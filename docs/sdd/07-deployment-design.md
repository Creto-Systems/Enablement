---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-07: Deployment Design

## Purpose

This document defines the deployment architecture, infrastructure, CI/CD pipelines, and operational procedures for the Enablement Layer products.

## Scope

**In Scope:**
- Kubernetes deployment architecture
- Container images and registries
- CI/CD pipeline design
- Environment strategy
- Configuration management
- Observability infrastructure

**Out of Scope:**
- Application code implementation
- Platform layer infrastructure
- Bare-metal provisioning

---

## 1. Infrastructure Overview

### 1.1 Target Platforms

| Platform | Use Case | Status |
|----------|----------|--------|
| **Kubernetes** | Production, staging | Primary |
| **Local (Docker Compose)** | Development | Supported |
| **Kind/k3s** | CI testing | Supported |

### 1.2 Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         KUBERNETES CLUSTER                               │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     creto-enablement Namespace                       │ │
│  │                                                                      │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │ │
│  │  │  Metering   │  │  Oversight  │  │   Runtime   │  │  Messaging  │ │ │
│  │  │  Deployment │  │  Deployment │  │  Deployment │  │  Deployment │ │ │
│  │  │  (3 pods)   │  │  (3 pods)   │  │  (3 pods)   │  │  (3 pods)   │ │ │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │ │
│  │         │                │                │                │        │ │
│  │  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐ │ │
│  │  │   Service   │  │   Service   │  │   Service   │  │   Service   │ │ │
│  │  │ (ClusterIP) │  │ (ClusterIP) │  │ (ClusterIP) │  │ (ClusterIP) │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │ │
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
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Container Images

### 2.1 Image Strategy

| Image | Base | Size Target | Contents |
|-------|------|-------------|----------|
| `creto-metering` | `gcr.io/distroless/cc-debian12` | <30MB | Metering binary |
| `creto-oversight` | `gcr.io/distroless/cc-debian12` | <30MB | Oversight binary |
| `creto-runtime` | `gcr.io/distroless/cc-debian12` | <50MB | Runtime controller |
| `creto-messaging` | `gcr.io/distroless/cc-debian12` | <30MB | Messaging binary |

### 2.2 Multi-Stage Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build with release optimizations
RUN cargo build --release -p creto-metering

# Runtime stage
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/creto-metering /usr/local/bin/

# Non-root user (65532 = nonroot in distroless)
USER 65532:65532

