---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
parent_sdd: docs/sdd/products/oversight/01-requirements.md
---

# SDD-OVS-07: Oversight Operational Runbook

## 1. Operational Overview

### 1.1 Service Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Production Deployment                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Oversight-1  │  │ Oversight-2  │  │ Oversight-3  │      │
│  │ (Active)     │  │ (Active)     │  │ (Active)     │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         └──────────────────┴──────────────────┘             │
│                          │                                  │
│  ┌───────────────────────┴──────────────────────────────┐  │
│  │         Load Balancer (Round-robin)                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│         ┌────────────────┼────────────────┐                │
│         │                │                │                │
│  ┌──────▼───────┐ ┌──────▼───────┐ ┌─────▼──────┐         │
│  │ PostgreSQL   │ │ Redis        │ │ Slack API  │         │
│  │ (Primary)    │ │ (Master)     │ │            │         │
│  └──────────────┘ └──────────────┘ └────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Key Metrics Dashboard

**Grafana Dashboard:** `https://grafana.company.com/d/oversight`

| Metric | Normal Range | Warning Threshold | Critical Threshold |
|--------|--------------|-------------------|-------------------|
| **Request Creation Rate** | 10-100 req/s | >800 req/s | >1,000 req/s |
| **Request Creation Latency (p99)** | <5ms | >8ms | >10ms |
| **State Transition Latency (p99)** | <500µs | >800µs | >1ms |
| **Pending Requests** | 10-100 | >5,000 | >8,000 |
| **Notification Delivery Success Rate** | >99.5% | <99% | <98% |
| **Database Connection Pool Usage** | 20-60% | >80% | >95% |
| **Redis Memory Usage** | <70% | >85% | >95% |
| **CPU Usage** | 20-50% | >70% | >85% |
| **Memory Usage** | <60% | >80% | >90% |

### 1.3 On-Call Responsibilities

**Primary On-Call:**
- Respond to PagerDuty alerts within 15 minutes
- Triage incidents (P1/P2/P3/P4)
- Execute runbook procedures
- Escalate to secondary if needed

**Secondary On-Call:**
- Backup for primary on-call
- Escalation point for complex incidents
- Coordinate with other teams (Database, Infrastructure)

**Escalation Contacts:**
- **Primary On-Call:** Slack #oversight-oncall, PagerDuty
- **Secondary On-Call:** Slack #oversight-leads
- **Engineering Manager:** Escalate for P1 incidents
- **Security Team:** Escalate for security incidents

---

## 2. Common Operational Procedures

### 2.1 Stuck Request Resolution

**Symptom:** Request in `PENDING` state for >24 hours, no approver response

**Diagnosis:**
```bash
# Query stuck requests
psql -d creto_oversight -c "
SELECT request_id, agent_nhi, state, tier_index, created_at, updated_at
FROM oversight_requests
WHERE state IN ('PENDING', 'ESCALATED')
  AND created_at < NOW() - INTERVAL '24 hours'
ORDER BY created_at ASC
LIMIT 10;
"

# Check notification delivery status
psql -d creto_oversight -c "
SELECT request_id, channel_type, status, delivered_at, error_message
FROM notification_log
WHERE request_id = 'REQUEST_ID'
ORDER BY delivered_at DESC;
"
```

**Root Causes & Remediation:**

| Root Cause | Diagnosis | Remediation |
|-----------|-----------|-------------|
| **Notification delivery failed** | `notification_log.status = 'FAILED'` | Manually resend notification via admin API |
| **Approver unavailable** | No error, just no response | Escalate to next tier manually |
| **Timeout scheduler stopped** | No timeout events in last 24h | Restart timeout scheduler, verify pending tasks |
| **Approver forgot** | Notification delivered, no response | Send reminder notification |

**Manual Escalation:**
```bash
# Escalate request to next tier
curl -X POST https://oversight.company.com/api/v1/admin/requests/{request_id}/escalate \
  -H "Authorization: Bearer $ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Manual escalation due to stuck request",
    "notify_next_tier": true
  }'
```

