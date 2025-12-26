---
status: approved
author: Claude (AI Assistant)
created: 2025-12-26
updated: 2025-12-26
reviewers: []
---

# Edge Cases and Failure Modes

## Executive Summary

This document catalogs edge cases, failure modes, and recovery procedures across all Enablement Layer products. Each section provides detection mechanisms, mitigation strategies, and testing approaches for production resilience.

---

## 1. Runtime Edge Cases

### 1.1 Sandbox Resource Exhaustion

#### 1.1.1 Out-of-Memory (OOM) in Sandbox

**Scenario**: Agent code allocates memory beyond sandbox limits.

**Detection**:
```rust
// Memory pressure monitor in sandbox controller
pub struct MemoryMonitor {
    threshold_warning: f64,   // 0.80 = 80%
    threshold_critical: f64,  // 0.95 = 95%
    poll_interval: Duration,
}

impl MemoryMonitor {
    pub async fn check(&self, sandbox_id: &SandboxId) -> MemoryStatus {
        let usage = self.get_cgroup_memory_usage(sandbox_id).await?;
        let limit = self.get_cgroup_memory_limit(sandbox_id).await?;
        let ratio = usage as f64 / limit as f64;

        match ratio {
            r if r >= self.threshold_critical => MemoryStatus::Critical { usage, limit },
            r if r >= self.threshold_warning => MemoryStatus::Warning { usage, limit },
            _ => MemoryStatus::Healthy { usage, limit },
        }
    }
}
```

**Mitigation**:
1. **Graceful Termination**: Send SIGTERM with 5s grace period before SIGKILL
2. **Checkpoint**: Attempt state snapshot before termination
3. **Quota Increment**: For long-running agents, offer quota upgrade via Oversight

**Recovery**:
```rust
pub async fn handle_oom(sandbox_id: SandboxId, agent_nhi: AgentIdentity) -> Result<OomRecovery> {
    // 1. Attempt graceful checkpoint
    let checkpoint = match timeout(Duration::from_secs(5), create_checkpoint(&sandbox_id)).await {
        Ok(Ok(cp)) => Some(cp),
        _ => None,
    };

    // 2. Force terminate sandbox
    force_terminate(&sandbox_id).await?;

    // 3. Create Oversight request for quota upgrade
    let oversight_request = OversightRequest {
        agent_nhi: agent_nhi.clone(),
        action: Action::ResourceUpgrade {
            resource: "memory",
            current: "512MB",
            requested: "1GB",
        },
        context: format!("OOM in sandbox {}", sandbox_id),
        checkpoint: checkpoint.map(|c| c.id),
    };

    // 4. Return recovery state
    Ok(OomRecovery {
        sandbox_id,
        checkpoint,
        oversight_request_id: submit_oversight(oversight_request).await?,
    })
}
```

**Testing Strategy**:
```rust
#[tokio::test]
async fn test_oom_graceful_handling() {
    let sandbox = create_sandbox(SandboxConfig {
        memory_limit: 64 * 1024 * 1024, // 64MB
        ..default()
    }).await;

    // Execute memory bomb
    let result = sandbox.execute(r#"
        let arrays = [];
        while (true) {
            arrays.push(new Array(1024 * 1024).fill('x'));
        }
    "#).await;

    assert!(matches!(result, Err(SandboxError::MemoryExhausted { .. })));
    assert!(sandbox.checkpoint_exists().await);
    assert!(sandbox.oversight_request_pending().await);
}
```

#### 1.1.2 GPU Memory Exhaustion

**Scenario**: GPU-enabled sandbox runs out of VRAM.

**Detection**:
```rust
pub async fn monitor_gpu_memory(sandbox_id: &SandboxId) -> GpuMemoryStatus {
    let nvidia_smi = Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
        .output()
        .await?;

    let output = String::from_utf8_lossy(&nvidia_smi.stdout);
    let parts: Vec<&str> = output.trim().split(',').collect();
    let used: u64 = parts[0].trim().parse()?;
    let total: u64 = parts[1].trim().parse()?;

    GpuMemoryStatus { used_mb: used, total_mb: total }
}
```

**Mitigation**:
1. Model offloading to CPU RAM
2. Gradient checkpointing for training workloads
3. Batch size reduction with retry

#### 1.1.3 Disk Space Exhaustion

**Scenario**: Sandbox fills allocated ephemeral storage.

**Detection**: Monitor overlay filesystem usage via `statfs`.

**Mitigation**:
```rust
pub async fn handle_disk_full(sandbox_id: &SandboxId) -> Result<()> {
    // 1. Identify large temp files
    let large_files = find_large_files(sandbox_id, 10 * 1024 * 1024).await?;

    // 2. Offer cleanup options via Oversight
    if !large_files.is_empty() {
        let request = OversightRequest {
            action: Action::ApproveCleanup {
                files: large_files,
                reclaim_estimate: calculate_reclaim(&large_files),
            },
            ..default()
        };
        submit_oversight(request).await?;
    }

    // 3. If critical, move to degraded mode
    set_sandbox_mode(sandbox_id, SandboxMode::ReadOnly).await?;

    Ok(())
}
```

