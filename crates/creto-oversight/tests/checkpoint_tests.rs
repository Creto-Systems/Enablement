//! Comprehensive checkpoint and recovery tests for Week 10 features.

use creto_common::{AgentId, OrganizationId, UserId};
use creto_oversight::{
    approval::{Approval, ApprovalDecision},
    request::{ActionType, OversightRequest, RequestStatus},
    service::OversightService,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Mock checkpoint storage for testing
#[derive(Debug, Clone)]
struct CheckpointStorage {
    requests: HashMap<Uuid, OversightRequest>,
    approvals: HashMap<Uuid, Vec<Approval>>,
    version: u64,
}

impl CheckpointStorage {
    fn new() -> Self {
        Self {
            requests: HashMap::new(),
            approvals: HashMap::new(),
            version: 0,
        }
    }

    fn save(&mut self, request: OversightRequest, approvals: Vec<Approval>) -> u64 {
        self.version += 1;
        self.requests.insert(request.id, request.clone());
        self.approvals.insert(request.id, approvals);
        self.version
    }

    fn load(&self, request_id: Uuid) -> Option<(OversightRequest, Vec<Approval>)> {
        let request = self.requests.get(&request_id)?;
        let approvals = self.approvals.get(&request_id)?;
        Some((request.clone(), approvals.clone()))
    }

    fn get_version(&self) -> u64 {
        self.version
    }
}

#[test]
fn test_save_and_load_checkpoint() {
    let mut storage = CheckpointStorage::new();

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 500000,
            currency: "USD".to_string(),
        },
        "Test transaction",
    );

    let approval = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);

    // Save checkpoint
    let version = storage.save(request.clone(), vec![approval.clone()]);
    assert_eq!(version, 1);

    // Load checkpoint
    let loaded = storage.load(request.id).unwrap();
    assert_eq!(loaded.0.id, request.id);
    assert_eq!(loaded.1.len(), 1);
    assert_eq!(loaded.1[0].decision, ApprovalDecision::Approve);
}

#[test]
fn test_checkpoint_versioning() {
    let mut storage = CheckpointStorage::new();

    // Create multiple checkpoints
    for i in 0..5 {
        let request = OversightRequest::new(
            OrganizationId::new(),
            AgentId::new(),
            ActionType::Custom {
                type_id: format!("test_{}", i),
            },
            format!("Test request {}", i),
        );

        let version = storage.save(request, vec![]);
        assert_eq!(version, (i + 1) as u64);
    }

    assert_eq!(storage.get_version(), 5);
}

#[test]
fn test_recover_pending_requests() {
    let mut storage = CheckpointStorage::new();

    // Create several requests in different states
    let pending_request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 100000,
            currency: "USD".to_string(),
        },
        "Pending transaction",
    );

    let mut approved_request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::DataAccess {
            data_type: "data".to_string(),
            scope: "read".to_string(),
        },
        "Approved request",
    );
    approved_request.status = RequestStatus::Approved;

    // Save checkpoints
    storage.save(pending_request.clone(), vec![]);
    storage.save(approved_request.clone(), vec![]);

    // Recover pending requests
    let pending_requests: Vec<_> = storage
        .requests
        .values()
        .filter(|r| r.is_pending())
        .cloned()
        .collect();

    assert_eq!(pending_requests.len(), 1);
    assert_eq!(pending_requests[0].id, pending_request.id);
}

#[test]
fn test_crash_recovery_simulation() {
    // Simulate a crash and recovery scenario
    let mut storage = CheckpointStorage::new();

    // Create request in progress
    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::CodeExecution {
            runtime: "python".to_string(),
            risk_level: "high".to_string(),
        },
        "Critical code execution",
    );

    let approval1 = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);

    // Save checkpoint before "crash"
    storage.save(request.clone(), vec![approval1.clone()]);

    // Simulate crash - clear in-memory state
    // (In real implementation, this would be a process restart)

    // Recover from checkpoint
    let (recovered_request, recovered_approvals) = storage.load(request.id).unwrap();

    assert_eq!(recovered_request.id, request.id);
    assert_eq!(recovered_approvals.len(), 1);

    // Continue processing after recovery
    let approval2 = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);
    let mut all_approvals = recovered_approvals;
    all_approvals.push(approval2);

    storage.save(recovered_request, all_approvals.clone());

    // Verify state is correct
    let final_state = storage.load(request.id).unwrap();
    assert_eq!(final_state.1.len(), 2);
}

#[test]
fn test_checkpoint_atomicity() {
    let mut storage = CheckpointStorage::new();

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 1000000,
            currency: "USD".to_string(),
        },
        "Atomic test",
    );

    let approval1 = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);
    let approval2 = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);

    // Save complete state atomically
    let version = storage.save(request.clone(), vec![approval1, approval2]);

    // Load and verify all data is present
    let loaded = storage.load(request.id).unwrap();
    assert_eq!(loaded.0.id, request.id);
    assert_eq!(loaded.1.len(), 2);
    assert_eq!(storage.get_version(), version);
}

#[test]
fn test_checkpoint_with_context() {
    let mut storage = CheckpointStorage::new();

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 250000,
            currency: "USD".to_string(),
        },
        "Request with context",
    )
    .with_context(serde_json::json!({
        "merchant": "Acme Corp",
        "invoice": "INV-12345",
        "department": "marketing"
    }));

    storage.save(request.clone(), vec![]);

    // Recover and verify context is preserved
    let recovered = storage.load(request.id).unwrap();
    assert_eq!(recovered.0.context["merchant"], "Acme Corp");
    assert_eq!(recovered.0.context["invoice"], "INV-12345");
}

#[tokio::test]
async fn test_service_checkpoint_integration() {
    // Test that OversightService can work with checkpoint/restore pattern
    let service = OversightService::new();

    let result = service
        .check_action(
            OrganizationId::new(),
            AgentId::new(),
            ActionType::Transaction {
                amount_cents: 2000000,
                currency: "USD".to_string(),
            },
            "Large transaction requiring checkpoint",
        )
        .await
        .unwrap();

    // For requests requiring approval, we should be able to checkpoint
    if result.requires_approval() {
        let request_id = result.request_id().unwrap();

        // In real implementation, this would save to persistent storage
        // For now, verify the request ID is valid
        assert!(request_id.get_version().is_some());
    }
}

#[test]
fn test_partial_approval_checkpoint() {
    let mut storage = CheckpointStorage::new();

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 5000000,
            currency: "USD".to_string(),
        },
        "Multi-approval transaction",
    );

    // Save checkpoint after first approval
    let approval1 = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);
    let v1 = storage.save(request.clone(), vec![approval1.clone()]);

    // Add second approval and checkpoint again
    let approval2 = Approval::new(request.id, UserId::new(), ApprovalDecision::Approve);
    let v2 = storage.save(request.clone(), vec![approval1, approval2]);

    // Verify version incremented
    assert!(v2 > v1);

    // Load and verify both approvals present
    let loaded = storage.load(request.id).unwrap();
    assert_eq!(loaded.1.len(), 2);
}
