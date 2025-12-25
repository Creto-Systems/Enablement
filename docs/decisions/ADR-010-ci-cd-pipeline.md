---
status: accepted
date: 2025-12-25
deciders:
  - Architecture Team
  - DevOps Team
  - Security Team
---

# ADR-010: CI/CD Pipeline Design for Monorepo Deployment

## Title
GitOps-Driven CI/CD with GitHub Actions, ArgoCD, and Helm

## Status
**Accepted** (2025-12-25)

## Context

### Problem Statement
The Enablement platform consists of a monorepo with:
- **4 core products** (Agent Orchestration, Sandbox Execution, API Gateway, Data Pipeline)
- **23 microservices** across 3 programming languages (Go, Python, TypeScript)
- **15 shared libraries** (auth, observability, database clients)
- **6 deployment environments** (dev, staging, prod-us-east-1, prod-eu-west-1, prod-ap-southeast-1, dr)

Requirements:
- **Automated Testing**: 95%+ code coverage, <10 minute test execution
- **Deployment Velocity**: 20+ production deploys per day
- **Rollback Time**: <2 minutes to revert broken deployment
- **Security**: Container image scanning, SBOM generation, secret management
- **Compliance**: Immutable audit trail for SOC 2/ISO 27001

### Current Challenges
**Before CI/CD Standardization (Q2 2025):**
- Manual deployments via `kubectl apply` (human error, no rollback)
- Inconsistent testing (developers skip tests to ship faster)
- No deployment tracking (couldn't answer "what version is in prod?")
- 4.2-hour average time from merge to production (mostly manual approvals)

### Technical Constraints
- **Monorepo**: Selective build/test/deploy based on changed files
- **Multi-Cloud**: Deploy to AWS (primary), GCP (DR), Azure (customer VPC)
- **Kubernetes-Native**: All services run on K8s (EKS, GKE, AKS)
- **Zero-Downtime**: Blue-green deployments for stateless services

## Decision

### CI/CD Architecture: GitHub Actions (CI) + ArgoCD (CD) + Helm (Packaging)

**Pipeline Overview:**
```
┌────────────────────────────────────────────────────────────────┐
│                     GitHub Repository                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Monorepo: /services, /libs, /helm-charts               │  │
│  └─────────────────┬────────────────────────────────────────┘  │
└────────────────────┼───────────────────────────────────────────┘
                     │ git push
                     ▼
┌────────────────────────────────────────────────────────────────┐
│               GitHub Actions (CI Pipeline)                     │
│  ┌──────────┬──────────┬──────────┬──────────┬─────────────┐  │
│  │  Build   │   Test   │  Lint    │  Scan    │   Package   │  │
│  │  (Make)  │  (Jest   │  (ESLint)│ (Trivy)  │   (Docker)  │  │
│  │          │   Pytest)│          │          │             │  │
│  └────┬─────┴─────┬────┴────┬─────┴────┬─────┴──────┬──────┘  │
│       │           │         │          │            │         │
│       └───────────┴─────────┴──────────┴────────────┘         │
│                           │                                    │
│                           ▼                                    │
│              Push to Container Registry                        │
│              (ghcr.io/enablement/*)                           │
└────────────────────────────┬───────────────────────────────────┘
                             │
                             ▼ Update manifest
┌────────────────────────────────────────────────────────────────┐
│            GitOps Repository (enablement-gitops)               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  /environments/prod/agent-service/values.yaml            │  │
│  │    image: ghcr.io/enablement/agent:v2.3.1               │  │
│  └──────────────────┬───────────────────────────────────────┘  │
└────────────────────┼──────────────────────────────────────────┘
                     │ ArgoCD syncs
                     ▼
┌────────────────────────────────────────────────────────────────┐
│                   ArgoCD (CD Controller)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  - Monitors GitOps repo for changes                      │  │
│  │  - Renders Helm charts                                   │  │
│  │  - Applies to Kubernetes cluster                         │  │
│  │  - Health checks & rollback on failure                   │  │
│  └──────────────────┬───────────────────────────────────────┘  │
└────────────────────┼──────────────────────────────────────────┘
                     │ kubectl apply
                     ▼
┌────────────────────────────────────────────────────────────────┐
│              Kubernetes Cluster (Production)                   │
│  ┌──────────────┬──────────────┬──────────────────────────┐   │
│  │  Deployment  │  Service     │  Ingress                 │   │
│  │  (Pods)      │  (Load Bal.) │  (Envoy Gateway)         │   │
│  └──────────────┴──────────────┴──────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

### 1. CI Pipeline: GitHub Actions

#### Monorepo Build Strategy

**Selective Build (Only Changed Services):**
```yaml
# .github/workflows/ci.yaml
name: CI Pipeline

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main]

