---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
parent_sdd: docs/sdd/products/oversight/01-requirements.md
---

# SDD-OVS-06: Oversight Testing Strategy

## 1. Testing Overview

### 1.1 Testing Pyramid

```
                    ┌─────────────────┐
                    │  E2E Tests      │  ~50 tests
                    │  (5% of suite)  │
                    └────────┬────────┘
                             │
                ┌────────────┴────────────┐
                │  Integration Tests      │  ~200 tests
                │  (20% of suite)         │
                └────────────┬────────────┘
                             │
            ┌────────────────┴────────────────┐
            │     Unit Tests                  │  ~750 tests
            │     (75% of suite)              │
            └─────────────────────────────────┘
```

**Test Distribution:**
- **Unit Tests (75%):** Component-level logic, state machine transitions, policy matching
- **Integration Tests (20%):** Database operations, channel delivery, service-to-service communication
- **End-to-End Tests (5%):** Complete approval flows across all components

**Coverage Target:** 90% code coverage (enforced in CI)

### 1.2 Testing Principles

1. **Fast Feedback:** Unit tests run in <5s, integration tests in <30s
2. **Deterministic:** No flaky tests, no time-based race conditions
3. **Isolated:** Tests don't depend on external services (use mocks)
4. **Comprehensive:** Cover happy paths, edge cases, and error scenarios
5. **Maintainable:** Clear test names, DRY test utilities, minimal boilerplate

---

## 2. Unit Testing

### 2.1 State Machine Transition Tests

**Test Coverage:**
- All state transitions (PENDING → APPROVED, PENDING → DENIED, etc.)
- Invalid transitions rejected
- Concurrent response handling (quorum counting)
- Idempotency (duplicate responses)

**Example Test:**
```rust
#[tokio::test]
async fn test_pending_to_approved_on_quorum_met() {
    // Setup
    let mut request_manager = setup_request_manager().await;
    let request = create_test_request(QuorumConfig::any());
    let request_id = request_manager.create_request(request).await.unwrap();

    // Exercise: Submit approval response
    let response = create_approval_response("approver1@company.com", ApprovalDecision::Approve);
    let outcome = request_manager.submit_response(request_id, response).await.unwrap();

    // Verify
    assert_eq!(outcome.new_state, State::Approved);
    assert_eq!(outcome.quorum_met, true);

    // Verify state persisted in database
    let state = request_manager.get_request(request_id).await.unwrap();
    assert_eq!(state.state, State::Approved);
}

#[tokio::test]
async fn test_pending_to_denied_on_any_denial() {
    let mut request_manager = setup_request_manager().await;
    let request = create_test_request(QuorumConfig::all());  // Requires all approvals
    let request_id = request_manager.create_request(request).await.unwrap();

    // Submit approval from approver1
    let approval = create_approval_response("approver1@company.com", ApprovalDecision::Approve);
    request_manager.submit_response(request_id, approval).await.unwrap();

    // Submit denial from approver2 (should immediately deny)
    let denial = create_approval_response("approver2@company.com", ApprovalDecision::Deny);
    let outcome = request_manager.submit_response(request_id, denial).await.unwrap();

    assert_eq!(outcome.new_state, State::Denied);
}

#[tokio::test]
async fn test_duplicate_response_idempotent() {
    let mut request_manager = setup_request_manager().await;
    let request = create_test_request(QuorumConfig::threshold(2));
    let request_id = request_manager.create_request(request).await.unwrap();

    let response = create_approval_response("approver1@company.com", ApprovalDecision::Approve);

    // Submit response twice
    request_manager.submit_response(request_id, response.clone()).await.unwrap();
    let outcome = request_manager.submit_response(request_id, response).await.unwrap();

    // Second submission should be ignored (idempotent)
    assert_eq!(outcome, StateTransitionOutcome::Duplicate);

    // Verify only one response recorded
    let state = request_manager.get_request(request_id).await.unwrap();
    assert_eq!(state.responses.len(), 1);
}

#[tokio::test]
async fn test_invalid_transition_rejected() {
    let mut request_manager = setup_request_manager().await;
    let request = create_test_request(QuorumConfig::any());
    let request_id = request_manager.create_request(request).await.unwrap();

    // Approve request
    let approval = create_approval_response("approver1@company.com", ApprovalDecision::Approve);
    request_manager.submit_response(request_id, approval).await.unwrap();

    // Attempt to submit another response (invalid: already in terminal state)
    let response = create_approval_response("approver2@company.com", ApprovalDecision::Approve);
    let result = request_manager.submit_response(request_id, response).await;

    assert!(matches!(result, Err(Error::RequestAlreadyResolved)));
}
```

