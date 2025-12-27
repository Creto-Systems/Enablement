//! Integration tests for checkpoint crash recovery scenarios.

use chrono::Utc;
use creto_common::{AgentId, OrganizationId, UserId};
use creto_oversight::{
    checkpoint::{Checkpoint, CheckpointManager, CheckpointRepository, CHECKPOINT_VERSION},
    request::{ActionType, OversightRequest, RequestStatus},
    service::OversightService,
    state::{Actor, StateMachine},
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// In-memory checkpoint repository for testing.
struct InMemoryCheckpointRepository {
    checkpoints: Arc<Mutex<Vec<Checkpoint>>>,
}

impl InMemoryCheckpointRepository {
    fn new() -> Self {
        Self {
            checkpoints: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl CheckpointRepository for InMemoryCheckpointRepository {
    async fn create(&self, checkpoint: &Checkpoint) -> Result<Uuid, creto_common::CretoError> {
        let mut checkpoints = self.checkpoints.lock().unwrap();
        checkpoints.push(checkpoint.clone());
        Ok(checkpoint.id)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Checkpoint>, creto_common::CretoError> {
        let checkpoints = self.checkpoints.lock().unwrap();
        Ok(checkpoints.iter().find(|c| c.id == id).cloned())
    }

    async fn get_latest(
        &self,
        request_id: Uuid,
    ) -> Result<Option<Checkpoint>, creto_common::CretoError> {
        let checkpoints = self.checkpoints.lock().unwrap();
        Ok(checkpoints
            .iter()
            .filter(|c| c.request_id == request_id)
            .max_by_key(|c| c.timestamp)
            .cloned())
    }

    async fn list_by_request(
        &self,
        request_id: Uuid,
    ) -> Result<Vec<Checkpoint>, creto_common::CretoError> {
        let checkpoints = self.checkpoints.lock().unwrap();
        Ok(checkpoints
            .iter()
            .filter(|c| c.request_id == request_id)
            .cloned()
            .collect())
    }

    async fn delete_before(
        &self,
        before: chrono::DateTime<Utc>,
    ) -> Result<usize, creto_common::CretoError> {
        let mut checkpoints = self.checkpoints.lock().unwrap();
        let initial_len = checkpoints.len();
        checkpoints.retain(|c| c.timestamp >= before);
        Ok(initial_len - checkpoints.len())
    }
}

#[tokio::test]
async fn test_checkpoint_save_and_restore() {
    let repo = Box::new(InMemoryCheckpointRepository::new());
    let manager = CheckpointManager::new(repo);

    let request_id = Uuid::now_v7();
    let mut state_machine = StateMachine::new();

    state_machine
        .transition(
            RequestStatus::InReview,
            Actor::User {
                user_id: UserId::new(),
            },
            Some("Starting review".to_string()),
        )
        .unwrap();

    let context = serde_json::json!({
        "conversation": ["User requested approval", "System acknowledged"]
    });

    // Create and save checkpoint
    let checkpoint = Checkpoint::new(
        request_id,
        RequestStatus::InReview,
        &state_machine,
        context.clone(),
    )
    .with_reason("test_checkpoint");

    let checkpoint_id = manager.save_checkpoint(&checkpoint).await.unwrap();
    assert_eq!(checkpoint_id, checkpoint.id);

    // Load checkpoint
    let loaded = manager.load_checkpoint(request_id).await.unwrap();
    assert!(loaded.is_some());

    let loaded = loaded.unwrap();
    assert_eq!(loaded.request_id, request_id);
    assert_eq!(loaded.status, RequestStatus::InReview);
    assert_eq!(loaded.context, context);
    assert_eq!(loaded.reason, Some("test_checkpoint".to_string()));

    // Restore state machine
    let restored_machine = loaded.restore_state_machine().unwrap();
    assert_eq!(restored_machine.current(), RequestStatus::InReview);
}

#[tokio::test]
async fn test_checkpoint_crash_recovery_scenario() {
    let repo = Box::new(InMemoryCheckpointRepository::new());
    let manager = CheckpointManager::new(repo);
    let service = OversightService::with_checkpoints(manager);

    // Simulate a request in progress
    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 100000,
            currency: "USD".to_string(),
        },
        "Large transaction",
    );

    let mut state_machine = StateMachine::new();
    state_machine
        .transition(
            RequestStatus::InReview,
            Actor::User {
                user_id: UserId::new(),
            },
            None,
        )
        .unwrap();

    let context = serde_json::json!({
        "messages": [
            {"role": "agent", "content": "Requesting approval for $1000 transaction"},
            {"role": "system", "content": "Notified reviewers"}
        ]
    });

    // Save checkpoint before "crash"
    let checkpoint_id = service
        .save_checkpoint(
            &request,
            &state_machine,
            context.clone(),
            Some("before_crash".to_string()),
        )
        .await
        .unwrap();

    // Simulate system restart - load checkpoint
    let loaded_checkpoint = service.load_checkpoint(request.id).await.unwrap();
    assert!(loaded_checkpoint.is_some());

    let loaded = loaded_checkpoint.unwrap();
    assert_eq!(loaded.id, checkpoint_id);
    assert_eq!(loaded.request_id, request.id);
    assert_eq!(loaded.status, RequestStatus::InReview);

    // Restore state and continue processing
    let restored_machine = loaded.restore_state_machine().unwrap();
    assert_eq!(restored_machine.current(), RequestStatus::InReview);
    assert_eq!(loaded.context, context);
}

#[tokio::test]
async fn test_multiple_checkpoints_returns_latest() {
    let repo = Box::new(InMemoryCheckpointRepository::new());
    let manager = CheckpointManager::new(repo);

    let request_id = Uuid::now_v7();
    let machine = StateMachine::new();

    // Save multiple checkpoints
    for i in 0..3 {
        let context = serde_json::json!({ "step": i });
        let checkpoint = Checkpoint::new(request_id, RequestStatus::Pending, &machine, context);
        manager.save_checkpoint(&checkpoint).await.unwrap();

        // Add small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Load latest
    let latest = manager.load_checkpoint(request_id).await.unwrap();
    assert!(latest.is_some());

    let latest = latest.unwrap();
    assert_eq!(latest.context["step"], 2);
}

#[tokio::test]
async fn test_checkpoint_version_compatibility() {
    let request_id = Uuid::now_v7();
    let machine = StateMachine::new();
    let context = serde_json::json!({});

    let mut checkpoint = Checkpoint::new(request_id, RequestStatus::Pending, &machine, context);

    // Current version is compatible
    assert!(checkpoint.is_compatible());

    // Future version is incompatible
    checkpoint.version = CHECKPOINT_VERSION + 1;
    assert!(!checkpoint.is_compatible());

    // Attempting to restore incompatible checkpoint fails
    let result = checkpoint.restore_state_machine();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_checkpoint_list_history() {
    let repo = Box::new(InMemoryCheckpointRepository::new());
    let manager = CheckpointManager::new(repo);

    let request_id = Uuid::now_v7();
    let machine = StateMachine::new();

    // Create checkpoint history
    let reasons = vec!["initial", "after_approval_1", "after_approval_2"];
    for reason in &reasons {
        let checkpoint = Checkpoint::new(
            request_id,
            RequestStatus::Pending,
            &machine,
            serde_json::json!({}),
        )
        .with_reason(*reason);

        manager.save_checkpoint(&checkpoint).await.unwrap();
    }

    // List all checkpoints
    let history = manager.list_checkpoints(request_id).await.unwrap();
    assert_eq!(history.len(), 3);

    // Verify reasons are preserved
    let checkpoint_reasons: Vec<_> = history.iter().filter_map(|c| c.reason.as_deref()).collect();

    assert!(checkpoint_reasons.contains(&"initial"));
    assert!(checkpoint_reasons.contains(&"after_approval_1"));
    assert!(checkpoint_reasons.contains(&"after_approval_2"));
}

#[tokio::test]
async fn test_checkpoint_pruning() {
    let repo = Box::new(InMemoryCheckpointRepository::new());
    let manager = CheckpointManager::new(repo);

    let request_id = Uuid::now_v7();
    let machine = StateMachine::new();

    // Create old checkpoint
    let old_checkpoint = Checkpoint::new(
        request_id,
        RequestStatus::Pending,
        &machine,
        serde_json::json!({}),
    );
    manager.save_checkpoint(&old_checkpoint).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let cutoff = Utc::now();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Create new checkpoint
    let new_checkpoint = Checkpoint::new(
        request_id,
        RequestStatus::Pending,
        &machine,
        serde_json::json!({}),
    );
    manager.save_checkpoint(&new_checkpoint).await.unwrap();

    // Prune old checkpoints
    let deleted = manager.prune_old_checkpoints(cutoff).await.unwrap();
    assert_eq!(deleted, 1);

    // Verify only new checkpoint remains
    let remaining = manager.list_checkpoints(request_id).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, new_checkpoint.id);
}