**Manual Notification Resend:**
```bash
# Resend notification to approver
curl -X POST https://oversight.company.com/api/v1/admin/requests/{request_id}/resend-notification \
  -H "Authorization: Bearer $ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "approver_subject": "cfo@company.com",
    "channels": ["SLACK", "EMAIL"]
  }'
```

### 2.2 Channel Failover

**Symptom:** Slack notifications failing with high error rate

**Diagnosis:**
```bash
# Check Slack delivery errors
psql -d creto_oversight -c "
SELECT COUNT(*) AS error_count, error_message
FROM notification_log
WHERE channel_type = 'SLACK'
  AND status = 'FAILED'
  AND delivered_at > NOW() - INTERVAL '1 hour'
GROUP BY error_message
ORDER BY error_count DESC;
"

# Check Slack API status
curl https://status.slack.com/api/v2.0.0/current
```

**Remediation:**

1. **Temporary Failover to Email:**
   ```sql
   -- Update pending requests to use email channel
   UPDATE oversight_requests
   SET notification_channels = ARRAY['EMAIL']
   WHERE state IN ('PENDING', 'ESCALATED')
     AND notification_channels @> ARRAY['SLACK'];
   ```

2. **Resend Failed Notifications via Email:**
   ```bash
   # Get failed Slack notifications from last hour
   psql -d creto_oversight -c "
   SELECT DISTINCT request_id
   FROM notification_log
   WHERE channel_type = 'SLACK'
     AND status = 'FAILED'
     AND delivered_at > NOW() - INTERVAL '1 hour';
   " | tail -n +3 | head -n -2 | while read request_id; do
     curl -X POST https://oversight.company.com/api/v1/admin/requests/$request_id/resend-notification \
       -H "Authorization: Bearer $ADMIN_API_KEY" \
       -d '{"channels": ["EMAIL"]}'
   done
   ```

3. **Monitor Slack API Recovery:**
   ```bash
   # Check if Slack API is back online
   while true; do
     curl -s https://status.slack.com/api/v2.0.0/current | jq -r '.status'
     sleep 60
   done
   ```

4. **Restore Slack Channel:**
   ```sql
   -- Restore Slack channel once API is healthy
   UPDATE oversight_requests
   SET notification_channels = ARRAY['SLACK', 'EMAIL']
   WHERE state IN ('PENDING', 'ESCALATED')
     AND notification_channels = ARRAY['EMAIL'];
   ```

### 2.3 Database Connection Pool Exhaustion

**Symptom:** `database connection pool exhausted` errors, high query latency

**Diagnosis:**
```bash
# Check active connections
psql -d creto_oversight -c "
SELECT state, COUNT(*) AS count
FROM pg_stat_activity
WHERE datname = 'creto_oversight'
GROUP BY state
ORDER BY count DESC;
"

# Check long-running queries
psql -d creto_oversight -c "
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > INTERVAL '5 minutes'
  AND state = 'active';
"
```

**Remediation:**

1. **Terminate Long-Running Queries:**
   ```sql
   -- Kill queries running >10 minutes
   SELECT pg_terminate_backend(pid)
   FROM pg_stat_activity
   WHERE (now() - pg_stat_activity.query_start) > INTERVAL '10 minutes'
     AND state = 'active'
     AND datname = 'creto_oversight';
   ```

2. **Increase Connection Pool Size (Temporary):**
   ```bash
   # Update environment variable (requires pod restart)
   kubectl set env deployment/creto-oversight -n oversight \
     DATABASE_POOL_SIZE=50  # Increased from 25

   # Rolling restart
   kubectl rollout restart deployment/creto-oversight -n oversight
   ```

3. **Analyze Query Performance:**
   ```sql
   -- Find slow queries
   SELECT query, calls, total_exec_time, mean_exec_time, max_exec_time
   FROM pg_stat_statements
   WHERE query LIKE '%oversight_requests%'
   ORDER BY mean_exec_time DESC
   LIMIT 10;
   ```