### 2.2 Quorum Calculation Tests

```rust
#[test]
fn test_quorum_any_met_with_single_approval() {
    let quorum = QuorumConfig::any();
    let responses = vec![
        create_approval("approver1", Approve),
    ];

    assert!(is_quorum_met(&quorum, &responses, 3));  // 1 of 3 approvers
}

#[test]
fn test_quorum_all_not_met_until_all_approve() {
    let quorum = QuorumConfig::all();
    let responses = vec![
        create_approval("approver1", Approve),
        create_approval("approver2", Approve),
    ];

    assert!(!is_quorum_met(&quorum, &responses, 3));  // 2 of 3 approvers

    let responses = vec![
        create_approval("approver1", Approve),
        create_approval("approver2", Approve),
        create_approval("approver3", Approve),
    ];

    assert!(is_quorum_met(&quorum, &responses, 3));  // 3 of 3 approvers
}

#[test]
fn test_quorum_threshold_2_of_3() {
    let quorum = QuorumConfig::threshold(2);
    let responses = vec![
        create_approval("approver1", Approve),
    ];

    assert!(!is_quorum_met(&quorum, &responses, 3));  // 1 of 3 approvers

    let responses = vec![
        create_approval("approver1", Approve),
        create_approval("approver2", Approve),
    ];

    assert!(is_quorum_met(&quorum, &responses, 3));  // 2 of 3 approvers (threshold met)
}

#[test]
fn test_quorum_ignores_denials() {
    let quorum = QuorumConfig::threshold(2);
    let responses = vec![
        create_approval("approver1", Approve),
        create_approval("approver2", Deny),
        create_approval("approver3", Approve),
    ];

    // Denial triggers immediate rejection (separate path)
    // Quorum calculation only counts approvals
    assert!(is_quorum_met(&quorum, &responses, 3));
}
```

### 2.3 Policy Matching Tests

```rust
#[tokio::test]
async fn test_policy_matches_agent_pattern() {
    let engine = setup_policy_engine().await;

    let request = OversightMatchRequest {
        agent_nhi: "agent:payment-bot-v3@company.creto",
        action: "TransferFunds",
        resource: json!({"amount": 50000}),
    };

    let requirement = engine.match_policy(&request).await.unwrap();
    assert!(requirement.is_some());
    assert_eq!(requirement.unwrap().escalation_chain.tiers.len(), 2);
}

#[tokio::test]
async fn test_policy_no_match_for_small_amount() {
    let engine = setup_policy_engine().await;

    let request = OversightMatchRequest {
        agent_nhi: "agent:payment-bot-v3@company.creto",
        action: "TransferFunds",
        resource: json!({"amount": 5000}),  // Below $10K threshold
    };

    let requirement = engine.match_policy(&request).await.unwrap();
    assert!(requirement.is_none());  // No oversight required
}

#[tokio::test]
async fn test_bloom_filter_fast_rejection() {
    let engine = setup_policy_engine().await;

    let request = OversightMatchRequest {
        agent_nhi: "agent:unknown-agent@company.creto",
        action: "UnknownAction",
        resource: json!({}),
    };

    // Bloom filter should reject without database lookup
    let start = Instant::now();
    let requirement = engine.match_policy(&request).await.unwrap();
    let elapsed = start.elapsed();

    assert!(requirement.is_none());
    assert!(elapsed < Duration::from_micros(100));  // <100µs (Bloom filter path)
}
```

### 2.4 Timeout Calculation Tests

