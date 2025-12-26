//! gRPC service implementation for metering.
//!
//! This service handles event ingestion with validation, deduplication,
//! and batching for high throughput.

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error, instrument};

use crate::dedup::{DedupResult, Deduplicator};
use crate::events::EventIngestion;
use crate::grpc::types::*;
use crate::quota::QuotaEnforcer;
use crate::validation::{EventValidator, ValidationConfig};

/// Configuration for the gRPC metering service.
#[derive(Debug, Clone)]
pub struct MeteringServiceConfig {
    /// Maximum batch size for ingestion.
    pub max_batch_size: usize,
    /// Whether to enforce quotas on ingestion.
    pub enforce_quotas: bool,
    /// Validation configuration.
    pub validation: ValidationConfig,
}

impl Default for MeteringServiceConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            enforce_quotas: true,
            validation: ValidationConfig::default(),
        }
    }
}

/// gRPC service for metering operations.
///
/// This service provides:
/// - Single event ingestion with validation and deduplication
/// - Batch ingestion for high throughput
/// - Quota checking
pub struct MeteringGrpcService<I: EventIngestion> {
    ingestion: Arc<I>,
    deduplicator: Arc<Deduplicator>,
    quota_enforcer: Arc<QuotaEnforcer>,
    validator: EventValidator,
    config: MeteringServiceConfig,
    /// Metrics for monitoring.
    metrics: Arc<RwLock<ServiceMetrics>>,
}

impl<I: EventIngestion> MeteringGrpcService<I> {
    /// Create a new metering gRPC service.
    pub fn new(
        ingestion: Arc<I>,
        deduplicator: Arc<Deduplicator>,
        quota_enforcer: Arc<QuotaEnforcer>,
        config: MeteringServiceConfig,
    ) -> Self {
        Self {
            ingestion,
            deduplicator,
            quota_enforcer,
            validator: EventValidator::new(config.validation.clone()),
            config,
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
        }
    }

    /// Ingest a single event.
    #[instrument(skip(self, request), fields(txn_id = %request.event.transaction_id))]
    pub async fn ingest_event(&self, request: IngestEventRequest) -> IngestEventResponse {
        // Convert gRPC event to internal event
        let event = match request.event.to_usage_event() {
            Ok(e) => e,
            Err(msg) => {
                return IngestEventResponse {
                    success: false,
                    status: IngestStatus::ValidationError,
                    error_message: Some(msg),
                };
            }
        };

        // Validate
        if let Err(e) = self.validator.validate(&event) {
            self.record_validation_error().await;
            return IngestEventResponse {
                success: false,
                status: IngestStatus::ValidationError,
                error_message: Some(e.to_string()),
            };
        }

        // Deduplicate
        match self.deduplicator.check_and_mark(&event.transaction_id).await {
            Ok(DedupResult::Duplicate) => {
                self.record_duplicate().await;
                return IngestEventResponse {
                    success: true,
                    status: IngestStatus::Duplicate,
                    error_message: None,
                };
            }
            Ok(DedupResult::New) => {}
            Err(e) => {
                error!("Deduplication check failed: {}", e);
                // Continue without dedup on error - DB will handle via ON CONFLICT
            }
        }

        // Check quota if enabled
        if self.config.enforce_quotas {
            let quota_result = self
                .quota_enforcer
                .check(&event.organization_id, &event.agent_id, &event.code, event.quantity);

            match quota_result {
                Ok(check) if !check.allowed => {
                    self.record_quota_exceeded().await;
                    return IngestEventResponse {
                        success: false,
                        status: IngestStatus::QuotaExceeded,
                        error_message: Some(format!("Quota exceeded: {}% used", check.usage_percentage * 100.0)),
                    };
                }
                Err(e) => {
                    self.record_quota_exceeded().await;
                    return IngestEventResponse {
                        success: false,
                        status: IngestStatus::QuotaExceeded,
                        error_message: Some(e.to_string()),
                    };
                }
                Ok(_) => {} // Quota check passed
            }
        }

        // Ingest
        match self.ingestion.ingest(event).await {
            Ok(()) => {
                self.record_accepted().await;
                IngestEventResponse {
                    success: true,
                    status: IngestStatus::Accepted,
                    error_message: None,
                }
            }
            Err(e) => {
                self.record_internal_error().await;
                IngestEventResponse {
                    success: false,
                    status: IngestStatus::InternalError,
                    error_message: Some(e.to_string()),
                }
            }
        }
    }

