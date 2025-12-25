# Metering Product - Component Diagram

## Overview

This component diagram illustrates the internal architecture of the Metering product, showing how it tracks AI agent resource usage, enforces quotas in real-time, aggregates events for billing, and integrates with external payment systems like Stripe.

## Purpose

- Detail the components within the Metering product container
- Show data flows from event ingestion to invoice generation
- Illustrate the three-tier quota enforcement strategy (Bloom → Redis → PostgreSQL)
- Visualize integration points with other Enablement products

## Diagram

```mermaid
graph TB
    subgraph External["External Systems"]
        Agent[AI Agent]
        Stripe[Stripe API]
        Audit[Audit Logger]
    end

    subgraph APIGateway["API Gateway Layer"]
        LB[Load Balancer]
        Gateway[API Gateway<br/>Kong/Envoy]
    end

    subgraph MeteringProduct["Metering Product Container"]

        subgraph Ingestion["Event Ingestion Layer"]
            EventAPI[Event Ingestion API<br/>gRPC Service]
            SchemaValidator[Schema Validator<br/>Protobuf/JSON Schema]
            Deduplicator[Event Deduplicator<br/>Bloom Filter]
            EventPublisher[Event Publisher<br/>Kafka Producer]
        end

        subgraph QuotaEnforcement["Quota Enforcement Layer"]
            QuotaAPI[Quota Enforcer API<br/>gRPC Service]
            BloomCheck[Bloom Filter Cache<br/>In-Memory, 0.1% FPR]
            RedisCheck[Redis Counter Cache<br/>TTL=3600s]
            PostgresCheck[PostgreSQL Quota Table<br/>Source of Truth]
            QuotaEngine[Quota Decision Engine<br/>CEL Expressions]
        end

        subgraph Aggregation["Aggregation Layer"]
            KafkaConsumer[Kafka Consumer<br/>Consumer Group]
            TimeBucketer[Time Bucketer<br/>Hourly/Daily/Monthly]
            AggregationEngine[Aggregation Engine<br/>Streaming SQL]
            DatabaseWriter[Database Writer<br/>Batch Inserter]
        end

        subgraph Billing["Billing Layer"]
            BillingAPI[Billing Service API<br/>gRPC Service]
            UsageCalculator[Usage Calculator<br/>SQL Queries]
            InvoiceGenerator[Invoice Generator<br/>Stripe SDK]
            ArchiveService[Archive Service<br/>S3 Uploader]
        end

        subgraph DataStores["Data Stores"]
            KafkaTopic[(Kafka Topic<br/>metering.events.v1<br/>30-day retention)]
            RedisCache[(Redis Cluster<br/>Quota Counters<br/>1-hour TTL)]
            PostgresDB[(PostgreSQL<br/>metering_events<br/>metering_quotas)]
            S3Bucket[(S3 Bucket<br/>billing-archives<br/>7-year retention)]
        end
    end

    %% External → API Gateway
    Agent -->|POST /metering/events<br/>Usage Event| LB
    Agent -->|GET /metering/quotas/{agent_id}<br/>Quota Check| LB
    LB -->|Route| Gateway

    %% API Gateway → Metering Components
    Gateway -->|gRPC: RecordEvent| EventAPI
    Gateway -->|gRPC: CheckQuota| QuotaAPI
    Gateway -->|gRPC: GetInvoice| BillingAPI

    %% Event Ingestion Flow
    EventAPI -->|1. Validate Schema| SchemaValidator
    SchemaValidator -->|2. Check Duplicate| Deduplicator
    Deduplicator -->|3. Publish if New| EventPublisher
    EventPublisher -->|4. Write Event| KafkaTopic

    %% Quota Enforcement Flow (Three-Tier Cache)
    QuotaAPI -->|1. Check Bloom| BloomCheck
    BloomCheck -->|2. Miss → Check Redis| RedisCheck
    RedisCheck -->|3. Miss → Check DB| PostgresCheck
    PostgresCheck -->|4. Evaluate Rules| QuotaEngine
    QuotaEngine -->|5. Allow/Deny| QuotaAPI

    %% Cache Warming
    PostgresCheck -.->|Warm Cache| RedisCheck
    RedisCheck -.->|Rebuild Bloom| BloomCheck

    %% Aggregation Flow
    KafkaConsumer -->|Consume Events| KafkaTopic
    KafkaConsumer -->|Time-bucket Events| TimeBucketer
    TimeBucketer -->|Aggregate Metrics| AggregationEngine
    AggregationEngine -->|Batch Write| DatabaseWriter
    DatabaseWriter -->|Insert Aggregated Data| PostgresDB

    %% Billing Flow
    BillingAPI -->|Query Usage| UsageCalculator
    UsageCalculator -->|Read Aggregated Data| PostgresDB
    UsageCalculator -->|Calculate Costs| InvoiceGenerator
    InvoiceGenerator -->|Create Invoice| Stripe
    InvoiceGenerator -->|Archive Record| ArchiveService
    ArchiveService -->|Upload JSON| S3Bucket

    %% Audit Logging
    EventAPI -.->|Log: Event Received| Audit
    QuotaAPI -.->|Log: Quota Decision| Audit
    BillingAPI -.->|Log: Invoice Generated| Audit

    %% Cross-Product Integration
    QuotaEngine -.->|Trigger Oversight<br/>Cost Threshold Alert| Gateway

    classDef api fill:#e1f5ff,stroke:#0066cc,stroke-width:2px
    classDef cache fill:#fff4e1,stroke:#ff9900,stroke-width:2px
    classDef db fill:#f0f0f0,stroke:#333,stroke-width:2px
    classDef processor fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px

    class EventAPI,QuotaAPI,BillingAPI api
    class BloomCheck,RedisCheck cache
    class KafkaTopic,RedisCache,PostgresDB,S3Bucket db
    class AggregationEngine,QuotaEngine processor
```

