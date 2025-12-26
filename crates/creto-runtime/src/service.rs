//! Runtime service facade.

use creto_common::{AgentId, CretoError, CretoResult, OrganizationId};

use crate::{
    execution::{ExecutionRequest, ExecutionResult, Executor},
    pool::{PoolConfig, WarmPool},
    sandbox::{Sandbox, SandboxConfig, SandboxId},
    secrets::{SecretMount, SecretProvider},
};

/// Main entry point for the runtime system.
pub struct RuntimeService {
    /// Warm pool for sandboxes.
    pool: WarmPool,

    /// Code executor.
    executor: Executor,

    /// Secret provider.
    secret_provider: Option<Box<dyn SecretProvider>>,
}

impl RuntimeService {
    /// Create a new runtime service with default configuration.
    pub fn new() -> Self {
        Self {
            pool: WarmPool::new(PoolConfig::default()),
            executor: Executor::new(),
            secret_provider: None,
        }
    }

    /// Create a runtime service with custom pool configuration.
    pub fn with_pool_config(config: PoolConfig) -> Self {
        Self {
            pool: WarmPool::new(config),
            executor: Executor::new(),
            secret_provider: None,
        }
    }

    /// Set the secret provider.
    pub fn with_secret_provider(mut self, provider: Box<dyn SecretProvider>) -> Self {
        self.secret_provider = Some(provider);
        self
    }

    /// Initialize the runtime (pre-warm pools).
    pub async fn initialize(&self) -> CretoResult<()> {
        self.pool.initialize().await
    }

    /// Create a new sandbox.
    ///
    /// Attempts to acquire from warm pool first, creates new if none available.
    pub async fn create_sandbox(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        config: SandboxConfig,
    ) -> CretoResult<Sandbox> {
        // Try to acquire from warm pool
        if let Some(mut sandbox) = self.pool.acquire(&config.runtime).await {
            // Update ownership
            sandbox.organization_id = organization_id;
            sandbox.agent_id = agent_id;
            tracing::debug!(
                sandbox_id = %sandbox.id,
                runtime = %config.runtime,
                "Acquired sandbox from warm pool"
            );
            return Ok(sandbox);
        }

        // Create new sandbox
        tracing::debug!(
            runtime = %config.runtime,
            "Creating new sandbox (pool miss)"
        );

        let sandbox = Sandbox::new(organization_id, agent_id, config);

        // TODO: Actually create the sandbox via backend
        // let handle = backend.create(&sandbox.config).await?;
        // sandbox.mark_ready(handle);

        Ok(sandbox)
    }

    /// Execute code in a sandbox.
    pub async fn execute(
        &self,
        sandbox_id: SandboxId,
        code: impl Into<String>,
    ) -> CretoResult<ExecutionResult> {
        let request = ExecutionRequest::new(sandbox_id, code);
        self.executor.execute(request).await
    }

    /// Execute code with secrets injected.
    pub async fn execute_with_secrets(
        &self,
        sandbox_id: SandboxId,
        organization_id: OrganizationId,
        agent_id: AgentId,
        code: impl Into<String>,
        secrets: Vec<SecretMount>,
    ) -> CretoResult<ExecutionResult> {
        // Resolve secrets
        if let Some(provider) = &self.secret_provider {
            for secret in &secrets {
                // Verify authorization
                if !provider.authorize(organization_id, agent_id, &secret.source).await? {
                    return Err(CretoError::NotAuthorized {
                        resource: format!("secret:{}", secret.name),
                        action: "access".to_string(),
                    });
                }

                // TODO: Inject secret into sandbox
                let _value = provider.resolve(organization_id, agent_id, &secret.source).await?;
                // backend.inject_secret(sandbox_id, &secret.name, value).await?;
            }
        }

        // Execute
        let request = ExecutionRequest::new(sandbox_id, code);
        self.executor.execute(request).await
    }

    /// Release a sandbox back to the pool.
    pub async fn release_sandbox(&self, sandbox_id: SandboxId) -> CretoResult<()> {
        self.pool.release(sandbox_id).await
    }

    /// Terminate a sandbox.
    pub async fn terminate_sandbox(&self, sandbox_id: SandboxId) -> CretoResult<()> {
        // Remove from pool and terminate
        if let Some(mut sandbox) = self.pool.remove(sandbox_id).await {
            sandbox.mark_terminated();
            // TODO: Actually terminate via backend
            // backend.terminate(&sandbox.runtime_handle.unwrap()).await?;
        }
        Ok(())
    }

    /// Get pool statistics.
    pub async fn pool_stats(&self) -> crate::pool::PoolStats {
        self.pool.stats().await
    }

    /// Cleanup idle sandboxes.
    pub async fn cleanup_idle(&self) -> CretoResult<usize> {
        let removed = self.pool.cleanup_idle().await?;
        Ok(removed.len())
    }
}

impl Default for RuntimeService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = RuntimeService::new();
        let stats = service.pool_stats().await;
        assert_eq!(stats.total, 0);
    }

    #[tokio::test]
    async fn test_create_sandbox() {
        let service = RuntimeService::new();

        let sandbox = service
            .create_sandbox(
                OrganizationId::new(),
                AgentId::new(),
                SandboxConfig::default(),
            )
            .await
            .unwrap();

        assert_eq!(sandbox.config.runtime, "python3.11");
    }

    #[tokio::test]
    async fn test_execute() {
        let service = RuntimeService::new();

        let sandbox = service
            .create_sandbox(
                OrganizationId::new(),
                AgentId::new(),
                SandboxConfig::default(),
            )
            .await
            .unwrap();

        let result = service.execute(sandbox.id, "print('hello')").await.unwrap();
        assert!(result.is_success());
    }
}
