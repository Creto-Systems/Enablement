---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# SDD-MTR-07: Metering Operations Runbook

## Overview

This runbook provides operational procedures for **creto-metering**, including:

1. **Quota management**: Reset quotas, adjust limits, handle overages
2. **Billing reconciliation**: Fix billing errors, void invoices, apply credits
3. **Event replay/recovery**: Recover lost events, replay from backup
4. **Performance troubleshooting**: Diagnose slow queries, cache issues
5. **Incident response**: Handle security incidents, data breaches, outages

**Target Audience**: On-call engineers, SREs, billing operations team

---

## 1. Quota Management

### 1.1 Reset Quota for Subscription

**When**: Customer requests quota reset (e.g., after planned maintenance)

**Procedure**:
```sql
-- Step 1: Verify current quota usage
SELECT
    subscription_id,
    event_type,
    limit_value,
    period,
    current_usage
FROM quotas q
LEFT JOIN (
    SELECT subscription_id, event_type, SUM(event_count) AS current_usage
    FROM usage_hourly
    WHERE hour >= date_trunc('hour', now())
    GROUP BY subscription_id, event_type
) u USING (subscription_id, event_type)
WHERE subscription_id = '<SUBSCRIPTION_ID>';

-- Step 2: Clear usage counters (soft reset: keep historical data)
DELETE FROM usage_hourly
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND event_type = '<EVENT_TYPE>'
  AND hour >= date_trunc('hour', now());

-- Step 3: Invalidate quota cache
-- (Run via Redis CLI or admin API)
EVAL "return redis.call('DEL', unpack(redis.call('KEYS', 'quota:<SUBSCRIPTION_ID>:*')))" 0

-- Step 4: Verify quota reset
-- (Should show 0 usage)
```

**Post-Reset Verification**:
```bash
# Check quota via API
curl -H "Authorization: Bearer $TOKEN" \
  "https://api.creto.io/metering/v1/quotas/<AGENT_ID>?event_type=<EVENT_TYPE>"

# Expected response:
# {
#   "current_usage": 0,
#   "limit": 1000,
#   "remaining": 1000
# }
```

**Audit**:
- Log quota reset in audit trail
- Notify customer via email
- Record in Jira ticket

---

### 1.2 Adjust Quota Limit

**When**: Customer upgrades plan or requests temporary increase

**Procedure**:
```sql
-- Step 1: Verify current quota config
SELECT * FROM quotas
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND event_type = '<EVENT_TYPE>';

-- Step 2: Update quota limit
UPDATE quotas
SET limit_value = <NEW_LIMIT>,
    updated_at = now()
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND event_type = '<EVENT_TYPE>';

-- Step 3: Invalidate cache (same as quota reset)
EVAL "return redis.call('DEL', 'quota:<SUBSCRIPTION_ID>:<EVENT_TYPE>:*')" 0

-- Step 4: Log to audit
INSERT INTO audit_log (actor, action, resource, metadata)
VALUES (
    '<ADMIN_USER>',
    'quota.limit.updated',
    '<SUBSCRIPTION_ID>',
    jsonb_build_object(
        'event_type', '<EVENT_TYPE>',
        'old_limit', <OLD_LIMIT>,
        'new_limit', <NEW_LIMIT>
    )
);
```

**Notification**:
```bash
# Send email to customer
curl -X POST https://api.sendgrid.com/v3/mail/send \
  -H "Authorization: Bearer $SENDGRID_KEY" \
  -d '{
    "to": "customer@example.com",
    "subject": "Quota Limit Updated",
    "body": "Your quota for <EVENT_TYPE> has been increased to <NEW_LIMIT>."
  }'
```

---

### 1.3 Handle Quota Overage

**When**: Customer exceeded quota with `allow_with_overage` action

