//! Redis-backed event deduplication for idempotency.
//!
//! Provides fast O(1) duplicate detection using Redis sets with automatic
//! expiration. This ensures that duplicate events are rejected quickly
//! without hitting the database.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │   Ingest Event  │────►│  Redis Check    │────►│  PostgreSQL     │
//! │                 │     │  (Bloom/Set)    │     │  (Persistence)  │
//! └─────────────────┘     └─────────────────┘     └─────────────────┘
//!                                │
//!                                ▼
//!                         ┌─────────────────┐
//!                         │  Duplicate!     │
//!                         │  (Fast reject)  │
//!                         └─────────────────┘
//! ```

use std::collections::HashSet;
use std::sync::Arc;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Deduplication errors.
#[derive(Debug, Error)]
pub enum DedupError {
    #[error("Redis connection error: {0}")]
    Connection(#[from] redis::RedisError),

    #[error("Deduplication service unavailable")]
    Unavailable,
}

/// Configuration for deduplication.
#[derive(Debug, Clone)]
pub struct DedupConfig {
    /// TTL for transaction IDs in Redis (default: 24 hours).
    pub ttl_seconds: u64,
    /// Redis key prefix for transaction IDs.
    pub key_prefix: String,
    /// Whether to use local cache as fallback.
    pub use_local_fallback: bool,
    /// Maximum size of local cache (LRU eviction).
    pub local_cache_max_size: usize,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 86400, // 24 hours
            key_prefix: "creto:metering:txn:".to_string(),
            use_local_fallback: true,
            local_cache_max_size: 100_000,
        }
    }
}

/// Result of a deduplication check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupResult {
    /// This is a new transaction ID, proceed with ingestion.
    New,
    /// This transaction ID has been seen before, skip ingestion.
    Duplicate,
}

impl DedupResult {
    pub fn is_new(&self) -> bool {
        matches!(self, DedupResult::New)
    }

    pub fn is_duplicate(&self) -> bool {
        matches!(self, DedupResult::Duplicate)
    }
}

/// Deduplication service using Redis.
pub struct Deduplicator {
    redis: Option<ConnectionManager>,
    config: DedupConfig,
    /// Local cache for fallback when Redis is unavailable.
    local_cache: Arc<RwLock<HashSet<String>>>,
}

impl Deduplicator {
    /// Create a new deduplicator with Redis connection.
    pub async fn new(redis_url: &str, config: DedupConfig) -> Result<Self, DedupError> {
        let client = redis::Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            redis: Some(connection),
            config,
            local_cache: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    /// Create a deduplicator with only local cache (for testing or when Redis unavailable).
    pub fn local_only(config: DedupConfig) -> Self {
        Self {
            redis: None,
            config,
            local_cache: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check if a transaction ID is a duplicate and mark it as seen.
    ///
    /// This is an atomic check-and-set operation:
    /// - Returns `DedupResult::New` if this is the first time seeing this ID
    /// - Returns `DedupResult::Duplicate` if this ID was already processed
    pub async fn check_and_mark(&self, transaction_id: &str) -> Result<DedupResult, DedupError> {
        let key = format!("{}{}", self.config.key_prefix, transaction_id);

        // Try Redis first
        if let Some(ref redis) = self.redis {
            match self.redis_check_and_mark(redis.clone(), &key).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("Redis dedup check failed: {}, falling back to local cache", e);
                    if !self.config.use_local_fallback {
                        return Err(e);
                    }
                }
            }
        }

        // Fallback to local cache
        self.local_check_and_mark(transaction_id).await
    }

    /// Check multiple transaction IDs in batch.
    ///
    /// Returns a vector of results in the same order as input.
    pub async fn check_and_mark_batch(
        &self,
        transaction_ids: &[&str],
    ) -> Result<Vec<DedupResult>, DedupError> {
        if let Some(ref redis) = self.redis {
            match self.redis_batch_check(redis.clone(), transaction_ids).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    warn!("Redis batch dedup check failed: {}, falling back to local", e);
                    if !self.config.use_local_fallback {
                        return Err(e);
                    }
                }
            }
        }