```rust
#[test]
fn test_calculate_remaining_timeout() {
    let created_at = Timestamp::from_secs(1000);
    let current_time = Timestamp::from_secs(1500);
    let timeout_duration = Duration::from_secs(1000);

    let remaining = calculate_remaining_timeout(created_at, current_time, timeout_duration);

    assert_eq!(remaining, Duration::from_secs(500));  // 500 seconds remaining
}

#[test]
fn test_timeout_already_expired() {
    let created_at = Timestamp::from_secs(1000);
    let current_time = Timestamp::from_secs(2500);
    let timeout_duration = Duration::from_secs(1000);

    let remaining = calculate_remaining_timeout(created_at, current_time, timeout_duration);

    assert_eq!(remaining, Duration::ZERO);  // Timeout expired
}

#[tokio::test]
async fn test_timeout_triggers_escalation() {
    let mut request_manager = setup_request_manager().await;
    let request = create_test_request_with_escalation();
    let request_id = request_manager.create_request(request).await.unwrap();

    // Fast-forward time to trigger tier timeout
    let timeout_scheduler = request_manager.timeout_scheduler.clone();
    timeout_scheduler.fast_forward(Duration::from_secs(3600)).await;  // 1 hour

    // Verify state transitioned to ESCALATED
    let state = request_manager.get_request(request_id).await.unwrap();
    assert_eq!(state.state, State::Escalated);
    assert_eq!(state.tier_index, 1);  // Moved to next tier
}
```

---

## 3. Integration Testing

### 3.1 Database Integration Tests

**Test Coverage:**
- CRUD operations (create, read, update, delete)
- Transaction atomicity (rollback on error)
- Optimistic concurrency control (version conflicts)
- Foreign key constraints

**Example Test:**
```rust
#[sqlx::test]
async fn test_create_and_retrieve_request(pool: PgPool) {
    let state_store = PostgresStateStore::new(pool);

    let request = create_test_request_state();
    state_store.save_checkpoint(&request).await.unwrap();

    let loaded = state_store.load_checkpoint(request.request_id).await.unwrap();

    assert_eq!(loaded.request_id, request.request_id);
    assert_eq!(loaded.state, request.state);
    assert_eq!(loaded.agent_nhi, request.agent_nhi);
}

#[sqlx::test]
async fn test_optimistic_concurrency_conflict(pool: PgPool) {
    let state_store = PostgresStateStore::new(pool.clone());

    let request = create_test_request_state();
    state_store.save_checkpoint(&request).await.unwrap();

    // Load state (version 0)
    let mut state1 = state_store.load_checkpoint(request.request_id).await.unwrap();
    let mut state2 = state_store.load_checkpoint(request.request_id).await.unwrap();

    // Update state1 (version 0 → 1)
    state1.state = State::Approved;
    state1.version += 1;
    state_store.save_checkpoint_with_version(&state1).await.unwrap();

    // Attempt to update state2 (version 0 → 1, should fail)
    state2.state = State::Denied;
    state2.version += 1;
    let result = state_store.save_checkpoint_with_version(&state2).await;

    assert!(matches!(result, Err(Error::ConcurrentModification)));
}

#[sqlx::test]
async fn test_foreign_key_cascade_delete(pool: PgPool) {
    let state_store = PostgresStateStore::new(pool.clone());

    let request = create_test_request_state();
    state_store.save_checkpoint(&request).await.unwrap();

    // Add approval response
    let response = create_approval_response("approver1@company.com", ApprovalDecision::Approve);
    sqlx::query!(
        "INSERT INTO approval_responses (request_id, approver_subject, decision, signature_algorithm, signature_value)
         VALUES ($1, $2, $3, $4, $5)",
        request.request_id,
        response.approver.subject,
        response.decision.to_string(),
        response.signature.algorithm,
        response.signature.value,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Delete request (should cascade delete response)
    sqlx::query!("DELETE FROM oversight_requests WHERE request_id = $1", request.request_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify response also deleted
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM approval_responses WHERE request_id = $1", request.request_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(count, 0);
}
```

### 3.2 Channel Integration Tests

**Test Coverage:**
- Slack message delivery
- Email sending
- Webhook POST requests
- Signature verification
- Retry logic