### 1.2 Network Edge Cases

#### 1.2.1 Network Partition During Execution

**Scenario**: Network connectivity lost mid-execution.

**Detection**:
```rust
pub struct NetworkHealthChecker {
    egress_endpoints: Vec<String>,
    check_interval: Duration,
    failure_threshold: u32,
}

impl NetworkHealthChecker {
    pub async fn check(&self) -> NetworkHealth {
        let mut failures = 0;
        for endpoint in &self.egress_endpoints {
            if let Err(_) = timeout(Duration::from_secs(5), tcp_connect(endpoint)).await {
                failures += 1;
            }
        }

        if failures >= self.failure_threshold {
            NetworkHealth::Partitioned { failed_endpoints: failures }
        } else {
            NetworkHealth::Healthy
        }
    }
}
```

**Mitigation**:
1. **Request Queuing**: Buffer outbound requests for replay
2. **Checkpoint**: Save state for recovery
3. **Exponential Backoff**: Retry with jitter

**Recovery**:
```rust
pub async fn handle_network_partition(sandbox_id: &SandboxId) -> Result<()> {
    // 1. Enable request buffering
    enable_request_buffer(sandbox_id, BufferConfig {
        max_requests: 1000,
        max_age: Duration::from_secs(300),
    }).await?;

    // 2. Create checkpoint
    create_checkpoint(sandbox_id).await?;

    // 3. Notify agent of degraded mode
    send_sandbox_event(sandbox_id, SandboxEvent::DegradedMode {
        reason: "network_partition",
        capabilities_affected: vec!["egress"],
    }).await?;

    // 4. Start reconnection loop
    spawn(reconnection_loop(sandbox_id.clone()));

    Ok(())
}
```

#### 1.2.2 DNS Resolution Failure

**Scenario**: DNS servers unreachable or returning NXDOMAIN for valid domains.

**Mitigation**:
1. Multi-resolver fallback (primary → secondary → DoH)
2. Positive cache with extended TTL during failures
3. IP address fallback for critical services

#### 1.2.3 TLS Certificate Expiry

**Scenario**: Sandbox attempts to connect to service with expired certificate.

**Detection**: Custom TLS verifier with expiry pre-check.

**Mitigation**:
```rust
pub struct TlsVerifierWithExpiry {
    warning_days: i64,
    block_expired: bool,
}

impl TlsVerifierWithExpiry {
    pub fn verify(&self, cert: &X509Certificate) -> TlsVerifyResult {
        let now = SystemTime::now();
        let not_after = cert.not_after();

        if now > not_after {
            if self.block_expired {
                return TlsVerifyResult::Reject(TlsError::CertificateExpired);
            }
            return TlsVerifyResult::WarnAndProceed(TlsWarning::Expired);
        }

        let days_until_expiry = (not_after - now).num_days();
        if days_until_expiry < self.warning_days {
            return TlsVerifyResult::WarnAndProceed(TlsWarning::ExpiringSoon {
                days: days_until_expiry,
            });
        }

        TlsVerifyResult::Ok
    }
}
```

### 1.3 Warm Pool Edge Cases

#### 1.3.1 Pool Exhaustion

**Scenario**: All warm sandboxes claimed, cold start required.

**Detection**: Pool size falls below minimum threshold.

**Mitigation**:
1. **Priority Queue**: High-priority agents get preference
2. **Preemption**: Lower-priority sandboxes can be reclaimed
3. **Elastic Scaling**: Trigger additional pool expansion

```rust
pub async fn handle_pool_exhaustion(
    pool_id: &PoolId,
    claim: &SandboxClaim,
) -> Result<SandboxAllocation> {
    // 1. Check if preemption is allowed
    if claim.priority >= Priority::High {
        if let Some(victim) = find_preemption_candidate(pool_id, claim.priority).await? {
            // Checkpoint and reclaim
            checkpoint_and_reclaim(&victim).await?;
            return allocate_reclaimed(pool_id, claim).await;
        }
    }

    // 2. Check queue position
    let position = get_queue_position(pool_id, claim).await?;
    if position <= 3 {
        // Wait for next available
        return wait_for_available(pool_id, claim, Duration::from_secs(30)).await;
    }

    // 3. Fall back to cold start with SLA warning
    emit_sla_warning(pool_id, "cold_start_fallback");
    cold_start_sandbox(claim).await
}
```

#### 1.3.2 Warm Sandbox Stale State