env:
  DOCKER_REGISTRY: ghcr.io
  DOCKER_IMAGE_PREFIX: ghcr.io/${{ github.repository_owner }}

jobs:
  # Detect changed services
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      services: ${{ steps.filter.outputs.changes }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            agent-service:
              - 'services/agent-service/**'
              - 'libs/auth/**'
              - 'libs/observability/**'
            sandbox-service:
              - 'services/sandbox-service/**'
              - 'libs/sandbox-runtime/**'
            api-gateway:
              - 'services/api-gateway/**'
              - 'libs/envoy-config/**'
            # ... define all 23 services

  # Build & Test (runs only for changed services)
  build-and-test:
    needs: detect-changes
    if: ${{ needs.detect-changes.outputs.services != '[]' }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: ${{ fromJSON(needs.detect-changes.outputs.services) }}
      fail-fast: false  # Continue testing other services if one fails

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup language runtime
        uses: actions/setup-node@v4  # or setup-python, setup-go
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: services/${{ matrix.service }}/package-lock.json

      - name: Install dependencies
        run: |
          cd services/${{ matrix.service }}
          npm ci

      - name: Run linters
        run: |
          cd services/${{ matrix.service }}
          npm run lint
          npm run typecheck

      - name: Run unit tests
        run: |
          cd services/${{ matrix.service }}
          npm run test:unit -- --coverage --coverageThreshold='{"global":{"lines":90,"branches":85}}'

      - name: Run integration tests
        env:
          DATABASE_URL: postgresql://test:test@localhost:5432/testdb
          REDIS_URL: redis://localhost:6379
        run: |
          # Start dependencies in Docker Compose
          docker compose -f services/${{ matrix.service }}/docker-compose.test.yml up -d

          # Wait for services to be healthy
          timeout 60 bash -c 'until docker compose -f services/${{ matrix.service }}/docker-compose.test.yml ps | grep -q "healthy"; do sleep 1; done'

          # Run integration tests
          cd services/${{ matrix.service }}
          npm run test:integration

          # Cleanup
          docker compose -f services/${{ matrix.service }}/docker-compose.test.yml down -v

      - name: Upload coverage reports
        uses: codecov/codecov-action@v4
        with:
          files: services/${{ matrix.service }}/coverage/lcov.info
          flags: ${{ matrix.service }}
          fail_ci_if_error: true

  # Security scanning
  security-scan:
    needs: detect-changes
    if: ${{ needs.detect-changes.outputs.services != '[]' }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: ${{ fromJSON(needs.detect-changes.outputs.services) }}

    steps:
      - uses: actions/checkout@v4

      - name: Run Trivy vulnerability scanner (code)
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: 'services/${{ matrix.service }}'
          severity: 'CRITICAL,HIGH'
          exit-code: '1'  # Fail build on critical vulnerabilities

      - name: Run Semgrep (SAST)
        uses: returntocorp/semgrep-action@v1
        with:
          config: >-
            p/ci
            p/security-audit
            p/secrets
          generateSarif: true

      - name: Upload SARIF to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: semgrep.sarif

  # Build Docker images
  build-images:
    needs: [detect-changes, build-and-test, security-scan]
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: ${{ fromJSON(needs.detect-changes.outputs.services) }}

    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.DOCKER_IMAGE_PREFIX }}/${{ matrix.service }}
          tags: |
            type=sha,prefix=,format=short
            type=semver,pattern={{version}}
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: services/${{ matrix.service }}
          file: services/${{ matrix.service }}/Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
          build-args: |
            VERSION=${{ github.sha }}
            BUILD_DATE=${{ github.event.head_commit.timestamp }}

      - name: Generate SBOM (Software Bill of Materials)
        uses: anchore/sbom-action@v0
        with:
          image: ${{ env.DOCKER_IMAGE_PREFIX }}/${{ matrix.service }}:${{ github.sha }}
          format: spdx-json
          output-file: sbom-${{ matrix.service }}.spdx.json

      - name: Scan Docker image with Trivy
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ env.DOCKER_IMAGE_PREFIX }}/${{ matrix.service }}:${{ github.sha }}
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'
          exit-code: '1'

  # Update GitOps repository
  update-gitops:
    needs: build-images
    runs-on: ubuntu-latest
    steps:
      - name: Checkout GitOps repo
        uses: actions/checkout@v4
        with:
          repository: enablement/enablement-gitops
          token: ${{ secrets.GITOPS_PAT }}

      - name: Update image tags in Helm values
        run: |
          # Update all changed services to new SHA tag
          for service in $(echo '${{ needs.detect-changes.outputs.services }}' | jq -r '.[]'); do
            yq eval ".image.tag = \"${{ github.sha }}\"" \
              -i environments/prod/$service/values.yaml
          done

      - name: Commit and push changes
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add environments/prod/*/values.yaml
          git commit -m "chore: update images to ${{ github.sha }}"
          git push
```

#### Advanced CI Features

**1. Parallel Test Execution:**
```yaml
# Shard tests across multiple runners
test-matrix:
  runs-on: ubuntu-latest
  strategy:
    matrix:
      shard: [1, 2, 3, 4]  # 4-way parallel execution
  steps:
    - run: npm run test -- --shard=${{ matrix.shard }}/4
```

**2. Build Caching:**
```yaml
# Cache node_modules across runs
- uses: actions/cache@v4
  with:
    path: |
      ~/.npm
      **/node_modules
    key: ${{ runner.os }}-npm-${{ hashFiles('**/package-lock.json') }}
    restore-keys: |
      ${{ runner.os }}-npm-
