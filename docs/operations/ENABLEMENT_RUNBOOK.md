---
status: draft
author: Creto Operations Team
created: 2024-12-26
updated: 2024-12-26
reviewers: []
---

# Enablement Layer Operations Runbook

## Document Purpose

This runbook provides comprehensive day-2 operations procedures for the Enablement Layer, covering deployment, monitoring, troubleshooting, and incident response across all four core products: Metering, Oversight, Runtime, and Messaging.

**Target Audience**: On-call SREs, Operations Engineers, Platform Engineers

**Related Documents**:
- Architecture: [/docs/sdd/02-architecture.md](../sdd/02-architecture.md)
- Deployment: [/docs/sdd/07-deployment-design.md](../sdd/07-deployment-design.md)
- Observability: [/docs/decisions/ADR-009-observability-stack.md](../decisions/ADR-009-observability-stack.md)

---

## Table of Contents

1. [Service Overview](#1-service-overview)
2. [Deployment Procedures](#2-deployment-procedures)
3. [Monitoring & Alerting](#3-monitoring--alerting)
4. [Common Operational Tasks](#4-common-operational-tasks)
5. [Incident Response](#5-incident-response)
6. [Troubleshooting Guides](#6-troubleshooting-guides)
7. [Disaster Recovery](#7-disaster-recovery)
8. [Security Operations](#8-security-operations)
9. [Performance Tuning](#9-performance-tuning)
10. [Runbook Checklists](#10-runbook-checklists)

---

## 1. Service Overview

### 1.1 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         KUBERNETES CLUSTER                               │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     creto-enablement Namespace                       │ │
│  │                                                                      │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │ │
│  │  │  Metering    │  │  Oversight   │  │   Runtime    │  │Messaging │ │ │
│  │  │  (3 pods)    │  │  (3 pods)    │  │  (3 pods)    │  │ (3 pods) │ │ │
│  │  │              │  │              │  │              │  │          │ │ │
│  │  │  gRPC:50051  │  │  gRPC:50052  │  │  gRPC:50053  │  │gRPC:50054│ │ │
│  │  │  Metrics:9090│  │  Metrics:9091│  │  Metrics:9092│  │Mtx:9093  │ │ │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘ │ │
│  │         │                 │                 │               │       │ │
│  │  ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐  ┌────▼─────┐ │ │
│  │  │   Service   │   │   Service   │   │   Service   │  │ Service  │ │ │
│  │  │ (ClusterIP) │   │ (ClusterIP) │   │ (ClusterIP) │  │(ClusterIP)│ │ │
│  │  └─────────────┘   └─────────────┘   └─────────────┘  └──────────┘ │ │
│  │                                                                      │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                        Ingress / API Gateway                        │ │
│  │                      (Istio / Kong / Envoy)                         │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                          Data Stores                                 │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │ │
│  │  │   Redis     │  │  PostgreSQL │  │   NATS      │  │  S3/MinIO   │ │ │
│  │  │  (StatefulS)│  │ (Operator)  │  │ (Operator)  │  │  (External) │ │ │
│  │  │  Port:6379  │  │  Port:5432  │  │  Port:4222  │  │             │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                    Observability Stack                               │ │
│  │  ┌──────────┐  ┌─────────┐  ┌────────┐  ┌────────┐  ┌────────────┐ │ │
│  │  │Prometheus│  │  Loki   │  │ Jaeger │  │Grafana │  │Alertmanager│ │ │
│  │  └──────────┘  └─────────┘  └────────┘  └────────┘  └────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Component Inventory

| Component | Type | Instances | Resources | Health Check |
|-----------|------|-----------|-----------|--------------|
| **creto-metering** | Deployment | 3 | CPU: 100m-1000m, RAM: 128Mi-512Mi | gRPC:50051, /metrics:9090 |
| **creto-oversight** | Deployment | 3 | CPU: 100m-1000m, RAM: 256Mi-1Gi | gRPC:50052, /metrics:9091 |
| **creto-runtime** | Deployment | 3 | CPU: 500m-2000m, RAM: 1Gi-4Gi | gRPC:50053, /metrics:9092 |
| **creto-messaging** | Deployment | 3 | CPU: 200m-1000m, RAM: 256Mi-1Gi | gRPC:50054, /metrics:9093 |
| **Redis** | StatefulSet | 3 (HA) | CPU: 500m-2000m, RAM: 2Gi-8Gi | Port 6379 |
| **PostgreSQL** | StatefulSet | 3 (HA) | CPU: 1000m-4000m, RAM: 4Gi-16Gi | Port 5432 |
| **NATS** | StatefulSet | 3 (Cluster) | CPU: 200m-1000m, RAM: 512Mi-2Gi | Port 4222 |

### 1.3 Dependencies

#### Internal Dependencies (Platform/Security Layers)

| Service | Purpose | Endpoint | Criticality |
|---------|---------|----------|-------------|
| **creto-authz** | Authorization (168ns) | grpc://authz.creto.local:50051 | **CRITICAL** |
| **creto-audit** | Audit logging | grpc://audit.creto.local:50052 | **HIGH** |
| **creto-nhi** | Agent identity | grpc://nhi.creto.local:50053 | **CRITICAL** |
| **creto-crypto** | Crypto-agile primitives | grpc://crypto.creto.local:50054 | **HIGH** |
| **creto-consensus** | Timestamp/ordering | grpc://consensus.creto.local:50055 | **HIGH** |
| **creto-memory** | Vector memory | grpc://memory.creto.local:50056 | **MEDIUM** |

#### External Dependencies

| Service | Purpose | Endpoint | Criticality |
|---------|---------|----------|-------------|
| **Slack API** | Oversight notifications | https://hooks.slack.com/services/* | **MEDIUM** |
| **Email (SMTP)** | Oversight notifications | smtp://smtp.creto.local:587 | **MEDIUM** |
| **S3/MinIO** | Object storage | s3://enablement-* | **HIGH** |
| **gVisor** | Runtime isolation | Local (kernel) | **CRITICAL** |
| **Kata Containers** | Runtime isolation (alt) | Local (QEMU) | **MEDIUM** |

### 1.4 SLA Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Availability** | 99.9% | Monthly uptime (43.2 min downtime/month) |
| **Latency (P95)** | <100ms | API request duration |
| **Latency (P99)** | <500ms | API request duration |
| **Error Rate** | <0.1% | Failed requests / total requests |
| **MTTD** | <5 min | Mean time to detect critical incidents |
| **MTTR** | <15 min | Mean time to resolve critical incidents |

---

## 2. Deployment Procedures

### 2.1 Kubernetes Manifests Overview

**Manifest Locations**:
```
k8s/
├── base/                          # Base Kustomize resources
│   ├── metering/
│   ├── oversight/
│   ├── runtime/
│   └── messaging/
├── overlays/
│   ├── dev/                       # Development environment
│   ├── staging/                   # Staging environment
│   └── prod/                      # Production environment
└── helm/                          # Helm charts (alternative)
    └── creto-enablement/
```

**Key Resources per Service**:
- `deployment.yaml` - Pod specification
- `service.yaml` - ClusterIP service
- `configmap.yaml` - Environment configuration
- `secret.yaml` - Sealed secrets
- `hpa.yaml` - Horizontal Pod Autoscaler
- `pdb.yaml` - Pod Disruption Budget
- `servicemonitor.yaml` - Prometheus scraping

### 2.2 Helm Chart Configuration

**Chart Structure**:
```bash
helm/creto-enablement/
├── Chart.yaml
├── values.yaml                 # Default values
├── values-dev.yaml
├── values-staging.yaml
├── values-prod.yaml
└── templates/
    ├── metering/
    ├── oversight/
    ├── runtime/
    └── messaging/
```

**Install Chart**:
```bash
# Add Helm repository (if published)
helm repo add creto https://charts.creto.io
helm repo update

# Install to staging
helm install creto-enablement creto/creto-enablement \
  --namespace creto-enablement-staging \
  --create-namespace \
  --values helm/creto-enablement/values-staging.yaml

# Verify installation
helm status creto-enablement -n creto-enablement-staging
kubectl get pods -n creto-enablement-staging
```

**Upgrade Chart**:
```bash
# Upgrade with new values
helm upgrade creto-enablement creto/creto-enablement \
  --namespace creto-enablement-staging \
  --values helm/creto-enablement/values-staging.yaml \
  --reuse-values

# Rollback if needed
helm rollback creto-enablement 1 -n creto-enablement-staging
```

### 2.3 Rolling Update Procedures

**Pre-Deployment Checklist**:
- [ ] All tests passing in CI
- [ ] Image signed with Cosign
- [ ] SBOM generated
- [ ] Security scan passed (Trivy/Snyk)
- [ ] Staging deployment successful
- [ ] Load test passed on staging

**Rolling Update Command**:
```bash
# Update image tag in Kustomize overlay
cd k8s/overlays/prod
kustomize edit set image gcr.io/creto-prod/enablement/metering:v1.2.0

# Apply changes
kubectl apply -k k8s/overlays/prod

# Watch rollout
kubectl rollout status deployment/creto-metering -n creto-enablement-prod

# Monitor pods
kubectl get pods -n creto-enablement-prod -l app.kubernetes.io/name=creto-metering -w
```

**Rollout Configuration**:
```yaml
# In deployment.yaml
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1  # At most 1 pod down during update
      maxSurge: 1        # At most 4 pods total during update
```

**Rollback Procedure**:
```bash
# View rollout history
kubectl rollout history deployment/creto-metering -n creto-enablement-prod

# Rollback to previous version
kubectl rollout undo deployment/creto-metering -n creto-enablement-prod

# Rollback to specific revision
kubectl rollout undo deployment/creto-metering -n creto-enablement-prod --to-revision=3
```

### 2.4 Blue-Green Deployment

**Use Case**: Zero-downtime deployments with instant rollback capability

**Procedure**:
```bash
# Step 1: Deploy green environment
kubectl apply -f k8s/blue-green/green-deployment.yaml -n creto-enablement-prod

# Step 2: Wait for green pods to be ready
kubectl wait --for=condition=ready pod \
  -l app.kubernetes.io/name=creto-metering,version=green \
  -n creto-enablement-prod \
  --timeout=300s

# Step 3: Run smoke tests against green
curl -H "Host: green.metering.creto.local" \
  https://api.creto.io/metering/v1/health

# Step 4: Switch traffic to green
kubectl patch service creto-metering -n creto-enablement-prod \
  -p '{"spec":{"selector":{"version":"green"}}}'

# Step 5: Monitor for 10 minutes
watch -n 5 'kubectl get pods -n creto-enablement-prod -l app.kubernetes.io/name=creto-metering'

# Step 6: If stable, delete blue deployment
kubectl delete deployment creto-metering-blue -n creto-enablement-prod
```

**Rollback (if issues detected)**:
```bash
# Immediate rollback to blue
kubectl patch service creto-metering -n creto-enablement-prod \
  -p '{"spec":{"selector":{"version":"blue"}}}'

# Delete green deployment
kubectl delete deployment creto-metering-green -n creto-enablement-prod
```

### 2.5 Canary Release Process

**Use Case**: Gradual rollout to subset of users, validate before full deployment

**Procedure**:
```bash
# Step 1: Deploy canary with 10% traffic
kubectl apply -f k8s/canary/canary-deployment.yaml -n creto-enablement-prod

# Step 2: Configure Istio VirtualService for traffic split
kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: creto-metering-canary
  namespace: creto-enablement-prod
spec:
  hosts:
    - creto-metering
  http:
    - match:
        - headers:
            canary:
              exact: "true"
      route:
        - destination:
            host: creto-metering
            subset: canary
    - route:
        - destination:
            host: creto-metering
            subset: stable
          weight: 90
        - destination:
            host: creto-metering
            subset: canary
          weight: 10
EOF

# Step 3: Monitor canary metrics
kubectl logs -f -l app.kubernetes.io/name=creto-metering,version=canary \
  -n creto-enablement-prod

# Step 4: Gradually increase traffic (20%, 50%, 100%)
kubectl patch virtualservice creto-metering-canary -n creto-enablement-prod \
  --type merge -p '{"spec":{"http":[{"route":[{"destination":{"subset":"stable"},"weight":80},{"destination":{"subset":"canary"},"weight":20}]}]}}'

# Step 5: Promote canary to stable
kubectl patch deployment creto-metering -n creto-enablement-prod \
  --type merge -p '{"spec":{"template":{"metadata":{"labels":{"version":"canary"}}}}}'

# Step 6: Delete canary resources
kubectl delete virtualservice creto-metering-canary -n creto-enablement-prod
kubectl delete deployment creto-metering-canary -n creto-enablement-prod
```

**Automated Canary with Flagger**:
```yaml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: creto-metering
  namespace: creto-enablement-prod
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: creto-metering
  service:
    port: 50051
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
      - name: request-success-rate
        thresholdRange:
          min: 99
        interval: 1m
      - name: request-duration
        thresholdRange:
          max: 500
        interval: 1m
  webhooks:
    - name: load-test
      url: http://flagger-loadtester.creto-system/
      timeout: 5s
      metadata:
        cmd: "hey -z 1m -q 10 -c 2 http://creto-metering-canary:50051/health"
```

---

## 3. Monitoring & Alerting

### 3.1 Prometheus Metrics by Service

#### Metering Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `metering_events_total` | Counter | `subscription_id`, `event_type` | Total events ingested |
| `metering_events_invalid_total` | Counter | `reason` | Invalid events rejected |
| `metering_quota_check_duration_seconds` | Histogram | `action` | Quota check latency |
| `metering_quota_exceeded_total` | Counter | `subscription_id`, `event_type` | Quota exceeded events |
| `metering_aggregation_duration_seconds` | Histogram | `aggregation_type` | Aggregation job duration |
| `metering_invoice_generation_total` | Counter | `status` | Invoices generated |
| `metering_redis_operations_total` | Counter | `operation`, `status` | Redis ops (get/set/del) |
| `metering_postgres_queries_total` | Counter | `query_type`, `status` | PostgreSQL queries |

**PromQL Examples**:
```promql
# Events ingestion rate
rate(metering_events_total[5m])

# P99 quota check latency
histogram_quantile(0.99, rate(metering_quota_check_duration_seconds_bucket[5m]))

# Error rate
rate(metering_events_invalid_total[5m]) / rate(metering_events_total[5m])
```

#### Oversight Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `oversight_requests_total` | Counter | `agent_id`, `policy_id` | Oversight requests created |
| `oversight_responses_total` | Counter | `decision`, `channel` | Human responses (approve/deny) |
| `oversight_pending_requests` | Gauge | `priority` | Current pending requests |
| `oversight_timeout_total` | Counter | `policy_id` | Requests timed out |
| `oversight_notification_duration_seconds` | Histogram | `channel` | Notification delivery time |
| `oversight_state_transitions_total` | Counter | `from_state`, `to_state` | State machine transitions |

**PromQL Examples**:
```promql
# Pending requests by priority
oversight_pending_requests{priority="high"}

# Response time distribution
histogram_quantile(0.95, rate(oversight_notification_duration_seconds_bucket{channel="slack"}[5m]))

# Approval rate
rate(oversight_responses_total{decision="approved"}[1h]) / rate(oversight_responses_total[1h])
```

#### Runtime Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `runtime_sandbox_spawns_total` | Counter | `backend`, `status` | Sandbox spawn attempts |
| `runtime_sandbox_spawn_duration_seconds` | Histogram | `backend`, `pool_type` | Spawn latency (cold/warm) |
| `runtime_sandbox_active` | Gauge | `backend` | Currently active sandboxes |
| `runtime_warmpool_size` | Gauge | `backend` | Warm pool current size |
| `runtime_warmpool_claims_total` | Counter | `backend` | Warm pool claims |
| `runtime_egress_checks_total` | Counter | `decision` | Network egress authz checks |
| `runtime_attestation_generation_duration_seconds` | Histogram | - | Attestation generation time |

**PromQL Examples**:
```promql
# Warm pool utilization
runtime_warmpool_claims_total / runtime_warmpool_size

# Cold start rate
rate(runtime_sandbox_spawns_total{pool_type="cold"}[5m])

# P95 warm pool claim latency
histogram_quantile(0.95, rate(runtime_sandbox_spawn_duration_seconds_bucket{pool_type="warm"}[5m]))
```

#### Messaging Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `messaging_messages_sent_total` | Counter | `sender_id`, `status` | Messages sent |
| `messaging_messages_received_total` | Counter | `recipient_id` | Messages delivered |
| `messaging_encryption_duration_seconds` | Histogram | `algorithm` | Encryption latency |
| `messaging_delivery_duration_seconds` | Histogram | `status` | End-to-end delivery time |
| `messaging_queue_depth` | Gauge | `recipient_id` | Pending messages |
| `messaging_signature_verification_total` | Counter | `status` | Signature verifications |

**PromQL Examples**:
```promql
# Message throughput
rate(messaging_messages_sent_total[5m])

# Encryption overhead
histogram_quantile(0.95, rate(messaging_encryption_duration_seconds_bucket[5m]))

# Delivery success rate
rate(messaging_messages_received_total[5m]) / rate(messaging_messages_sent_total[5m])
```

### 3.2 Grafana Dashboard List

| Dashboard | Panels | Purpose | URL |
|-----------|--------|---------|-----|
| **Enablement Overview** | 12 | High-level health across all products | /d/enablement-overview |
| **Metering Dashboard** | 16 | Event ingestion, quota checks, billing | /d/metering |
| **Oversight Dashboard** | 10 | Pending requests, response times, channels | /d/oversight |
| **Runtime Dashboard** | 14 | Sandbox lifecycle, warm pool, egress | /d/runtime |
| **Messaging Dashboard** | 12 | Message flow, encryption, delivery | /d/messaging |
| **Golden Signals** | 8 | Latency, traffic, errors, saturation | /d/golden-signals |
| **Cost Attribution** | 6 | Per-customer resource usage | /d/cost-attribution |
| **Kubernetes Resources** | 10 | Pod health, CPU, memory, network | /d/k8s-resources |

**Dashboard Import**:
```bash
# Import dashboard from JSON
curl -X POST http://grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $GRAFANA_API_KEY" \
  -d @dashboards/enablement-overview.json

# Or via ConfigMap
kubectl create configmap grafana-dashboards \
  --from-file=dashboards/ \
  -n creto-observability
```

### 3.3 Alert Definitions with Thresholds

**Alertmanager Configuration**:
```yaml
# /etc/prometheus/alerts/enablement.yaml
groups:
  - name: enablement_critical
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          (
            sum(rate(grpc_server_handled_total{grpc_code!="OK",namespace="creto-enablement"}[5m]))
            /
            sum(rate(grpc_server_handled_total{namespace="creto-enablement"}[5m]))
          ) > 0.01
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate in {{ $labels.job }}"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 1%)"

      # Pod down
      - alert: PodDown
        expr: |
          kube_deployment_status_replicas_available{namespace="creto-enablement"} < 2
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "{{ $labels.deployment }} has less than 2 pods"
          description: "Only {{ $value }} pods available (expected: 3)"

      # High latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95,
            rate(grpc_server_handling_seconds_bucket{namespace="creto-enablement"}[5m])
          ) > 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High P95 latency in {{ $labels.job }}"
          description: "P95 latency is {{ $value }}s (threshold: 0.5s)"

      # Quota check slow
      - alert: QuotaCheckSlow
        expr: |
          histogram_quantile(0.99,
            rate(metering_quota_check_duration_seconds_bucket[5m])
          ) > 0.00001
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Quota checks exceeding 10µs target"
          description: "P99 quota check latency: {{ $value }}s"

      # Runtime warm pool depleted
      - alert: WarmPoolDepleted
        expr: runtime_warmpool_size < 5
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Warm pool size below minimum"
          description: "Warm pool size: {{ $value }} (minimum: 10)"

      # Oversight backlog
      - alert: OversightBacklog
        expr: oversight_pending_requests > 50
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High number of pending oversight requests"
          description: "Pending requests: {{ $value }} (threshold: 50)"

      # Database connection issues
      - alert: DatabaseConnectionFailures
        expr: |
          rate(postgres_connection_errors_total[5m]) > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "PostgreSQL connection failures"
          description: "Connection error rate: {{ $value }}/s"

      # Redis unavailable
      - alert: RedisDown
        expr: |
          redis_up{namespace="creto-enablement"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Redis instance down"
          description: "Redis at {{ $labels.instance }} is unavailable"

      # Disk usage high
      - alert: HighDiskUsage
        expr: |
          (
            node_filesystem_avail_bytes{mountpoint="/",namespace="creto-enablement"}
            /
            node_filesystem_size_bytes{mountpoint="/",namespace="creto-enablement"}
          ) < 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Disk usage >90% on {{ $labels.instance }}"
          description: "Available: {{ $value | humanizePercentage }}"

      # Memory pressure
      - alert: MemoryPressure
        expr: |
          (
            container_memory_working_set_bytes{namespace="creto-enablement"}
            /
            container_spec_memory_limit_bytes{namespace="creto-enablement"}
          ) > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Container {{ $labels.pod }} high memory usage"
          description: "Memory usage: {{ $value | humanizePercentage }}"

  - name: enablement_warnings
    interval: 60s
    rules:
      # Certificate expiring
      - alert: CertificateExpiring
        expr: |
          (cert_exporter_not_after - time()) / 86400 < 30
        labels:
          severity: warning
        annotations:
          summary: "Certificate expiring in <30 days"
          description: "{{ $labels.cn }} expires in {{ $value }} days"

      # Kafka lag
      - alert: KafkaLagHigh
        expr: |
          kafka_consumergroup_lag{topic="metering-events"} > 10000
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Kafka consumer lag high"
          description: "Lag: {{ $value }} messages"
```

### 3.4 PagerDuty Escalation Policies

**Alertmanager Routing**:
```yaml
# /etc/alertmanager/alertmanager.yml
global:
  resolve_timeout: 5m
  pagerduty_url: https://events.pagerduty.com/v2/enqueue

route:
  receiver: default
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    # Critical alerts -> PagerDuty
    - match:
        severity: critical
      receiver: pagerduty-critical
      continue: false

    # Warnings -> Slack
    - match:
        severity: warning
      receiver: slack-warnings
      continue: false

receivers:
  - name: default
    slack_configs:
      - api_url: https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXX
        channel: '#creto-ops'
        title: 'Alert: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: pagerduty-critical
    pagerduty_configs:
      - service_key: '<PAGERDUTY_SERVICE_KEY>'
        severity: 'critical'
        description: '{{ .GroupLabels.alertname }}: {{ .CommonAnnotations.summary }}'
        details:
          firing: '{{ .Alerts.Firing | len }}'
          resolved: '{{ .Alerts.Resolved | len }}'

  - name: slack-warnings
    slack_configs:
      - api_url: https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXX
        channel: '#creto-warnings'
        title: 'Warning: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
        color: 'warning'
```

**PagerDuty Escalation Policy**:
```
Level 1: On-call SRE (immediate)
  └─> If no ack in 5 min
Level 2: SRE Team Lead (5 min delay)
  └─> If no ack in 10 min
Level 3: Engineering Manager (15 min delay)
  └─> If no ack in 15 min
Level 4: VP Engineering (30 min delay)
```

**On-Call Schedule**:
```
Primary: Weekly rotation (Mon 9am - Mon 9am)
Secondary: Monthly rotation
Holidays: Separate schedule with double coverage
```

---

## 4. Common Operational Tasks

### 4.1 Scaling Services Up/Down

**Horizontal Scaling (Pods)**:
```bash
# Scale up metering to 5 replicas
kubectl scale deployment creto-metering -n creto-enablement-prod --replicas=5

# Verify scaling
kubectl get pods -n creto-enablement-prod -l app.kubernetes.io/name=creto-metering

# Check HPA status
kubectl get hpa creto-metering -n creto-enablement-prod

# Manually trigger HPA recalculation (if needed)
kubectl patch hpa creto-metering -n creto-enablement-prod \
  -p '{"spec":{"minReplicas":5,"maxReplicas":20}}'
```

**Vertical Scaling (Resources)**:
```bash
# Update resource limits
kubectl set resources deployment creto-metering -n creto-enablement-prod \
  --requests=cpu=200m,memory=256Mi \
  --limits=cpu=2000m,memory=1Gi

# Restart pods to apply new limits
kubectl rollout restart deployment creto-metering -n creto-enablement-prod
```

**Auto-Scaling Configuration**:
```yaml
# Adjust HPA thresholds
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: creto-metering
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: creto-metering
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70  # Scale when avg CPU >70%
    - type: Pods
      pods:
        metric:
          name: grpc_requests_per_second
        target:
          type: AverageValue
          averageValue: 1000  # Scale when >1000 RPS per pod
```

### 4.2 Database Maintenance

#### PostgreSQL Vacuum and Analyze

```bash
# Connect to PostgreSQL pod
kubectl exec -it postgres-0 -n creto-enablement-prod -- psql -U creto

# Manual VACUUM ANALYZE
VACUUM ANALYZE usage_hourly;
VACUUM ANALYZE quotas;
VACUUM ANALYZE invoices;

# Check table bloat
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) AS external_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

# VACUUM FULL (locks table, use during maintenance window)
VACUUM FULL usage_hourly;
```

**Automated Vacuum Configuration**:
```sql
-- Enable autovacuum (should be on by default)
ALTER TABLE usage_hourly SET (autovacuum_enabled = true);

-- Tune autovacuum thresholds
ALTER TABLE usage_hourly SET (
    autovacuum_vacuum_threshold = 50,
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_analyze_threshold = 50,
    autovacuum_analyze_scale_factor = 0.05
);
```

#### PostgreSQL Index Maintenance

```sql
-- Identify missing indexes
SELECT
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    seq_tup_read / seq_scan AS avg_seq_tup
FROM pg_stat_user_tables
WHERE seq_scan > 0
ORDER BY seq_tup_read DESC
LIMIT 20;

-- Rebuild bloated index
REINDEX INDEX CONCURRENTLY usage_hourly_subscription_idx;

-- Update statistics
ANALYZE usage_hourly;
```

#### PostgreSQL Backups

```bash
# Full database dump
kubectl exec postgres-0 -n creto-enablement-prod -- \
  pg_dump -U creto -Fc creto_metering > metering-$(date +%Y%m%d).dump

# Upload to S3
aws s3 cp metering-$(date +%Y%m%d).dump \
  s3://creto-backups/postgres/metering/

# Point-in-time recovery setup (WAL archiving)
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "SELECT pg_start_backup('manual-backup', true);"

# Rsync data directory
kubectl cp postgres-0:/var/lib/postgresql/data \
  ./postgres-backup-$(date +%Y%m%d) \
  -n creto-enablement-prod

kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "SELECT pg_stop_backup();"
```

#### Redis Maintenance

```bash
# Connect to Redis
kubectl exec -it redis-0 -n creto-enablement-prod -- redis-cli

# Manual save (RDB snapshot)
SAVE

# Background save (non-blocking)
BGSAVE

# Check memory usage
INFO memory

# Evict volatile keys (if memory pressure)
FLUSHDB ASYNC

# Replication status (for HA setup)
INFO replication

# Backup RDB file
kubectl cp redis-0:/data/dump.rdb \
  ./redis-backup-$(date +%Y%m%d).rdb \
  -n creto-enablement-prod
```

### 4.3 Log Rotation

**Kubernetes Log Rotation** (via logrotate in nodes):
```bash
# /etc/logrotate.d/kubernetes
/var/log/pods/*/*.log {
    daily
    rotate 7
    compress
    missingok
    notifempty
    create 0644 root root
    postrotate
        /usr/bin/docker kill -s USR1 $(docker ps -q) 2>/dev/null || true
    endscript
}
```

**Loki Log Retention**:
```yaml
# loki-config.yaml
table_manager:
  retention_deletes_enabled: true
  retention_period: 8760h  # 1 year

# S3 lifecycle policy (for older logs)
{
  "Rules": [{
    "Id": "logs-lifecycle",
    "Filter": {"Prefix": "loki/"},
    "Status": "Enabled",
    "Transitions": [
      {"Days": 30, "StorageClass": "GLACIER"}
    ],
    "Expiration": {"Days": 365}
  }]
}
```

### 4.4 Certificate Rotation

**TLS Certificate Rotation (cert-manager)**:
```bash
# Check certificate expiration
kubectl get certificate -n creto-enablement-prod

# Force renewal (if <30 days until expiry)
kubectl delete certificaterequest <cert-request-name> -n creto-enablement-prod

# cert-manager will auto-create new request

# Verify new certificate
kubectl describe certificate creto-enablement-tls -n creto-enablement-prod

# Restart pods to pick up new cert
kubectl rollout restart deployment -n creto-enablement-prod
```

**Manual Certificate Rotation**:
```bash
# Generate new certificate (if not using cert-manager)
openssl req -x509 -newkey rsa:4096 -keyout tls.key -out tls.crt \
  -days 365 -nodes -subj "/CN=*.creto.io"

# Update Kubernetes secret
kubectl create secret tls creto-enablement-tls \
  --cert=tls.crt --key=tls.key \
  -n creto-enablement-prod \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart ingress controller
kubectl rollout restart deployment ingress-nginx-controller -n ingress-nginx
```

### 4.5 Key Rotation (NHI Keys)

**NHI Agent Key Rotation**:
```bash
# Step 1: Generate new key pair via NHI service
curl -X POST https://nhi.creto.io/v1/agents/rotate-key \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"agent_id": "agent-12345"}'

# Response:
# {
#   "old_key_id": "key-abc123",
#   "new_key_id": "key-def456",
#   "transition_period": "2024-01-01T00:00:00Z to 2024-01-08T00:00:00Z"
# }

# Step 2: During transition, both keys are valid

# Step 3: After transition period, old key is revoked
curl -X POST https://nhi.creto.io/v1/keys/revoke \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"key_id": "key-abc123", "reason": "rotation"}'

# Step 4: Verify all agents using new key
curl https://nhi.creto.io/v1/keys/key-def456/usage
```

**Database Encryption Key Rotation**:
```bash
# PostgreSQL LUKS key rotation (for encrypted volumes)
kubectl exec postgres-0 -n creto-enablement-prod -- \
  cryptsetup luksChangeKey /dev/sda1

# Kubernetes Secrets encryption key rotation
# (Requires cluster admin access)
kubectl get secrets -A -o json | \
  kubectl replace -f -
```

---

## 5. Incident Response

### 5.1 Severity Classification

| Severity | Definition | Examples | Response Time |
|----------|------------|----------|---------------|
| **SEV-1** | Total service outage or data loss | All pods down, database corruption | <5 min |
| **SEV-2** | Major functionality degraded | Single product down, high latency | <15 min |
| **SEV-3** | Partial functionality impaired | Feature not working, elevated errors | <1 hour |
| **SEV-4** | Minor issue, no user impact | Metrics gap, log noise | <4 hours |

### 5.2 On-Call Procedures

**Incident Alert Workflow**:
```
1. Alert fires in Prometheus
   └─> Alertmanager routes to PagerDuty
       └─> On-call SRE receives page
           └─> Acknowledge within 5 minutes
               ├─> SEV-1: Immediate response
               │   └─> Create incident channel (#incident-YYYYMMDD-001)
               │   └─> Post initial status update
               │   └─> Engage additional responders
               │
               ├─> SEV-2: 15-minute response window
               │   └─> Investigate and triage
               │   └─> Escalate if needed
               │
               └─> SEV-3/4: Work during business hours
                   └─> Create ticket, schedule fix
```

**Initial Response Checklist**:
- [ ] Acknowledge alert in PagerDuty
- [ ] Check Grafana dashboards for symptoms
- [ ] Verify incident scope (single service vs. multi-service)
- [ ] Create incident Slack channel (SEV-1/2 only)
- [ ] Post initial status update
- [ ] Engage additional responders if needed
- [ ] Start incident timeline doc

### 5.3 Communication Templates

#### Initial Incident Notification (SEV-1/2)

**Slack #incident-YYYYMMDD-001**:
```
🚨 INCIDENT DECLARED - SEV-1

**Service**: creto-metering
**Impact**: Quota checks failing, all customers affected
**Detected**: 2024-12-26 14:32 UTC
**Status**: Investigating

**Responders**:
- Incident Commander: @alice
- Tech Lead: @bob
- SRE: @charlie

**Next Update**: 14:45 UTC (15 min)

**Customer Notification**: SENT to status page
```

#### Status Update Template

```
**UPDATE** (14:45 UTC)

**Current Status**: Root cause identified - Redis connection pool exhausted
**Action**: Scaling Redis from 3→6 pods
**ETA to Resolution**: 15:00 UTC
**Next Update**: 15:00 UTC
```

#### Resolution Notification

```
🟢 INCIDENT RESOLVED - SEV-1

**Service**: creto-metering
**Duration**: 28 minutes (14:32 - 15:00 UTC)
**Root Cause**: Redis connection pool exhaustion due to unexpected traffic spike
**Fix**: Scaled Redis pods 3→6, increased connection pool size

**Follow-Up Actions**:
- [ ] Post-incident review scheduled for 2024-12-27 10:00 UTC
- [ ] Update runbook with learnings
- [ ] Review HPA thresholds for Redis

**Timeline**: See incident doc (link)
```

### 5.4 Post-Incident Review Process

**Post-Incident Review Template**:
```markdown
# Post-Incident Review: SEV-1 Metering Outage (2024-12-26)

## Incident Summary
- **Severity**: SEV-1
- **Duration**: 28 minutes
- **Services Affected**: creto-metering (quota checks)
- **Customer Impact**: 100% of customers unable to check quotas
- **Financial Impact**: $0 (no SLA breach)

## Timeline
| Time (UTC) | Event |
|------------|-------|
| 14:32 | Alert: HighErrorRate (metering) |
| 14:33 | On-call SRE acknowledges, begins investigation |
| 14:38 | Root cause identified: Redis connection pool exhausted |
| 14:40 | Incident Commander declares SEV-1 |
| 14:45 | Fix applied: Scale Redis 3→6 pods |
| 14:55 | Error rate returns to normal |
| 15:00 | Incident resolved |

## Root Cause
Traffic spike from customer "acme-corp" running batch quota checks (10K req/s) exhausted Redis connection pool (max 100 connections). Metering service could not establish new connections, causing quota checks to fail.

## What Went Well
- Alert fired within 1 minute of error spike
- On-call SRE quickly identified Redis as bottleneck
- Fix applied within 13 minutes

## What Went Poorly
- No auto-scaling configured for Redis
- Connection pool size hardcoded (not tunable via config)
- No rate limiting per customer

## Action Items
| Item | Owner | Due Date | Status |
|------|-------|----------|--------|
| Implement HPA for Redis | @charlie | 2024-12-27 | ✅ Done |
| Make connection pool size configurable | @bob | 2024-12-30 | 🔄 In Progress |
| Add per-customer rate limiting | @alice | 2025-01-05 | 📋 Planned |
| Update runbook with Redis scaling procedure | @charlie | 2024-12-26 | ✅ Done |
```

**Post-Incident Review Meeting Agenda**:
1. Incident overview (5 min)
2. Timeline walkthrough (10 min)
3. Root cause analysis (10 min)
4. What went well / What went poorly (10 min)
5. Action items assignment (10 min)
6. Questions & discussion (15 min)

---

## 6. Troubleshooting Guides

### 6.1 High Latency Diagnosis

**Symptoms**:
- P95/P99 latency exceeds SLA targets
- User reports of slow API responses
- Grafana dashboard shows latency spike

**Diagnosis Steps**:

**Step 1: Identify Slow Component**
```bash
# Check Prometheus for latency by service
# In Grafana or Prometheus UI, run:
histogram_quantile(0.95,
  rate(grpc_server_handling_seconds_bucket{namespace="creto-enablement"}[5m])
) by (grpc_service)

# Expected output:
# creto-metering: 0.05s
# creto-oversight: 0.02s
# creto-runtime: 2.5s  ← High latency!
# creto-messaging: 0.01s
```

**Step 2: Check Dependencies (if Runtime is slow)**
```bash
# Check gRPC call latency to dependencies
histogram_quantile(0.95,
  rate(grpc_client_handling_seconds_bucket{grpc_service="creto.authz"}[5m])
)

# Check Redis latency
redis_command_duration_seconds{quantile="0.95"}

# Check PostgreSQL query duration
postgres_query_duration_seconds{quantile="0.95"}
```

**Step 3: Inspect Slow Traces (Jaeger)**
```bash
# Search Jaeger for slow requests
# UI: http://jaeger.creto.io
# Filter: service=creto-runtime, minDuration=2s

# Or via API:
curl "http://jaeger.creto.io/api/traces?service=creto-runtime&minDuration=2000000"
```

**Step 4: Analyze Logs (Loki)**
```bash
# Query slow requests in Loki
{namespace="creto-enablement", app="creto-runtime"}
  | json
  | duration > 2s
  | line_format "{{.timestamp}} {{.method}} {{.duration}} {{.error}}"
```

**Step 5: Profile Application (if code issue suspected)**
```bash
# Enable pprof endpoint (if not already exposed)
kubectl port-forward svc/creto-runtime 6060:6060 -n creto-enablement-prod

# Capture CPU profile
curl http://localhost:6060/debug/pprof/profile?seconds=30 > cpu.prof

# Analyze with pprof
go tool pprof -http=:8080 cpu.prof

# Or heap profile for memory issues
curl http://localhost:6060/debug/pprof/heap > heap.prof
go tool pprof -http=:8080 heap.prof
```

**Common Causes & Fixes**:

| Cause | Diagnosis | Fix |
|-------|-----------|-----|
| **Database slow query** | High `postgres_query_duration` | Add index, optimize query |
| **Redis cache miss** | High cache miss rate | Increase cache TTL, pre-warm cache |
| **Cold start (Runtime)** | High `runtime_sandbox_spawn_duration` (cold) | Increase warm pool size |
| **Network latency** | High gRPC client latency to authz/audit | Check network policies, DNS |
| **CPU saturation** | High `container_cpu_usage_seconds_total` | Scale up pods or increase CPU limits |
| **Garbage collection** | GC pauses in logs | Tune GC settings, increase heap |

### 6.2 Error Rate Spikes

**Symptoms**:
- `HighErrorRate` alert firing
- Grafana shows >1% error rate
- User reports of 5xx errors

**Diagnosis Steps**:

**Step 1: Identify Error Type**
```bash
# Check error distribution by status code
sum by (grpc_code) (
  rate(grpc_server_handled_total{namespace="creto-enablement", grpc_code!="OK"}[5m])
)

# Common codes:
# UNAUTHENTICATED: authz issue
# RESOURCE_EXHAUSTED: quota exceeded or rate limit
# UNAVAILABLE: dependency down
# INTERNAL: application bug
```

**Step 2: Check Recent Deployments**
```bash
# List recent rollouts
kubectl rollout history deployment/creto-metering -n creto-enablement-prod

# Check if error spike correlates with deployment
# In Grafana, overlay deployment annotations on error rate graph
```

**Step 3: Inspect Error Logs**
```bash
# Query error logs in Loki
{namespace="creto-enablement", level="error"}
  | json
  | line_format "{{.timestamp}} {{.service}} {{.error}} {{.trace_id}}"
  | __error__=""

# Count errors by type
sum by (error_type) (
  count_over_time({namespace="creto-enablement", level="error"} [5m])
)
```

**Step 4: Check Dependencies**
```bash
# Verify PostgreSQL connectivity
kubectl exec -it postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "SELECT 1;"

# Verify Redis connectivity
kubectl exec -it redis-0 -n creto-enablement-prod -- \
  redis-cli PING

# Check authz service health
curl http://authz.creto.local:50051/health
```

**Step 5: Review Recent Changes**
```bash
# Check git commits in last 24h
git log --since="24 hours ago" --oneline

# Check ConfigMap/Secret changes
kubectl diff -f k8s/overlays/prod/

# Check HPA scaling events
kubectl get events -n creto-enablement-prod --sort-by='.lastTimestamp' | grep HPA
```

**Common Causes & Fixes**:

| Error Type | Cause | Fix |
|------------|-------|-----|
| **UNAUTHENTICATED** | authz service down or policy issue | Check authz logs, verify policies |
| **RESOURCE_EXHAUSTED** | Quota exceeded | Adjust quota limits or investigate usage spike |
| **UNAVAILABLE** | Dependency (DB, Redis) down | Restart dependency, check resources |
| **INTERNAL** | Application bug (panic, nil pointer) | Rollback deployment, fix bug |
| **DEADLINE_EXCEEDED** | Timeout | Increase timeout, optimize slow operation |

### 6.3 Memory/CPU Issues

**Symptoms**:
- `MemoryPressure` alert firing
- Pods OOMKilled (evicted)
- High CPU usage, throttling

**Diagnosis Steps**:

**Step 1: Identify Resource-Constrained Pods**
```bash
# Check current resource usage
kubectl top pods -n creto-enablement-prod --sort-by=memory

# Check for OOMKilled pods
kubectl get pods -n creto-enablement-prod \
  -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.containerStatuses[0].lastState.terminated.reason}{"\n"}{end}' \
  | grep OOMKilled

# Check resource limits vs usage
kubectl describe pod <pod-name> -n creto-enablement-prod | grep -A 10 "Limits:"
```

**Step 2: Memory Leak Detection**
```bash
# Capture heap profile
kubectl exec -it <pod-name> -n creto-enablement-prod -- \
  curl http://localhost:6060/debug/pprof/heap > heap-$(date +%s).prof

# Compare heap over time (capture again after 5 min)
# Increasing heap usage indicates leak

# Analyze with pprof
go tool pprof -http=:8080 heap-<timestamp>.prof
# Look for "inuse_space" and "alloc_space" top consumers
```

**Step 3: CPU Profiling**
```bash
# Capture CPU profile (30 seconds)
kubectl exec -it <pod-name> -n creto-enablement-prod -- \
  curl "http://localhost:6060/debug/pprof/profile?seconds=30" > cpu-$(date +%s).prof

# Analyze with pprof
go tool pprof -http=:8080 cpu-<timestamp>.prof
# Look for hot functions
```

**Step 4: Check Goroutine Leaks (Rust equivalent: thread leaks)**
```bash
# Check goroutine count (if Go service)
kubectl exec -it <pod-name> -n creto-enablement-prod -- \
  curl http://localhost:6060/debug/pprof/goroutine?debug=1

# For Rust services, check thread count
kubectl exec -it <pod-name> -n creto-enablement-prod -- \
  ps -eLf | wc -l
```

**Common Fixes**:

| Issue | Fix |
|-------|-----|
| **OOMKilled** | Increase memory limits, optimize memory usage |
| **CPU Throttling** | Increase CPU limits, optimize hot paths |
| **Memory Leak** | Fix leak in code, restart pods as interim |
| **Goroutine/Thread Leak** | Fix leak (unclosed channels/handles), restart pods |

### 6.4 Database Connection Issues

**Symptoms**:
- `DatabaseConnectionFailures` alert
- Error logs: "connection refused", "too many connections"
- Queries timing out

**Diagnosis Steps**:

**Step 1: Check Connection Pool Status**
```sql
-- Connect to PostgreSQL
kubectl exec -it postgres-0 -n creto-enablement-prod -- psql -U creto

-- Check active connections
SELECT count(*) FROM pg_stat_activity;

-- Check max connections limit
SHOW max_connections;

-- Check connections by application
SELECT application_name, count(*)
FROM pg_stat_activity
GROUP BY application_name;

-- Check long-running queries (potential blockers)
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE state = 'active' AND now() - pg_stat_activity.query_start > interval '5 seconds';
```

**Step 2: Check PostgreSQL Logs**
```bash
# View PostgreSQL logs
kubectl logs postgres-0 -n creto-enablement-prod --tail=100

# Look for:
# - "too many connections"
# - "FATAL: remaining connection slots reserved"
# - "FATAL: no pg_hba.conf entry"
```

**Step 3: Check Application Connection Pool**
```bash
# Check connection pool metrics (if exposed)
# In Grafana or Prometheus:
postgres_pool_connections_active
postgres_pool_connections_idle
postgres_pool_connections_max

# Should see:
# active < max
# idle > 0 (pool has available connections)
```

**Step 4: Network Connectivity Test**
```bash
# Test from application pod to PostgreSQL
kubectl exec -it creto-metering-<pod-id> -n creto-enablement-prod -- \
  nc -zv postgres 5432

# Expected output: "Connection to postgres 5432 port [tcp/postgresql] succeeded!"

# Check DNS resolution
kubectl exec -it creto-metering-<pod-id> -n creto-enablement-prod -- \
  nslookup postgres
```

**Common Fixes**:

| Issue | Fix |
|-------|-----|
| **Too many connections** | Increase `max_connections` in PostgreSQL, reduce pool size per app |
| **Connection pool exhausted** | Increase app connection pool size, reduce query latency |
| **Long-running query blocking** | Kill blocking query: `SELECT pg_terminate_backend(pid);` |
| **Network policy blocking** | Update NetworkPolicy to allow pod→postgres traffic |

### 6.5 Kafka Lag Issues

**Symptoms**:
- `KafkaLagHigh` alert firing
- Delayed event processing
- Consumer group falling behind

**Diagnosis Steps**:

**Step 1: Check Consumer Lag**
```bash
# Using kafka-consumer-groups
kubectl exec -it kafka-0 -n creto-enablement-prod -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --group metering-consumer --describe

# Output:
# TOPIC          PARTITION  CURRENT-OFFSET  LOG-END-OFFSET  LAG
# metering-events    0          12500           22500        10000
# metering-events    1          12500           22500        10000

# Total lag: 20,000 messages
```

**Step 2: Check Consumer Processing Rate**
```bash
# Check message consumption rate
rate(kafka_consumer_records_consumed_total{group="metering-consumer"}[5m])

# Compare to production rate
rate(kafka_producer_records_sent_total{topic="metering-events"}[5m])

# If consumption < production, consumer is falling behind
```

**Step 3: Check Consumer Health**
```bash
# Check consumer pod CPU/memory
kubectl top pods -n creto-enablement-prod -l app=metering-consumer

# Check consumer logs for errors
kubectl logs -l app=metering-consumer -n creto-enablement-prod --tail=100
```

**Step 4: Check Kafka Broker Health**
```bash
# Check broker metrics
kubectl exec -it kafka-0 -n creto-enablement-prod -- \
  kafka-broker-api-versions.sh --bootstrap-server localhost:9092

# Check disk usage
kubectl exec -it kafka-0 -n creto-enablement-prod -- df -h /var/lib/kafka

# Check under-replicated partitions
kubectl exec -it kafka-0 -n creto-enablement-prod -- \
  kafka-topics.sh --bootstrap-server localhost:9092 --describe --under-replicated-partitions
```

**Common Fixes**:

| Issue | Fix |
|-------|-----|
| **Slow consumer processing** | Optimize consumer code, increase parallelism |
| **Too few consumer instances** | Scale consumer pods (up to partition count) |
| **Large messages** | Increase batch size, adjust `max.poll.records` |
| **Kafka broker overloaded** | Add more brokers, increase replication factor |

### 6.6 AuthZ Outage

**Symptoms**:
- Alert: `AuthzUnavailable` (Prometheus)
- All Enablement services returning 503 errors
- Logs showing `AuthzError::ServiceUnavailable`

**Immediate Actions**:
```bash
# 1. Verify AuthZ is actually down
kubectl -n creto-authz get pods
kubectl -n creto-authz logs -l app=creto-authz --tail=100

# 2. Check if it's a network partition
kubectl -n creto-enablement exec -it deploy/creto-metering -- \
  nc -zv creto-authz.creto-authz.svc.cluster.local 8080

# 3. If AuthZ pod is healthy but unreachable, check NetworkPolicy
kubectl -n creto-authz get networkpolicy

# 4. Check AuthZ dependencies (consensus, vault)
kubectl -n creto-consensus logs -l app=creto-consensus --tail=50
kubectl -n creto-vault logs -l app=creto-vault --tail=50
```

**Mitigation Steps**:

**Option A: Restart AuthZ pods (if crashed)**
```bash
kubectl -n creto-authz rollout restart deploy/creto-authz
kubectl -n creto-authz rollout status deploy/creto-authz --timeout=120s
```

**Option B: Scale up (if overloaded)**
```bash
kubectl -n creto-authz scale deploy/creto-authz --replicas=5
```

**Option C: Enable circuit breaker fallback (EMERGENCY ONLY)**
```bash
# This allows DENY-by-default fallback for 15 minutes
# REQUIRES SECURITY TEAM APPROVAL
kubectl -n creto-enablement set env deploy/creto-metering \
  AUTHZ_CIRCUIT_BREAKER_ENABLED=true \
  AUTHZ_FALLBACK_DECISION=DENY \
  AUTHZ_FALLBACK_TTL_SECONDS=900

# Apply to all Enablement services
for svc in oversight runtime messaging; do
  kubectl -n creto-enablement set env deploy/creto-$svc \
    AUTHZ_CIRCUIT_BREAKER_ENABLED=true \
    AUTHZ_FALLBACK_DECISION=DENY \
    AUTHZ_FALLBACK_TTL_SECONDS=900
done
```

**Verification**:
```bash
# Verify AuthZ is responding
curl -s http://creto-authz:8080/health | jq .

# Verify Enablement services recovered
for port in 9090 9091 9092 9093; do
  curl -s "http://localhost:$port/health" | jq .status
done

# Check error rate dropping
promtool query instant 'rate(authz_check_errors_total[5m])'
```

**Post-Incident**:
- Disable circuit breaker fallback if enabled
- Review AuthZ capacity planning
- Check if policy complexity increased (slow policy evaluation)

### 6.7 Storage Cleanup

**Symptoms**:
- Alert: `StorageQuotaExceeded`
- Errors: `StorageError::QuotaExceeded`
- Disk usage >85%

**Investigation**:
```bash
# Check storage usage
kubectl -n creto-storage exec -it deploy/creto-storage -- df -h

# Check largest tenants
kubectl -n creto-storage exec -it deploy/creto-storage -- \
  psql -c "SELECT organization_id, pg_size_pretty(sum(size))
           FROM storage_objects
           GROUP BY organization_id
           ORDER BY sum(size) DESC
           LIMIT 10;"

# Check object age distribution
kubectl -n creto-storage exec -it deploy/creto-storage -- \
  psql -c "SELECT date_trunc('month', created_at), count(*), pg_size_pretty(sum(size))
           FROM storage_objects
           GROUP BY 1 ORDER BY 1;"
```

**Cleanup Procedures**:

**Automated: Trigger retention policy**
```bash
# Run retention cleanup job
kubectl -n creto-storage create job --from=cronjob/storage-retention cleanup-$(date +%s)
kubectl -n creto-storage logs -f job/cleanup-$(date +%s)
```

**Manual: Delete old checkpoints (Runtime)**
```bash
# Delete checkpoints older than 30 days
kubectl -n creto-storage exec -it deploy/creto-storage -- \
  psql -c "DELETE FROM storage_objects
           WHERE object_type = 'checkpoint'
           AND created_at < now() - interval '30 days';"
```

**Manual: Archive cold data to S3**
```bash
# Archive messages older than 90 days
kubectl -n creto-storage exec -it deploy/creto-storage -- \
  /scripts/archive-to-s3.sh --older-than 90d --type message_envelope
```

**Emergency: Expand storage**
```bash
# Expand PVC (if storage class supports)
kubectl -n creto-storage patch pvc storage-data -p \
  '{"spec":{"resources":{"requests":{"storage":"500Gi"}}}}'
```

### 6.8 HSM Recovery

**Symptoms**:
- Alert: `VaultHSMUnavailable`
- Errors: `VaultError::HSMConnectionFailed`
- Signing operations failing

**Investigation**:
```bash
# Check HSM connectivity
kubectl -n creto-vault exec -it deploy/creto-vault -- \
  /scripts/hsm-health-check.sh

# Check HSM partition status (Luna HSM)
kubectl -n creto-vault exec -it deploy/creto-vault -- \
  lunacm -c slot list

# Check recent HSM operations
kubectl -n creto-vault logs -l app=creto-vault --tail=200 | grep -i hsm
```

**Recovery Steps**:

**Step 1: Verify HSM hardware status**
```bash
# Contact data center / cloud provider
# AWS CloudHSM: Check AWS Health Dashboard
# Thales Luna: Contact Thales support
```

**Step 2: Failover to backup HSM (if available)**
```bash
kubectl -n creto-vault set env deploy/creto-vault \
  HSM_PRIMARY_SLOT=backup-hsm-slot-id \
  HSM_FAILOVER_ENABLED=true
kubectl -n creto-vault rollout restart deploy/creto-vault
```

**Step 3: If HSM unrecoverable, enable software fallback (EMERGENCY)**
```bash
# WARNING: This reduces security posture
# REQUIRES CISO APPROVAL

kubectl -n creto-vault set env deploy/creto-vault \
  HSM_SOFTWARE_FALLBACK_ENABLED=true \
  HSM_SOFTWARE_FALLBACK_KEY_FILE=/secrets/fallback-key
kubectl -n creto-vault rollout restart deploy/creto-vault
```

**Post-Recovery**:
- Schedule HSM hardware replacement
- Rotate all keys after hardware fix
- Review HSM HA configuration

### 6.9 Token Refresh Failures

**Symptoms**:
- Alert: `NHITokenRefreshFailed`
- Agents unable to authenticate
- Errors: `NHIError::TokenExpired`

**Investigation**:
```bash
# Check NHI service health
kubectl -n creto-nhi get pods
kubectl -n creto-nhi logs -l app=creto-nhi --tail=100 | grep -i token

# Check token store connectivity
kubectl -n creto-nhi exec -it deploy/creto-nhi -- \
  redis-cli -h creto-nhi-redis ping

# Check certificate validity
kubectl -n creto-nhi exec -it deploy/creto-nhi -- \
  openssl x509 -in /certs/nhi-signing.crt -noout -dates
```

**Recovery Steps**:

**If Redis unavailable**:
```bash
kubectl -n creto-nhi rollout restart statefulset/creto-nhi-redis
kubectl -n creto-nhi rollout status statefulset/creto-nhi-redis --timeout=120s
```

**If signing certificate expired**:
```bash
# Rotate certificate (requires Vault)
kubectl -n creto-nhi exec -it deploy/creto-nhi -- \
  /scripts/rotate-signing-cert.sh

# Restart NHI service
kubectl -n creto-nhi rollout restart deploy/creto-nhi
```

**Force token refresh for all agents**:
```bash
# Broadcast token refresh event
kubectl -n creto-nhi exec -it deploy/creto-nhi -- \
  curl -X POST http://localhost:8080/admin/force-refresh-all
```

### 6.10 Cluster Healing (Consensus Split Brain)

**Symptoms**:
- Alert: `ConsensusSplitBrain`
- Multiple leaders detected
- Inconsistent state across nodes

**Investigation**:
```bash
# Check Raft cluster status
kubectl -n creto-consensus exec -it creto-consensus-0 -- \
  /bin/raftadmin status

# Check leader election
kubectl -n creto-consensus logs -l app=creto-consensus --tail=200 | grep -i leader

# Check network connectivity between nodes
for i in 0 1 2; do
  kubectl -n creto-consensus exec -it creto-consensus-$i -- \
    nc -zv creto-consensus-$((($i + 1) % 3)).creto-consensus 8081
done
```

**Healing Steps**:

**Step 1: Identify the correct leader**
```bash
# Find node with most recent committed log
for i in 0 1 2; do
  echo "Node $i:"
  kubectl -n creto-consensus exec -it creto-consensus-$i -- \
    /bin/raftadmin log-info
done
```

**Step 2: Stop minority partition nodes**
```bash
# Stop nodes not in majority partition
kubectl -n creto-consensus delete pod creto-consensus-X --grace-period=0
```

**Step 3: Wait for leader stabilization**
```bash
# Monitor until single leader elected
watch 'kubectl -n creto-consensus exec -it creto-consensus-0 -- /bin/raftadmin status'
```

**Step 4: Rejoin recovered nodes**
```bash
# Nodes will auto-rejoin after pod recreation
kubectl -n creto-consensus get pods -w
```

**Data Reconciliation**:
After split brain, verify data consistency:
```bash
# Compare state checksums across nodes
for i in 0 1 2; do
  echo "Node $i checksum:"
  kubectl -n creto-consensus exec -it creto-consensus-$i -- \
    /bin/raftadmin state-checksum
done
```

---

## 7. Disaster Recovery

### 7.1 Backup Procedures

#### PostgreSQL Backups

**Automated Daily Backups** (via CronJob):
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: creto-enablement-prod
spec:
  schedule: "0 2 * * *"  # Daily at 2am UTC
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: postgres-backup
              image: postgres:15
              command:
                - /bin/bash
                - -c
                - |
                  pg_dump -h postgres -U creto -Fc creto_metering > /backup/metering-$(date +\%Y\%m\%d).dump
                  aws s3 cp /backup/metering-$(date +\%Y\%m\%d).dump s3://creto-backups/postgres/metering/
                  # Retain only last 30 days
                  aws s3 ls s3://creto-backups/postgres/metering/ | \
                    awk '{print $4}' | head -n -30 | xargs -I {} aws s3 rm s3://creto-backups/postgres/metering/{}
              env:
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: postgres-credentials
                      key: password
              volumeMounts:
                - name: backup-volume
                  mountPath: /backup
          restartPolicy: OnFailure
          volumes:
            - name: backup-volume
              emptyDir: {}
```

**Manual Backup**:
```bash
# Full database dump
kubectl exec postgres-0 -n creto-enablement-prod -- \
  pg_dump -U creto -Fc creto_metering > metering-manual-$(date +%Y%m%d).dump

# Upload to S3
aws s3 cp metering-manual-$(date +%Y%m%d).dump \
  s3://creto-backups/postgres/metering/manual/

# Verify backup
aws s3 ls s3://creto-backups/postgres/metering/manual/
```

#### Redis Backups

**Automated RDB Snapshots** (configured in Redis):
```yaml
# redis-config.yaml
save 900 1     # Save if 1 key changed in 15 min
save 300 10    # Save if 10 keys changed in 5 min
save 60 10000  # Save if 10K keys changed in 1 min

# Upload to S3 via sidecar
apiVersion: v1
kind: Pod
metadata:
  name: redis-0
spec:
  containers:
    - name: redis
      image: redis:7
    - name: backup-sidecar
      image: amazon/aws-cli
      command:
        - /bin/bash
        - -c
        - |
          while true; do
            sleep 900  # Every 15 min
            aws s3 cp /data/dump.rdb s3://creto-backups/redis/$(date +\%Y\%m\%d-\%H\%M).rdb
          done
      volumeMounts:
        - name: redis-data
          mountPath: /data
```

#### Kubernetes State Backup (Velero)

**Install Velero**:
```bash
# Install Velero CLI
wget https://github.com/vmware-tanzu/velero/releases/download/v1.12.0/velero-v1.12.0-linux-amd64.tar.gz
tar -xvf velero-v1.12.0-linux-amd64.tar.gz
sudo mv velero-v1.12.0-linux-amd64/velero /usr/local/bin/

# Install Velero in cluster
velero install \
  --provider aws \
  --bucket creto-velero-backups \
  --secret-file ./credentials-velero \
  --backup-location-config region=us-east-1 \
  --snapshot-location-config region=us-east-1 \
  --use-volume-snapshots=true
```

**Create Backup**:
```bash
# Backup entire namespace
velero backup create enablement-backup-$(date +%Y%m%d) \
  --include-namespaces creto-enablement-prod \
  --wait

# Backup specific resources
velero backup create metering-backup-$(date +%Y%m%d) \
  --include-namespaces creto-enablement-prod \
  --include-resources deployments,configmaps,secrets \
  --selector app.kubernetes.io/name=creto-metering

# Schedule daily backups
velero schedule create daily-enablement-backup \
  --schedule="0 3 * * *" \
  --include-namespaces creto-enablement-prod \
  --ttl 720h0m0s  # 30 days retention
```

### 7.2 Restore Procedures

#### PostgreSQL Restore

```bash
# Step 1: Stop application pods (prevent writes during restore)
kubectl scale deployment creto-metering --replicas=0 -n creto-enablement-prod

# Step 2: Download backup from S3
aws s3 cp s3://creto-backups/postgres/metering/metering-20241225.dump ./

# Step 3: Drop existing database (CAUTION!)
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "DROP DATABASE creto_metering;"

# Step 4: Recreate database
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "CREATE DATABASE creto_metering;"

# Step 5: Restore from backup
kubectl exec -i postgres-0 -n creto-enablement-prod -- \
  pg_restore -U creto -d creto_metering < metering-20241225.dump

# Step 6: Verify data
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -d creto_metering -c "SELECT count(*) FROM usage_hourly;"

# Step 7: Restart application pods
kubectl scale deployment creto-metering --replicas=3 -n creto-enablement-prod
```

#### Redis Restore

```bash
# Step 1: Stop Redis pod
kubectl scale statefulset redis --replicas=0 -n creto-enablement-prod

# Step 2: Download backup
aws s3 cp s3://creto-backups/redis/20241225-0200.rdb ./dump.rdb

# Step 3: Copy backup to persistent volume
kubectl cp dump.rdb redis-0:/data/dump.rdb -n creto-enablement-prod

# Step 4: Start Redis (loads dump.rdb automatically)
kubectl scale statefulset redis --replicas=3 -n creto-enablement-prod

# Step 5: Verify data
kubectl exec redis-0 -n creto-enablement-prod -- redis-cli DBSIZE
```

#### Velero Restore

```bash
# List available backups
velero backup get

# Restore entire namespace
velero restore create --from-backup enablement-backup-20241225 \
  --wait

# Restore specific resources
velero restore create --from-backup metering-backup-20241225 \
  --include-resources deployments,configmaps \
  --namespace-mappings creto-enablement-prod:creto-enablement-staging

# Check restore status
velero restore describe <restore-name>
velero restore logs <restore-name>
```

### 7.3 RTO/RPO Targets

| Service | RTO (Recovery Time) | RPO (Recovery Point) | Backup Frequency | Retention |
|---------|---------------------|----------------------|------------------|-----------|
| **PostgreSQL** | 30 min | 1 hour | Hourly (WAL) + Daily (full) | 30 days |
| **Redis** | 15 min | 15 min | Every 15 min (RDB) | 7 days |
| **Kubernetes State** | 1 hour | 24 hours | Daily (Velero) | 30 days |
| **Application Secrets** | 15 min | Real-time | Continuous (sync to Vault) | 90 days |
| **Audit Logs** | 4 hours | 0 min | Real-time to S3 | 7 years |

### 7.4 Failover Procedures

#### Multi-Region Failover (PostgreSQL)

**Primary Region Failure Scenario**:
```bash
# Step 1: Promote standby to primary (in secondary region)
kubectl exec postgres-0 -n creto-enablement-prod-eu-west -- \
  psql -U creto -c "SELECT pg_promote();"

# Step 2: Update DNS to point to new region
aws route53 change-resource-record-sets \
  --hosted-zone-id Z1234567890ABC \
  --change-batch file://failover-dns.json

# failover-dns.json:
# {
#   "Changes": [{
#     "Action": "UPSERT",
#     "ResourceRecordSet": {
#       "Name": "postgres.creto.io",
#       "Type": "CNAME",
#       "TTL": 60,
#       "ResourceRecords": [{"Value": "postgres-eu-west.creto.local"}]
#     }
#   }]
# }

# Step 3: Update application config to use new endpoint
kubectl set env deployment/creto-metering \
  DATABASE_URL=postgresql://creto@postgres-eu-west:5432/creto_metering \
  -n creto-enablement-prod

# Step 4: Restart application pods
kubectl rollout restart deployment -n creto-enablement-prod

# Step 5: Verify failover
curl -H "Authorization: Bearer $TOKEN" \
  https://api-eu.creto.io/metering/v1/health
```

#### Redis Sentinel Failover (Automatic)

**Sentinel Configuration** (automatic failover):
```yaml
# redis-sentinel-config.yaml
sentinel monitor mymaster redis-0.redis 6379 2  # 2 sentinels must agree
sentinel down-after-milliseconds mymaster 5000  # 5s to detect failure
sentinel failover-timeout mymaster 10000        # 10s failover timeout
sentinel parallel-syncs mymaster 1              # 1 replica syncs at a time
```

**Manual Failover (if needed)**:
```bash
# Connect to Sentinel
kubectl exec -it redis-sentinel-0 -n creto-enablement-prod -- redis-cli -p 26379

# Trigger manual failover
SENTINEL failover mymaster

# Check new master
SENTINEL get-master-addr-by-name mymaster
```

---

## 8. Security Operations

### 8.1 Secret Rotation

**Rotate Database Password**:
```bash
# Step 1: Generate new password
NEW_PASSWORD=$(openssl rand -base64 32)

# Step 2: Update password in PostgreSQL
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "ALTER USER creto WITH PASSWORD '$NEW_PASSWORD';"

# Step 3: Update Kubernetes secret
kubectl create secret generic postgres-credentials \
  --from-literal=password=$NEW_PASSWORD \
  -n creto-enablement-prod \
  --dry-run=client -o yaml | kubectl apply -f -

# Step 4: Restart application pods (picks up new secret)
kubectl rollout restart deployment -n creto-enablement-prod

# Step 5: Verify connectivity
kubectl exec -it creto-metering-<pod> -n creto-enablement-prod -- \
  psql -h postgres -U creto -d creto_metering -c "SELECT 1;"
```

**Rotate API Keys** (for external services):
```bash
# Step 1: Generate new key in external service (e.g., Slack)
# (Manual step via Slack UI)

# Step 2: Update Kubernetes secret
kubectl create secret generic oversight-secrets \
  --from-literal=slack-webhook-url=$NEW_SLACK_URL \
  -n creto-enablement-prod \
  --dry-run=client -o yaml | kubectl apply -f -

# Step 3: Restart oversight pods
kubectl rollout restart deployment/creto-oversight -n creto-enablement-prod

# Step 4: Test notification
curl -X POST $NEW_SLACK_URL \
  -H "Content-Type: application/json" \
  -d '{"text": "Test notification after key rotation"}'
```

### 8.2 Access Audit

**Review RBAC Permissions**:
```bash
# List all RoleBindings in namespace
kubectl get rolebindings -n creto-enablement-prod

# Check who has access to secrets
kubectl get rolebindings -n creto-enablement-prod -o json | \
  jq '.items[] | select(.roleRef.name == "secret-reader") | .subjects'

# Review ServiceAccount permissions
kubectl describe serviceaccount creto-metering -n creto-enablement-prod
```

**Audit Kubernetes API Access**:
```bash
# Enable audit logging (in kube-apiserver)
--audit-policy-file=/etc/kubernetes/audit-policy.yaml
--audit-log-path=/var/log/kubernetes/audit.log

# Query audit logs for secret access
kubectl logs kube-apiserver-<node> -n kube-system | \
  grep "secrets" | grep "get"

# Or if using audit webhook:
kubectl get events -n creto-enablement-prod --sort-by='.lastTimestamp' | \
  grep "secrets"
```

### 8.3 Vulnerability Scanning

**Scan Container Images** (Trivy):
```bash
# Install Trivy
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | \
  sudo sh -s -- -b /usr/local/bin

# Scan image
trivy image gcr.io/creto-prod/enablement/metering:v1.2.0

# Output:
# Total: 5 (UNKNOWN: 0, LOW: 2, MEDIUM: 2, HIGH: 1, CRITICAL: 0)

# Fail build if HIGH or CRITICAL vulnerabilities
trivy image --exit-code 1 --severity HIGH,CRITICAL \
  gcr.io/creto-prod/enablement/metering:v1.2.0
```

**Scan Kubernetes Manifests** (kube-bench):
```bash
# Install kube-bench
kubectl apply -f https://raw.githubusercontent.com/aquasecurity/kube-bench/main/job.yaml

# Check results
kubectl logs job/kube-bench -n kube-system

# Expected output: CIS Kubernetes Benchmark compliance report
```

**Dependency Scanning** (cargo-audit for Rust):
```bash
# Install cargo-audit
cargo install cargo-audit

# Scan dependencies for vulnerabilities
cd creto-enablement/
cargo audit

# Output:
# Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
#   Loaded 500 security advisories
#   Scanning Cargo.lock for vulnerabilities (234 crate dependencies)
# No vulnerabilities found!
```

### 8.4 Incident Response (Security)

**Security Incident Workflow**:
```
1. Detect security event
   └─> Automated alert (Falco, SIEM) or manual report
       └─> Triage severity (P1: breach, P2: vulnerability, P3: policy violation)
           └─> Engage security team
               ├─> P1 Breach: Immediate containment
               │   └─> Isolate affected systems
               │   └─> Preserve evidence (logs, memory dumps)
               │   └─> Notify legal/compliance
               │
               ├─> P2 Vulnerability: 24-hour remediation
               │   └─> Patch and deploy fix
               │   └─> Verify exploitation attempts in logs
               │
               └─> P3 Policy Violation: Investigate and document
                   └─> Review access logs
                   └─> Update policies/training
```

**Containment Procedures**:

**Suspected Compromised Pod**:
```bash
# Step 1: Isolate pod (deny all network traffic)
kubectl label pod <pod-name> security-incident=true -n creto-enablement-prod

# Apply NetworkPolicy to block traffic
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: isolate-compromised-pod
  namespace: creto-enablement-prod
spec:
  podSelector:
    matchLabels:
      security-incident: "true"
  policyTypes:
    - Ingress
    - Egress
  # No ingress/egress rules = deny all
EOF

# Step 2: Capture evidence (logs, filesystem)
kubectl logs <pod-name> -n creto-enablement-prod > evidence-logs.txt
kubectl exec <pod-name> -n creto-enablement-prod -- tar czf /tmp/evidence.tar.gz /

# Step 3: Preserve memory dump (if possible)
kubectl exec <pod-name> -n creto-enablement-prod -- \
  gcore $(pgrep creto-metering)

# Step 4: Terminate pod
kubectl delete pod <pod-name> -n creto-enablement-prod --force --grace-period=0

# Step 5: Analyze evidence offline
```

**Suspected API Key Leak**:
```bash
# Step 1: Revoke compromised key immediately
# (Varies by service, e.g., Slack: regenerate webhook URL)

# Step 2: Search audit logs for unauthorized usage
kubectl logs -l app=oversight -n creto-enablement-prod | \
  grep "slack_webhook_url" | grep -v "200 OK"

# Step 3: Rotate key (see Section 8.1)

# Step 4: Review access controls
# - Who had access to the secret?
# - Was it logged/transmitted insecurely?

# Step 5: Update incident response plan
```

---

## 9. Performance Tuning

### 9.1 Connection Pool Sizing

**PostgreSQL Connection Pool** (in application):
```rust
// creto-metering/src/db.rs

// Calculate pool size based on:
// - Number of replicas (3)
// - Max concurrent requests per replica (100)
// - Percentage using DB per request (30%)
// Pool size = replicas * concurrent * percentage = 3 * 100 * 0.3 = 90

let pool = PgPoolOptions::new()
    .max_connections(30)  // Per replica: 90 / 3 = 30
    .min_connections(5)   // Keep 5 connections warm
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(600))  // Close idle after 10 min
    .max_lifetime(Duration::from_secs(1800)) // Recycle after 30 min
    .connect(&database_url)
    .await?;
```

**PostgreSQL Server Settings**:
```sql
-- Increase max connections (default: 100)
ALTER SYSTEM SET max_connections = 200;

-- Connection pooler (PgBouncer) in transaction mode
-- max_client_conn = 1000 (application connections)
-- default_pool_size = 25 (per database)
-- Total DB connections = databases * pool_size = 4 * 25 = 100
```

**Redis Connection Pool**:
```rust
// creto-metering/src/cache.rs

let redis_pool = redis::Client::open(redis_url)?
    .get_tokio_connection_manager()
    .await?;

// Or with deadpool-redis:
let pool = deadpool_redis::Config {
    url: Some(redis_url),
    pool: Some(deadpool_redis::PoolConfig {
        max_size: 50,      // Max connections
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(2)),
            create: Some(Duration::from_secs(2)),
            recycle: Some(Duration::from_secs(2)),
        },
    }),
    connection: None,
}
.create_pool(Some(Runtime::Tokio1))?;
```

### 9.2 Cache Configuration

**Redis Caching Strategy**:
```rust
// Cache quota check results (TTL: 60s)
async fn check_quota_cached(
    redis: &RedisPool,
    postgres: &PgPool,
    subscription_id: &str,
    event_type: &str,
) -> Result<QuotaStatus> {
    let cache_key = format!("quota:{}:{}", subscription_id, event_type);

    // Try cache first
    if let Some(cached) = redis.get::<_, Option<String>>(&cache_key).await? {
        return Ok(serde_json::from_str(&cached)?);
    }

    // Cache miss: query database
    let status = query_quota_from_db(postgres, subscription_id, event_type).await?;

    // Store in cache (TTL: 60s)
    redis.set_ex(&cache_key, serde_json::to_string(&status)?, 60).await?;

    Ok(status)
}
```

**Cache Eviction Policies**:
```yaml
# redis.conf
maxmemory 2gb
maxmemory-policy allkeys-lru  # Evict least recently used keys

# Alternative policies:
# - volatile-lru: Evict LRU keys with TTL set
# - allkeys-lfu: Evict least frequently used (better for hot keys)
# - volatile-ttl: Evict keys with shortest TTL
```

**Cache Warm-Up** (pre-populate hot keys):
```bash
# CronJob to pre-warm cache daily
apiVersion: batch/v1
kind: CronJob
metadata:
  name: cache-warmup
spec:
  schedule: "0 0 * * *"  # Daily at midnight
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: warmup
              image: gcr.io/creto-prod/enablement/metering:v1.2.0
              command: ["/usr/local/bin/cache-warmup"]
              # Script queries top 1000 subscriptions, loads quota configs into cache
```

### 9.3 Query Optimization

**Identify Slow Queries**:
```sql
-- Enable slow query logging
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log queries >1s

-- View slowest queries
SELECT
    mean_exec_time,
    calls,
    query
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;

-- Expected output:
-- mean_exec_time | calls |              query
-- ----------------+-------+---------------------------------
--  3542.12        | 1234  | SELECT * FROM usage_hourly WHERE...
```

**Add Missing Indexes**:
```sql
-- Identify missing indexes
SELECT
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    seq_tup_read / NULLIF(seq_scan, 0) AS avg_seq_tup
FROM pg_stat_user_tables
WHERE seq_scan > 0
ORDER BY seq_tup_read DESC
LIMIT 10;

-- Add index for common query
CREATE INDEX CONCURRENTLY idx_usage_hourly_subscription_hour
ON usage_hourly (subscription_id, hour DESC);

-- Verify index usage
EXPLAIN ANALYZE
SELECT * FROM usage_hourly
WHERE subscription_id = 'sub_123'
  AND hour >= '2024-12-01';

-- Expected: "Index Scan using idx_usage_hourly_subscription_hour"
```

**Query Rewrite Example**:
```sql
-- SLOW (sequential scan on large table)
SELECT * FROM usage_hourly
WHERE hour >= now() - interval '7 days'
  AND event_type = 'agent.execution';

-- FAST (use index on event_type + hour)
CREATE INDEX idx_usage_hourly_event_hour
ON usage_hourly (event_type, hour DESC);

-- Even faster: Materialized view for common aggregations
CREATE MATERIALIZED VIEW usage_daily AS
SELECT
    subscription_id,
    event_type,
    date_trunc('day', hour) AS day,
    SUM(event_count) AS total_count
FROM usage_hourly
GROUP BY subscription_id, event_type, day;

-- Refresh daily via CronJob
REFRESH MATERIALIZED VIEW CONCURRENTLY usage_daily;
```

### 9.4 Resource Limits Tuning

**CPU Limits**:
```yaml
# Start conservative, adjust based on metrics
resources:
  requests:
    cpu: 100m     # Minimum guaranteed
    memory: 128Mi
  limits:
    cpu: 1000m    # Max allowed (throttled beyond this)
    memory: 512Mi

# Monitor throttling
kubectl top pods -n creto-enablement-prod
# If CPU usage consistently near limit and latency high, increase limit

# Check CPU throttling metrics in Prometheus:
rate(container_cpu_cfs_throttled_seconds_total[5m]) > 0.1
```

**Memory Limits**:
```yaml
# Set limits based on actual usage + 20% headroom
resources:
  requests:
    memory: 256Mi  # Minimum (used for scheduling)
  limits:
    memory: 512Mi  # Max (OOMKilled if exceeded)

# Monitor OOMKills:
kubectl get pods -n creto-enablement-prod \
  -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.containerStatuses[0].lastState.terminated.reason}{"\n"}{end}' \
  | grep OOMKilled

# If frequent OOMKills, increase limit or fix memory leak
```

**Garbage Collection Tuning** (Rust - less relevant, but for completeness):
```rust
// For services with large heaps, consider jemalloc for better memory management
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

// Or mimalloc
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

---

## 10. Runbook Checklists

### 10.1 Daily Checks

**Morning Standup Checklist** (5 minutes):
- [ ] Check Grafana "Enablement Overview" dashboard
  - [ ] All services healthy (green status)
  - [ ] No active alerts in Alertmanager
  - [ ] Error rate <0.1%
  - [ ] P95 latency <100ms
- [ ] Review overnight incidents (if any)
  - [ ] Check #incident-* Slack channels
  - [ ] Verify all incidents resolved
- [ ] Check backup status
  - [ ] PostgreSQL backup completed (check S3 bucket)
  - [ ] Redis snapshot generated
  - [ ] Velero backup succeeded
- [ ] Review capacity trends
  - [ ] Database size growth
  - [ ] Redis memory usage
  - [ ] Disk usage <80%
- [ ] Check certificate expiration
  - [ ] No certs expiring in <30 days

**Commands**:
```bash
# Quick health check script
#!/bin/bash
kubectl get pods -n creto-enablement-prod | grep -v Running && echo "⚠️ Unhealthy pods detected" || echo "✅ All pods healthy"

aws s3 ls s3://creto-backups/postgres/metering/ | tail -1 | \
  awk '{if ($1 == strftime("%Y-%m-%d", systime())) print "✅ Backup today"; else print "⚠️ No backup today"}'

kubectl get certificate -n creto-enablement-prod -o json | \
  jq -r '.items[] | select(.status.notAfter | fromdateiso8601 < (now + 30*86400)) | .metadata.name' | \
  (read cert && echo "⚠️ Certificate expiring: $cert" || echo "✅ All certs valid >30 days")
```

### 10.2 Weekly Maintenance

**Monday Maintenance Window** (30 minutes, 10am UTC):
- [ ] Review and acknowledge Prometheus alerts
  - [ ] Silence non-actionable alerts
  - [ ] Create tickets for recurring warnings
- [ ] Check resource utilization trends
  - [ ] CPU/memory usage by pod (last 7 days)
  - [ ] Plan scaling if >70% utilized
- [ ] Review slow queries
  - [ ] Check `pg_stat_statements` for queries >1s
  - [ ] Add indexes or optimize queries
- [ ] Update documentation
  - [ ] Add new runbook entries from last week's incidents
  - [ ] Update on-call schedule
- [ ] Dependency updates
  - [ ] Check for security patches (Rust crates, Docker images)
  - [ ] Plan upgrade if critical CVEs
- [ ] Test disaster recovery
  - [ ] Restore latest backup to staging
  - [ ] Verify data integrity

**Automation Script**:
```bash
#!/bin/bash
# weekly-checks.sh

echo "=== Weekly Maintenance Checklist ==="

echo "1. Checking resource utilization..."
kubectl top nodes
kubectl top pods -n creto-enablement-prod --sort-by=memory | head -10

echo "2. Checking slow queries..."
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "SELECT mean_exec_time, calls, query FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

echo "3. Checking for security updates..."
cd ~/creto-enablement
cargo audit

echo "4. Testing backup restore..."
aws s3 cp s3://creto-backups/postgres/metering/$(date +%Y%m%d).dump /tmp/
kubectl exec postgres-0 -n creto-enablement-staging -- \
  pg_restore -U creto -d creto_metering < /tmp/$(date +%Y%m%d).dump

echo "✅ Weekly maintenance complete"
```

### 10.3 Monthly Reviews

**First Monday of Month** (2 hours):
- [ ] Capacity planning review
  - [ ] Project resource needs for next quarter
  - [ ] Plan infrastructure scaling (nodes, storage)
- [ ] Cost analysis
  - [ ] Review cloud spend (GCP, AWS)
  - [ ] Identify cost optimization opportunities
- [ ] Security audit
  - [ ] Review RBAC permissions
  - [ ] Rotate long-lived credentials
  - [ ] Scan for vulnerabilities (Trivy, cargo-audit)
- [ ] SLA review
  - [ ] Calculate actual uptime vs. target (99.9%)
  - [ ] Review P95/P99 latency trends
  - [ ] Identify top error sources
- [ ] On-call retrospective
  - [ ] Review incident count and severity
  - [ ] Identify toil reduction opportunities
  - [ ] Update runbooks from learnings
- [ ] Dependency updates
  - [ ] Update Rust crates to latest stable
  - [ ] Update Docker base images
  - [ ] Update Kubernetes manifests (if new versions available)

**Monthly Report Template**:
```markdown
# Enablement Layer Monthly Report - January 2025

## Service Health
- **Uptime**: 99.95% (target: 99.9%) ✅
- **Incidents**: 2 SEV-2, 5 SEV-3
- **Mean Time to Detect**: 4.2 minutes
- **Mean Time to Resolve**: 18 minutes

## Performance
- **P95 Latency**: 87ms (target: <100ms) ✅
- **P99 Latency**: 342ms (target: <500ms) ✅
- **Error Rate**: 0.08% (target: <0.1%) ✅

## Capacity & Cost
- **Pod Count**: 12 (3 per service)
- **CPU Usage**: 62% avg
- **Memory Usage**: 58% avg
- **Storage**: 2.3TB PostgreSQL, 450GB Redis
- **Monthly Cost**: $12,340 (-8% vs last month)

## Security
- **Vulnerabilities**: 0 critical, 2 high (patched)
- **Access Reviews**: Completed (removed 3 stale accounts)
- **Key Rotations**: 4 keys rotated

## Action Items
- [ ] Scale PostgreSQL storage (projected 80% full in 2 months)
- [ ] Implement Redis auto-scaling (currently manual)
- [ ] Update metering dependency (security patch)
```

### 10.4 Quarterly Audits

**Quarterly Business Review** (4 hours, stakeholders: SRE, Engineering, Product):
- [ ] **Reliability**
  - [ ] Calculate SLI/SLO compliance (last 90 days)
  - [ ] Review error budget consumption
  - [ ] Plan reliability improvements
- [ ] **Performance**
  - [ ] Analyze latency trends (P50/P95/P99)
  - [ ] Identify performance regressions
  - [ ] Benchmark against targets
- [ ] **Scalability**
  - [ ] Review usage growth (events/sec, agents, sandboxes)
  - [ ] Project capacity needs for next year
  - [ ] Plan infrastructure scaling
- [ ] **Security & Compliance**
  - [ ] Audit access logs (SOC 2 requirement)
  - [ ] Review incident response readiness
  - [ ] Verify backup/restore procedures
  - [ ] Penetration testing (annual)
- [ ] **Cost Optimization**
  - [ ] Analyze spend by service
  - [ ] Identify idle resources
  - [ ] Evaluate reserved instances vs on-demand
- [ ] **Documentation**
  - [ ] Update architecture diagrams
  - [ ] Review and update runbooks
  - [ ] Update disaster recovery procedures
- [ ] **Team Development**
  - [ ] On-call rotation retrospective
  - [ ] Training needs assessment
  - [ ] Process improvements

**Quarterly Audit Script**:
```bash
#!/bin/bash
# quarterly-audit.sh

echo "=== Quarterly Audit Report ==="

echo "## SLI/SLO Compliance (Last 90 days)"
# Calculate uptime
kubectl get pods -n creto-enablement-prod --sort-by=.status.startTime | \
  awk '{print $1, $3}' | grep -v NAME

echo "## Top 10 Error Sources"
# Query Prometheus for error sources
curl -s 'http://prometheus:9090/api/v1/query?query=topk(10,sum by (grpc_method)(rate(grpc_server_handled_total{grpc_code!="OK"}[90d])))'

echo "## Storage Growth Trend"
# Check database size over time
kubectl exec postgres-0 -n creto-enablement-prod -- \
  psql -U creto -c "SELECT pg_database.datname, pg_size_pretty(pg_database_size(pg_database.datname)) FROM pg_database;"

echo "## Cost by Service (Last 90 days)"
# Query cloud provider billing API (example for GCP)
gcloud billing reports list --billing-account=$BILLING_ACCOUNT --format=json | \
  jq '.[] | select(.timeRange.startTime > (now - 90*86400)) | {service: .labels.service, cost: .cost}'

echo "✅ Quarterly audit data collected. Review with stakeholders."
```

---

## Appendix

### A. Useful Commands Reference

**Kubernetes**:
```bash
# Get pod status
kubectl get pods -n creto-enablement-prod

# Describe pod (detailed info)
kubectl describe pod <pod-name> -n creto-enablement-prod

# View logs
kubectl logs -f <pod-name> -n creto-enablement-prod

# Execute command in pod
kubectl exec -it <pod-name> -n creto-enablement-prod -- /bin/bash

# Port forward
kubectl port-forward svc/creto-metering 50051:50051 -n creto-enablement-prod

# Scale deployment
kubectl scale deployment creto-metering --replicas=5 -n creto-enablement-prod

# Restart deployment
kubectl rollout restart deployment creto-metering -n creto-enablement-prod

# Check deployment status
kubectl rollout status deployment creto-metering -n creto-enablement-prod

# View events
kubectl get events -n creto-enablement-prod --sort-by='.lastTimestamp'
```

**PostgreSQL**:
```bash
# Connect to PostgreSQL
kubectl exec -it postgres-0 -n creto-enablement-prod -- psql -U creto -d creto_metering

# Common queries
SELECT version();  -- PostgreSQL version
\dt               -- List tables
\di               -- List indexes
\du               -- List users
SELECT count(*) FROM usage_hourly;  -- Row count
```

**Redis**:
```bash
# Connect to Redis
kubectl exec -it redis-0 -n creto-enablement-prod -- redis-cli

# Common commands
PING          # Test connection
DBSIZE        # Number of keys
INFO memory   # Memory usage
KEYS quota:*  # List keys matching pattern (use sparingly in prod)
GET quota:sub_123:agent.execution  # Get key value
```

**Prometheus/Grafana**:
```bash
# Query Prometheus API
curl 'http://prometheus:9090/api/v1/query?query=up{namespace="creto-enablement"}'

# Import Grafana dashboard
curl -X POST http://grafana:3000/api/dashboards/db \
  -H "Authorization: Bearer $GRAFANA_API_KEY" \
  -H "Content-Type: application/json" \
  -d @dashboard.json
```

### B. Contact Information

| Role | Contact | Escalation |
|------|---------|------------|
| **Primary On-Call** | #oncall-sre (Slack) | PagerDuty |
| **Secondary On-Call** | See PagerDuty schedule | PagerDuty |
| **SRE Team Lead** | alice@creto.io | Direct call |
| **Engineering Manager** | bob@creto.io | Direct call |
| **VP Engineering** | charlie@creto.io | Email/Slack |
| **Security Team** | security@creto.io | Urgent: #security-incidents |
| **Customer Support** | support@creto.io | #customer-support |

### C. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-26 | 1.0 | Creto Operations Team | Initial runbook creation |

---

**END OF RUNBOOK**