**Scenario**: Warm sandbox has stale environment variables or dependencies.

**Detection**:
```rust
pub struct SandboxFreshnessChecker {
    max_age: Duration,
    config_version: String,
}

impl SandboxFreshnessChecker {
    pub fn check(&self, sandbox: &WarmSandbox) -> Freshness {
        if sandbox.config_version != self.config_version {
            return Freshness::ConfigMismatch;
        }

        if sandbox.created_at.elapsed() > self.max_age {
            return Freshness::TooOld { age: sandbox.created_at.elapsed() };
        }

        Freshness::Fresh
    }
}
```

**Mitigation**: Rolling refresh of warm pool based on configuration changes.

---

## 2. Metering Edge Cases

### 2.1 Event Processing Edge Cases

#### 2.1.1 Clock Skew Between Services

**Scenario**: Event timestamps from different services disagree.

**Detection**:
```rust
pub fn detect_clock_skew(
    event_timestamp: DateTime<Utc>,
    received_timestamp: DateTime<Utc>,
) -> ClockSkewResult {
    let drift = received_timestamp - event_timestamp;

    if drift < Duration::seconds(-60) {
        ClockSkewResult::FutureEvent { drift }
    } else if drift > Duration::seconds(300) {
        ClockSkewResult::StaleEvent { drift }
    } else {
        ClockSkewResult::Acceptable { drift }
    }
}
```

**Mitigation**:
1. **Consensus Timestamps**: Use platform consensus for authoritative time
2. **Grace Window**: Accept events within ±5 minute window
3. **Clock Sync Alerts**: Trigger ops alert for persistent drift

#### 2.1.2 Duplicate Events (Idempotency Violation)

**Scenario**: Same event submitted multiple times.

**Detection**:
```rust
pub async fn check_duplicate(
    event: &BillableEvent,
    dedup_store: &RedisPool,
) -> DuplicateStatus {
    let key = format!("dedup:{}:{}", event.subscription_id, event.transaction_id);

    match dedup_store.get(&key).await {
        Ok(Some(existing)) => DuplicateStatus::Duplicate {
            original_received: existing.timestamp,
        },
        Ok(None) => DuplicateStatus::New,
        Err(e) => DuplicateStatus::Unknown { error: e },
    }
}
```

**Mitigation**:
1. **Bloom Filter**: Fast probabilistic pre-check (10µs)
2. **Redis SET NX**: Atomic deduplication with TTL
3. **Idempotency Token**: Client-provided unique identifier

#### 2.1.3 Late-Arriving Events

**Scenario**: Events arrive after billing period has closed.

**Handling**:
```rust
pub async fn handle_late_event(
    event: BillableEvent,
    period_end: DateTime<Utc>,
) -> Result<LateEventResolution> {
    let lateness = Utc::now() - period_end;

    if lateness <= Duration::hours(24) {
        // Grace period - include in final invoice
        Ok(LateEventResolution::IncludeInPeriod)
    } else if lateness <= Duration::days(7) {
        // Create adjustment on next invoice
        Ok(LateEventResolution::CreateAdjustment {
            original_period: period_end,
        })
    } else {
        // Require manual review
        Ok(LateEventResolution::RequireReview {
            reason: "Event too old for automatic processing",
        })
    }
}
```

### 2.2 Quota Enforcement Edge Cases

#### 2.2.1 Race Condition: Concurrent Quota Checks

**Scenario**: Two agents check quota simultaneously, both see available, both exceed.

**Mitigation**: Atomic increment with limit check.

```rust
pub async fn atomic_quota_check_and_decrement(
    quota_key: &str,
    amount: i64,
    redis: &RedisPool,
) -> Result<QuotaResult> {
    let script = r#"
        local current = redis.call('GET', KEYS[1])
        if current == false then
            return {err = 'QUOTA_NOT_FOUND'}
        end
        local remaining = tonumber(current)
        if remaining < tonumber(ARGV[1]) then
            return {remaining = remaining, allowed = false}
        end
        local new_remaining = redis.call('DECRBY', KEYS[1], ARGV[1])
        return {remaining = new_remaining, allowed = true}
    "#;

    redis.eval(script, &[quota_key], &[&amount.to_string()]).await
}
```

#### 2.2.2 Quota Inheritance Conflicts

**Scenario**: Organization quota < sum of team quotas.

**Detection**:
```rust
pub fn validate_quota_hierarchy(org_id: &OrgId) -> Vec<QuotaConflict> {
    let mut conflicts = Vec::new();
    let org_quota = get_org_quota(org_id);

    let team_sum: i64 = get_teams(org_id)
        .iter()
        .map(|t| t.quota_limit)
        .sum();

    if team_sum > org_quota.limit {
        conflicts.push(QuotaConflict::SumExceedsParent {
            parent: org_id.clone(),
            parent_limit: org_quota.limit,
            children_sum: team_sum,
        });
    }

    conflicts
}
```

