//! Warm pool for pre-initialized sandboxes.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use creto_common::CretoResult;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::sandbox::{Sandbox, SandboxConfig, SandboxId, SandboxState};

/// Configuration for the warm pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum sandboxes to keep warm per runtime.
    pub min_warm: usize,

    /// Maximum sandboxes to keep warm per runtime.
    pub max_warm: usize,

    /// How long to keep idle sandboxes (seconds).
    pub idle_timeout_seconds: u64,

    /// How often to check for idle sandboxes (seconds).
    pub cleanup_interval_seconds: u64,

    /// Supported runtimes with their configs.
    pub runtimes: Vec<RuntimePoolConfig>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_warm: 2,
            max_warm: 10,
            idle_timeout_seconds: 300, // 5 minutes
            cleanup_interval_seconds: 60,
            runtimes: vec![
                RuntimePoolConfig {
                    name: "python3.11".to_string(),
                    min_warm: 2,
                    max_warm: 5,
                },
                RuntimePoolConfig {
                    name: "node20".to_string(),
                    min_warm: 1,
                    max_warm: 3,
                },
            ],
        }
    }
}

/// Per-runtime pool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePoolConfig {
    /// Runtime name.
    pub name: String,
    /// Minimum warm sandboxes for this runtime.
    pub min_warm: usize,
    /// Maximum warm sandboxes for this runtime.
    pub max_warm: usize,
}

/// Statistics about the warm pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total sandboxes in pool.
    pub total: usize,
    /// Sandboxes ready for use.
    pub ready: usize,
    /// Sandboxes currently in use.
    pub in_use: usize,
    /// Pool hits (sandbox reused).
    pub hits: u64,
    /// Pool misses (new sandbox created).
    pub misses: u64,
    /// Sandboxes evicted due to idle timeout.
    pub evictions: u64,
    /// Per-runtime breakdown.
    pub by_runtime: HashMap<String, RuntimePoolStats>,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total: 0,
            ready: 0,
            in_use: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            by_runtime: HashMap::new(),
        }
    }
}

/// Per-runtime statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimePoolStats {
    /// Ready sandboxes.
    pub ready: usize,
    /// In-use sandboxes.
    pub in_use: usize,
}

/// Warm pool for pre-initialized sandboxes.
///
/// Maintains a pool of ready-to-use sandboxes to minimize cold start latency.
pub struct WarmPool {
    config: PoolConfig,
    /// Sandboxes indexed by ID.
    sandboxes: Arc<RwLock<HashMap<SandboxId, PooledSandbox>>>,
    /// Ready sandboxes by runtime (for fast acquisition).
    ready_by_runtime: Arc<RwLock<HashMap<String, Vec<SandboxId>>>>,
    /// Statistics.
    stats: Arc<RwLock<PoolStats>>,
}

/// A sandbox in the pool with metadata.
struct PooledSandbox {
    sandbox: Sandbox,
    acquired: bool,
    acquired_at: Option<DateTime<Utc>>,
}

impl WarmPool {
    /// Create a new warm pool.
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            sandboxes: Arc::new(RwLock::new(HashMap::new())),
            ready_by_runtime: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    /// Initialize the pool (pre-warm sandboxes).
    pub async fn initialize(&self) -> CretoResult<()> {
        // TODO: Pre-create sandboxes based on config
        // For each runtime in config.runtimes:
        //   Create min_warm sandboxes
        //   Add to pool

        tracing::info!(
            "Initializing warm pool with {} runtimes",
            self.config.runtimes.len()
        );

        Ok(())
    }

    /// Acquire a sandbox from the pool.
    ///
    /// Returns a ready sandbox if available, or None if pool is empty.
    pub async fn acquire(&self, runtime: &str) -> Option<Sandbox> {
        let mut ready_map = self.ready_by_runtime.write().await;
        let mut sandboxes = self.sandboxes.write().await;
        let mut stats = self.stats.write().await;

        // Try to get a ready sandbox for this runtime
        if let Some(ready_list) = ready_map.get_mut(runtime) {
            if let Some(sandbox_id) = ready_list.pop() {
                if let Some(pooled) = sandboxes.get_mut(&sandbox_id) {
                    pooled.acquired = true;
                    pooled.acquired_at = Some(Utc::now());
                    stats.hits += 1;
                    stats.ready -= 1;
                    stats.in_use += 1;

                    if let Some(runtime_stats) = stats.by_runtime.get_mut(runtime) {
                        runtime_stats.ready -= 1;
                        runtime_stats.in_use += 1;
                    }

                    return Some(pooled.sandbox.clone());
                }
            }
        }

        stats.misses += 1;
        None
    }