**Procedure**:
```sql
-- Step 1: Query overage usage
SELECT
    subscription_id,
    event_type,
    limit_value AS quota_limit,
    SUM(event_count) AS actual_usage,
    SUM(event_count) - limit_value AS overage
FROM quotas q
JOIN usage_hourly u USING (subscription_id, event_type)
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND hour >= '<PERIOD_START>'
  AND hour < '<PERIOD_END>'
GROUP BY subscription_id, event_type, limit_value
HAVING SUM(event_count) > limit_value;

-- Step 2: Calculate overage fee
-- Example: Overage rate = $0.10/unit (vs normal $0.02/unit)
SELECT
    overage * <OVERAGE_RATE> AS overage_charge
FROM (
    SELECT SUM(event_count) - limit_value AS overage
    FROM quotas q
    JOIN usage_hourly u USING (subscription_id, event_type)
    WHERE subscription_id = '<SUBSCRIPTION_ID>'
    GROUP BY limit_value
);

-- Step 3: Add overage line item to invoice
INSERT INTO invoice_line_items (invoice_id, description, quantity, unit_price, amount)
VALUES (
    '<INVOICE_ID>',
    'Overage: <EVENT_TYPE> (above quota)',
    <OVERAGE_QUANTITY>,
    <OVERAGE_RATE>,
    <OVERAGE_CHARGE>
);

-- Step 4: Update invoice total
UPDATE invoices
SET subtotal = subtotal + <OVERAGE_CHARGE>,
    total = subtotal + tax,
    updated_at = now()
WHERE invoice_id = '<INVOICE_ID>';
```

---

## 2. Billing Reconciliation

### 2.1 Fix Billing Error (Incorrect Charge)

**When**: Customer disputes charge, investigation confirms error

**Root Cause Analysis**:
```sql
-- Step 1: Retrieve disputed invoice
SELECT * FROM invoices WHERE invoice_id = '<INVOICE_ID>';

-- Step 2: Audit trail for invoice generation
SELECT * FROM audit_log
WHERE resource = '<INVOICE_ID>'
  AND action LIKE 'invoice.%'
ORDER BY timestamp DESC;

-- Step 3: Verify event data (check for duplicates, incorrect pricing)
SELECT
    event_id,
    idempotency_key,
    event_type,
    properties,
    timestamp
FROM events
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND timestamp >= '<PERIOD_START>'
  AND timestamp < '<PERIOD_END>'
ORDER BY timestamp DESC;
```

**Correction Procedure**:

**Option A: Void Invoice + Regenerate**
```sql
-- Step 1: Void incorrect invoice
UPDATE invoices
SET status = 'void',
    voided_at = now()
WHERE invoice_id = '<INVOICE_ID>';

-- Step 2: Delete incorrect line items
DELETE FROM invoice_line_items
WHERE invoice_id = '<INVOICE_ID>';

-- Step 3: Regenerate invoice with correct data
-- (Run billing service with corrected config)
```

**Option B: Issue Credit Note**
```sql
-- Step 1: Create credit line item (negative charge)
INSERT INTO invoice_line_items (invoice_id, description, quantity, unit_price, amount)
VALUES (
    '<NEXT_INVOICE_ID>',
    'Credit for billing error (Invoice <ORIGINAL_INVOICE_ID>)',
    1,
    -<ERROR_AMOUNT>,
    -<ERROR_AMOUNT>
);

-- Step 2: Notify customer
-- (Email with credit note details)
```

**Customer Communication**:
```
Subject: Billing Correction - Invoice <INVOICE_ID>

Dear Customer,

We identified an error in invoice <INVOICE_ID> dated <INVOICE_DATE>.

Error: <DESCRIPTION>
Original Charge: $<ORIGINAL_AMOUNT>
Correct Charge: $<CORRECT_AMOUNT>
Credit Applied: $<CREDIT_AMOUNT>

The credit will appear on your next invoice. We apologize for the inconvenience.
```

---

### 2.2 Apply Manual Credit

**When**: Customer goodwill credit, service outage compensation

**Procedure**:
```sql
-- Step 1: Create manual credit invoice
INSERT INTO invoices (subscription_id, period_start, period_end, subtotal, tax, total, status)
VALUES (
    '<SUBSCRIPTION_ID>',
    now(),
    now(),
    -<CREDIT_AMOUNT>,  -- Negative amount
    0,
    -<CREDIT_AMOUNT>,
    'issued'
);

-- Step 2: Add credit line item
INSERT INTO invoice_line_items (invoice_id, description, quantity, unit_price, amount)
VALUES (
    '<INVOICE_ID>',
    'Service credit: <REASON>',
    1,
    -<CREDIT_AMOUNT>,
    -<CREDIT_AMOUNT>
);

-- Step 3: Log to audit
INSERT INTO audit_log (actor, action, resource, metadata)
VALUES (
    '<ADMIN_USER>',
    'credit.applied',
    '<SUBSCRIPTION_ID>',
    jsonb_build_object(
        'amount', <CREDIT_AMOUNT>,
        'reason', '<REASON>',
        'invoice_id', '<INVOICE_ID>'
    )
);
```