4. **Add Missing Indexes (if needed):**
   ```sql
   -- Example: Add index if missing
   CREATE INDEX CONCURRENTLY idx_requests_state_created_at
   ON oversight_requests(state, created_at)
   WHERE state IN ('PENDING', 'ESCALATED');
   ```

### 2.4 Redis Cache Eviction Spike

**Symptom:** High cache miss rate, increased database load

**Diagnosis:**
```bash
# Check Redis memory usage
redis-cli INFO memory | grep used_memory_human

# Check eviction stats
redis-cli INFO stats | grep evicted_keys

# Check cache hit/miss ratio
redis-cli INFO stats | grep keyspace
```

**Remediation:**

1. **Increase Redis Memory Limit (if available):**
   ```bash
   # Update Redis config
   kubectl edit configmap redis-config -n oversight
   # Set maxmemory: 4gb (increased from 2gb)

   # Restart Redis
   kubectl rollout restart statefulset/redis -n oversight
   ```

2. **Reduce TTL for Less-Critical Keys:**
   ```bash
   # Reduce TTL for policy cache (from 3600s to 1800s)
   redis-cli CONFIG SET "policy:*" EX 1800
   ```

3. **Purge Stale Keys:**
   ```bash
   # Delete keys for completed requests (>7 days old)
   redis-cli --scan --pattern "checkpoint:*" | while read key; do
     ttl=$(redis-cli TTL $key)
     if [ $ttl -eq -1 ]; then  # No TTL set
       redis-cli DEL $key
     fi
   done
   ```

### 2.5 Timeout Scheduler Not Processing

**Symptom:** Requests not escalating despite tier timeout expiration

**Diagnosis:**
```bash
# Check if timeout scheduler is running
kubectl get pods -n oversight | grep timeout-scheduler

# Check timeout scheduler logs
kubectl logs -n oversight deployment/creto-oversight --tail=100 | grep timeout_scheduler

# Query overdue timeouts
psql -d creto_oversight -c "
SELECT r.request_id, r.created_at, et.timeout_seconds, r.tier_index
FROM oversight_requests r
INNER JOIN policies p ON r.policy_id = p.policy_id
INNER JOIN escalation_tiers et ON p.escalation_chain_id = et.chain_id AND r.tier_index = et.tier_index
WHERE r.state IN ('PENDING', 'ESCALATED')
  AND r.created_at + (et.timeout_seconds || ' seconds')::INTERVAL < NOW();
"
```

**Remediation:**

1. **Restart Timeout Scheduler:**
   ```bash
   # Restart Oversight service (includes timeout scheduler)
   kubectl rollout restart deployment/creto-oversight -n oversight

   # Wait for rollout to complete
   kubectl rollout status deployment/creto-oversight -n oversight
   ```

2. **Manually Trigger Timeout for Stuck Requests:**
   ```bash
   # Get overdue request IDs
   psql -d creto_oversight -t -c "
   SELECT r.request_id
   FROM oversight_requests r
   INNER JOIN policies p ON r.policy_id = p.policy_id
   INNER JOIN escalation_tiers et ON p.escalation_chain_id = et.chain_id AND r.tier_index = et.tier_index
   WHERE r.state IN ('PENDING', 'ESCALATED')
     AND r.created_at + (et.timeout_seconds || ' seconds')::INTERVAL < NOW();
   " | while read request_id; do
     curl -X POST https://oversight.company.com/api/v1/admin/requests/$request_id/timeout \
       -H "Authorization: Bearer $ADMIN_API_KEY"
   done
   ```

3. **Verify Recovery:**
   ```bash
   # Check recent timeout events in audit log
   psql -d creto_oversight -c "
   SELECT request_id, event_type, timestamp
   FROM approval_audit
   WHERE event_type = 'request.escalated'
     AND timestamp > NOW() - INTERVAL '10 minutes'
   ORDER BY timestamp DESC;
   "
   ```

---

## 3. Incident Response Procedures

### 3.1 P1 Incident: Complete Service Outage

**Definition:** Oversight service completely unavailable, 100% error rate

**Response Time:** Immediate (page on-call)

**Procedure:**