        // Fallback to local cache for batch
        let mut results = Vec::with_capacity(transaction_ids.len());
        for id in transaction_ids {
            results.push(self.local_check_and_mark(id).await?);
        }
        Ok(results)
    }

    /// Check if a transaction ID exists without marking it.
    pub async fn exists(&self, transaction_id: &str) -> Result<bool, DedupError> {
        let key = format!("{}{}", self.config.key_prefix, transaction_id);

        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            let exists: bool = conn.exists(&key).await?;
            return Ok(exists);
        }

        let cache = self.local_cache.read().await;
        Ok(cache.contains(transaction_id))
    }

    /// Clear a transaction ID (useful for testing or manual cleanup).
    pub async fn clear(&self, transaction_id: &str) -> Result<(), DedupError> {
        let key = format!("{}{}", self.config.key_prefix, transaction_id);

        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            let _: () = conn.del(&key).await?;
        }

        let mut cache = self.local_cache.write().await;
        cache.remove(transaction_id);

        Ok(())
    }

    /// Get statistics about the deduplicator.
    pub async fn stats(&self) -> DedupStats {
        let local_size = self.local_cache.read().await.len();
        DedupStats {
            local_cache_size: local_size,
            redis_available: self.redis.is_some(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private Methods
    // ─────────────────────────────────────────────────────────────────────────

    async fn redis_check_and_mark(
        &self,
        mut conn: ConnectionManager,
        key: &str,
    ) -> Result<DedupResult, DedupError> {
        // SETNX with expiration - atomic set-if-not-exists
        let set_result: bool = redis::cmd("SET")
            .arg(key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(self.config.ttl_seconds)
            .query_async(&mut conn)
            .await
            .map_err(DedupError::Connection)?;

        if set_result {
            debug!("New transaction ID: {}", key);
            Ok(DedupResult::New)
        } else {
            debug!("Duplicate transaction ID: {}", key);
            Ok(DedupResult::Duplicate)
        }
    }

    async fn redis_batch_check(
        &self,
        mut conn: ConnectionManager,
        transaction_ids: &[&str],
    ) -> Result<Vec<DedupResult>, DedupError> {
        let mut pipe = redis::pipe();

        // Build pipeline of SETNX commands
        for id in transaction_ids {
            let key = format!("{}{}", self.config.key_prefix, id);
            pipe.cmd("SET")
                .arg(&key)
                .arg("1")
                .arg("NX")
                .arg("EX")
                .arg(self.config.ttl_seconds);
        }

        let results: Vec<Option<()>> = pipe
            .query_async(&mut conn)
            .await
            .map_err(DedupError::Connection)?;

        Ok(results
            .into_iter()
            .map(|r| {
                if r.is_some() {
                    DedupResult::New
                } else {
                    DedupResult::Duplicate
                }
            })
            .collect())
    }

    async fn local_check_and_mark(&self, transaction_id: &str) -> Result<DedupResult, DedupError> {
        let mut cache = self.local_cache.write().await;

        // Simple LRU: if cache is full, clear half of it
        if cache.len() >= self.config.local_cache_max_size {
            let to_remove: Vec<_> = cache.iter().take(cache.len() / 2).cloned().collect();
            for id in to_remove {
                cache.remove(&id);
            }
        }

        if cache.insert(transaction_id.to_string()) {
            Ok(DedupResult::New)
        } else {
            Ok(DedupResult::Duplicate)
        }
    }
}

/// Statistics about the deduplicator.
#[derive(Debug, Clone)]
pub struct DedupStats {
    /// Number of entries in local cache.
    pub local_cache_size: usize,
    /// Whether Redis is available.
    pub redis_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_dedup_new() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        let result = dedup.check_and_mark("txn_001").await.unwrap();
        assert!(result.is_new());
    }

    #[tokio::test]
    async fn test_local_dedup_duplicate() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        // First check - new
        let result1 = dedup.check_and_mark("txn_002").await.unwrap();
        assert!(result1.is_new());

        // Second check - duplicate
        let result2 = dedup.check_and_mark("txn_002").await.unwrap();
        assert!(result2.is_duplicate());
    }

    #[tokio::test]
    async fn test_local_dedup_different_ids() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        let r1 = dedup.check_and_mark("txn_a").await.unwrap();
        let r2 = dedup.check_and_mark("txn_b").await.unwrap();
        let r3 = dedup.check_and_mark("txn_c").await.unwrap();

        assert!(r1.is_new());
        assert!(r2.is_new());
        assert!(r3.is_new());
    }

    #[tokio::test]
    async fn test_exists() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        assert!(!dedup.exists("txn_003").await.unwrap());

        dedup.check_and_mark("txn_003").await.unwrap();

        assert!(dedup.exists("txn_003").await.unwrap());
    }

    #[tokio::test]
    async fn test_clear() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        dedup.check_and_mark("txn_004").await.unwrap();
        assert!(dedup.exists("txn_004").await.unwrap());

        dedup.clear("txn_004").await.unwrap();
        assert!(!dedup.exists("txn_004").await.unwrap());

        // Can insert again after clear
        let result = dedup.check_and_mark("txn_004").await.unwrap();
        assert!(result.is_new());
    }

    #[tokio::test]
    async fn test_batch_check() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        // Pre-insert one ID
        dedup.check_and_mark("batch_2").await.unwrap();

        let ids = vec!["batch_1", "batch_2", "batch_3"];
        let results = dedup.check_and_mark_batch(&ids).await.unwrap();

        assert!(results[0].is_new());     // batch_1 is new
        assert!(results[1].is_duplicate()); // batch_2 was already inserted
        assert!(results[2].is_new());     // batch_3 is new
    }

    #[tokio::test]
    async fn test_stats() {
        let dedup = Deduplicator::local_only(DedupConfig::default());

        dedup.check_and_mark("stat_1").await.unwrap();
        dedup.check_and_mark("stat_2").await.unwrap();

        let stats = dedup.stats().await;
        assert_eq!(stats.local_cache_size, 2);
        assert!(!stats.redis_available);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let dedup = Deduplicator::local_only(DedupConfig {
            local_cache_max_size: 10,
            ..Default::default()
        });

        // Fill cache
        for i in 0..10 {
            dedup.check_and_mark(&format!("evict_{}", i)).await.unwrap();
        }

        assert_eq!(dedup.stats().await.local_cache_size, 10);

        // Add one more - should trigger eviction
        dedup.check_and_mark("evict_trigger").await.unwrap();

        // Cache should be smaller now (half evicted + 1 new)
        assert!(dedup.stats().await.local_cache_size < 10);
    }

    #[test]
    fn test_dedup_result_helpers() {
        assert!(DedupResult::New.is_new());
        assert!(!DedupResult::New.is_duplicate());
        assert!(DedupResult::Duplicate.is_duplicate());
        assert!(!DedupResult::Duplicate.is_new());
    }
}