---

### 2.3 Billing Reconciliation Report

**When**: Monthly close, financial audit

**Report Query**:
```sql
-- Summary of all invoices for period
WITH invoice_summary AS (
    SELECT
        DATE_TRUNC('month', period_start) AS billing_month,
        COUNT(*) AS total_invoices,
        SUM(total) AS total_revenue,
        SUM(CASE WHEN status = 'paid' THEN total ELSE 0 END) AS paid_revenue,
        SUM(CASE WHEN status = 'void' THEN total ELSE 0 END) AS voided_revenue
    FROM invoices
    WHERE period_start >= '<START_DATE>'
      AND period_end <= '<END_DATE>'
    GROUP BY DATE_TRUNC('month', period_start)
)
SELECT * FROM invoice_summary;

-- Disputed invoices
SELECT
    invoice_id,
    subscription_id,
    total,
    status,
    created_at
FROM invoices
WHERE status = 'disputed'
  AND created_at >= '<START_DATE>';

-- Revenue by metric
SELECT
    metric_code,
    SUM(quantity) AS total_quantity,
    SUM(amount) AS total_revenue
FROM invoice_line_items li
JOIN invoices i USING (invoice_id)
WHERE i.period_start >= '<START_DATE>'
  AND i.period_end <= '<END_DATE>'
  AND i.status != 'void'
GROUP BY metric_code
ORDER BY total_revenue DESC;
```

**Export to CSV**:
```bash
psql $DATABASE_URL -c "COPY (
    SELECT * FROM invoices
    WHERE period_start >= '2024-12-01'
      AND period_end <= '2024-12-31'
) TO STDOUT WITH CSV HEADER" > invoices_2024_12.csv
```

---

## 3. Event Replay & Recovery

### 3.1 Recover Lost Events from Kafka

**When**: Database failure caused event loss, but Kafka has durable copy

**Procedure**:
```bash
# Step 1: Identify time range of lost events
START_OFFSET=$(kafka-run-class kafka.tools.GetOffsetShell \
  --broker-list kafka:9092 \
  --topic events.ingestion \
  --time $(date -d '2024-12-25 10:00:00' +%s)000)

END_OFFSET=$(kafka-run-class kafka.tools.GetOffsetShell \
  --broker-list kafka:9092 \
  --topic events.ingestion \
  --time $(date -d '2024-12-25 12:00:00' +%s)000)

# Step 2: Replay events from Kafka to database
kafka-console-consumer \
  --bootstrap-server kafka:9092 \
  --topic events.ingestion \
  --from-beginning \
  --max-messages $((END_OFFSET - START_OFFSET)) \
  --offset $START_OFFSET \
| while read event; do
    # Reingest event via API (idempotency prevents duplicates)
    curl -X POST https://api.creto.io/metering/v1/events \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d "$event"
done
```

**Verification**:
```sql
-- Check event count before/after replay
SELECT COUNT(*) FROM events
WHERE timestamp >= '2024-12-25 10:00:00'
  AND timestamp <= '2024-12-25 12:00:00';
```

---

### 3.2 Replay Events from S3 Cold Storage

**When**: Need to regenerate historical invoices, data recovery

**Procedure**:
```bash
# Step 1: Download Parquet file from S3
aws s3 cp s3://creto-metering-cold-storage/events/year=2024/month=12/day=25/events.parquet.gz \
  /tmp/events.parquet.gz

gunzip /tmp/events.parquet.gz

# Step 2: Convert Parquet to JSON
parquet-tools cat /tmp/events.parquet --json > /tmp/events.json

# Step 3: Replay events
cat /tmp/events.json | jq -c '.' | while read event; do
    curl -X POST https://api.creto.io/metering/v1/events \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d "$event"
done
```

---

### 3.3 Backfill Missing Aggregations