1. **Acknowledge Incident (0-5 minutes):**
   ```bash
   # Acknowledge PagerDuty alert
   pd incident acknowledge --id $INCIDENT_ID

   # Post to #incidents Slack channel
   /incident declare "Oversight service outage"
   ```

2. **Assess Scope (5-10 minutes):**
   ```bash
   # Check pod status
   kubectl get pods -n oversight

   # Check recent errors
   kubectl logs -n oversight deployment/creto-oversight --since=10m | grep ERROR

   # Check database connectivity
   psql -d creto_oversight -c "SELECT 1;"

   # Check external dependencies (Slack API, Authorization service)
   curl https://status.slack.com/api/v2.0.0/current
   curl https://authz.company.com/health
   ```

3. **Immediate Mitigation (10-20 minutes):**

   **If pods crashed:**
   ```bash
   # Check pod events
   kubectl describe pod -n oversight $(kubectl get pods -n oversight -o name | head -n1)

   # Force restart
   kubectl rollout restart deployment/creto-oversight -n oversight
   ```

   **If database down:**
   ```bash
   # Check PostgreSQL status
   kubectl exec -n oversight postgres-0 -- pg_isready

   # If primary down, failover to standby
   patronictl -c /etc/patroni/patroni.yaml failover --force
   ```

   **If authorization service down:**
   ```bash
   # Check Authorization service status
   kubectl get pods -n authz

   # Contact Authorization on-call
   slack @authz-oncall "Oversight depends on Authorization, service is down"
   ```

4. **Restore Service (20-30 minutes):**
   ```bash
   # Verify pods healthy
   kubectl get pods -n oversight

   # Health check
   curl https://oversight.company.com/health

   # Test request creation
   curl -X POST https://oversight.company.com/api/v1/requests \
     -H "Authorization: Bearer $TEST_API_KEY" \
     -d '{ /* test request */ }'
   ```

5. **Post-Incident Actions (30-60 minutes):**
   ```bash
   # Verify pending requests recovered
   psql -d creto_oversight -c "
   SELECT COUNT(*) FROM oversight_requests WHERE state IN ('PENDING', 'ESCALATED');
   "

   # Check for stuck requests
   # See Section 2.1 "Stuck Request Resolution"

   # Update incident status
   /incident update "Service restored, verifying pending requests"
   ```

6. **Post-Mortem (Within 48 hours):**
   - Document root cause
   - Timeline of events
   - Mitigation steps taken
   - Action items to prevent recurrence

### 3.2 P2 Incident: Degraded Performance

**Definition:** High latency (p99 >50ms), elevated error rate (>1%)

**Response Time:** 30 minutes

**Procedure:**

1. **Identify Bottleneck:**
   ```bash
   # Check request creation latency
   curl -s https://oversight.company.com/metrics | grep oversight_request_creation_duration_seconds

   # Check database query performance
   psql -d creto_oversight -c "
   SELECT query, calls, mean_exec_time, max_exec_time
   FROM pg_stat_statements
   ORDER BY mean_exec_time DESC
   LIMIT 10;
   "

   # Check Redis latency
   redis-cli --latency-history
   ```

2. **Scale Resources:**
   ```bash
   # Scale up Oversight pods
   kubectl scale deployment/creto-oversight -n oversight --replicas=5

   # Scale up database connections
   kubectl set env deployment/creto-oversight -n oversight DATABASE_POOL_SIZE=50
   ```

3. **Monitor Recovery:**
   ```bash
   # Watch metrics dashboard
   open https://grafana.company.com/d/oversight

   # Check error rate
   curl -s https://oversight.company.com/metrics | grep oversight_request_creation_errors_total
   ```

### 3.3 P3 Incident: Notification Delivery Degraded

**Definition:** Notification delivery success rate <98%

**Response Time:** 1 hour

**Procedure:**

1. **Identify Failing Channel:**
   ```sql
   SELECT channel_type, COUNT(*) AS failed_count
   FROM notification_log
   WHERE status = 'FAILED'
     AND delivered_at > NOW() - INTERVAL '1 hour'
   GROUP BY channel_type
   ORDER BY failed_count DESC;
   ```