    /// Release a sandbox back to the pool.
    pub async fn release(&self, sandbox_id: SandboxId) -> CretoResult<()> {
        let mut sandboxes = self.sandboxes.write().await;
        let mut ready_map = self.ready_by_runtime.write().await;
        let mut stats = self.stats.write().await;

        if let Some(pooled) = sandboxes.get_mut(&sandbox_id) {
            if pooled.acquired {
                pooled.acquired = false;
                pooled.acquired_at = None;
                pooled.sandbox.last_used_at = Some(Utc::now());

                let runtime = pooled.sandbox.config.runtime.clone();

                // Add back to ready list
                ready_map
                    .entry(runtime.clone())
                    .or_default()
                    .push(sandbox_id);

                stats.in_use -= 1;
                stats.ready += 1;

                if let Some(runtime_stats) = stats.by_runtime.get_mut(&runtime) {
                    runtime_stats.in_use -= 1;
                    runtime_stats.ready += 1;
                }
            }
        }

        Ok(())
    }

    /// Add a new sandbox to the pool.
    pub async fn add(&self, sandbox: Sandbox) -> CretoResult<()> {
        let mut sandboxes = self.sandboxes.write().await;
        let mut ready_map = self.ready_by_runtime.write().await;
        let mut stats = self.stats.write().await;

        let sandbox_id = sandbox.id;
        let runtime = sandbox.config.runtime.clone();
        let is_ready = sandbox.state == SandboxState::Ready;

        sandboxes.insert(
            sandbox_id,
            PooledSandbox {
                sandbox,
                acquired: false,
                acquired_at: None,
            },
        );

        if is_ready {
            ready_map.entry(runtime.clone()).or_default().push(sandbox_id);
            stats.ready += 1;
        }

        stats.total += 1;

        let runtime_stats = stats.by_runtime.entry(runtime).or_default();
        if is_ready {
            runtime_stats.ready += 1;
        }

        Ok(())
    }

    /// Remove a sandbox from the pool.
    pub async fn remove(&self, sandbox_id: SandboxId) -> Option<Sandbox> {
        let mut sandboxes = self.sandboxes.write().await;
        let mut ready_map = self.ready_by_runtime.write().await;
        let mut stats = self.stats.write().await;

        if let Some(pooled) = sandboxes.remove(&sandbox_id) {
            let runtime = &pooled.sandbox.config.runtime;

            // Remove from ready list if present
            if let Some(ready_list) = ready_map.get_mut(runtime) {
                ready_list.retain(|id| *id != sandbox_id);
            }

            stats.total -= 1;
            if pooled.acquired {
                stats.in_use -= 1;
            } else {
                stats.ready -= 1;
            }

            if let Some(runtime_stats) = stats.by_runtime.get_mut(runtime) {
                if pooled.acquired {
                    runtime_stats.in_use -= 1;
                } else {
                    runtime_stats.ready -= 1;
                }
            }

            return Some(pooled.sandbox);
        }

        None
    }

    /// Get current pool statistics.
    pub async fn stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }

    /// Cleanup idle sandboxes.
    pub async fn cleanup_idle(&self) -> CretoResult<Vec<SandboxId>> {
        let sandboxes = self.sandboxes.read().await;
        let idle_timeout = self.config.idle_timeout_seconds;

        let expired: Vec<SandboxId> = sandboxes
            .iter()
            .filter(|(_, pooled)| {
                !pooled.acquired && pooled.sandbox.is_idle_expired(idle_timeout)
            })
            .map(|(id, _)| *id)
            .collect();

        drop(sandboxes);

        // Remove expired sandboxes
        let mut removed = Vec::new();
        for id in expired {
            if self.remove(id).await.is_some() {
                removed.push(id);
            }
        }

        // Update eviction stats
        if !removed.is_empty() {
            let mut stats = self.stats.write().await;
            stats.evictions += removed.len() as u64;
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = WarmPool::new(PoolConfig::default());
        let stats = pool.stats().await;
        assert_eq!(stats.total, 0);
    }

    #[tokio::test]
    async fn test_pool_add_and_acquire() {
        use creto_common::{AgentId, OrganizationId};

        let pool = WarmPool::new(PoolConfig::default());

        // Create and add a sandbox
        let mut sandbox = Sandbox::new(
            OrganizationId::new(),
            AgentId::new(),
            SandboxConfig {
                runtime: "python3.11".to_string(),
                ..Default::default()
            },
        );
        sandbox.mark_ready("handle_123".to_string());

        pool.add(sandbox).await.unwrap();

        let stats = pool.stats().await;
        assert_eq!(stats.total, 1);
        assert_eq!(stats.ready, 1);

        // Acquire the sandbox
        let acquired = pool.acquire("python3.11").await;
        assert!(acquired.is_some());

        let stats = pool.stats().await;
        assert_eq!(stats.ready, 0);
        assert_eq!(stats.in_use, 1);
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test]
    async fn test_pool_miss() {
        let pool = WarmPool::new(PoolConfig::default());

        // Try to acquire from empty pool
        let acquired = pool.acquire("python3.11").await;
        assert!(acquired.is_none());

        let stats = pool.stats().await;
        assert_eq!(stats.misses, 1);
    }
}