**When**: Aggregation job failed, materialized views stale

**Procedure**:
```sql
-- Step 1: Identify missing aggregation windows
SELECT DISTINCT DATE_TRUNC('hour', timestamp) AS missing_hour
FROM events
WHERE timestamp >= '<START_DATE>'
  AND DATE_TRUNC('hour', timestamp) NOT IN (
      SELECT DISTINCT hour FROM usage_hourly
      WHERE subscription_id = '<SUBSCRIPTION_ID>'
  )
ORDER BY missing_hour;

-- Step 2: Backfill aggregations
INSERT INTO usage_hourly (subscription_id, event_type, hour, event_count, total_tokens)
SELECT
    subscription_id,
    event_type,
    DATE_TRUNC('hour', timestamp) AS hour,
    COUNT(*) AS event_count,
    SUM((properties->>'tokens')::bigint) AS total_tokens
FROM events
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND timestamp >= '<START_DATE>'
  AND timestamp < '<END_DATE>'
GROUP BY subscription_id, event_type, DATE_TRUNC('hour', timestamp)
ON CONFLICT (subscription_id, event_type, hour)
DO UPDATE SET
    event_count = EXCLUDED.event_count,
    total_tokens = EXCLUDED.total_tokens,
    updated_at = now();
```

---

## 4. Performance Troubleshooting

### 4.1 Slow Quota Checks

**Symptoms**:
- Quota check latency >100ms (p99)
- Authorization service timeouts
- Customer complaints about slow API

**Diagnosis**:
```sql
-- Check cache hit rate
SELECT
    COUNT(*) FILTER (WHERE cache_hit = true) AS cache_hits,
    COUNT(*) FILTER (WHERE cache_hit = false) AS cache_misses,
    (COUNT(*) FILTER (WHERE cache_hit = true))::float / COUNT(*) AS hit_rate
FROM quota_check_logs
WHERE timestamp >= now() - interval '1 hour';

-- Identify slow queries
SELECT
    agent_nhi,
    event_type,
    AVG(duration_ms) AS avg_latency,
    MAX(duration_ms) AS max_latency,
    COUNT(*) AS check_count
FROM quota_check_logs
WHERE timestamp >= now() - interval '1 hour'
  AND duration_ms > 100
GROUP BY agent_nhi, event_type
ORDER BY avg_latency DESC
LIMIT 10;
```

**Remediation**:

**Issue 1: Low Cache Hit Rate (<90%)**
```bash
# Increase Redis memory
redis-cli CONFIG SET maxmemory 4gb

# Adjust eviction policy (LRU)
redis-cli CONFIG SET maxmemory-policy allkeys-lru

# Preload hot quotas
psql $DATABASE_URL -c "
    SELECT subscription_id, event_type, limit_value
    FROM quotas
    WHERE active = true
" | redis-cli --pipe SET quota:<sub_id>:<event_type> '<quota_json>'
```

**Issue 2: Database Slow Queries**
```sql
-- Check for missing indexes
SELECT schemaname, tablename, indexname
FROM pg_indexes
WHERE tablename = 'quotas'
  AND schemaname = 'public';

-- Add missing index
CREATE INDEX CONCURRENTLY idx_quotas_subscription_type
ON quotas(subscription_id, event_type)
WHERE active = true;

-- Update table statistics
ANALYZE quotas;
```

---

### 4.2 Event Ingestion Backlog

**Symptoms**:
- Kafka consumer lag increasing
- Event ingestion latency >1s
- Database write throughput saturated

**Diagnosis**:
```bash
# Check Kafka consumer lag
kafka-consumer-groups \
  --bootstrap-server kafka:9092 \
  --group metering-ingesters \
  --describe

# Expected output:
# TOPIC              PARTITION  CURRENT-OFFSET  LOG-END-OFFSET  LAG
# events.ingestion   0          1000000         1050000         50000  # <-- Lag!

# Check database write throughput
psql $DATABASE_URL -c "
    SELECT
        schemaname,
        tablename,
        pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
        n_tup_ins AS inserts_since_startup,
        n_tup_upd AS updates_since_startup
    FROM pg_stat_user_tables
    WHERE tablename = 'events'
"
```

**Remediation**:

**Option 1: Scale Ingesters Horizontally**
```bash
# Increase Kafka consumer group instances
kubectl scale deployment metering-ingester --replicas=5
```

**Option 2: Batch Inserts**
```rust
// Instead of single inserts, batch 1000 events
async fn ingest_events_batched(events: Vec<Event>) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    for chunk in events.chunks(1000) {
        sqlx::query(
            "INSERT INTO events (event_id, subscription_id, event_type, timestamp, properties, signature)
             SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[], $4::timestamptz[], $5::jsonb[], $6::bytea[])"
        )
        .bind(&chunk.iter().map(|e| e.event_id).collect::<Vec<_>>())
        .bind(&chunk.iter().map(|e| e.subscription_id).collect::<Vec<_>>())
        .bind(&chunk.iter().map(|e| &e.event_type).collect::<Vec<_>>())
        .bind(&chunk.iter().map(|e| e.timestamp).collect::<Vec<_>>())
        .bind(&chunk.iter().map(|e| &e.properties).collect::<Vec<_>>())
        .bind(&chunk.iter().map(|e| &e.signature).collect::<Vec<_>>())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

---

### 4.3 Slow Invoice Generation

**Symptoms**:
- Invoice generation >5s (target: <1s)
- Month-end processing takes hours
- Timeouts in billing service

**Diagnosis**:
```sql
-- Profile invoice generation query
EXPLAIN (ANALYZE, BUFFERS)
SELECT
    event_type,
    SUM(event_count) AS total_usage
FROM usage_hourly
WHERE subscription_id = '<SUBSCRIPTION_ID>'
  AND hour >= '2024-12-01'
  AND hour < '2025-01-01'
GROUP BY event_type;
```

**Remediation**:

**Option 1: Precompute Monthly Aggregates**
```sql
-- Create monthly rollup table
CREATE TABLE usage_monthly AS
SELECT
    subscription_id,
    event_type,
    DATE_TRUNC('month', hour) AS month,
    SUM(event_count) AS event_count,
    SUM(total_tokens) AS total_tokens
FROM usage_hourly
GROUP BY subscription_id, event_type, DATE_TRUNC('month', hour);

CREATE INDEX idx_usage_monthly_lookup
ON usage_monthly(subscription_id, month);
```

**Option 2: Partition Invoices Table**
```sql
-- Partition invoices by year-month
CREATE TABLE invoices_2024_12 PARTITION OF invoices
FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');
```

---

## 5. Monitoring & Alerts

### 5.1 Key Metrics to Monitor

**Prometheus Metrics**:
```yaml
# Event ingestion
- metering_events_ingested_total{subscription_id}
- metering_event_ingestion_duration_seconds{quantile}
- metering_batch_size{quantile}

# Quota checks
- metering_quota_checks_total{event_type, decision}
- metering_quota_check_duration_micros{quantile, cache_layer}
- metering_quota_cache_hit_ratio

# Billing
- metering_invoices_generated_total{status}
- metering_invoice_generation_duration_seconds{quantile}

# Errors
- metering_signature_verification_failures_total
- metering_idempotency_conflicts_total
- metering_database_errors_total{operation}
```

### 5.2 Alert Rules

**Critical Alerts** (PagerDuty):
```yaml
groups:
  - name: metering_critical
    rules:
      - alert: QuotaCheckLatencyHigh
        expr: histogram_quantile(0.99, metering_quota_check_duration_micros) > 10000
        for: 5m
        annotations:
          summary: "Quota check p99 latency >10ms"

      - alert: EventIngestionBacklog
        expr: rate(metering_events_ingested_total[1m]) < 5000
        for: 10m
        annotations:
          summary: "Event ingestion rate dropped below 5K/s"

      - alert: DatabaseConnectionPoolExhausted
        expr: metering_db_connections_active / metering_db_connections_max > 0.9
        for: 2m
        annotations:
          summary: "Database connection pool >90% utilized"
```

**Warning Alerts** (Slack):
```yaml
  - name: metering_warnings
    rules:
      - alert: QuotaCacheHitRateLow
        expr: metering_quota_cache_hit_ratio < 0.90
        for: 10m
        annotations:
          summary: "Quota cache hit rate <90%"

      - alert: SignatureVerificationFailureSpike
        expr: rate(metering_signature_verification_failures_total[5m]) > 10
        for: 5m
        annotations:
          summary: "Signature verification failures >10/min"