2. **Failover to Backup Channel:**
   - See Section 2.2 "Channel Failover"

3. **Contact Channel Provider:**
   - Slack: Check https://status.slack.com
   - Email: Contact SMTP provider support
   - Webhook: Contact external system team

---

## 4. Audit and Compliance Procedures

### 4.1 Approval Audit Review

**Frequency:** Weekly (automated), Monthly (manual review)

**Procedure:**

1. **Generate Approval Report:**
   ```sql
   -- All approvals in last 30 days
   SELECT
     r.request_id,
     r.agent_nhi,
     r.action_description,
     resp.approver_subject,
     resp.decision,
     resp.responded_at,
     r.state
   FROM oversight_requests r
   LEFT JOIN approval_responses resp ON r.request_id = resp.request_id
   WHERE r.created_at > NOW() - INTERVAL '30 days'
   ORDER BY r.created_at DESC;
   ```

2. **Review Anomalies:**
   ```sql
   -- Approvals outside business hours
   SELECT *
   FROM approval_responses
   WHERE EXTRACT(HOUR FROM responded_at) NOT BETWEEN 8 AND 18
     OR EXTRACT(DOW FROM responded_at) IN (0, 6)  -- Weekend
     AND responded_at > NOW() - INTERVAL '30 days';

   -- High-value approvals (>$100K)
   SELECT *
   FROM oversight_requests
   WHERE (resource->>'amount')::NUMERIC > 100000
     AND state = 'APPROVED'
     AND created_at > NOW() - INTERVAL '30 days';
   ```

3. **Verify Signature Integrity:**
   ```bash
   # Export approvals for external verification
   psql -d creto_oversight -t -A -F"," -c "
   SELECT
     request_id,
     approver_subject,
     decision,
     signature_algorithm,
     encode(signature_value, 'hex') AS signature_hex
   FROM approval_responses
   WHERE responded_at > NOW() - INTERVAL '30 days';
   " > approvals_last_30_days.csv

   # Verify signatures with external tool
   python verify_signatures.py approvals_last_30_days.csv
   ```

### 4.2 Policy Compliance Report

**Frequency:** Quarterly

**Procedure:**

1. **Generate Coverage Report:**
   ```sql
   -- Actions requiring oversight vs. total actions
   WITH total_actions AS (
     SELECT COUNT(*) AS total
     FROM authz_decisions
     WHERE timestamp > NOW() - INTERVAL '90 days'
   ),
   oversight_required AS (
     SELECT COUNT(*) AS required
     FROM authz_decisions
     WHERE decision = 'REQUIRES_OVERSIGHT'
       AND timestamp > NOW() - INTERVAL '90 days'
   )
   SELECT
     total_actions.total AS total_actions,
     oversight_required.required AS oversight_required,
     ROUND(100.0 * oversight_required.required / total_actions.total, 2) AS oversight_percentage
   FROM total_actions, oversight_required;
   ```

2. **Approval Response Time Analysis:**
   ```sql
   -- Average time to approval by tier
   SELECT
     tier_index,
     COUNT(*) AS request_count,
     AVG(EXTRACT(EPOCH FROM (resp.responded_at - r.created_at))) AS avg_seconds_to_approval,
     PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (resp.responded_at - r.created_at))) AS median_seconds
   FROM oversight_requests r
   INNER JOIN approval_responses resp ON r.request_id = resp.request_id
   WHERE r.state = 'APPROVED'
     AND r.created_at > NOW() - INTERVAL '90 days'
   GROUP BY tier_index
   ORDER BY tier_index;
   ```

3. **Timeout Rate Analysis:**
   ```sql
   -- Requests that timed out vs. total requests
   SELECT
     COUNT(*) FILTER (WHERE state = 'TIMEOUT') AS timeout_count,
     COUNT(*) AS total_requests,
     ROUND(100.0 * COUNT(*) FILTER (WHERE state = 'TIMEOUT') / COUNT(*), 2) AS timeout_percentage
   FROM oversight_requests
   WHERE created_at > NOW() - INTERVAL '90 days';
   ```