**Mitigation**: Enforce "pool" mode where children share parent quota vs "reserved" mode where children have guaranteed allocation.

#### 2.2.3 Prepaid Credits Expiry Race

**Scenario**: Credits expire while transaction is in flight.

**Handling**:
```rust
pub async fn use_credits_with_expiry_check(
    wallet_id: &WalletId,
    amount: Decimal,
) -> Result<CreditUseResult> {
    // 1. Find non-expired credits
    let available_credits = get_credits(wallet_id)
        .filter(|c| c.expires_at > Utc::now())
        .sorted_by(|a, b| a.expires_at.cmp(&b.expires_at)) // FIFO by expiry
        .collect::<Vec<_>>();

    let total_available: Decimal = available_credits.iter().map(|c| c.remaining).sum();

    if total_available < amount {
        return Err(CreditError::InsufficientCredits {
            requested: amount,
            available: total_available,
        });
    }

    // 2. Deduct from earliest-expiring first
    let mut remaining = amount;
    for credit in available_credits {
        if remaining <= Decimal::ZERO {
            break;
        }
        let deduct = std::cmp::min(credit.remaining, remaining);
        deduct_credit(&credit.id, deduct).await?;
        remaining -= deduct;
    }

    Ok(CreditUseResult::Success { amount })
}
```

### 2.3 Billing Edge Cases

#### 2.3.1 Mid-Period Plan Change

**Scenario**: Customer upgrades/downgrades mid-billing period.

**Handling**:
```rust
pub enum PlanChangeStrategy {
    /// Pro-rate old plan, pro-rate new plan
    ProRate,
    /// Complete old period at old rate, start new period
    NextPeriod,
    /// Credit remaining old plan, charge full new plan
    ImmediateSwitch,
}

pub fn calculate_plan_change_charges(
    old_plan: &Plan,
    new_plan: &Plan,
    change_date: DateTime<Utc>,
    period_end: DateTime<Utc>,
    strategy: PlanChangeStrategy,
) -> PlanChangeCharges {
    let days_remaining = (period_end - change_date).num_days() as f64;
    let period_days = 30.0; // Simplified

    match strategy {
        PlanChangeStrategy::ProRate => {
            let old_refund = old_plan.monthly_fee * Decimal::from_f64(days_remaining / period_days).unwrap();
            let new_charge = new_plan.monthly_fee * Decimal::from_f64(days_remaining / period_days).unwrap();
            PlanChangeCharges { refund: old_refund, charge: new_charge }
        }
        // ... other strategies
    }
}
```

#### 2.3.2 Payment Failure After Usage

**Scenario**: Usage recorded but payment fails.

**Handling**:
1. **Grace Period**: 7-day retry window
2. **Degraded Mode**: Reduce quota to 10% of normal
3. **Notification Escalation**: Day 1 email, Day 3 in-app, Day 7 service restriction

---

## 3. Oversight Edge Cases

### 3.1 Approval Flow Edge Cases

#### 3.1.1 Approver Unavailable

**Scenario**: All designated approvers are unreachable.

**Escalation Chain**:
```rust
pub async fn handle_approver_unavailable(
    request: &OversightRequest,
    escalation_config: &EscalationConfig,
) -> Result<EscalationResult> {
    for tier in &escalation_config.tiers {
        let approvers = get_approvers(&tier.group_id).await?;

        for approver in approvers {
            let reachable = check_reachability(&approver, &tier.channels).await;
            if reachable {
                return Ok(EscalationResult::Escalated {
                    to: approver,
                    tier: tier.level,
                });
            }
        }

        // Wait before next tier
        if tier.wait_before_escalate > Duration::ZERO {
            tokio::time::sleep(tier.wait_before_escalate).await;
        }
    }

    // All tiers exhausted
    Ok(EscalationResult::AllTiersExhausted {
        action: escalation_config.final_action.clone(),
    })
}
```

**Final Actions**:
- `AutoApprove`: Proceed with audit trail
- `AutoDeny`: Fail safely with explanation
- `Queue`: Hold indefinitely for manual processing
- `AlertOps`: Page on-call SRE

#### 3.1.2 Approval Timeout During Checkpoint

**Scenario**: Agent checkpointed awaiting approval, timeout reached.