    /// Ingest a batch of events.
    #[instrument(skip(self, request), fields(batch_size = request.events.len()))]
    pub async fn ingest_event_batch(
        &self,
        request: IngestEventBatchRequest,
    ) -> IngestEventBatchResponse {
        let mut accepted_count = 0u32;
        let mut duplicate_count = 0u32;
        let mut failed_count = 0u32;
        let mut results = Vec::new();

        // Enforce max batch size
        if request.events.len() > self.config.max_batch_size {
            return IngestEventBatchResponse {
                accepted_count: 0,
                duplicate_count: 0,
                failed_count: request.events.len() as u32,
                results: vec![EventResult {
                    index: 0,
                    status: IngestStatus::ValidationError,
                    error_message: Some(format!(
                        "Batch size {} exceeds maximum {}",
                        request.events.len(),
                        self.config.max_batch_size
                    )),
                }],
            };
        }

        // Convert and validate all events first
        let mut valid_events = Vec::with_capacity(request.events.len());
        let mut event_indices = Vec::with_capacity(request.events.len());

        for (idx, grpc_event) in request.events.iter().enumerate() {
            // Convert
            let event = match grpc_event.to_usage_event() {
                Ok(e) => e,
                Err(msg) => {
                    failed_count += 1;
                    if !request.continue_on_error {
                        results.push(EventResult {
                            index: idx as u32,
                            status: IngestStatus::ValidationError,
                            error_message: Some(msg),
                        });
                        return IngestEventBatchResponse {
                            accepted_count,
                            duplicate_count,
                            failed_count,
                            results,
                        };
                    }
                    results.push(EventResult {
                        index: idx as u32,
                        status: IngestStatus::ValidationError,
                        error_message: Some(msg),
                    });
                    continue;
                }
            };

            // Validate
            if let Err(e) = self.validator.validate(&event) {
                failed_count += 1;
                if !request.continue_on_error {
                    results.push(EventResult {
                        index: idx as u32,
                        status: IngestStatus::ValidationError,
                        error_message: Some(e.to_string()),
                    });
                    return IngestEventBatchResponse {
                        accepted_count,
                        duplicate_count,
                        failed_count,
                        results,
                    };
                }
                results.push(EventResult {
                    index: idx as u32,
                    status: IngestStatus::ValidationError,
                    error_message: Some(e.to_string()),
                });
                continue;
            }

            valid_events.push(event);
            event_indices.push(idx);
        }

        // Batch deduplication check
        let txn_ids: Vec<&str> = valid_events.iter().map(|e| e.transaction_id.as_str()).collect();
        let dedup_results = match self.deduplicator.check_and_mark_batch(&txn_ids).await {
            Ok(r) => r,
            Err(e) => {
                error!("Batch dedup check failed: {}", e);
                // Treat all as new on error
                vec![DedupResult::New; valid_events.len()]
            }
        };

        // Filter out duplicates
        let mut events_to_ingest = Vec::new();
        for (event, dedup_result) in valid_events.into_iter().zip(dedup_results.iter()) {
            if dedup_result.is_duplicate() {
                duplicate_count += 1;
            } else {
                events_to_ingest.push(event);
            }
        }

        // Batch ingest remaining events
        if !events_to_ingest.is_empty() {
            match self.ingestion.ingest_batch(events_to_ingest).await {
                Ok(count) => {
                    accepted_count = count as u32;
                }
                Err(e) => {
                    error!("Batch ingestion failed: {}", e);
                    // All remaining events failed
                    failed_count += accepted_count;
                    accepted_count = 0;
                }
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_accepted += accepted_count as u64;
            metrics.total_duplicates += duplicate_count as u64;
            metrics.total_failed += failed_count as u64;
        }

        IngestEventBatchResponse {
            accepted_count,
            duplicate_count,
            failed_count,
            results,
        }
    }

    /// Check if a quota allows usage.
    #[instrument(skip(self))]
    pub async fn check_quota(&self, request: CheckQuotaRequest) -> CheckQuotaResponse {
        let org_id = match uuid::Uuid::parse_str(&request.organization_id) {
            Ok(id) => creto_common::OrganizationId::from_uuid(id),
            Err(_) => {
                return CheckQuotaResponse {
                    allowed: false,
                    current_usage: 0,
                    limit: 0,
                    remaining: 0,
                    denial_reason: Some("Invalid organization_id".to_string()),
                };
            }
        };

        let agent_id = match &request.agent_id {
            Some(id) => match uuid::Uuid::parse_str(id) {
                Ok(uuid) => creto_common::AgentId::from_uuid(uuid),
                Err(_) => {
                    return CheckQuotaResponse {
                        allowed: false,
                        current_usage: 0,
                        limit: 0,
                        remaining: 0,
                        denial_reason: Some("Invalid agent_id".to_string()),
                    };
                }
            },
            None => creto_common::AgentId::new(),
        };

        let result = self
            .quota_enforcer
            .check(&org_id, &agent_id, &request.metric_code, request.quantity);

        match result {
            Ok(check) => {
                CheckQuotaResponse {
                    allowed: check.allowed,
                    current_usage: check.current_usage,
                    limit: check.limit,
                    remaining: check.remaining,
                    denial_reason: if check.allowed { None } else { Some("Quota exceeded".to_string()) },
                }
            }
            Err(e) => CheckQuotaResponse {
                allowed: false,
                current_usage: 0,
                limit: 0,
                remaining: 0,
                denial_reason: Some(e.to_string()),
            },
        }
    }

    /// Get current quota status.
    #[instrument(skip(self))]
    pub async fn get_quota_status(
        &self,
        request: GetQuotaStatusRequest,
    ) -> Option<GetQuotaStatusResponse> {
        let org_id = uuid::Uuid::parse_str(&request.organization_id).ok()?;
        let agent_id = request
            .agent_id
            .as_ref()
            .and_then(|id| uuid::Uuid::parse_str(id).ok())
            .map(creto_common::AgentId::from_uuid)
            .unwrap_or_default();

        let org = creto_common::OrganizationId::from_uuid(org_id);
        let status = self
            .quota_enforcer
            .check(&org, &agent_id, &request.metric_code, 0)
            .ok()?;

        Some(GetQuotaStatusResponse {
            metric_code: request.metric_code.clone(),
            limit: status.limit,
            current_usage: status.current_usage,
            remaining: status.remaining,
            usage_percentage: status.usage_percentage * 100.0, // Convert to percentage
            period: GrpcQuotaPeriod::Daily, // TODO: Map from actual period
            period_start: status.resets_at - chrono::Duration::days(1), // Approximate start
            period_end: status.resets_at,
        })
    }

    /// Get service metrics.
    pub async fn get_metrics(&self) -> ServiceMetrics {
        self.metrics.read().await.clone()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private Methods
    // ─────────────────────────────────────────────────────────────────────────

    async fn record_accepted(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_accepted += 1;
    }

    async fn record_duplicate(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_duplicates += 1;
    }

    async fn record_validation_error(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_validation_errors += 1;
        metrics.total_failed += 1;
    }

    async fn record_quota_exceeded(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_quota_exceeded += 1;
        metrics.total_failed += 1;
    }

    async fn record_internal_error(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_internal_errors += 1;
        metrics.total_failed += 1;
    }
}

/// Metrics for the metering service.
#[derive(Debug, Clone, Default)]
pub struct ServiceMetrics {
    pub total_accepted: u64,
    pub total_duplicates: u64,
    pub total_failed: u64,
    pub total_validation_errors: u64,
    pub total_quota_exceeded: u64,
    pub total_internal_errors: u64,
}

impl ServiceMetrics {
    /// Get total events processed (success + failure + duplicates).
    pub fn total_processed(&self) -> u64 {
        self.total_accepted + self.total_duplicates + self.total_failed
    }

    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        let total = self.total_processed();
        if total == 0 {
            0.0
        } else {
            (self.total_accepted as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dedup::DedupConfig;
    use crate::UsageEvent;
    use creto_common::CretoError;

    /// Mock ingestion for testing.
    struct MockIngestion;

    impl EventIngestion for MockIngestion {
        async fn ingest(&self, _event: UsageEvent) -> Result<(), CretoError> {
            Ok(())
        }

        async fn ingest_batch(&self, events: Vec<UsageEvent>) -> Result<usize, CretoError> {
            Ok(events.len())
        }
    }

    fn create_test_service() -> MeteringGrpcService<MockIngestion> {
        let ingestion = Arc::new(MockIngestion);
        let deduplicator = Arc::new(Deduplicator::local_only(DedupConfig::default()));
        let quota_enforcer = Arc::new(QuotaEnforcer::new());

        MeteringGrpcService::new(
            ingestion,
            deduplicator,
            quota_enforcer,
            MeteringServiceConfig {
                enforce_quotas: false, // Disable for tests
                ..Default::default()
            },
        )
    }

    fn test_grpc_event() -> GrpcUsageEvent {
        GrpcUsageEvent {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            organization_id: uuid::Uuid::new_v4().to_string(),
            agent_id: uuid::Uuid::new_v4().to_string(),
            external_subscription_id: None,
            event_type: GrpcUsageEventType::ApiCall,
            code: "api_calls".to_string(),
            quantity: 1,
            timestamp: None,
            properties: None,
            delegation_depth: 0,
        }
    }

    #[tokio::test]
    async fn test_ingest_single_event() {
        let service = create_test_service();

        let response = service
            .ingest_event(IngestEventRequest {
                event: test_grpc_event(),
            })
            .await;

        assert!(response.success);
        assert_eq!(response.status, IngestStatus::Accepted);
    }

    #[tokio::test]
    async fn test_ingest_duplicate_event() {
        let service = create_test_service();
        let event = test_grpc_event();

        // First ingestion
        let r1 = service
            .ingest_event(IngestEventRequest {
                event: event.clone(),
            })
            .await;
        assert_eq!(r1.status, IngestStatus::Accepted);

        // Second ingestion - should be duplicate
        let r2 = service
            .ingest_event(IngestEventRequest { event })
            .await;
        assert_eq!(r2.status, IngestStatus::Duplicate);
    }

    #[tokio::test]
    async fn test_ingest_invalid_event() {
        let service = create_test_service();

        let mut event = test_grpc_event();
        event.quantity = -1; // Invalid

        let response = service.ingest_event(IngestEventRequest { event }).await;

        assert!(!response.success);
        assert_eq!(response.status, IngestStatus::ValidationError);
    }

    #[tokio::test]
    async fn test_batch_ingestion() {
        let service = create_test_service();

        let events = vec![test_grpc_event(), test_grpc_event(), test_grpc_event()];

        let response = service
            .ingest_event_batch(IngestEventBatchRequest {
                events,
                continue_on_error: true,
            })
            .await;

        assert_eq!(response.accepted_count, 3);
        assert_eq!(response.duplicate_count, 0);
        assert_eq!(response.failed_count, 0);
    }

    #[tokio::test]
    async fn test_metrics() {
        let service = create_test_service();

        // Ingest some events
        service
            .ingest_event(IngestEventRequest {
                event: test_grpc_event(),
            })
            .await;
        service
            .ingest_event(IngestEventRequest {
                event: test_grpc_event(),
            })
            .await;

        let metrics = service.get_metrics().await;
        assert_eq!(metrics.total_accepted, 2);
        assert_eq!(metrics.total_processed(), 2);
    }

    #[test]
    fn test_service_metrics_calculations() {
        let metrics = ServiceMetrics {
            total_accepted: 90,
            total_duplicates: 5,
            total_failed: 5,
            ..Default::default()
        };

        assert_eq!(metrics.total_processed(), 100);
        assert_eq!(metrics.success_rate(), 90.0);
    }
}