---

## 5. Backup and Recovery

### 5.1 Database Backup

**Schedule:** Daily full backup at 02:00 UTC, continuous WAL archiving

**Verification Procedure:**
```bash
# List recent backups
aws s3 ls s3://creto-backups/oversight/base/ --recursive | tail -n5

# Verify backup integrity
pg_verifybackup /backups/oversight/base/backup-2024-12-25

# Test restore (monthly)
pg_basebackup -D /tmp/restore_test -Fp -Xs -P
psql -d template1 -c "DROP DATABASE IF EXISTS oversight_restore_test;"
psql -d template1 -c "CREATE DATABASE oversight_restore_test;"
pg_restore -d oversight_restore_test /tmp/restore_test
```

### 5.2 Disaster Recovery Procedure

**RTO (Recovery Time Objective):** 4 hours
**RPO (Recovery Point Objective):** 1 hour

**Procedure:**

1. **Assess Damage (0-30 minutes):**
   ```bash
   # Check database corruption
   psql -d creto_oversight -c "SELECT pg_database.datname, pg_stat_database.* FROM pg_database JOIN pg_stat_database ON pg_database.oid = pg_stat_database.datid;"

   # Check data loss window
   psql -d creto_oversight -c "SELECT MAX(created_at) FROM oversight_requests;"
   ```

2. **Restore from Backup (30 minutes - 2 hours):**
   ```bash
   # Stop application
   kubectl scale deployment/creto-oversight -n oversight --replicas=0

   # Restore base backup
   pg_basebackup -D /var/lib/postgresql/data -Fp -Xs -P

   # Restore WAL archives (point-in-time recovery)
   cat > /var/lib/postgresql/data/recovery.conf <<EOF
   restore_command = 'aws s3 cp s3://creto-backups/oversight/wal/%f %p'
   recovery_target_time = '2024-12-25 14:30:00'
   EOF

   # Start PostgreSQL (will replay WAL to recovery_target_time)
   systemctl start postgresql
   ```

3. **Verify Data Integrity (2-3 hours):**
   ```bash
   # Check request counts
   psql -d creto_oversight -c "SELECT COUNT(*) FROM oversight_requests;"

   # Verify recent requests present
   psql -d creto_oversight -c "SELECT * FROM oversight_requests ORDER BY created_at DESC LIMIT 10;"

   # Check audit log continuity
   psql -d creto_oversight -c "SELECT MIN(timestamp), MAX(timestamp) FROM approval_audit;"
   ```

4. **Resume Service (3-4 hours):**
   ```bash
   # Scale up application
   kubectl scale deployment/creto-oversight -n oversight --replicas=3

   # Health check
   curl https://oversight.company.com/health

   # Verify checkpoint recovery
   kubectl logs -n oversight deployment/creto-oversight | grep "Checkpoint recovery complete"
   ```

---

## 6. Performance Tuning

### 6.1 Database Optimization

**Query Performance Tuning:**
```sql
-- Analyze slow queries
SELECT query, calls, total_exec_time, mean_exec_time
FROM pg_stat_statements
WHERE query LIKE '%oversight_requests%'
ORDER BY mean_exec_time DESC
LIMIT 20;

-- Vacuum analyze (monthly)
VACUUM ANALYZE oversight_requests;
VACUUM ANALYZE approval_responses;

-- Reindex (quarterly)
REINDEX TABLE oversight_requests;
REINDEX TABLE approval_responses;
```

**Connection Pool Tuning:**
```bash
# Monitor connection pool usage
psql -d creto_oversight -c "
SELECT COUNT(*) AS active_connections, state
FROM pg_stat_activity
WHERE datname = 'creto_oversight'
GROUP BY state;
"

# Adjust pool size based on load
kubectl set env deployment/creto-oversight -n oversight \
  DATABASE_POOL_SIZE=40  # Tune based on metrics
```

### 6.2 Redis Optimization