**Example Test:**
```rust
#[tokio::test]
async fn test_slack_channel_sends_notification() {
    let mut slack_mock = MockSlackClient::new();

    slack_mock
        .expect_chat_post_message()
        .withf(|req| req.channel == "U123456" && req.blocks.is_some())
        .times(1)
        .returning(|_| Ok(ChatPostMessageResponse {
            ts: "1703520000.123456".to_string(),
            channel: "U123456".to_string(),
        }));

    let slack_channel = SlackChannel::new(slack_mock);

    let request = create_test_request();
    let context = create_test_context();
    let approver = create_test_approver("alice@company.com");

    let delivery = slack_channel.send_notification(&request, &context, &approver).await.unwrap();

    assert_eq!(delivery.channel_type, ChannelType::Slack);
    assert_eq!(delivery.status, DeliveryStatus::Delivered);
}

#[tokio::test]
async fn test_slack_channel_retries_on_failure() {
    let mut slack_mock = MockSlackClient::new();

    // First attempt fails
    slack_mock
        .expect_chat_post_message()
        .times(1)
        .returning(|_| Err(SlackError::RateLimited));

    // Second attempt succeeds
    slack_mock
        .expect_chat_post_message()
        .times(1)
        .returning(|_| Ok(ChatPostMessageResponse {
            ts: "1703520000.123456".to_string(),
            channel: "U123456".to_string(),
        }));

    let slack_channel = SlackChannel::new(slack_mock);

    let request = create_test_request();
    let context = create_test_context();
    let approver = create_test_approver("alice@company.com");

    // Should succeed after retry
    let delivery = slack_channel.send_notification(&request, &context, &approver).await.unwrap();

    assert_eq!(delivery.status, DeliveryStatus::Delivered);
}

#[tokio::test]
async fn test_email_channel_generates_secure_link() {
    let mut smtp_mock = MockSmtpClient::new();
    let email_channel = EmailChannel::new(smtp_mock, template_engine, link_generator);

    smtp_mock
        .expect_send()
        .withf(|msg| {
            msg.to == "cfo@company.com"
                && msg.html_body.contains("https://oversight.company.com/approve?token=")
        })
        .times(1)
        .returning(|_| Ok("message-id-123".to_string()));

    let request = create_test_request();
    let context = create_test_context();
    let approver = create_test_approver("cfo@company.com");

    let delivery = email_channel.send_notification(&request, &context, &approver).await.unwrap();

    assert_eq!(delivery.channel_type, ChannelType::Email);
}
```

### 3.3 Checkpoint Recovery Tests

```rust
#[tokio::test]
async fn test_recover_pending_requests_on_startup() {
    let pool = setup_test_database().await;
    let durability_manager = DurabilityManager::new(pool.clone());

    // Create pending requests
    let request1 = create_test_request_state_with_state(State::Pending);
    let request2 = create_test_request_state_with_state(State::Escalated);
    let request3 = create_test_request_state_with_state(State::Approved);  // Terminal state

    sqlx::query!("INSERT INTO oversight_requests (...) VALUES (...)", /* request1 */).execute(&pool).await.unwrap();
    sqlx::query!("INSERT INTO oversight_requests (...) VALUES (...)", /* request2 */).execute(&pool).await.unwrap();
    sqlx::query!("INSERT INTO oversight_requests (...) VALUES (...)", /* request3 */).execute(&pool).await.unwrap();

    // Simulate startup recovery
    durability_manager.recover_on_startup().await.unwrap();

    // Verify pending requests recovered (request1, request2)
    // Verify terminal requests not recovered (request3)
    let timeout_scheduler = durability_manager.timeout_scheduler.clone();
    let scheduled_tasks = timeout_scheduler.list_tasks().await.unwrap();

    assert_eq!(scheduled_tasks.len(), 2);  // Only pending and escalated
    assert!(scheduled_tasks.iter().any(|t| t.request_id == request1.request_id));
    assert!(scheduled_tasks.iter().any(|t| t.request_id == request2.request_id));
}

#[tokio::test]
async fn test_recalculate_remaining_timeout_on_recovery() {
    let pool = setup_test_database().await;
    let durability_manager = DurabilityManager::new(pool.clone());

    // Create request with 1-hour timeout, created 30 minutes ago
    let created_at = Timestamp::now() - Duration::from_secs(1800);  // 30 minutes ago
    let timeout_duration = Duration::from_secs(3600);  // 1 hour
    let request = create_test_request_with_timeout(created_at, timeout_duration);

    sqlx::query!("INSERT INTO oversight_requests (...) VALUES (...)", /* request */).execute(&pool).await.unwrap();

    // Simulate startup recovery
    durability_manager.recover_on_startup().await.unwrap();

    // Verify timeout rescheduled with remaining 30 minutes
    let timeout_scheduler = durability_manager.timeout_scheduler.clone();
    let task = timeout_scheduler.get_task(request.request_id).await.unwrap();

    let expected_expiration = created_at + timeout_duration;
    assert_eq!(task.expiration_time, expected_expiration);
}
```