**Handling**:
```rust
pub async fn handle_checkpoint_timeout(
    checkpoint_id: &CheckpointId,
    request_id: &RequestId,
) -> Result<TimeoutResolution> {
    let checkpoint = get_checkpoint(checkpoint_id).await?;
    let request = get_request(request_id).await?;

    // 1. Check if request has default timeout action
    if let Some(default) = request.on_timeout {
        return execute_timeout_action(default, checkpoint).await;
    }

    // 2. Check policy for timeout handling
    let policy = get_timeout_policy(&request.agent_nhi).await?;

    match policy.action {
        TimeoutAction::ExtendAndNotify { extension } => {
            extend_timeout(request_id, extension).await?;
            notify_escalation_chain(request_id).await?;
            Ok(TimeoutResolution::Extended { new_deadline: Utc::now() + extension })
        }
        TimeoutAction::DenyAndResume => {
            deny_request(request_id, "Timeout - auto-denied").await?;
            resume_with_denial(checkpoint_id).await?;
            Ok(TimeoutResolution::Denied)
        }
        TimeoutAction::TerminateAgent => {
            terminate_agent_session(&checkpoint.session_id).await?;
            Ok(TimeoutResolution::Terminated)
        }
    }
}
```

#### 3.1.3 Conflicting Approvals

**Scenario**: Two approvers respond simultaneously with different decisions.

**Resolution**:
```rust
pub fn resolve_conflicting_approvals(
    approvals: Vec<ApprovalResponse>,
    strategy: ConflictStrategy,
) -> Decision {
    match strategy {
        ConflictStrategy::FirstWins => {
            approvals.into_iter()
                .min_by_key(|a| a.timestamp)
                .map(|a| a.decision)
                .unwrap_or(Decision::Deny)
        }
        ConflictStrategy::AnyApprove => {
            if approvals.iter().any(|a| a.decision == Decision::Approve) {
                Decision::Approve
            } else {
                Decision::Deny
            }
        }
        ConflictStrategy::AllApprove => {
            if approvals.iter().all(|a| a.decision == Decision::Approve) {
                Decision::Approve
            } else {
                Decision::Deny
            }
        }
        ConflictStrategy::HighestAuthority => {
            approvals.into_iter()
                .max_by_key(|a| a.approver_authority_level)
                .map(|a| a.decision)
                .unwrap_or(Decision::Deny)
        }
    }
}
```

### 3.2 Channel Edge Cases

#### 3.2.1 Slack Rate Limiting

**Scenario**: Too many Slack notifications hit rate limits.

**Mitigation**:
```rust
pub struct SlackRateLimiter {
    tokens: AtomicU32,
    max_tokens: u32,
    refill_rate: Duration, // 1 token per second typical
    last_refill: AtomicU64,
}

impl SlackRateLimiter {
    pub async fn acquire(&self) -> Result<RateLimitResult> {
        self.refill();

        let current = self.tokens.load(Ordering::SeqCst);
        if current == 0 {
            return Ok(RateLimitResult::Throttled {
                retry_after: self.refill_rate,
            });
        }

        self.tokens.fetch_sub(1, Ordering::SeqCst);
        Ok(RateLimitResult::Acquired)
    }
}
```

**Fallback**: Queue messages and batch-send, or fall back to email channel.

#### 3.2.2 Email Delivery Failure

**Scenario**: SMTP server rejects or bounces approval email.

**Handling**:
1. **Immediate Retry**: 3 attempts with exponential backoff
2. **Alternative Address**: Try secondary email if configured
3. **Channel Fallback**: Escalate to Slack or webhook
4. **Bounce Tracking**: Mark email as invalid after 3 hard bounces

#### 3.2.3 Webhook Endpoint Down

**Scenario**: Customer's webhook endpoint returns 5xx or times out.

**Circuit Breaker**:
```rust
pub struct WebhookCircuitBreaker {
    state: AtomicU8, // 0=closed, 1=open, 2=half-open
    failure_count: AtomicU32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure: AtomicU64,
}

impl WebhookCircuitBreaker {
    pub fn can_send(&self) -> bool {
        match self.state.load(Ordering::SeqCst) {
            0 => true, // Closed - allow
            1 => {
                // Open - check if recovery timeout passed
                let last = self.last_failure.load(Ordering::SeqCst);
                let elapsed = Duration::from_millis(now_millis() - last);
                if elapsed > self.recovery_timeout {
                    self.state.store(2, Ordering::SeqCst); // Half-open
                    true
                } else {
                    false
                }
            }
            2 => true, // Half-open - allow one attempt
            _ => false,
        }
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.failure_threshold {
            self.state.store(1, Ordering::SeqCst); // Open
            self.last_failure.store(now_millis(), Ordering::SeqCst);
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.state.store(0, Ordering::SeqCst); // Closed
    }
}
```

---

## 4. Messaging Edge Cases

### 4.1 Key Management Edge Cases

#### 4.1.1 PreKey Exhaustion

**Scenario**: All one-time prekeys consumed, only signed prekey remains.

