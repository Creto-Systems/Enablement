//! Runtime service facade.

use creto_common::{AgentId, CretoError, CretoResult, OrganizationId};

use crate::{
    checkpoint::{CheckpointConfig, CheckpointId, CheckpointManager, InMemoryCheckpointStore},
    execution::{ExecutionRequest, ExecutionResult, Executor},
    pool::{PoolConfig, WarmPool},
    sandbox::{Sandbox, SandboxConfig, SandboxId, SandboxState},
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

    /// Checkpoint manager.
    checkpoint_manager: Box<dyn CheckpointManager>,
}

impl RuntimeService {
    /// Create a new runtime service with default configuration.
    pub fn new() -> Self {
        Self {
            pool: WarmPool::new(PoolConfig::default()),
            executor: Executor::new(),
            secret_provider: None,
            checkpoint_manager: Box::new(InMemoryCheckpointStore::new()),
        }
    }

    /// Create a runtime service with custom pool configuration.
    pub fn with_pool_config(config: PoolConfig) -> Self {
        Self {
            pool: WarmPool::new(config),
            executor: Executor::new(),
            secret_provider: None,
            checkpoint_manager: Box::new(InMemoryCheckpointStore::new()),
        }
    }

    /// Set the secret provider.
    pub fn with_secret_provider(mut self, provider: Box<dyn SecretProvider>) -> Self {
        self.secret_provider = Some(provider);
        self
    }

    /// Set the checkpoint manager.
    pub fn with_checkpoint_manager(mut self, manager: Box<dyn CheckpointManager>) -> Self {
        self.checkpoint_manager = manager;
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

    /// Create a checkpoint of a sandbox.
    ///
    /// The sandbox must be in a state that allows checkpointing (Ready, Paused, or Stopped).
    /// Returns the ID of the created checkpoint.
    pub async fn checkpoint(&self, sandbox_id: SandboxId) -> CretoResult<CheckpointId> {
        // TODO: Get the actual sandbox from pool or repository
        // For now, use default config
        let config = CheckpointConfig::default();

        tracing::info!(
            sandbox_id = %sandbox_id,
            "Creating checkpoint for sandbox"
        );

        let checkpoint_id = self.checkpoint_manager.checkpoint(sandbox_id, config).await?;

        tracing::info!(
            sandbox_id = %sandbox_id,
            checkpoint_id = %checkpoint_id,
            "Checkpoint created successfully"
        );

        Ok(checkpoint_id)
    }

    /// Restore a sandbox from a checkpoint.
    ///
    /// Creates a new sandbox (or reuses an existing one) and restores it to the
    /// state captured in the checkpoint.
    pub async fn restore(&self, checkpoint_id: CheckpointId) -> CretoResult<Sandbox> {
        tracing::info!(
            checkpoint_id = %checkpoint_id,
            "Restoring sandbox from checkpoint"
        );

        let sandbox_id = self.checkpoint_manager.restore(checkpoint_id).await?;

        // TODO: Actually create/retrieve the sandbox instance
        // For now, create a mock sandbox
        let checkpoint = self.checkpoint_manager.get_checkpoint(checkpoint_id).await?;

        let mut sandbox = Sandbox::new(
            OrganizationId::new(),
            checkpoint.agent_id,
            SandboxConfig::default(),
        );
        sandbox.id = sandbox_id;
        sandbox.state = SandboxState::Ready;

        tracing::info!(
            checkpoint_id = %checkpoint_id,
            sandbox_id = %sandbox_id,
            "Sandbox restored successfully"
        );

        Ok(sandbox)
    }

    /// List checkpoints, optionally filtered by sandbox or agent.
    pub async fn list_checkpoints(
        &self,
        sandbox_id: Option<SandboxId>,
        agent_id: Option<AgentId>,
    ) -> CretoResult<Vec<crate::checkpoint::Checkpoint>> {
        self.checkpoint_manager
            .list_checkpoints(sandbox_id, agent_id)
            .await
    }

    /// Delete a checkpoint.
    pub async fn delete_checkpoint(&self, checkpoint_id: CheckpointId) -> CretoResult<()> {
        tracing::info!(
            checkpoint_id = %checkpoint_id,
            "Deleting checkpoint"
        );

        self.checkpoint_manager
            .delete_checkpoint(checkpoint_id)
            .await?;

        tracing::info!(
            checkpoint_id = %checkpoint_id,
            "Checkpoint deleted successfully"
        );

        Ok(())
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

    #[tokio::test]
    async fn test_checkpoint_and_restore() {
        let service = RuntimeService::new();

        let sandbox = service
            .create_sandbox(
                OrganizationId::new(),
                AgentId::new(),
                SandboxConfig::default(),
            )
            .await
            .unwrap();

        // Create checkpoint
        let checkpoint_id = service.checkpoint(sandbox.id).await.unwrap();
        assert!(!checkpoint_id.to_string().is_empty());

        // Restore from checkpoint
        let restored = service.restore(checkpoint_id).await.unwrap();
        assert_eq!(restored.state, SandboxState::Ready);
    }

    #[tokio::test]
    async fn test_list_checkpoints() {
        let service = RuntimeService::new();
        let sandbox_id = SandboxId::new();

        // Create multiple checkpoints
        service.checkpoint(sandbox_id).await.unwrap();
        service.checkpoint(sandbox_id).await.unwrap();

        let checkpoints = service.list_checkpoints(Some(sandbox_id), None).await.unwrap();
        assert_eq!(checkpoints.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_checkpoint() {
        let service = RuntimeService::new();
        let sandbox_id = SandboxId::new();

        let checkpoint_id = service.checkpoint(sandbox_id).await.unwrap();

        // Delete should succeed
        service.delete_checkpoint(checkpoint_id).await.unwrap();

        // Deleting again should fail
        let result = service.delete_checkpoint(checkpoint_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_checkpoint_state_transitions() {
        let service = RuntimeService::new();
        let mut sandbox = Sandbox::new(
            OrganizationId::new(),
            AgentId::new(),
            SandboxConfig::default(),
        );

        // Can't checkpoint while Creating
        assert!(!sandbox.state.can_checkpoint());

        // Can checkpoint when Ready
        sandbox.mark_ready("handle".to_string());
        assert!(sandbox.state.can_checkpoint());

        // Can't checkpoint when Running
        sandbox.mark_running();
        assert!(!sandbox.state.can_checkpoint());

        // Can't checkpoint when Failed
        sandbox.mark_failed();
        assert!(!sandbox.state.can_checkpoint());

        // Can't checkpoint when Terminated
        sandbox.mark_terminated();
        assert!(!sandbox.state.can_checkpoint());
    }
}