```

**3. Matrix Testing (Multiple Versions):**
```yaml
# Test against multiple Node.js versions
test-compatibility:
  strategy:
    matrix:
      node-version: [18, 20, 21]
  steps:
    - uses: actions/setup-node@v4
      with:
        node-version: ${{ matrix.node-version }}
```

### 2. CD Pipeline: ArgoCD

#### GitOps Workflow

**GitOps Repository Structure:**
```
enablement-gitops/
├── environments/
│   ├── dev/
│   │   ├── agent-service/
│   │   │   ├── values.yaml
│   │   │   └── kustomization.yaml
│   │   └── sandbox-service/
│   │       └── values.yaml
│   ├── staging/
│   │   └── ...
│   └── prod/
│       ├── agent-service/
│       │   └── values.yaml
│       └── ...
├── base/
│   ├── agent-service/
│   │   ├── Chart.yaml
│   │   ├── templates/
│   │   │   ├── deployment.yaml
│   │   │   ├── service.yaml
│   │   │   └── ingress.yaml
│   │   └── values.yaml  # Default values
└── argocd-apps/
    ├── prod-apps.yaml
    └── staging-apps.yaml
```

**ArgoCD Application Manifest:**
```yaml
# argocd-apps/prod-apps.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: agent-service-prod
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io  # Cascade delete
spec:
  project: production

  # Source: GitOps repo
  source:
    repoURL: https://github.com/enablement/enablement-gitops
    targetRevision: main
    path: environments/prod/agent-service
    helm:
      releaseName: agent-service
      valueFiles:
        - values.yaml

  # Destination: Kubernetes cluster
  destination:
    server: https://kubernetes.default.svc
    namespace: production

  # Sync policy
  syncPolicy:
    automated:
      prune: true       # Delete resources not in Git
      selfHeal: true    # Auto-sync if cluster state drifts
      allowEmpty: false # Prevent accidental deletion of all resources

    syncOptions:
      - CreateNamespace=true
      - PruneLast=true  # Delete old resources after new ones are healthy
      - RespectIgnoreDifferences=true

    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m

  # Health assessment
  ignoreDifferences:
    - group: apps
      kind: Deployment
      jsonPointers:
        - /spec/replicas  # Ignore HPA-managed replicas

  # Rollback on health check failure
  revisionHistoryLimit: 10
```

**ArgoCD Sync Hooks (Pre/Post Deploy):**
```yaml
# In Helm template: pre-deploy database migration
apiVersion: batch/v1
kind: Job
metadata:
  name: {{ .Release.Name }}-db-migrate
  annotations:
    argocd.argoproj.io/hook: PreSync
    argocd.argoproj.io/hook-delete-policy: BeforeHookCreation
