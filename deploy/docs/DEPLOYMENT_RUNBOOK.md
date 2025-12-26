# Creto Enablement Deployment Runbook

## Overview

This runbook provides step-by-step instructions for deploying, upgrading, and managing the Creto Enablement services in production.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Initial Deployment](#initial-deployment)
4. [Upgrade Procedures](#upgrade-procedures)
5. [Rollback Procedures](#rollback-procedures)
6. [Health Checks](#health-checks)
7. [Troubleshooting](#troubleshooting)
8. [Emergency Procedures](#emergency-procedures)

---

## Prerequisites

### Required Tools

| Tool | Minimum Version | Purpose |
|------|-----------------|---------|
| kubectl | v1.28+ | Kubernetes CLI |
| helm | v3.13+ | Package manager |
| gcloud/aws/az | latest | Cloud CLI |
| jq | v1.6+ | JSON processing |

### Access Requirements

- [ ] Kubernetes cluster admin access
- [ ] Container registry push/pull permissions
- [ ] Secrets management access (Vault/Secret Manager)
- [ ] Monitoring dashboards access (Grafana)
- [ ] PagerDuty/OpsGenie access for alerts

### Pre-Deployment Checklist

- [ ] All tests passing in CI/CD pipeline
- [ ] Security scan completed with no critical vulnerabilities
- [ ] Change request approved (for production)
- [ ] Rollback plan reviewed
- [ ] On-call engineer notified

---

## Environment Setup

### Configure kubectl Context

```bash
# Production
kubectl config use-context creto-prod

# Staging
kubectl config use-context creto-staging

# Verify context
kubectl config current-context
kubectl get nodes
```

### Set Environment Variables

```bash
# Common variables
export NAMESPACE="creto-enablement"
export RELEASE_NAME="creto-enablement"
export CHART_PATH="deploy/helm/creto-enablement"

# Production
export IMAGE_REGISTRY="gcr.io/creto-prod/enablement"
export IMAGE_TAG="v1.0.0"

# Staging
export IMAGE_REGISTRY="ghcr.io/creto-systems/enablement"
export IMAGE_TAG="main"
```

---

## Initial Deployment

### Step 1: Create Namespace

```bash
kubectl create namespace $NAMESPACE
kubectl label namespace $NAMESPACE \
  app.kubernetes.io/part-of=creto-platform \
  environment=production
```

### Step 2: Create Secrets

```bash
# Create secrets from Vault/Secret Manager
kubectl create secret generic creto-enablement-oversight-secrets \
  --namespace $NAMESPACE \
  --from-literal=slack-webhook-url="$SLACK_WEBHOOK_URL" \
  --from-literal=smtp-host="$SMTP_HOST" \
  --from-literal=smtp-username="$SMTP_USERNAME" \
  --from-literal=smtp-password="$SMTP_PASSWORD"

kubectl create secret generic creto-enablement-metering-secrets \
  --namespace $NAMESPACE \
  --from-literal=stripe-api-key="$STRIPE_API_KEY" \
  --from-literal=database-url="$DATABASE_URL"

kubectl create secret generic creto-enablement-messaging-secrets \
  --namespace $NAMESPACE \
  --from-literal=encryption-master-key="$ENCRYPTION_MASTER_KEY"
```

### Step 3: Deploy with Helm

```bash
# Update Helm dependencies
helm dependency update $CHART_PATH

# Dry-run first
helm upgrade --install $RELEASE_NAME $CHART_PATH \
  --namespace $NAMESPACE \
  --set global.imageRegistry=$IMAGE_REGISTRY \
  --set metering.image.tag=$IMAGE_TAG \
  --set oversight.image.tag=$IMAGE_TAG \
  --set runtime.image.tag=$IMAGE_TAG \
  --set messaging.image.tag=$IMAGE_TAG \
  --dry-run

# Deploy
helm upgrade --install $RELEASE_NAME $CHART_PATH \
  --namespace $NAMESPACE \
  --set global.imageRegistry=$IMAGE_REGISTRY \
  --set metering.image.tag=$IMAGE_TAG \
  --set oversight.image.tag=$IMAGE_TAG \
  --set runtime.image.tag=$IMAGE_TAG \
  --set messaging.image.tag=$IMAGE_TAG \
  --wait --timeout 10m
```

### Step 4: Verify Deployment

```bash
# Check rollout status
kubectl rollout status deployment/$RELEASE_NAME-metering -n $NAMESPACE
kubectl rollout status deployment/$RELEASE_NAME-oversight -n $NAMESPACE
kubectl rollout status deployment/$RELEASE_NAME-runtime -n $NAMESPACE
kubectl rollout status deployment/$RELEASE_NAME-messaging -n $NAMESPACE

# Check pods
kubectl get pods -n $NAMESPACE -l app.kubernetes.io/instance=$RELEASE_NAME

# Check services
kubectl get svc -n $NAMESPACE
```

---

## Upgrade Procedures

### Standard Upgrade (Non-Breaking Changes)

```bash
# Set new version
export NEW_VERSION="v1.1.0"

# Upgrade with Helm
helm upgrade $RELEASE_NAME $CHART_PATH \
  --namespace $NAMESPACE \
  --set global.imageRegistry=$IMAGE_REGISTRY \
  --set metering.image.tag=$NEW_VERSION \
  --set oversight.image.tag=$NEW_VERSION \
  --set runtime.image.tag=$NEW_VERSION \
  --set messaging.image.tag=$NEW_VERSION \
  --wait --timeout 10m

# Verify
kubectl rollout status deployment/$RELEASE_NAME-metering -n $NAMESPACE --timeout=5m
```

### Canary Deployment

```bash
# Deploy canary (10% traffic)
kubectl patch deployment $RELEASE_NAME-metering -n $NAMESPACE \
  -p '{"spec":{"replicas":1}}'

kubectl set image deployment/$RELEASE_NAME-metering \
  metering=$IMAGE_REGISTRY/metering:$NEW_VERSION \
  -n $NAMESPACE

# Monitor for 10 minutes
sleep 600

# If healthy, proceed with full rollout
kubectl set image deployment/$RELEASE_NAME-metering \
  metering=$IMAGE_REGISTRY/metering:$NEW_VERSION \
  -n $NAMESPACE --record

kubectl rollout status deployment/$RELEASE_NAME-metering -n $NAMESPACE
```

### Blue-Green Deployment

```bash
# Create green deployment
helm upgrade --install $RELEASE_NAME-green $CHART_PATH \
  --namespace $NAMESPACE \
  --set nameOverride="creto-enablement-green" \
  --set metering.image.tag=$NEW_VERSION \
  --wait

# Switch traffic (update ingress/service)
kubectl patch service $RELEASE_NAME -n $NAMESPACE \
  -p '{"spec":{"selector":{"app.kubernetes.io/instance":"creto-enablement-green"}}}'

# Cleanup blue after verification
helm uninstall $RELEASE_NAME -n $NAMESPACE
helm upgrade --install $RELEASE_NAME $CHART_PATH \
  --namespace $NAMESPACE \
  --set metering.image.tag=$NEW_VERSION
```

---

## Rollback Procedures

### Using Helm

```bash
# List revisions
helm history $RELEASE_NAME -n $NAMESPACE

# Rollback to previous revision
helm rollback $RELEASE_NAME -n $NAMESPACE

# Rollback to specific revision
helm rollback $RELEASE_NAME 3 -n $NAMESPACE --wait --timeout 5m
```

### Using kubectl

```bash
# Rollback all deployments to previous
for deployment in metering oversight runtime messaging; do
  kubectl rollout undo deployment/$RELEASE_NAME-$deployment -n $NAMESPACE
done

# Wait for rollouts
for deployment in metering oversight runtime messaging; do
  kubectl rollout status deployment/$RELEASE_NAME-$deployment -n $NAMESPACE --timeout=5m
done
```

### Using Rollback Script

```bash
# Rollback to previous (kubectl method)
./deploy/scripts/rollback.sh

# Rollback to specific Helm revision
./deploy/scripts/rollback.sh 3
```

---

## Health Checks

### Service Health

```bash
# Check all pods
kubectl get pods -n $NAMESPACE -o wide

# Check pod health
for pod in $(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/instance=$RELEASE_NAME -o name); do
  echo "=== $pod ==="
  kubectl exec $pod -n $NAMESPACE -- wget -qO- http://localhost:8080/health/ready
done
```

### Metrics Health

```bash
# Check Prometheus targets
kubectl port-forward svc/prometheus -n monitoring 9090:9090 &
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.labels.job | contains("creto"))'

# Check error rates
curl -s 'http://localhost:9090/api/v1/query?query=sum(rate(creto_metering_errors_total[5m]))/sum(rate(creto_metering_requests_total[5m]))'
```

### Database Connectivity

```bash
# Test PostgreSQL connection
kubectl exec -it deployment/$RELEASE_NAME-metering -n $NAMESPACE -- \
  psql "$DATABASE_URL" -c "SELECT 1"

# Test Redis connection
kubectl exec -it deployment/$RELEASE_NAME-messaging -n $NAMESPACE -- \
  redis-cli -h $RELEASE_NAME-redis-master ping
```

---

## Troubleshooting

### Pod Not Starting

```bash
# Check events
kubectl describe pod -n $NAMESPACE -l app.kubernetes.io/component=metering

# Check logs
kubectl logs -n $NAMESPACE -l app.kubernetes.io/component=metering --tail=100

# Check previous container logs
kubectl logs -n $NAMESPACE -l app.kubernetes.io/component=metering --previous
```

### High Latency

```bash
# Check resource utilization
kubectl top pods -n $NAMESPACE

# Check HPA status
kubectl get hpa -n $NAMESPACE

# Scale manually if needed
kubectl scale deployment/$RELEASE_NAME-metering --replicas=5 -n $NAMESPACE
```

### Service Unreachable

```bash
# Check endpoints
kubectl get endpoints -n $NAMESPACE

# Test DNS resolution
kubectl run test-dns --image=busybox --rm -it --restart=Never -- \
  nslookup $RELEASE_NAME-metering.$NAMESPACE.svc.cluster.local

# Test service connectivity
kubectl run test-curl --image=curlimages/curl --rm -it --restart=Never -- \
  curl -v http://$RELEASE_NAME-metering.$NAMESPACE.svc.cluster.local:8080/health/live
```

---

## Emergency Procedures

### Complete Service Outage

1. **Assess impact**
   ```bash
   kubectl get pods -n $NAMESPACE
   kubectl get events -n $NAMESPACE --sort-by='.lastTimestamp' | tail -20
   ```

2. **Immediate rollback**
   ```bash
   helm rollback $RELEASE_NAME -n $NAMESPACE --wait --timeout 5m
   ```

3. **Scale up if needed**
   ```bash
   kubectl scale deployment/$RELEASE_NAME-metering --replicas=10 -n $NAMESPACE
   ```

4. **Notify stakeholders**
   - Update status page
   - Send incident notification
   - Page on-call SRE

### Database Corruption

1. **Isolate affected service**
   ```bash
   kubectl scale deployment/$RELEASE_NAME-metering --replicas=0 -n $NAMESPACE
   ```

2. **Check database status**
   ```bash
   kubectl exec -it $RELEASE_NAME-postgresql-0 -n $NAMESPACE -- \
     pg_isready -U postgres
   ```

3. **Restore from backup if needed**
   ```bash
   kubectl exec -it $RELEASE_NAME-postgresql-0 -n $NAMESPACE -- \
     pg_restore -U postgres -d creto_enablement /backup/latest.dump
   ```

### Security Incident

1. **Isolate affected pods**
   ```bash
   kubectl label pods -n $NAMESPACE -l app.kubernetes.io/instance=$RELEASE_NAME \
     quarantine=true
   kubectl apply -f - <<EOF
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: quarantine
     namespace: $NAMESPACE
   spec:
     podSelector:
       matchLabels:
         quarantine: "true"
     policyTypes:
     - Ingress
     - Egress
   EOF
   ```

2. **Capture forensic data**
   ```bash
   kubectl logs -n $NAMESPACE -l quarantine=true --all-containers > incident-logs.txt
   kubectl get pods -n $NAMESPACE -l quarantine=true -o yaml > incident-pods.yaml
   ```

3. **Rotate secrets**
   ```bash
   kubectl delete secret creto-enablement-oversight-secrets -n $NAMESPACE
   # Recreate with new values
   ```

---

## Contact Information

| Role | Name | Contact |
|------|------|---------|
| On-Call SRE | Rotation | pagerduty.com/creto-enablement |
| Platform Lead | [Name] | slack: #platform-team |
| Security | [Name] | security@creto.io |

---

## Appendix

### Useful Commands Cheatsheet

```bash
# Get all resources
kubectl get all -n $NAMESPACE

# Watch pods
kubectl get pods -n $NAMESPACE -w

# Port forward to service
kubectl port-forward svc/$RELEASE_NAME-metering 8080:8080 -n $NAMESPACE

# Execute into pod
kubectl exec -it deployment/$RELEASE_NAME-metering -n $NAMESPACE -- /bin/sh

# Get Helm values
helm get values $RELEASE_NAME -n $NAMESPACE

# Check resource quotas
kubectl describe resourcequota -n $NAMESPACE
```

### Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2024-12-26 | Platform Team | Initial runbook |
