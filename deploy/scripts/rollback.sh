#!/bin/bash
# Creto Enablement Rollback Script
# Phase 6: Production Release - Rollback Procedures

set -euo pipefail

NAMESPACE="${NAMESPACE:-creto-enablement}"
ROLLBACK_REVISION="${1:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found. Please install kubectl."
        exit 1
    fi

    if ! command -v helm &> /dev/null; then
        log_error "helm not found. Please install helm."
        exit 1
    fi

    if ! kubectl auth can-i get deployments -n "$NAMESPACE" &> /dev/null; then
        log_error "Insufficient permissions for namespace $NAMESPACE"
        exit 1
    fi

    log_info "Prerequisites check passed."
}

# Show current state
show_current_state() {
    log_info "Current deployment state:"
    echo ""

    kubectl get deployments -n "$NAMESPACE" -o wide
    echo ""

    log_info "Recent helm releases:"
    helm history creto-enablement -n "$NAMESPACE" 2>/dev/null | tail -5 || echo "No helm history found"
    echo ""
}

# Rollback using Helm
helm_rollback() {
    local revision="$1"

    log_info "Rolling back to Helm revision $revision..."

    helm rollback creto-enablement "$revision" -n "$NAMESPACE" --wait --timeout 5m

    log_info "Helm rollback completed."
}

# Rollback using kubectl
kubectl_rollback() {
    local deployments=("creto-metering" "creto-oversight" "creto-runtime" "creto-messaging")

    log_info "Rolling back all deployments to previous revision..."

    for deployment in "${deployments[@]}"; do
        if kubectl get deployment "$deployment" -n "$NAMESPACE" &> /dev/null; then
            log_info "Rolling back $deployment..."
            kubectl rollout undo deployment/"$deployment" -n "$NAMESPACE"
        else
            log_warn "Deployment $deployment not found, skipping."
        fi
    done

    log_info "Waiting for rollouts to complete..."
    for deployment in "${deployments[@]}"; do
        if kubectl get deployment "$deployment" -n "$NAMESPACE" &> /dev/null; then
            kubectl rollout status deployment/"$deployment" -n "$NAMESPACE" --timeout=5m
        fi
    done

    log_info "kubectl rollback completed."
}

# Health check after rollback
health_check() {
    log_info "Running health checks..."

    local failed=0
    local services=("creto-metering" "creto-oversight" "creto-runtime" "creto-messaging")

    for service in "${services[@]}"; do
        local ready
        ready=$(kubectl get deployment "$service" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
        local desired
        desired=$(kubectl get deployment "$service" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")

        if [[ "$ready" -ge "$desired" && "$desired" -gt "0" ]]; then
            log_info "$service: ${ready}/${desired} pods ready ✓"
        else
            log_error "$service: ${ready}/${desired} pods ready ✗"
            failed=1
        fi
    done

    if [[ $failed -eq 1 ]]; then
        log_error "Health check failed!"
        exit 1
    fi

    log_info "All health checks passed!"
}

# Main execution
main() {
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║          Creto Enablement Rollback Procedure                  ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""

    check_prerequisites
    show_current_state

    if [[ -n "$ROLLBACK_REVISION" ]]; then
        # Specific revision provided - use Helm
        helm_rollback "$ROLLBACK_REVISION"
    else
        # No revision - use kubectl to rollback to previous
        kubectl_rollback
    fi

    health_check

    echo ""
    log_info "Rollback completed successfully!"
    echo ""

    show_current_state
}

# Usage
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    echo "Usage: $0 [HELM_REVISION]"
    echo ""
    echo "Rollback Creto Enablement services."
    echo ""
    echo "Arguments:"
    echo "  HELM_REVISION  Optional. Specific Helm revision to rollback to."
    echo "                 If not provided, rolls back each deployment to previous revision."
    echo ""
    echo "Environment variables:"
    echo "  NAMESPACE      Kubernetes namespace (default: creto-enablement)"
    echo ""
    echo "Examples:"
    echo "  $0           # Rollback to previous revision"
    echo "  $0 3         # Rollback to Helm revision 3"
    exit 0
fi

main "$@"