**Detection**:
```rust
pub async fn check_prekey_inventory(agent_nhi: &AgentIdentity) -> PrekeyStatus {
    let bundle = get_prekey_bundle(agent_nhi).await?;

    match bundle.one_time_prekeys.len() {
        0 => PrekeyStatus::Exhausted,
        n if n < 10 => PrekeyStatus::Low { remaining: n },
        n => PrekeyStatus::Healthy { remaining: n },
    }
}
```

**Mitigation**:
1. **Signed Prekey Fallback**: X3DH still works with just signed prekey (reduced forward secrecy)
2. **Auto-Replenish**: Background task generates new prekeys when count < threshold
3. **Priority Messaging**: High-priority messages get reserved prekeys

#### 4.1.2 Ratchet State Desynchronization

**Scenario**: Message counter mismatch between sender and receiver.

**Detection**:
```rust
pub fn detect_ratchet_desync(
    received_header: &MessageHeader,
    our_state: &RatchetState,
) -> RatchetSyncStatus {
    // Check if message is from future (we missed messages)
    if received_header.message_number > our_state.receiving_message_number + MAX_SKIP {
        return RatchetSyncStatus::TooFarAhead {
            expected: our_state.receiving_message_number,
            received: received_header.message_number,
        };
    }

    // Check if message is from past (already processed or skipped)
    if received_header.message_number < our_state.receiving_message_number {
        if our_state.skipped_keys.contains_key(&(
            received_header.ratchet_key.clone(),
            received_header.message_number,
        )) {
            return RatchetSyncStatus::SkippedMessage;
        }
        return RatchetSyncStatus::AlreadyProcessed;
    }

    RatchetSyncStatus::InSync
}
```

**Recovery**:
1. **Skipped Message Keys**: Store keys for out-of-order messages (up to MAX_SKIP)
2. **Session Reset Request**: Send special message requesting new X3DH handshake
3. **Audit Trail**: Log desync event for forensic analysis

#### 4.1.3 Key Rotation During Active Session

**Scenario**: Agent's long-term key rotates while sessions are active.

**Handling**:
```rust
pub async fn handle_key_rotation(
    agent_nhi: &AgentIdentity,
    new_key: &SigningPublicKey,
    active_sessions: Vec<SessionId>,
) -> Result<KeyRotationResult> {
    // 1. Generate new signed prekey with new identity key
    let new_signed_prekey = generate_signed_prekey(new_key).await?;

    // 2. Notify active session peers of key change
    for session_id in &active_sessions {
        let peer = get_session_peer(session_id).await?;
        send_key_rotation_notification(peer, KeyRotationNotice {
            agent_nhi: agent_nhi.clone(),
            new_identity_key: new_key.clone(),
            new_signed_prekey: new_signed_prekey.clone(),
            effective_from: Utc::now(),
        }).await?;
    }

    // 3. Mark old sessions for graceful migration
    for session_id in &active_sessions {
        mark_session_for_migration(session_id, new_key).await?;
    }

    Ok(KeyRotationResult::NotificationsSent {
        session_count: active_sessions.len(),
    })
}
```

### 4.2 Message Delivery Edge Cases

#### 4.2.1 Recipient Offline (Store-and-Forward)

**Scenario**: Recipient agent not connected, message must be queued.

**Handling**:
```rust
pub async fn deliver_message(
    envelope: EncryptedEnvelope,
    recipient: &AgentIdentity,
) -> DeliveryResult {
    // 1. Check if recipient is online
    if let Some(connection) = get_active_connection(recipient).await {
        match connection.send(&envelope).await {
            Ok(_) => return DeliveryResult::Delivered,
            Err(e) => log::warn!("Direct delivery failed: {}", e),
        }
    }

    // 2. Queue for later delivery
    let queue_result = message_queue.push(QueuedMessage {
        envelope,
        recipient: recipient.clone(),
        queued_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(30),
        retry_count: 0,
    }).await?;

    DeliveryResult::Queued {
        queue_id: queue_result.id,
        expires_at: queue_result.expires_at,
    }
}
```

#### 4.2.2 Message Expiry Before Delivery

**Scenario**: Queued message expires before recipient comes online.

**Handling**:
1. **Expiry Notification**: Notify sender that message expired
2. **Audit Log**: Record expiry event (not message content)
3. **Sender Retry Option**: Allow sender to re-send with fresh encryption

#### 4.2.3 Large Message Fragmentation

**Scenario**: Message exceeds single-packet size limit.