## Component Inventory

### Event Ingestion Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Event Ingestion API** | Go/gRPC | Receives usage events from agents, validates AuthZ |
| **Schema Validator** | Protobuf/JSON Schema | Validates event payload against schema version |
| **Event Deduplicator** | Bloom Filter (In-Memory) | Detects duplicate events using event_id hash |
| **Event Publisher** | Kafka Producer | Publishes validated events to Kafka topic |

**Key Metrics:**
- Throughput: 100,000 events/sec
- Latency: p99 < 10ms
- Duplicate rate: < 0.01%

### Quota Enforcement Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Quota Enforcer API** | Go/gRPC | Real-time quota checks with three-tier caching |
| **Bloom Filter Cache** | In-Memory (Probabilistic) | Fast negative lookups (0.1% false positive rate) |
| **Redis Counter Cache** | Redis Cluster | Hot quota counters with 1-hour TTL |
| **PostgreSQL Quota Table** | PostgreSQL | Source of truth for quota definitions and usage |
| **Quota Decision Engine** | CEL Expressions | Evaluates complex quota rules (time-based, hierarchical) |

**Three-Tier Cache Strategy:**
1. **Tier 1 (Bloom Filter)**: 95% of "no quota" requests rejected in <100ns
2. **Tier 2 (Redis)**: 90% of remaining requests served in <1ms
3. **Tier 3 (PostgreSQL)**: 10% of requests require DB lookup in <10ms

### Aggregation Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Kafka Consumer** | Kafka Consumer Group | Consumes events from `metering.events.v1` topic |
| **Time Bucketer** | Go/Time Windowing | Groups events by (agent_id, resource_type, time_bucket) |
| **Aggregation Engine** | Streaming SQL (ksqlDB) | Computes SUM, AVG, P99 metrics per time window |
| **Database Writer** | PostgreSQL Batch Inserter | Writes aggregated data in 1000-row batches |

**Aggregation Windows:**
- **Real-time**: 5-minute tumbling windows
- **Hourly**: 1-hour hopping windows (for billing)
- **Daily**: 24-hour tumbling windows (for dashboards)

### Billing Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Billing Service API** | Go/gRPC | Generates invoices, integrates with Stripe |
| **Usage Calculator** | SQL Queries | Queries aggregated usage from PostgreSQL |
| **Invoice Generator** | Stripe SDK | Creates invoices, line items, taxes |
| **Archive Service** | S3 SDK | Uploads billing records for 7-year retention |