**Memory Optimization:**
```bash
# Check memory fragmentation
redis-cli INFO memory | grep mem_fragmentation_ratio

# Defragment if ratio >1.5
redis-cli MEMORY DOCTOR

# Set eviction policy
redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

**Cache Prewarming:**
```bash
# Preload frequently-accessed policies on startup
curl -X POST https://oversight.company.com/api/v1/admin/cache/prewarm \
  -H "Authorization: Bearer $ADMIN_API_KEY" \
  -d '{"policy_ids": ["pol_large_transfer", "pol_database_access", ...]}'
```

---

## 7. Monitoring and Alerting

### 7.1 Alert Definitions

| Alert | Condition | Severity | Action |
|-------|----------|----------|--------|
| **HighRequestLatency** | p99 latency >10ms for 5 minutes | Warning | Investigate bottleneck (see 3.2) |
| **ServiceDown** | 0 healthy pods for 1 minute | Critical | Page on-call (see 3.1) |
| **DatabaseConnectionPoolExhausted** | >95% pool usage for 2 minutes | Critical | Scale pool (see 2.3) |
| **NotificationDeliveryFailed** | <98% success rate for 10 minutes | Warning | Channel failover (see 2.2) |
| **StuckRequests** | >10 requests pending >24 hours | Warning | Manual escalation (see 2.1) |
| **MemoryUsageHigh** | >90% memory for 5 minutes | Warning | Investigate memory leak |
| **DiskUsageHigh** | >85% disk usage | Warning | Cleanup old logs, backups |

### 7.2 Health Checks

**Liveness Probe:**
```bash
# Kubernetes liveness probe
curl http://localhost:9090/healthz

# Expected response: 200 OK
```

**Readiness Probe:**
```bash
# Kubernetes readiness probe
curl http://localhost:9090/ready

# Expected response: 200 OK
# Checks: Database connectivity, Redis connectivity
```

**Synthetic Monitoring:**
```bash
# End-to-end health check (runs every 60s)
curl -X POST https://oversight.company.com/api/v1/requests \
  -H "Authorization: Bearer $SYNTHETIC_TEST_KEY" \
  -d '{
    "agent_nhi": "agent:synthetic-test@company.creto",
    "action": "TestAction",
    "resource": {"test": true},
    "policy_id": "pol_synthetic_test"
  }'

# Expected response: 201 Created (within 100ms)
```

---

## 8. Security Operations

### 8.1 Signature Verification Failure Alert

**Alert Trigger:** >5 invalid signature errors in 5 minutes

**Response:**
1. **Identify Affected Approver:**
   ```sql
   SELECT approver_subject, COUNT(*) AS failure_count
   FROM approval_audit
   WHERE event_type = 'signature_verification_failed'
     AND timestamp > NOW() - INTERVAL '5 minutes'
   GROUP BY approver_subject
   ORDER BY failure_count DESC;
   ```

2. **Contact Approver:**
   ```bash
   # Send Slack DM to approver
   slack chat:send --user $APPROVER_SLACK_ID \
     --text "We detected multiple signature verification failures for your approvals. Please verify your signing key is correct."
   ```

3. **Rotate Approver Key (if compromised):**
   - See Section 5.3 in `05-security.md`

### 8.2 Suspicious Approval Pattern Detection

**Automated Checks (Daily):**
```sql
-- Approvals outside normal hours
SELECT *
FROM approval_responses
WHERE EXTRACT(HOUR FROM responded_at) NOT BETWEEN 8 AND 18
  AND responded_at > NOW() - INTERVAL '24 hours';

-- Unusually fast approvals (<10 seconds)
SELECT *
FROM approval_responses resp
INNER JOIN oversight_requests req ON resp.request_id = req.request_id
WHERE resp.responded_at - req.created_at < INTERVAL '10 seconds'
  AND resp.responded_at > NOW() - INTERVAL '24 hours';

-- High-value approvals
SELECT *
FROM oversight_requests
WHERE (resource->>'amount')::NUMERIC > 100000
  AND state = 'APPROVED'
  AND created_at > NOW() - INTERVAL '24 hours';
```

**Manual Review Trigger:**
- Email security team with flagged approvals
- Require justification from approver

---

**END OF DOCUMENT**