---

## 4. End-to-End Testing

### 4.1 Complete Approval Flow Tests

**Test Coverage:**
- Request creation → notification → approval → state update → audit log
- Multi-tier escalation flow
- Quorum-based approval flow
- Timeout-triggered auto-deny

**Example Test:**
```rust
#[tokio::test]
async fn test_e2e_simple_approval_flow() {
    // Setup: Start all services (Oversight, mock Authorization, mock Slack)
    let test_env = setup_e2e_environment().await;

    // Step 1: Authorization service triggers oversight
    let request_id = test_env.authz_client.trigger_oversight(OversightTrigger {
        agent_nhi: "agent:payment-bot@company.creto",
        action: "TransferFunds",
        amount: 50000,
    }).await.unwrap();

    // Step 2: Verify Slack notification sent
    let slack_messages = test_env.slack_mock.get_sent_messages().await;
    assert_eq!(slack_messages.len(), 1);
    assert_eq!(slack_messages[0].channel, "U_CFO");

    // Step 3: Approver clicks "Approve" button in Slack
    test_env.slack_mock.simulate_button_click(
        "U_CFO",
        "approve",
        request_id.to_string(),
    ).await.unwrap();

    // Step 4: Verify state transitioned to APPROVED
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.state, State::Approved);

    // Step 5: Verify audit log recorded approval
    let audit_events = test_env.audit_client.query_events(AuditQuery {
        request_id: Some(request_id),
    }).await.unwrap();

    assert!(audit_events.iter().any(|e| e.event_type == "request.approved"));

    // Step 6: Verify override token issued to Authorization service
    let override_token = test_env.authz_client.get_override_token(request_id).await.unwrap();
    assert!(override_token.is_some());
}

#[tokio::test]
async fn test_e2e_escalation_chain() {
    let test_env = setup_e2e_environment().await;

    // Create request with escalation chain (Team Lead 1h → CFO 2h → auto-deny)
    let request_id = test_env.authz_client.trigger_oversight_with_escalation().await.unwrap();

    // Verify Team Lead notified
    let slack_messages = test_env.slack_mock.get_sent_messages().await;
    assert_eq!(slack_messages.last().unwrap().channel, "U_TEAM_LEAD");

    // Fast-forward time by 1 hour (Team Lead timeout)
    test_env.time_controller.advance(Duration::from_secs(3600)).await;

    // Verify state transitioned to ESCALATED
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.state, State::Escalated);
    assert_eq!(request.tier_index, 1);

    // Verify CFO notified
    let slack_messages = test_env.slack_mock.get_sent_messages().await;
    assert_eq!(slack_messages.last().unwrap().channel, "U_CFO");

    // CFO approves
    test_env.slack_mock.simulate_button_click("U_CFO", "approve", request_id.to_string()).await.unwrap();

    // Verify state transitioned to APPROVED
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.state, State::Approved);
}

#[tokio::test]
async fn test_e2e_quorum_2_of_3_approvers() {
    let test_env = setup_e2e_environment().await;

    // Create request requiring 2-of-3 approvals
    let request_id = test_env.authz_client.trigger_oversight_with_quorum(QuorumConfig::threshold(2)).await.unwrap();

    // Verify all 3 approvers notified
    let slack_messages = test_env.slack_mock.get_sent_messages().await;
    assert_eq!(slack_messages.len(), 3);

    // First approver approves
    test_env.slack_mock.simulate_button_click("U_APPROVER1", "approve", request_id.to_string()).await.unwrap();

    // Verify state still PENDING (quorum not met)
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.state, State::Pending);

    // Second approver approves (quorum met)
    test_env.slack_mock.simulate_button_click("U_APPROVER2", "approve", request_id.to_string()).await.unwrap();

    // Verify state transitioned to APPROVED
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.state, State::Approved);

    // Third approver's response should be redundant
    test_env.slack_mock.simulate_button_click("U_APPROVER3", "approve", request_id.to_string()).await.unwrap();

    // Verify only 2 responses recorded
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.responses.len(), 2);
}
```

