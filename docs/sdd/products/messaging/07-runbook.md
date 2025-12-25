---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# Messaging Operational Runbook

## Table of Contents

1. [Routine Operations](#routine-operations)
2. [Incident Response](#incident-response)
3. [Key Compromise Response](#key-compromise-response)
4. [Performance Degradation](#performance-degradation)
5. [Capacity Planning](#capacity-planning)
6. [Maintenance Procedures](#maintenance-procedures)
7. [Monitoring & Alerting](#monitoring--alerting)

---

## Routine Operations

### Health Checks

**Service health endpoint:**

```bash
# Check messaging service health
curl https://messaging.creto.io/health

# Expected response (200 OK):
{
  "status": "healthy",
  "checks": {
    "database": "ok",
    "redis": "ok",
    "nhi_service": "ok",
    "authz_service": "ok"
  },
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

**Component health checks:**

```bash
# PostgreSQL
psql -h messaging-db.creto.io -U messaging -c "SELECT 1;" >/dev/null && echo "Postgres OK"

# Redis
redis-cli -h messaging-redis.creto.io PING | grep PONG && echo "Redis OK"

# NHI service
curl -f https://nhi.creto.io/health || echo "NHI DOWN"

# AuthZ service
curl -f https://authz.creto.io/health || echo "AuthZ DOWN"
```

**Automated health checks (every 30 seconds):**

```yaml
# kubernetes/health-check.yaml
apiVersion: v1
kind: Pod
metadata:
  name: messaging-health-check
spec:
  containers:
  - name: health-checker
    image: curlimages/curl
    command:
    - /bin/sh
    - -c
    - |
      while true; do
        curl -f http://messaging-service:8080/health || exit 1
        sleep 30
      done
```

### Daily Operations Checklist

**Morning (8:00 AM UTC):**
- [ ] Review overnight alerts (PagerDuty)
- [ ] Check canary test results (last 24h)
- [ ] Review error rate dashboard (p99 latency, failed deliveries)
- [ ] Check storage usage (Redis, PostgreSQL)
- [ ] Verify key rotation jobs ran successfully

**Midday (12:00 PM UTC):**
- [ ] Check peak load metrics (message throughput)
- [ ] Review AuthZ denial logs (identify policy issues)
- [ ] Monitor queue depths (recipients with >1000 pending messages)

**Evening (6:00 PM UTC):**
- [ ] Review security logs (signature verification failures)
- [ ] Check backup status (database snapshots)
- [ ] Update runbook if new issues found

### Weekly Operations

**Monday:**
- [ ] Review capacity trends (storage, compute, network)
- [ ] Plan scaling adjustments for upcoming week
- [ ] Update on-call schedule

**Wednesday:**
- [ ] Run chaos engineering tests in staging
- [ ] Review performance regression tests

**Friday:**
- [ ] Archive old delivery receipts to S3 (>30 days)
- [ ] Review and close resolved incidents
- [ ] Update documentation for operational changes

---

## Incident Response

### Severity Levels

| Severity | Impact | Response Time | Example |
|----------|--------|---------------|---------|
| **P1 (Critical)** | Complete service outage | <15 minutes | All messages failing |
| **P2 (High)** | Partial outage | <1 hour | Region down, degraded performance |
| **P3 (Medium)** | Minor degradation | <4 hours | Slow delivery, elevated errors |
| **P4 (Low)** | Cosmetic issues | <1 business day | Typos in logs |

### P1: Complete Service Outage

**Symptoms:**
- Health check failing
- 100% error rate on `/v1/messages`
- No messages delivered in last 5 minutes

**Immediate Actions (0-15 minutes):**

1. **Acknowledge alert:**
   ```bash
   # PagerDuty
   pd incident ack <incident-id>
   ```

2. **Check service status:**
   ```bash
   kubectl get pods -n messaging
   kubectl logs -n messaging messaging-service-<pod> --tail=100
   ```

3. **Identify failure mode:**
   - **Pods crashing?** Check logs for panic/OOM
   - **Database down?** Check PostgreSQL/Redis health
   - **Network issue?** Check connectivity to NHI/AuthZ

4. **Emergency mitigation:**
   ```bash
   # Restart pods
   kubectl rollout restart deployment/messaging-service -n messaging

   # Scale up replicas (if load issue)
   kubectl scale deployment/messaging-service -n messaging --replicas=10

   # Failover to secondary region (if regional outage)
   kubectl apply -f k8s/failover-to-us-west.yaml
   ```

5. **Communicate status:**
   ```bash
   # Post to status page
   curl -X POST https://status.creto.io/api/incidents \
     -d '{"status": "investigating", "message": "Messaging service outage detected"}'

   # Notify Slack
   curl -X POST https://hooks.slack.com/services/T00/B00/XXX \
     -d '{"text": "🚨 P1: Messaging service DOWN. Investigating..."}'
   ```

**Investigation (15-60 minutes):**

1. **Collect diagnostics:**
   ```bash
   # Service logs
   kubectl logs -n messaging messaging-service-<pod> --since=30m > /tmp/messaging-logs.txt

   # Database logs
   kubectl logs -n messaging messaging-db-0 --since=30m > /tmp/db-logs.txt

   # Metrics snapshot
   curl https://prometheus.creto.io/api/v1/query_range \
     -d 'query=up{job="messaging-service"}' \
     -d 'start=<30m ago>' > /tmp/metrics.json
   ```

2. **Check recent deployments:**
   ```bash
   kubectl rollout history deployment/messaging-service -n messaging

   # Rollback if recent deployment caused issue
   kubectl rollout undo deployment/messaging-service -n messaging
   ```

3. **Check dependencies:**
   ```bash
   # NHI service
   curl -f https://nhi.creto.io/health

   # AuthZ service
   curl -f https://authz.creto.io/health

   # Database connectivity
   psql -h messaging-db.creto.io -U messaging -c "SELECT NOW();"
   ```

**Resolution:**

1. **Restore service:**
   - Fix root cause (e.g., increase database connections, fix code bug)
   - Verify health checks passing
   - Monitor error rates for 15 minutes

2. **Post-incident:**
   ```bash
   # Mark incident resolved
   pd incident resolve <incident-id>

   # Update status page
   curl -X PATCH https://status.creto.io/api/incidents/<id> \
     -d '{"status": "resolved", "message": "Service restored"}'
   ```

3. **Schedule post-mortem:**
   - Create document: `incidents/YYYY-MM-DD-messaging-outage.md`
   - Include: Timeline, root cause, impact, action items
   - Review meeting within 48 hours

---

## Key Compromise Response

### Detection

**Indicators of compromise:**
- Signature verification failures spike (>100/hour)
- Messages from agent at unusual times/volumes
- Audit log shows key usage from unexpected IPs
- Security team notification (leaked private key)

### Response Procedure

**Phase 1: Containment (T+0 to T+5 minutes)**

1. **Confirm compromise:**
   ```bash
   # Query audit logs for anomalous activity
   psql -h messaging-db.creto.io -U messaging -c "
     SELECT event_type, actor_nhi, COUNT(*) as count
     FROM audit_events
     WHERE actor_nhi = 'agent-alice-01'
     AND event_time > NOW() - INTERVAL '1 hour'
     GROUP BY event_type, actor_nhi
     ORDER BY count DESC;
   "
   ```

2. **Emergency key rotation:**
   ```bash
   # Revoke compromised keys immediately (no grace period)
   curl -X POST https://messaging.creto.io/v1/keys/emergency-rotate \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "agent_nhi": "agent-alice-01",
       "reason": "private_key_compromised",
       "skip_grace_period": true
     }'
   ```

3. **Rate limit agent:**
   ```bash
   # Prevent further abuse
   redis-cli SET "ratelimit:agent-alice-01:blocked" "1" EX 3600

   # Or use AuthZ to deny all messages from agent
   curl -X POST https://authz.creto.io/v1/policies \
     -d '{
       "id": "deny-compromised-agent",
       "effect": "deny",
       "subjects": ["agent-alice-01"],
       "actions": ["send_message", "receive_message"]
     }'
   ```

**Phase 2: Investigation (T+5 to T+30 minutes)**

1. **Identify affected messages:**
   ```sql
   -- Find all messages signed with compromised key
   SELECT message_id, sender_nhi, recipient_nhi, created_at
   FROM delivery_receipts
   WHERE sender_nhi = 'agent-alice-01'
   AND sent_at > NOW() - INTERVAL '7 days'
   AND status IN ('delivered', 'pending')
   ORDER BY sent_at DESC;
   ```

2. **Notify affected recipients:**
   ```bash
   # Script to send notifications
   for recipient in $(psql -t -c "SELECT DISTINCT recipient_nhi FROM delivery_receipts WHERE sender_nhi='agent-alice-01'"); do
     curl -X POST https://messaging.creto.io/v1/messages \
       -H "Authorization: Bearer $SYSTEM_TOKEN" \
       -d "{
         \"recipient_nhi\": \"$recipient\",
         \"payload\": \"SECURITY ALERT: Messages from agent-alice-01 may be compromised. Key rotation completed.\"
       }"
   done
   ```

3. **Forensic analysis:**
   ```bash
   # Collect evidence
   mkdir -p /tmp/incident-$(date +%Y%m%d-%H%M%S)
   cd /tmp/incident-*

   # Export audit logs
   psql -c "COPY (SELECT * FROM audit_events WHERE actor_nhi='agent-alice-01') TO STDOUT CSV HEADER" > audit_logs.csv

   # Export message metadata (no plaintext)
   psql -c "COPY (SELECT message_id, sender_nhi, recipient_nhi, created_at FROM delivery_receipts WHERE sender_nhi='agent-alice-01') TO STDOUT CSV HEADER" > messages.csv

   # Preserve for security team
   tar czf incident-evidence.tar.gz audit_logs.csv messages.csv
   ```

**Phase 3: Recovery (T+30 to T+60 minutes)**

1. **Verify new keys active:**
   ```bash
   # Check key bundle
   curl https://messaging.creto.io/v1/keys/bundle/agent-alice-01 | jq '.status'
   # Expected: "active"
   ```

2. **Test message send/receive:**
   ```bash
   # Send test message with new keys
   curl -X POST https://messaging.creto.io/v1/messages \
     -H "Authorization: Bearer $AGENT_ALICE_TOKEN" \
     -d '{"recipient_nhi": "test-receiver", "payload": "<encrypted>"}'
   ```

3. **Remove rate limits (if appropriate):**
   ```bash
   redis-cli DEL "ratelimit:agent-alice-01:blocked"
   ```

**Phase 4: Post-Incident (T+24 hours)**

1. **Post-mortem meeting:**
   - How was compromise detected?
   - Was response timely?
   - What was the blast radius?
   - How to prevent recurrence?

2. **Update procedures:**
   - Document lessons learned
   - Update detection rules
   - Enhance monitoring

3. **Notify compliance/legal (if required):**
   - GDPR breach notification (72 hours)
   - HIPAA breach notification (60 days)

---

## Performance Degradation

### Symptoms

- Elevated latency (p99 > 50ms)
- Message queue depths increasing
- CPU/memory usage high
- Delivery failures increasing

### Diagnosis

**1. Check metrics dashboard:**

```bash
# Grafana dashboard
open https://grafana.creto.io/d/messaging-performance

# Key metrics:
# - Message throughput (msg/sec)
# - p50/p99 latency
# - Error rate
# - Queue depth
```

**2. Identify bottleneck:**

```bash
# CPU usage
kubectl top pods -n messaging

# Memory usage
kubectl top nodes

# Database connections
psql -c "SELECT count(*) FROM pg_stat_activity WHERE datname='messaging';"

# Redis slow queries
redis-cli SLOWLOG GET 10
```

**3. Query performance:**

```sql
-- Find slow queries (PostgreSQL)
SELECT
  query,
  mean_exec_time,
  calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

### Mitigation

**Short-term (immediate):**

```bash
# Scale up pods
kubectl scale deployment/messaging-service -n messaging --replicas=20

# Increase database connections
kubectl set env deployment/messaging-service DATABASE_POOL_SIZE=200

# Enable AuthZ fail-open (if AuthZ is slow)
kubectl set env deployment/messaging-service AUTHZ_FAIL_OPEN=true
```

**Medium-term (1-24 hours):**

```bash
# Add database replicas
kubectl scale statefulset/messaging-db -n messaging --replicas=3

# Add Redis cluster nodes
kubectl scale statefulset/messaging-redis -n messaging --replicas=6

# Increase resource limits
kubectl set resources deployment/messaging-service \
  --limits=cpu=4,memory=8Gi \
  --requests=cpu=2,memory=4Gi
```

**Long-term (1-7 days):**

- Optimize slow queries (add indexes)
- Implement caching (Redis)
- Partition database tables
- Shard message queues
- Enable horizontal pod autoscaling

---

## Capacity Planning

### Storage Capacity

**Current usage:**

```bash
# PostgreSQL database size
psql -c "SELECT pg_size_pretty(pg_database_size('messaging'));"

# Redis memory usage
redis-cli INFO memory | grep used_memory_human

# Breakdown by table
psql -c "
  SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
  FROM pg_tables
  WHERE schemaname = 'public'
  ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
  LIMIT 10;
"
```

**Projected growth:**

| Component | Current | Growth Rate | Projected (6 months) |
|-----------|---------|-------------|----------------------|
| Delivery receipts | 520 GB | +10 GB/month | 580 GB |
| Ratchet state | 50 GB | +5 GB/month | 80 GB |
| Channel metadata | 10 GB | +1 GB/month | 16 GB |
| Redis cache | 100 GB | +2 GB/month | 112 GB |

**Action items:**

- **Storage expansion:**
  ```bash
  # Increase PVC size
  kubectl patch pvc messaging-db-pvc -p '{"spec":{"resources":{"requests":{"storage":"1Ti"}}}}'
  ```

- **Data retention cleanup:**
  ```bash
  # Archive old delivery receipts
  psql -c "DELETE FROM delivery_receipts WHERE sent_at < NOW() - INTERVAL '30 days';"
  ```

### Compute Capacity

**Current resource usage:**

```bash
# CPU utilization (average over last 24h)
kubectl top pods -n messaging --containers

# Memory utilization
kubectl describe nodes | grep -A 5 "Allocated resources"
```

**Scaling triggers:**

| Metric | Threshold | Action |
|--------|-----------|--------|
| CPU usage | >70% for 5 min | Add 2 pods |
| Memory usage | >80% | Add 2 pods |
| Queue depth | >10,000 msgs | Add 5 pods |
| Latency p99 | >50ms for 10 min | Add 3 pods |

**Autoscaling configuration:**

```yaml
# kubernetes/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: messaging-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: messaging-service
  minReplicas: 5
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 2
        periodSeconds: 120
```

---

## Maintenance Procedures

### Database Maintenance

**Weekly vacuum (PostgreSQL):**

```bash
# Automated via cron job
psql -c "VACUUM ANALYZE delivery_receipts;"
psql -c "VACUUM ANALYZE ratchet_states;"
psql -c "VACUUM ANALYZE channels;"

# Check for bloat
psql -c "
  SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    n_dead_tup
  FROM pg_stat_user_tables
  ORDER BY n_dead_tup DESC
  LIMIT 10;
"
```

**Monthly index rebuild:**

```sql
-- Identify unused indexes
SELECT
  schemaname,
  tablename,
  indexname,
  idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0
ORDER BY pg_relation_size(indexrelid) DESC;

-- Rebuild fragmented indexes
REINDEX INDEX CONCURRENTLY idx_delivery_receipts_recipient_time;
```

### Key Rotation Scheduling

**Verify rotation jobs:**

```bash
# Check RotationScheduler logs
kubectl logs -n messaging messaging-key-rotation-cronjob-<pod> --tail=100

# List sessions due for rotation
psql -c "
  SELECT session_id, agent_a_nhi, agent_b_nhi, next_rotation_at
  FROM ratchet_states
  WHERE next_rotation_at < NOW() + INTERVAL '1 hour'
  ORDER BY next_rotation_at
  LIMIT 10;
"
```

**Manual rotation (emergency):**

```bash
# Rotate specific agent's keys
curl -X POST https://messaging.creto.io/v1/keys/rotate \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"agent_nhi": "agent-alice-01", "reason": "manual_rotation"}'
```

### Certificate Renewal

**TLS certificates (Let's Encrypt):**

```bash
# Check expiration
openssl s_client -connect messaging.creto.io:443 -servername messaging.creto.io \
  </dev/null 2>/dev/null | openssl x509 -noout -dates

# Renew (automated via cert-manager)
kubectl get certificate -n messaging
kubectl describe certificate messaging-tls -n messaging

# Force renewal
kubectl delete secret messaging-tls -n messaging
# cert-manager will automatically recreate
```

---

## Monitoring & Alerting

### Key Metrics

**SLIs (Service Level Indicators):**

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Availability | 99.99% | <99.95% over 5 min |
| Latency (p99) | <10ms | >50ms over 5 min |
| Error rate | <0.1% | >1% over 5 min |
| Message throughput | >100K msg/sec | <50K msg/sec (degraded) |
| Key rotation success | 100% | <99% |

**Prometheus queries:**

```promql
# Availability (uptime)
up{job="messaging-service"} == 1

# Latency (p99)
histogram_quantile(0.99, rate(messaging_request_duration_seconds_bucket[5m]))

# Error rate
rate(messaging_errors_total[5m]) / rate(messaging_requests_total[5m])

# Message throughput
rate(messaging_messages_sent_total[1m])
```

### Alert Rules

**Critical alerts (PagerDuty):**

```yaml
# prometheus/alerts.yaml
groups:
- name: messaging-critical
  interval: 30s
  rules:
  - alert: MessagingServiceDown
    expr: up{job="messaging-service"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Messaging service is down"
      description: "Messaging service {{ $labels.instance }} is unreachable"

  - alert: HighLatency
    expr: histogram_quantile(0.99, rate(messaging_request_duration_seconds_bucket[5m])) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High latency detected (p99 > 50ms)"

  - alert: HighErrorRate
    expr: rate(messaging_errors_total[5m]) / rate(messaging_requests_total[5m]) > 0.01
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Error rate exceeded 1%"

  - alert: KeyRotationFailed
    expr: rate(messaging_key_rotation_failures_total[15m]) > 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Key rotation failed"
      description: "Session {{ $labels.session_id }} failed to rotate keys"
```

**Warning alerts (Slack):**

```yaml
- alert: QueueDepthHigh
  expr: messaging_queue_depth > 10000
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "Queue depth exceeds 10,000 messages"

- alert: DatabaseConnectionsHigh
  expr: pg_stat_database_numbackends{datname="messaging"} > 150
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Database connections approaching limit"
```

### Dashboards

**Grafana dashboard panels:**

1. **Overview:**
   - Message throughput (line chart)
   - Latency (p50, p99, p999) (line chart)
   - Error rate (gauge)
   - Active agents (stat)

2. **Performance:**
   - CPU usage (heatmap)
   - Memory usage (heatmap)
   - Queue depth (line chart)
   - Database connections (line chart)

3. **Security:**
   - Signature verification failures (counter)
   - Key rotations (timeline)
   - AuthZ denials (table)

4. **Infrastructure:**
   - Pod status (table)
   - Node resource usage (heatmap)
   - Network I/O (line chart)

**Example dashboard JSON:**

```json
{
  "dashboard": {
    "title": "Messaging Service",
    "panels": [
      {
        "title": "Message Throughput",
        "targets": [
          {
            "expr": "rate(messaging_messages_sent_total[1m])",
            "legendFormat": "Messages/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Latency (p99)",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(messaging_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

---

## Summary

### On-Call Responsibilities

**Primary on-call:**
- Respond to P1/P2 alerts within SLA
- Triage and escalate as needed
- Update incident status page
- Coordinate with secondary on-call

**Secondary on-call:**
- Backup for primary
- Assist with investigation
- Handle P3/P4 issues

**Escalation path:**
```
P1 Alert
  ↓
Primary On-Call (15 min)
  ↓ (if no response)
Secondary On-Call (30 min)
  ↓ (if unresolved)
Engineering Manager (1 hour)
  ↓ (if critical)
VP Engineering
```

### Critical Contacts

| Role | Name | Slack | Phone | PagerDuty |
|------|------|-------|-------|-----------|
| **Primary On-Call** | Rotating | #messaging-oncall | - | @messaging-primary |
| **Secondary On-Call** | Rotating | #messaging-oncall | - | @messaging-secondary |
| **Engineering Manager** | Alice | @alice | +1-555-0100 | @alice-em |
| **Security Team** | Bob | @bob-security | +1-555-0200 | @security-oncall |
| **Platform Team** | Charlie | @charlie-platform | +1-555-0300 | @platform-oncall |

### Useful Links

- **Production Dashboard:** https://grafana.creto.io/d/messaging-production
- **Staging Dashboard:** https://grafana.creto.io/d/messaging-staging
- **Logs (Kibana):** https://kibana.creto.io/app/discover#/messaging
- **Status Page:** https://status.creto.io
- **PagerDuty:** https://creto.pagerduty.com
- **Runbook Repo:** https://github.com/creto/runbooks/tree/main/messaging
- **SDD Documentation:** /docs/sdd/products/messaging/

---

**Document Version:** 1.0
**Last Updated:** 2025-12-25
**Next Review:** 2026-01-25