**Invoice Generation Flow:**
1. Query aggregated usage for billing period
2. Apply pricing rules (tiered, volume discounts)
3. Calculate taxes based on customer location
4. Create Stripe invoice with line items
5. Archive invoice JSON to S3 (compliance)

## Data Flow Details

### Flow 1: Event Ingestion (Write Path)

```
Agent → API Gateway → Event Ingestion API
  ↓
Schema Validator (Protobuf validation)
  ↓
Event Deduplicator (Bloom filter check on event_id)
  ↓
Event Publisher (Kafka producer)
  ↓
Kafka Topic: metering.events.v1 (partitioned by agent_id)
```

**Event Schema:**
```protobuf
message UsageEvent {
  string event_id = 1;          // UUID v7 (time-sortable)
  string agent_id = 2;          // Agent public key hash
  string resource_type = 3;     // "llm_tokens", "sandbox_seconds", etc.
  int64 quantity = 4;           // Numeric quantity consumed
  google.protobuf.Timestamp timestamp = 5;
  map<string, string> metadata = 6;  // Tags, labels
  bytes delegation_chain_hmac = 7;   // AuthZ proof
}
```

### Flow 2: Quota Enforcement (Read Path)

```
Agent → API Gateway → Quota Enforcer API
  ↓
Tier 1: Bloom Filter (in-memory, <100ns)
  → Hit: Return ALLOW (95% of "no quota" cases)
  → Miss: Check Tier 2
  ↓
Tier 2: Redis (distributed cache, <1ms)
  → Hit: Return quota status (90% of remaining)
  → Miss: Check Tier 3
  ↓
Tier 3: PostgreSQL (source of truth, <10ms)
  → Read quota definition + current usage
  → Evaluate CEL rules (time-based, hierarchical)
  → Cache result in Redis + Bloom
  ↓
Return ALLOW/DENY to agent
```

**Quota Decision Logic:**
```cel
// CEL expression evaluated by Quota Engine
(agent.monthly_tokens + request.tokens) <= quota.monthly_limit &&
(agent.hourly_tokens + request.tokens) <= quota.hourly_limit &&
agent.payment_status == "active"
```

### Flow 3: Event Aggregation (Batch Processing)

```
Kafka Topic: metering.events.v1
  ↓
Kafka Consumer (consumer group, 8 partitions)
  ↓
Time Bucketer (tumbling window: 1 hour)
  ↓
Aggregation Engine (streaming SQL)
  SELECT agent_id, resource_type, date_trunc('hour', timestamp),
         SUM(quantity), AVG(quantity), percentile_cont(0.99)
  GROUP BY agent_id, resource_type, hour
  ↓
Database Writer (batch insert 1000 rows)
  ↓
PostgreSQL: metering_events (partitioned by time)
```

**Database Schema:**
```sql
CREATE TABLE metering_events (
  id BIGSERIAL,
  agent_id TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  time_bucket TIMESTAMPTZ NOT NULL,
  sum_quantity BIGINT NOT NULL,
  avg_quantity DOUBLE PRECISION,
  p99_quantity DOUBLE PRECISION,
  event_count INT,
  PRIMARY KEY (time_bucket, agent_id, resource_type)
) PARTITION BY RANGE (time_bucket);
```

### Flow 4: Billing & Invoicing

```
Billing Service API (triggered monthly cron)
  ↓
Usage Calculator
  SELECT agent_id, SUM(sum_quantity)
  FROM metering_events
  WHERE time_bucket BETWEEN start_date AND end_date
  GROUP BY agent_id
  ↓
Invoice Generator
  - Apply pricing rules (tiered: $0.01/1K tokens for 0-1M, $0.008/1K for 1M+)
  - Calculate taxes (Stripe Tax API)
  - Create Stripe invoice
  ↓
Archive Service
  - Serialize invoice to JSON
  - Upload to S3: s3://billing-archives/{year}/{month}/{agent_id}.json
  ↓
Return invoice_id to caller
```

## Implementation Considerations

### Performance Optimization