### 4.2 Multi-Channel Integration Tests

```rust
#[tokio::test]
async fn test_e2e_slack_and_email_parallel_delivery() {
    let test_env = setup_e2e_environment().await;

    // Create request with Slack + Email channels
    let request_id = test_env.authz_client.trigger_oversight_with_channels(vec![
        ChannelType::Slack,
        ChannelType::Email,
    ]).await.unwrap();

    // Verify both channels delivered
    let slack_messages = test_env.slack_mock.get_sent_messages().await;
    let email_messages = test_env.smtp_mock.get_sent_messages().await;

    assert_eq!(slack_messages.len(), 1);
    assert_eq!(email_messages.len(), 1);

    // Approver responds via email (clicks link)
    let email = email_messages.last().unwrap();
    let approval_link = extract_approval_link(&email.html_body);
    test_env.http_client.get(approval_link).send().await.unwrap();

    // Verify state transitioned to APPROVED
    let request = test_env.oversight_client.get_request(request_id).await.unwrap();
    assert_eq!(request.state, State::Approved);
}
```

---

## 5. Performance Testing

### 5.1 Load Testing

**Tools:** k6, Gatling

**Scenarios:**

1. **Request Creation Load**
   - Ramp up to 1,000 requests/second
   - Measure p50, p95, p99 latency
   - Target: p99 <10ms

2. **Concurrent Response Submission**
   - 100 concurrent approvers responding
   - Measure quorum calculation latency
   - Target: p99 <1ms

3. **Notification Delivery Load**
   - 500 notifications/second across all channels
   - Measure delivery success rate
   - Target: >99% success

**Example k6 Script:**
```javascript
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '1m', target: 100 },   // Ramp up to 100 RPS
    { duration: '3m', target: 1000 },  // Ramp up to 1000 RPS
    { duration: '1m', target: 0 },     // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(99)<10'],  // 99% of requests <10ms
  },
};

export default function() {
  let payload = JSON.stringify({
    agent_nhi: 'agent:test-bot@company.creto',
    action: 'TransferFunds',
    resource: { amount: 50000 },
    policy_id: 'pol_large_transfer',
    requirement: { /* escalation chain */ },
  });

  let res = http.post('https://oversight.company.com/api/v1/requests', payload, {
    headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer test_key' },
  });

  check(res, {
    'status is 201': (r) => r.status === 201,
    'response time <10ms': (r) => r.timings.duration < 10,
  });
}
```

### 5.2 Stress Testing

**Goal:** Find breaking point

**Test:**
- Continuously increase load until system degrades
- Measure failure modes (timeouts, errors, crashes)
- Target: Graceful degradation (return 503, no crashes)

**Metrics:**
- Max throughput before degradation
- Error rate at max load
- Recovery time after load reduction

---

## 6. Security Testing

### 6.1 Signature Verification Tests

```rust
#[tokio::test]
async fn test_invalid_signature_rejected() {
    let response_handler = setup_response_handler().await;

    let request_id = create_test_request_id();
    let response = ApprovalResponse {
        approver: create_test_approver("alice@company.com"),
        decision: ApprovalDecision::Approve,
        signature: Signature {
            algorithm: "ML-DSA-65".to_string(),
            value: vec![0; 64],  // Invalid signature
        },
        // ... other fields
    };

    let result = response_handler.submit_response(request_id, response).await;

    assert!(matches!(result, Err(Error::InvalidSignature)));
}

#[tokio::test]
async fn test_signature_replay_prevented() {
    let response_handler = setup_response_handler().await;

    // Create two different requests
    let request_id1 = create_test_request_id();
    let request_id2 = create_test_request_id();

    // Sign approval for request1
    let signature1 = sign_approval(request_id1, ApprovalDecision::Approve).await;

    // Attempt to use signature1 for request2 (should fail - signature bound to request_id)
    let response = ApprovalResponse {
        approver: create_test_approver("alice@company.com"),
        decision: ApprovalDecision::Approve,
        signature: signature1,  // Wrong request_id in signature
        // ... other fields
    };

    let result = response_handler.submit_response(request_id2, response).await;

    assert!(matches!(result, Err(Error::InvalidSignature)));
}
```

### 6.2 Authorization Tests