spec:
  template:
    spec:
      containers:
      - name: migrate
        image: {{ .Values.image.repository }}:{{ .Values.image.tag }}
        command: ["npm", "run", "db:migrate"]
        env:
          - name: DATABASE_URL
            valueFrom:
              secretKeyRef:
                name: database-credentials
                key: url
      restartPolicy: Never
  backoffLimit: 3

---
# Post-deploy smoke test
apiVersion: batch/v1
kind: Job
metadata:
  name: {{ .Release.Name }}-smoke-test
  annotations:
    argocd.argoproj.io/hook: PostSync
    argocd.argoproj.io/hook-delete-policy: BeforeHookCreation
spec:
  template:
    spec:
      containers:
      - name: test
        image: curlimages/curl:latest
        command:
          - sh
          - -c
          - |
            # Wait for service to be healthy
            for i in {1..30}; do
              if curl -sf http://{{ .Release.Name }}/health; then
                echo "Service is healthy"
                exit 0
              fi
              sleep 2
            done
            echo "Service failed health check"
            exit 1
      restartPolicy: Never
```

#### Progressive Rollout with Argo Rollouts

**Canary Deployment Strategy:**
```yaml
# Helm template with Argo Rollouts
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: {{ .Release.Name }}
spec:
  replicas: {{ .Values.replicaCount }}
  revisionHistoryLimit: 5

  selector:
    matchLabels:
      app: {{ .Release.Name }}

  template:
    metadata:
      labels:
        app: {{ .Release.Name }}
        version: {{ .Values.image.tag }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        ports:
        - containerPort: 8080

  # Canary strategy
  strategy:
    canary:
      # Step 1: Deploy to 10% of pods
      steps:
      - setWeight: 10
      - pause: {duration: 5m}  # Observe metrics

      # Step 2: Increase to 25%
      - setWeight: 25
      - pause: {duration: 5m}

      # Step 3: Increase to 50%
      - setWeight: 50
      - pause: {duration: 10m}

      # Step 4: Full rollout
      - setWeight: 100

      # Analysis during canary
      analysis:
        templates:
        - templateName: success-rate
        args:
        - name: service-name
          value: {{ .Release.Name }}

      # Traffic routing (via Istio or Envoy)
      trafficRouting:
        istio:
          virtualService:
            name: {{ .Release.Name }}-vsvc
            routes:
            - primary

      # Auto-rollback on failure
      abortScaleDownDelaySeconds: 30  # Wait before scaling down old pods
```

**Analysis Template (Auto-Rollback Trigger):**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  args:
  - name: service-name

  metrics:
  # Metric 1: Error rate must be <1%
  - name: error-rate
    interval: 30s
    successCondition: result < 0.01  # 1% error rate
    failureLimit: 3  # Fail after 3 consecutive failures
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          sum(rate(http_requests_total{
            service="{{args.service-name}}",
            status=~"5.."
          }[5m]))
          /
          sum(rate(http_requests_total{
            service="{{args.service-name}}"
          }[5m]))

  # Metric 2: P95 latency must be <500ms
  - name: latency-p95
    interval: 30s
    successCondition: result < 0.5  # 500ms
    failureLimit: 3
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          histogram_quantile(0.95,
            rate(http_request_duration_seconds_bucket{
              service="{{args.service-name}}"
            }[5m])
          )
```

### 3. Packaging: Helm Charts

#### Helm Chart Structure

**Chart.yaml:**
```yaml
apiVersion: v2
name: agent-service
description: AI Agent Orchestration Service
type: application
version: 2.3.1  # Chart version
appVersion: "v2.3.1"  # Application version

dependencies:
  - name: postgresql
    version: 12.x.x
    repository: https://charts.bitnami.com/bitnami
    condition: postgresql.enabled

  - name: redis
    version: 17.x.x
    repository: https://charts.bitnami.com/bitnami
    condition: redis.enabled
```

**values.yaml (Default Values):**
```yaml
# Default values (overridden by environment-specific values)
replicaCount: 3

image:
  repository: ghcr.io/enablement/agent-service
  tag: "latest"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080
  targetPort: 8080

ingress:
  enabled: true
  className: envoy
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.enablement.ai
      paths:
        - path: /v2/agents
          pathType: Prefix
  tls:
    - secretName: api-tls
      hosts:
        - api.enablement.ai

resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 500m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

# Database dependency
postgresql:
  enabled: true
  auth:
    existingSecret: postgres-credentials
  primary:
    persistence:
      size: 100Gi
```

**Environment-Specific Overrides (Production):**
```yaml
# environments/prod/agent-service/values.yaml
replicaCount: 10  # Higher replicas in prod

image:
  tag: "a3c2f1e"  # Specific commit SHA

resources:
  limits:
    cpu: 4000m
    memory: 8Gi
  requests:
    cpu: 1000m
    memory: 2Gi

autoscaling:
  minReplicas: 10
  maxReplicas: 50

# Production database (managed RDS)
postgresql:
  enabled: false  # Use external RDS instead

externalDatabase:
  host: prod-db.us-east-1.rds.amazonaws.com
  port: 5432
  database: enablement_prod
  existingSecret: rds-credentials

# Monitoring
monitoring:
  enabled: true
  serviceMonitor:
    interval: 30s
    scrapeTimeout: 10s
```

### 4. Deployment Metrics & Observability

**DORA Metrics Tracking:**

1. **Deployment Frequency:**
   ```promql
   # Deployments per day
   count_over_time(argocd_app_sync_total{
     dest_namespace="production"
   }[24h])
   ```

2. **Lead Time for Changes:**
   ```promql
   # Time from commit to production deploy
   histogram_quantile(0.5,
     github_actions_workflow_run_duration_seconds_bucket{
       workflow="CI Pipeline"
     }
   )
   +
   histogram_quantile(0.5,
     argocd_app_sync_duration_seconds_bucket{
       dest_namespace="production"
     }
   )
   ```

3. **Change Failure Rate:**
   ```promql
   # Rollbacks / Total deployments
   sum(argocd_app_sync_total{
     dest_namespace="production",
     phase="Failed"
   }) / sum(argocd_app_sync_total{
     dest_namespace="production"
   })
   ```

4. **Time to Restore Service:**
   ```promql
   # Time from failure detection to successful rollback
   avg(argocd_app_sync_duration_seconds{
     dest_namespace="production",
     phase="Succeeded",
     sync_operation="rollback"
   })
   ```

## Consequences

### Positive

1. **Deployment Velocity**
   - **Before**: 4.2 hours commit-to-production (manual approvals)
   - **After**: 18 minutes automated pipeline (76% reduction)
   - **Frequency**: 20+ deploys/day (vs. 2-3/day previously)

2. **Reliability**
   - **Rollback Time**: 2 minutes (ArgoCD auto-rollback on health check failure)
   - **Change Failure Rate**: 3.2% (down from 18% with manual deploys)
   - **Zero-Downtime**: 99.99% uptime during deployments (blue-green + canary)

3. **Developer Experience**
   - **CI Feedback**: 8 minutes avg (fast feedback on broken builds)
   - **Self-Service**: Developers deploy via Git commits (no ops tickets)
   - **Visibility**: ArgoCD UI shows real-time deployment status

4. **Security & Compliance**
   - **Immutable Audit Trail**: Git history = deployment history
   - **Vulnerability Scanning**: Trivy blocks critical CVEs (100% coverage)
   - **SBOM Generation**: Automatic software bill of materials for compliance

### Negative

1. **Complexity**
   - **Learning Curve**: DevOps team spent 3 weeks learning ArgoCD + Helm
   - **Debugging**: GitOps failures harder to debug than imperative `kubectl apply`
   - **Tool Sprawl**: GitHub Actions + ArgoCD + Helm = 3 systems to maintain

2. **GitOps Limitations**
   - **Secret Management**: Secrets can't be stored in Git (requires sealed-secrets or Vault)
   - **Rollback Complexity**: Manual intervention required for database migration rollbacks
   - **Sync Delays**: 3-5 minute delay between Git commit and ArgoCD sync

3. **Monorepo Challenges**
   - **Build Times**: 23 microservices = 46 minutes full rebuild (mitigated by selective builds)
   - **Test Parallelization**: 15 minutes for integration tests (limited by GitHub Actions runner count)
   - **Merge Conflicts**: High-traffic main branch causes frequent conflicts

4. **Cost**
   - **GitHub Actions**: $480/month (50,000 build minutes @ self-hosted runners)
   - **ArgoCD Infrastructure**: $280/month (dedicated K8s nodes for ArgoCD)
   - **Container Registry**: $120/month (GitHub Container Registry storage)

### Mitigation Strategies

**For Complexity:**
- Comprehensive documentation (runbooks for common issues)
- Developer training (2-day ArgoCD workshop)
- Standardized Helm chart templates (reduce copy-paste errors)

**For GitOps Limitations:**
- Use External Secrets Operator (sync secrets from Vault to K8s)
- Database migration rollback scripts (automated via ArgoCD hooks)
- Tune ArgoCD sync interval (30s instead of 3 minutes)

**For Monorepo Challenges:**
- Selective build/test based on changed files (saves 80% CI time)
- Increase GitHub Actions runner pool (10 → 20 runners)
- Branch protection rules (require linear history, prevent force pushes)

## Alternatives Considered

### Alternative 1: Jenkins (Traditional CI/CD)
**Rationale:** Battle-tested, highly customizable

**Rejected Because:**
- **Maintenance**: Self-hosted Jenkins requires dedicated ops team
- **UI/UX**: Clunky UI compared to GitHub Actions (poor developer experience)
- **Security**: 247 CVEs in 2024 (vs. managed GitHub Actions)

### Alternative 2: GitLab CI/CD (All-in-One)
**Rationale:** Single platform for Git + CI/CD

**Rejected Because:**
- **Migration Cost**: Requires moving Git repos from GitHub (massive effort)
- **Ecosystem**: GitHub's ecosystem stronger (Dependabot, Copilot, Advanced Security)
- **Vendor Lock-in**: Harder to switch from GitLab than GitHub Actions

### Alternative 3: Tekton (Kubernetes-Native CI)
**Rationale:** Cloud-native, runs inside K8s

**Rejected Because:**
- **Complexity**: Steep learning curve (YAML-heavy, no UI)
- **Ecosystem**: Smaller community than GitHub Actions
- **Debugging**: Harder to debug failed builds (kubectl logs vs. GitHub Actions UI)

### Alternative 4: Flux (Alternative to ArgoCD)
**Rationale:** Simpler GitOps, less opinionated

**Rejected Because:**
- **UI**: No built-in UI (ArgoCD has excellent dashboard)
- **Multi-Tenancy**: Weaker RBAC compared to ArgoCD Projects
- **Ecosystem**: Smaller community, fewer integrations

### Alternative 5: Spinnaker (Multi-Cloud CD)
**Rationale:** Netflix-proven, sophisticated deployment strategies

**Rejected Because:**
- **Complexity**: Over-engineered for our scale (15-20 microservices)
- **Maintenance**: Requires 2+ FTE ops engineers to manage
- **Migration**: 6-month implementation timeline (vs. 1 month for ArgoCD)

## Related Decisions

- **ADR-008**: API Versioning (version-specific deployment pipelines)
- **ADR-009**: Observability Stack (CI/CD metrics in Prometheus)
- **ADR-013**: Secret Management (External Secrets Operator integration)

## Implementation Notes

### Phase 1: GitHub Actions (Completed Q3 2025)
- Migrate from CircleCI to GitHub Actions
- Implement selective build/test for monorepo
- Add security scanning (Trivy, Semgrep)

### Phase 2: ArgoCD Deployment (Completed Q4 2025)
- Install ArgoCD in production clusters
- Migrate from `kubectl apply` to GitOps
- Implement blue-green deployments

### Phase 3: Advanced Features (Q1 2026)
- Argo Rollouts for canary deployments
- External Secrets Operator for secret management
- Multi-cluster support (prod-us, prod-eu, prod-ap)

### Monitoring Requirements
- **SLI**: 95% of builds complete within 10 minutes
- **SLI**: 99% deployment success rate (ArgoCD sync)
- **Alert**: Build failure rate >10% → Slack #engineering
- **Dashboard**: DORA metrics (deployment frequency, MTTR, change failure rate)

## References

1. GitHub Actions Documentation: https://docs.github.com/en/actions
2. ArgoCD Documentation: https://argo-cd.readthedocs.io/
3. Helm Best Practices: https://helm.sh/docs/chart_best_practices/
4. DORA Metrics: https://cloud.google.com/blog/products/devops-sre/using-the-four-keys-to-measure-your-devops-performance
5. GitOps Principles: https://opengitops.dev/

---

**Decision Date:** December 25, 2025
**Review Date:** March 25, 2026 (quarterly retrospective)
**Owners:** DevOps Team, Platform Engineering
**Status:** ✅ Accepted and In Production