**Bloom Filter Tuning:**
- Size: 10M bits (1.25 MB memory)
- Hash functions: 7 (optimal for 0.1% FPR)
- Rebuild frequency: Every 5 minutes
- Expected rejection rate: 95% of "no quota" requests

**Redis Caching Strategy:**
- Key pattern: `quota:{agent_id}:{resource_type}`
- Value: JSON object with `{current_usage, limit, last_reset}`
- TTL: 3600s (1 hour)
- Eviction policy: LRU
- Cluster: 6 nodes (3 primary + 3 replica)

**PostgreSQL Partitioning:**
- Partitioning key: `time_bucket` (RANGE partitioning)
- Partition size: 1 week per partition
- Retention: 13 months (52 weekly partitions + 1 overflow)
- Index: `(agent_id, resource_type, time_bucket)`

### Scalability

**Horizontal Scaling:**
- Event Ingestion API: 10 replicas (Kubernetes HPA on CPU > 70%)
- Quota Enforcer API: 8 replicas (HPA on request rate > 1000 rps)
- Aggregation Engine: 8 Kafka consumer instances (1 per partition)

**Vertical Scaling:**
- PostgreSQL: db.r6g.2xlarge (8 vCPU, 64 GB RAM)
- Redis: cache.r6g.xlarge (4 vCPU, 26 GB RAM)
- Kafka: kafka.m5.2xlarge (8 vCPU, 32 GB RAM)

### Resilience

**Kafka Consumer Group:**
- Consumer lag monitoring: Alert if lag > 10,000 messages
- Auto-rebalancing: On consumer failure, partitions reassigned
- Offset commit strategy: At-least-once (commit after DB write)

**Database Failover:**
- PostgreSQL: Patroni for automatic failover (<30s RTO)
- Redis: Redis Sentinel for automatic promotion
- Connection pooling: PgBouncer (1000 connections → 20 DB connections)

**Circuit Breakers:**
- Stripe API: Open after 5 consecutive failures, half-open after 30s
- PostgreSQL: Open after 10 failures, fallback to Redis-only quota checks

### Security

**AuthZ Integration:**
- Every event ingestion validates delegation chain HMAC
- Quota checks verify agent identity before allowing access
- Audit logs record all quota denials with reason codes

**Data Encryption:**
- At rest: PostgreSQL TDE, S3 SSE-KMS
- In transit: TLS 1.3 for all gRPC, mTLS between services
- Secrets: Stripe API keys in AWS Secrets Manager

### Monitoring & Alerting

**Key Metrics:**
- `metering_events_received_total` (counter)
- `metering_quota_checks_total{result="allow|deny"}` (counter)
- `metering_kafka_consumer_lag` (gauge)
- `metering_aggregation_latency_seconds` (histogram)
- `metering_invoice_generation_duration_seconds` (histogram)

**Alerts:**
- Kafka consumer lag > 10,000: Page on-call
- Quota check p99 > 50ms: Warning
- Event ingestion error rate > 1%: Critical
- Stripe API error rate > 5%: Page on-call

## Integration Points

### Cross-Product Triggers

**Metering → Oversight:**
- **Cost threshold alerts**: When agent exceeds 80% of monthly budget, create oversight request
- **Quota exhaustion**: When quota fully consumed, trigger approval workflow for increase

**Metering → Runtime:**
- **Resource limits**: Sandbox CPU/memory quotas fetched from Metering quota engine
- **Egress billing**: Runtime reports network egress bytes to Metering

**Metering → Messaging:**
- **Message volume billing**: Messaging reports encrypted message count to Metering

### External Integrations

**Stripe API:**
- Endpoints: `/v1/invoices`, `/v1/customers`, `/v1/prices`
- Authentication: Bearer token (API key)
- Retry policy: 3 retries with exponential backoff
- Webhook: Receive payment success/failure events

**Audit Logger:**
- All quota denials logged with reason code
- Invoice generation events logged with Stripe invoice ID
- Schema validation failures logged for debugging

## Related Diagrams

- [C4 Context Diagram](./c4-context.md) - System-level context
- [C4 Container Diagram](./c4-container.md) - Container-level architecture
- [Data Flow Matrix](./data-flow-matrix.md) - Cross-product data flows

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-25 | Initial Metering component diagram for Issue #61 |