```rust
#[tokio::test]
async fn test_unauthorized_approver_rejected() {
    let response_handler = setup_response_handler().await;
    let request = create_test_request_with_approvers(vec!["cfo@company.com"]);
    let request_id = response_handler.create_request(request).await.unwrap();

    // Attempt to approve as unauthorized user
    let response = create_signed_approval_response("attacker@company.com", ApprovalDecision::Approve);
    let result = response_handler.submit_response(request_id, response).await;

    assert!(matches!(result, Err(Error::ApproverNotAuthorized)));
}
```

---

## 7. Chaos Engineering

### 7.1 Failure Injection Tests

**Scenarios:**
1. **Database Connection Failure:** PostgreSQL becomes unreachable mid-request
2. **Redis Failure:** Cache unavailable (should fallback to PostgreSQL)
3. **Slack API Failure:** Notification delivery fails (should retry)
4. **Network Partition:** Service isolated from Authorization service

**Example Test:**
```rust
#[tokio::test]
async fn test_database_failure_during_state_transition() {
    let test_env = setup_chaos_environment().await;

    let request_id = test_env.oversight_client.create_request(/* ... */).await.unwrap();

    // Inject database failure
    test_env.chaos.inject_failure(FailureType::DatabaseUnavailable).await;

    // Attempt to submit response (should fail gracefully)
    let response = create_approval_response("alice@company.com", ApprovalDecision::Approve);
    let result = test_env.oversight_client.submit_response(request_id, response).await;

    assert!(matches!(result, Err(Error::DatabaseUnavailable)));

    // Clear failure
    test_env.chaos.clear_failures().await;

    // Retry should succeed
    let response = create_approval_response("alice@company.com", ApprovalDecision::Approve);
    let result = test_env.oversight_client.submit_response(request_id, response).await;

    assert!(result.is_ok());
}
```

---

## 8. Test Data Management

### 8.1 Test Fixtures

**Fixture Library:**
```rust
// Test request builders
pub fn create_test_request() -> OversightRequest { /* ... */ }
pub fn create_test_request_with_escalation() -> OversightRequest { /* ... */ }
pub fn create_test_request_with_quorum(quorum: QuorumConfig) -> OversightRequest { /* ... */ }

// Test approver identities
pub fn create_test_approver(subject: &str) -> ApproverIdentity { /* ... */ }
pub fn create_test_approver_with_key(subject: &str, public_key: Vec<u8>) -> ApproverIdentity { /* ... */ }

// Test responses
pub fn create_approval_response(approver: &str, decision: ApprovalDecision) -> ApprovalResponse { /* ... */ }
pub fn create_signed_approval_response(approver: &str, decision: ApprovalDecision) -> ApprovalResponse { /* ... */ }
```

### 8.2 Database Seeding

**Setup Test Database:**
```sql
-- Seed policies
INSERT INTO policies (policy_id, agent_pattern, action_pattern, approver_spec, escalation_chain_id, quorum_type)
VALUES ('pol_test_large_transfer', 'agent:payment-*', 'TransferFunds', '{"type": "individual", "value": ["cfo@company.com"]}', 'chain_test_cfo', 'ANY');

-- Seed escalation chains
INSERT INTO escalation_chains (chain_id, name, final_action)
VALUES ('chain_test_cfo', 'Test CFO Chain', 'AUTO_DENY');

INSERT INTO escalation_tiers (chain_id, tier_index, approvers, timeout_seconds, channels, quorum_type)
VALUES ('chain_test_cfo', 0, '["cfo@company.com"]', 3600, '["SLACK"]', 'ANY');
```

---

## 9. CI/CD Integration

### 9.1 GitHub Actions Workflow

```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run unit tests
        run: cargo test --lib --all-features

      - name: Run integration tests
        run: cargo test --test '*' --all-features
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/oversight_test
          REDIS_URL: redis://localhost:6379

      - name: Generate coverage report
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --all-features

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3

      - name: Run clippy (linting)
        run: cargo clippy -- -D warnings
```

### 9.2 Coverage Enforcement

**Minimum Coverage Thresholds:**
- Overall: 90%
- Critical paths (state machine, signature verification): 100%
- New code: 95%

**Coverage Report:**
```bash
cargo tarpaulin --out Html --output-dir coverage/
```

---

**END OF DOCUMENT**