**Handling**:
```rust
pub const MAX_FRAGMENT_SIZE: usize = 64 * 1024; // 64KB

pub fn fragment_message(
    message: &PlaintextMessage,
) -> Vec<MessageFragment> {
    let serialized = message.serialize();
    let total_fragments = (serialized.len() + MAX_FRAGMENT_SIZE - 1) / MAX_FRAGMENT_SIZE;
    let fragment_id = Uuid::new_v4();

    serialized
        .chunks(MAX_FRAGMENT_SIZE)
        .enumerate()
        .map(|(index, chunk)| MessageFragment {
            fragment_id,
            index: index as u32,
            total: total_fragments as u32,
            data: chunk.to_vec(),
        })
        .collect()
}

pub fn reassemble_fragments(
    fragments: Vec<MessageFragment>,
) -> Result<Vec<u8>> {
    // Verify all fragments present
    let expected: HashSet<u32> = (0..fragments[0].total).collect();
    let received: HashSet<u32> = fragments.iter().map(|f| f.index).collect();

    if expected != received {
        return Err(FragmentError::MissingFragments {
            expected: expected.difference(&received).cloned().collect(),
        });
    }

    // Reassemble in order
    let mut sorted = fragments;
    sorted.sort_by_key(|f| f.index);

    Ok(sorted.into_iter().flat_map(|f| f.data).collect())
}
```

### 4.3 Group Messaging Edge Cases

#### 4.3.1 Sender Key Distribution Race

**Scenario**: Multiple agents join group simultaneously, sender key distribution overlaps.

**Handling**:
```rust
pub async fn handle_group_join_race(
    group_id: &GroupId,
    joining_agents: Vec<AgentIdentity>,
) -> Result<()> {
    // Use distributed lock to serialize joins
    let lock = acquire_group_lock(group_id, Duration::from_secs(30)).await?;

    // Get current group membership
    let current_members = get_group_members(group_id).await?;

    // For each joining agent, distribute sender keys from all existing members
    for new_member in &joining_agents {
        for existing_member in &current_members {
            let sender_key = get_sender_key(existing_member, group_id).await?;
            distribute_sender_key(sender_key, new_member).await?;
        }
    }

    // Add new members to group
    add_members(group_id, &joining_agents).await?;

    // Generate and distribute new members' sender keys
    for new_member in &joining_agents {
        let sender_key = generate_sender_key(new_member, group_id).await?;
        for recipient in current_members.iter().chain(joining_agents.iter()) {
            if recipient != new_member {
                distribute_sender_key(sender_key.clone(), recipient).await?;
            }
        }
    }

    drop(lock);
    Ok(())
}
```

#### 4.3.2 Member Removal Key Update

**Scenario**: Member removed from group, sender keys must be invalidated.

**Handling**: All remaining members rotate their sender keys (post-compromise security).

---

## 5. Cross-Product Failure Modes

### 5.1 Cascade Failures

#### 5.1.1 AuthZ Service Degradation

**Impact**: All products depend on 168ns AuthZ path.

**Degradation Strategy**:
```rust
pub enum AuthZDegradationMode {
    /// Use cached decisions for repeat requests
    CacheOnly { max_age: Duration },
    /// Fail open for read operations, fail closed for writes
    FailOpenRead,
    /// Queue all requests for later evaluation
    QueueAll { max_queue_size: usize },
    /// Reject all requests (safest)
    FailClosed,
}

pub async fn get_authz_decision_degraded(
    request: &AuthZRequest,
    mode: AuthZDegradationMode,
) -> Result<Decision> {
    // 1. Try cache first
    if let Some(cached) = authz_cache.get(&request.cache_key()).await {
        if cached.age() < Duration::from_secs(300) {
            return Ok(cached.decision);
        }
    }

    // 2. Try live service with short timeout
    match timeout(Duration::from_millis(50), authz_service.evaluate(request)).await {
        Ok(Ok(decision)) => {
            authz_cache.set(&request.cache_key(), decision.clone()).await;
            return Ok(decision);
        }
        _ => { /* Service unavailable or timeout */ }
    }

    // 3. Apply degradation mode
    match mode {
        AuthZDegradationMode::CacheOnly { max_age } => {
            authz_cache.get_stale(&request.cache_key(), max_age)
                .ok_or(AuthZError::NoCache)
        }
        AuthZDegradationMode::FailOpenRead if request.is_read() => {
            Ok(Decision::Allow)
        }
        AuthZDegradationMode::FailClosed | _ => {
            Ok(Decision::Deny)
        }
    }
}
```

#### 5.1.2 Memory Service Outage

**Impact**: Oversight loses agent context, Messaging loses session state.

**Mitigation**:
1. **Local State Cache**: Each service maintains recent state in-memory
2. **WAL Recovery**: Write-ahead log enables state reconstruction
3. **Graceful Degradation**: Proceed without context, flag as incomplete

### 5.2 Data Consistency Failures

#### 5.2.1 Metering-Billing Sync Failure

**Scenario**: Events processed but not reflected in invoice.