ENTRYPOINT ["/usr/local/bin/creto-metering"]
```

### 2.3 Image Registry

| Environment | Registry | Path |
|-------------|----------|------|
| Development | GHCR | `ghcr.io/creto-systems/enablement/*` |
| Staging | GCR | `gcr.io/creto-staging/enablement/*` |
| Production | GCR | `gcr.io/creto-prod/enablement/*` |

### 2.4 Image Signing

```yaml
# Cosign signature verification
apiVersion: policy.sigstore.dev/v1beta1
kind: ClusterImagePolicy
metadata:
  name: creto-enablement-policy
spec:
  images:
    - glob: "gcr.io/creto-*/enablement/*"
  authorities:
    - keyless:
        url: https://fulcio.sigstore.dev
        identities:
          - issuer: https://accounts.google.com
            subject: ci@creto.io
```

---

## 3. Kubernetes Resources

### 3.1 Deployment Template

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: creto-metering
  namespace: creto-enablement
  labels:
    app.kubernetes.io/name: creto-metering
    app.kubernetes.io/component: metering
    app.kubernetes.io/part-of: enablement
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: creto-metering
  template:
    metadata:
      labels:
        app.kubernetes.io/name: creto-metering
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
    spec:
      serviceAccountName: creto-metering
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        runAsGroup: 65532
        fsGroup: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: metering
          image: gcr.io/creto-prod/enablement/metering:v1.0.0
          imagePullPolicy: IfNotPresent
          ports:
            - name: grpc
              containerPort: 50051
            - name: metrics
              containerPort: 9090
          env:
            - name: RUST_LOG
              value: "info,creto_metering=debug"
            - name: OTEL_SERVICE_NAME
              value: "creto-metering"
          envFrom:
            - configMapRef:
                name: creto-metering-config
            - secretRef:
                name: creto-metering-secrets
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 1000m
              memory: 512Mi
          livenessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            grpc:
              port: 50051
            initialDelaySeconds: 5
            periodSeconds: 5
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchLabels:
                    app.kubernetes.io/name: creto-metering
                topologyKey: kubernetes.io/hostname
```

### 3.2 Service Definition

```yaml
apiVersion: v1
kind: Service
metadata:
  name: creto-metering
  namespace: creto-enablement
spec:
  type: ClusterIP
  selector:
    app.kubernetes.io/name: creto-metering
  ports:
    - name: grpc
      port: 50051
      targetPort: grpc
    - name: metrics
      port: 9090
      targetPort: metrics
```

### 3.3 HorizontalPodAutoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: creto-metering
  namespace: creto-enablement
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
          averageUtilization: 70
    - type: Pods
      pods:
        metric:
          name: grpc_requests_per_second
        target:
          type: AverageValue
          averageValue: 1000
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

### 3.4 PodDisruptionBudget

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: creto-metering
  namespace: creto-enablement
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app.kubernetes.io/name: creto-metering
```

---

## 4. Configuration Management

### 4.1 ConfigMap Structure

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: creto-metering-config
  namespace: creto-enablement
data:
  # Server configuration
  GRPC_PORT: "50051"
  METRICS_PORT: "9090"

  # Redis configuration
  REDIS_URL: "redis://creto-redis:6379"
  REDIS_POOL_SIZE: "10"

  # PostgreSQL connection
  DATABASE_URL: "postgresql://creto-metering:${DB_PASSWORD}@creto-postgres:5432/metering"

  # Observability
  OTEL_EXPORTER_OTLP_ENDPOINT: "http://otel-collector:4317"

  # Feature flags
  FEATURE_ASYNC_BILLING: "true"
  FEATURE_REAL_TIME_ALERTS: "true"
```

### 4.2 Secrets Management

```yaml
# Sealed Secrets for GitOps
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: creto-metering-secrets
  namespace: creto-enablement
spec:
  encryptedData:
    DB_PASSWORD: AgBy3i4OJSWK+PiTySYZZA9rO4...
    REDIS_PASSWORD: AgBy3i4OJSWK+PiTySYZZA9rO4...
    NHI_SERVICE_KEY: AgBy3i4OJSWK+PiTySYZZA9rO4...
```

### 4.3 External Secrets (Alternative)

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: creto-metering-secrets
  namespace: creto-enablement
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: creto-metering-secrets
  data:
    - secretKey: DB_PASSWORD
      remoteRef:
        key: enablement/metering
        property: db_password
```

---

## 5. Environment Strategy

### 5.1 Environment Matrix

| Environment | Purpose | Cluster | Data | Access |
|-------------|---------|---------|------|--------|
| **local** | Development | Docker Compose | Synthetic | Developers |
| **ci** | PR validation | Kind (ephemeral) | Synthetic | CI system |
| **dev** | Integration | Shared dev cluster | Synthetic | Dev team |
| **staging** | Pre-prod | Dedicated cluster | Anonymized | QA team |
| **prod** | Production | Multi-region | Real | Operations |

### 5.2 Namespace Strategy

```
creto-enablement-dev     # Development environment
creto-enablement-staging # Staging environment
creto-enablement-prod    # Production environment
```

### 5.3 Resource Quotas

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: creto-enablement-quota
  namespace: creto-enablement-prod
spec:
  hard:
    requests.cpu: "50"
    requests.memory: 100Gi
    limits.cpu: "100"
    limits.memory: 200Gi
    pods: "100"
    services: "20"
    persistentvolumeclaims: "20"
```

---

## 6. CI/CD Pipeline

### 6.1 Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        GitHub Actions Pipeline                       │
│                                                                      │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌────────┐ │
│  │  Lint   │──►│  Test   │──►│  Build  │──►│  Push   │──►│ Deploy │ │
│  │ & Check │   │ & Cover │   │ Images  │   │ Registry│   │ (ArgoCD)│ │
│  └─────────┘   └─────────┘   └─────────┘   └─────────┘   └────────┘ │
│       │             │             │             │             │      │
│       ▼             ▼             ▼             ▼             ▼      │
│    clippy        cargo        docker       cosign        gitops     │
│    rustfmt       test         build        sign          sync       │
│    audit         llvm-cov     buildx       sbom          promote    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 GitHub Actions Workflow

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  REGISTRY: gcr.io/creto-prod/enablement

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Format
        run: cargo fmt --all -- --check
      - name: Audit
        run: cargo audit

  test:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7
        ports: [6379:6379]
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
        ports: [5432:5432]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --workspace
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/test
      - name: Coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v3
        with:
          files: lcov.info

  build:
    needs: [lint, test]
    runs-on: ubuntu-latest
    strategy:
      matrix:
        crate: [metering, oversight, runtime, messaging]
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: gcr.io
          username: _json_key
          password: ${{ secrets.GCR_JSON_KEY }}
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: docker/Dockerfile.${{ matrix.crate }}
          push: ${{ github.event_name != 'pull_request' }}
          tags: |
            ${{ env.REGISTRY }}/${{ matrix.crate }}:${{ github.sha }}
            ${{ env.REGISTRY }}/${{ matrix.crate }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

  sign:
    needs: build
    if: github.event_name != 'pull_request'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        crate: [metering, oversight, runtime, messaging]
    steps:
      - uses: sigstore/cosign-installer@v3
      - name: Sign image
        run: |
          cosign sign --yes ${{ env.REGISTRY }}/${{ matrix.crate }}:${{ github.sha }}
      - name: Generate SBOM
        uses: anchore/sbom-action@v0
        with:
          image: ${{ env.REGISTRY }}/${{ matrix.crate }}:${{ github.sha }}
          artifact-name: sbom-${{ matrix.crate }}.spdx

  deploy-staging:
    needs: sign
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v4
      - name: Update manifest
        run: |
          yq e -i '.spec.template.spec.containers[0].image = "${{ env.REGISTRY }}/metering:${{ github.sha }}"' \
            k8s/overlays/staging/metering-patch.yaml
      - name: Commit and push
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add .
          git commit -m "Deploy ${{ github.sha }} to staging"
          git push

  deploy-prod:
    needs: deploy-staging
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v4
      - name: Create production tag
        run: |
          git tag v${{ github.run_number }}
          git push --tags
```

### 6.3 ArgoCD Application

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: creto-enablement
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/creto-systems/enablement
    targetRevision: HEAD
    path: k8s/overlays/prod
  destination:
    server: https://kubernetes.default.svc
    namespace: creto-enablement-prod
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

---

## 7. Observability Stack

### 7.1 Stack Components

| Component | Purpose | Implementation |
|-----------|---------|----------------|
| **Metrics** | Time-series data | Prometheus + Thanos |
| **Logs** | Structured logging | Loki + Promtail |
| **Traces** | Distributed tracing | Tempo + OpenTelemetry |
| **Dashboards** | Visualization | Grafana |
| **Alerts** | Incident detection | Alertmanager |

### 7.2 Prometheus ServiceMonitor

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: creto-metering
  namespace: creto-enablement
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: creto-metering
  endpoints:
    - port: metrics
      interval: 15s
      path: /metrics
```

### 7.3 Grafana Dashboard ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: creto-enablement-dashboards
  labels:
    grafana_dashboard: "1"
data:
  metering.json: |
    {
      "title": "Creto Metering",
      "panels": [
        {
          "title": "Events Ingested/s",
          "targets": [{"expr": "rate(metering_events_total[5m])"}]
        },
        {
          "title": "Quota Check Latency",
          "targets": [{"expr": "histogram_quantile(0.99, rate(metering_quota_check_duration_seconds_bucket[5m]))"}]
        }
      ]
    }
```

---

## 8. Disaster Recovery

### 8.1 Backup Strategy

| Data | Method | Frequency | Retention |
|------|--------|-----------|-----------|
| PostgreSQL | pg_dump to S3 | Hourly | 30 days |
| Redis | RDB snapshots | Every 15 min | 7 days |
| Kubernetes state | Velero | Daily | 30 days |
| Secrets | External backup | Real-time | 90 days |

### 8.2 RTO/RPO Targets

| Tier | RTO | RPO | Services |
|------|-----|-----|----------|
| **Critical** | 15 min | 1 min | Metering quota, Runtime egress |
| **High** | 1 hour | 5 min | Oversight, Messaging delivery |
| **Standard** | 4 hours | 1 hour | Billing aggregation, Reports |

### 8.3 Multi-Region Strategy

```
Primary: us-east-1 (active)
Secondary: eu-west-1 (standby)
Tertiary: ap-southeast-1 (standby)

Failover: DNS-based (Route53 health checks)
Data sync: PostgreSQL logical replication
```

---

## 9. Security Hardening

### 9.1 Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: creto-metering-policy
  namespace: creto-enablement
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: creto-metering
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: istio-system
        - podSelector:
            matchLabels:
              app.kubernetes.io/part-of: enablement
      ports:
        - port: 50051
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: redis
        - podSelector:
            matchLabels:
              app: postgres
      ports:
        - port: 6379
        - port: 5432
    - to:
        - namespaceSelector:
            matchLabels:
              name: creto-platform
      ports:
        - port: 50051
```

### 9.2 Pod Security Standards

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: creto-enablement-prod
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/enforce-version: latest
```

---

## 10. Decisions

| Decision | Rationale |
|----------|-----------|
| Distroless base images | Minimal attack surface, no shell |
| ArgoCD for GitOps | Declarative, auditable deployments |
| Cosign image signing | Supply chain security |
| Sealed Secrets | GitOps-compatible secret management |
| Prometheus/Grafana | Standard observability stack |

---

## 11. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