```

---

## 6. Incident Response

### 6.1 Incident Severity Classification

| Severity | Description | Examples | Response Time |
|----------|-------------|----------|---------------|
| **SEV-1** | Service down, billing stopped | Database outage, API 5xx errors | <15 minutes |
| **SEV-2** | Degraded performance, quota failures | Slow quota checks, cache down | <1 hour |
| **SEV-3** | Minor issues, workaround available | Single subscription issue | <4 hours |
| **SEV-4** | Cosmetic, no customer impact | Typo in logs | <24 hours |

---

### 6.2 SEV-1: Database Outage

**Symptoms**:
- API returns 500 errors
- Event ingestion fails
- Quota checks timeout

**Response Procedure**:

**Phase 1: Immediate Mitigation (0-15 min)**
```bash
# Step 1: Promote PostgreSQL replica to primary
pg_ctl promote -D /var/lib/postgresql/data

# Step 2: Update DNS/connection string
kubectl set env deployment/metering-api DATABASE_URL=postgres://new-primary:5432/metering

# Step 3: Enable read-only mode (if writes still failing)
kubectl set env deployment/metering-api READ_ONLY_MODE=true
```

**Phase 2: Recovery (15 min - 4 hours)**
```bash
# Step 1: Restore failed primary database
pg_basebackup -D /var/lib/postgresql/data -R -X stream

# Step 2: Replay WAL from archive
pg_ctl start

# Step 3: Verify data integrity
psql -c "SELECT COUNT(*) FROM events WHERE created_at > now() - interval '1 hour'"

# Step 4: Restore write operations
kubectl set env deployment/metering-api READ_ONLY_MODE=false
```

**Phase 3: Post-Incident (4 hours+)**
- Root cause analysis
- Update runbook
- Improve monitoring

---

### 6.3 SEV-2: Quota Cache Failure

**Symptoms**:
- Quota check latency >500ms
- Redis connection errors
- Authorization service slow

**Response Procedure**:
```bash
# Step 1: Check Redis cluster health
redis-cli --cluster check redis-cluster:6379

# Step 2: Restart failed Redis nodes
kubectl rollout restart statefulset redis-cluster

# Step 3: Rebuild cache from database
psql $DATABASE_URL -c "
    SELECT subscription_id, event_type, limit_value, current_usage
    FROM quotas
    WHERE active = true
" | redis-cli --pipe SET quota:<sub_id>:<event_type>

# Step 4: Monitor recovery
watch 'redis-cli INFO | grep connected_clients'
```

---

## 7. Disaster Recovery Procedures

### 7.1 Restore from Backup

**Scenario**: Complete data loss (database corruption, ransomware)

**Recovery Steps**:
```bash
# Step 1: Restore PostgreSQL from last base backup
aws s3 cp s3://creto-metering-backups/base/backup_20241225.tar.gz /tmp/
tar -xzf /tmp/backup_20241225.tar.gz -C /var/lib/postgresql/data

# Step 2: Replay WAL to target point-in-time
recovery_target_time = '2024-12-25 14:30:00 UTC'
pg_ctl start

# Step 3: Verify data integrity
psql -c "SELECT MAX(created_at) FROM events"  # Should match recovery_target_time

# Step 4: Rebuild materialized views
psql -c "REFRESH MATERIALIZED VIEW CONCURRENTLY usage_hourly"

# Step 5: Repopulate Redis cache
# (Run cache rebuild script)
```

**RTO (Recovery Time Objective)**: <1 hour
**RPO (Recovery Point Objective)**: <5 minutes (WAL archiving)

---

## 8. Runbook Maintenance

**Update Frequency**: Monthly (or after major incidents)

**Change Log**:
| Date | Author | Changes |
|------|--------|---------|
| 2024-12-25 | Claude | Initial runbook creation |

**Review Schedule**:
- After each SEV-1 incident
- Quarterly review by SRE team
- Annual disaster recovery drill

---

**End of Runbook**

This runbook covers the most common operational scenarios for creto-metering. For issues not covered here, escalate to the on-call engineering team.