**Detection**:
```rust
pub async fn audit_billing_consistency(
    subscription_id: &SubscriptionId,
    billing_period: &BillingPeriod,
) -> ConsistencyReport {
    let processed_events = count_processed_events(subscription_id, billing_period).await?;
    let billed_events = count_billed_events(subscription_id, billing_period).await?;

    if processed_events != billed_events {
        ConsistencyReport::Mismatch {
            processed: processed_events,
            billed: billed_events,
            difference: processed_events.saturating_sub(billed_events),
        }
    } else {
        ConsistencyReport::Consistent
    }
}
```

**Remediation**: Nightly reconciliation job with automatic adjustment creation.

#### 5.2.2 Oversight-Audit Trail Gap

**Scenario**: Approval decision made but audit record missing.

**Prevention**: Transactional write of decision + audit in same database transaction.

**Detection**: Hourly audit completeness check.

---

## 6. Testing Strategies

### 6.1 Chaos Engineering Tests

```rust
#[tokio::test]
async fn test_network_partition_recovery() {
    let cluster = TestCluster::new(3).await;

    // Partition node 2 from nodes 0 and 1
    cluster.partition(vec![2], vec![0, 1]).await;

    // Operations should continue with quorum
    let result = cluster.node(0).process_event(test_event()).await;
    assert!(result.is_ok());

    // Heal partition
    cluster.heal().await;

    // Node 2 should catch up
    eventually(Duration::from_secs(30), || async {
        cluster.node(2).event_count() == cluster.node(0).event_count()
    }).await;
}

#[tokio::test]
async fn test_service_dependency_failure() {
    let harness = TestHarness::new().await;

    // Kill AuthZ service
    harness.kill_service("authz").await;

    // Verify graceful degradation
    let result = harness.metering().process_event(test_event()).await;
    assert!(matches!(result, Ok(ProcessResult::DegradedMode { .. })));

    // Restore AuthZ
    harness.restore_service("authz").await;

    // Verify recovery
    let result = harness.metering().process_event(test_event()).await;
    assert!(matches!(result, Ok(ProcessResult::Normal { .. })));
}
```

### 6.2 Fuzz Testing

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzedEvent {
    transaction_id: String,
    timestamp: i64,
    properties: HashMap<String, String>,
}

#[test]
fn fuzz_event_parsing() {
    arbtest::arbtest(|u| {
        let fuzzed: FuzzedEvent = u.arbitrary()?;

        // Should never panic, always return Result
        let result = BillableEvent::try_from(fuzzed);

        // Validate error handling
        match result {
            Ok(event) => {
                assert!(!event.transaction_id.is_empty());
            }
            Err(e) => {
                // Error message should be helpful
                assert!(!e.to_string().is_empty());
            }
        }

        Ok(())
    });
}
```

### 6.3 Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn quota_never_goes_negative(
        initial in 0i64..1_000_000,
        decrements in prop::collection::vec(1i64..1000, 0..100),
    ) {
        let quota = AtomicI64::new(initial);

        for decrement in decrements {
            let result = atomic_decrement(&quota, decrement);

            // Quota should never go negative
            prop_assert!(quota.load(Ordering::SeqCst) >= 0);

            // If rejected, quota should be unchanged
            if result.is_err() {
                prop_assert!(quota.load(Ordering::SeqCst) == initial);
            }
        }
    }

    #[test]
    fn ratchet_always_advances(
        message_count in 1usize..1000,
    ) {
        let (mut alice, mut bob) = create_session_pair();
        let mut prev_root_key = alice.root_key.clone();

        for i in 0..message_count {
            let (header, ciphertext) = alice.encrypt(b"test");
            bob.decrypt(&header, &ciphertext).unwrap();

            // Root key should change after DH ratchet
            if header.ratchet_key != prev_root_key {
                prop_assert_ne!(alice.root_key, prev_root_key);
                prev_root_key = alice.root_key.clone();
            }

            // Swap roles
            std::mem::swap(&mut alice, &mut bob);
        }
    }
}
```

---

## 7. Monitoring and Alerting

### 7.1 Edge Case Metrics

```rust
pub struct EdgeCaseMetrics {
    // Runtime
    oom_events: Counter,
    network_partitions: Counter,
    warm_pool_exhaustion: Counter,

    // Metering
    duplicate_events: Counter,
    late_events: Counter,
    quota_races: Counter,

    // Oversight
    escalation_timeouts: Counter,
    channel_failures: Counter,
    conflicting_approvals: Counter,

    // Messaging
    prekey_exhaustion: Counter,
    ratchet_desyncs: Counter,
    message_expirations: Counter,
}
```

### 7.2 Alert Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| OOM events/hour | 5 | 20 | Scale sandbox limits |
| Prekey exhaustion | 10% low | 1% exhausted | Force replenishment |
| Escalation timeouts/day | 3 | 10 | Review approver availability |
| Ratchet desyncs/hour | 5 | 20 | Investigate client bugs |
| Late events % | 1% | 5% | Check clock sync |

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-26 | 1.0 | Claude | Initial edge cases documentation |
