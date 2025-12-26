//! Event validation for usage metering.
//!
//! Provides comprehensive validation of usage events before ingestion,
//! ensuring data quality and preventing invalid events from entering the system.

use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

use crate::events::UsageEvent;

/// Validation error types for usage events.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Transaction ID is required and must be non-empty")]
    EmptyTransactionId,

    #[error("Transaction ID exceeds maximum length of {max} characters")]
    TransactionIdTooLong { max: usize },

    #[error("Quantity must be positive, got {0}")]
    NonPositiveQuantity(i64),

    #[error("Quantity exceeds maximum allowed value of {max}")]
    QuantityTooLarge { value: i64, max: i64 },

    #[error("Event timestamp {timestamp} is too far in the future (max {max_hours}h ahead)")]
    TimestampTooFuture { timestamp: DateTime<Utc>, max_hours: i64 },

    #[error("Event timestamp {timestamp} is too old (max {max_days}d in the past)")]
    TimestampTooOld { timestamp: DateTime<Utc>, max_days: i64 },

    #[error("Metric code is required and must be non-empty")]
    EmptyMetricCode,

    #[error("Metric code '{0}' contains invalid characters (alphanumeric and underscore only)")]
    InvalidMetricCode(String),

    #[error("Properties JSON exceeds maximum size of {max_bytes} bytes")]
    PropertiesTooLarge { size: usize, max_bytes: usize },

    #[error("Delegation depth {depth} exceeds maximum of {max}")]
    DelegationDepthTooDeep { depth: u8, max: u8 },

    #[error("External subscription ID exceeds maximum length of {max} characters")]
    ExternalSubscriptionIdTooLong { max: usize },

    #[error("Multiple validation errors: {0:?}")]
    Multiple(Vec<ValidationError>),
}

impl ValidationError {
    /// Check if this is a critical error that should reject the event.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            ValidationError::EmptyTransactionId
                | ValidationError::NonPositiveQuantity(_)
                | ValidationError::EmptyMetricCode
        )
    }
}

/// Configuration for event validation.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum length of transaction ID.
    pub max_transaction_id_length: usize,
    /// Maximum quantity value per event.
    pub max_quantity: i64,
    /// Maximum hours in the future for event timestamp.
    pub max_future_hours: i64,
    /// Maximum days in the past for event timestamp.
    pub max_past_days: i64,
    /// Maximum size of properties JSON in bytes.
    pub max_properties_bytes: usize,
    /// Maximum delegation depth.
    pub max_delegation_depth: u8,
    /// Maximum length of external subscription ID.
    pub max_external_subscription_id_length: usize,
    /// Whether to collect all errors or fail fast.
    pub collect_all_errors: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_transaction_id_length: 255,
            max_quantity: 1_000_000_000, // 1 billion
            max_future_hours: 1,          // 1 hour ahead
            max_past_days: 30,            // 30 days back
            max_properties_bytes: 65536,  // 64KB
            max_delegation_depth: 10,
            max_external_subscription_id_length: 255,
            collect_all_errors: false,
        }
    }
}

impl ValidationConfig {
    /// Create a strict configuration for production use.
    pub fn strict() -> Self {
        Self {
            max_transaction_id_length: 128,
            max_quantity: 100_000_000,
            max_future_hours: 0,  // No future events
            max_past_days: 7,     // Only 7 days back
            max_properties_bytes: 16384, // 16KB
            max_delegation_depth: 5,
            max_external_subscription_id_length: 128,
            collect_all_errors: true,
        }
    }

    /// Create a lenient configuration for testing.
    pub fn lenient() -> Self {
        Self {
            max_transaction_id_length: 512,
            max_quantity: i64::MAX,
            max_future_hours: 24,
            max_past_days: 365,
            max_properties_bytes: 1048576, // 1MB
            max_delegation_depth: 20,
            max_external_subscription_id_length: 512,
            collect_all_errors: true,
        }
    }
}

/// Validator for usage events.
pub struct EventValidator {
    config: ValidationConfig,
}

impl EventValidator {
    /// Create a new validator with the given configuration.
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Create a validator with default configuration.
    pub fn default_validator() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Validate a usage event.
    pub fn validate(&self, event: &UsageEvent) -> Result<(), ValidationError> {
        let mut errors = Vec::new();

        // Transaction ID validation
        if event.transaction_id.is_empty() {
            if !self.config.collect_all_errors {
                return Err(ValidationError::EmptyTransactionId);
            }
            errors.push(ValidationError::EmptyTransactionId);
        } else if event.transaction_id.len() > self.config.max_transaction_id_length {
            let err = ValidationError::TransactionIdTooLong {
                max: self.config.max_transaction_id_length,
            };
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        // Quantity validation
        if event.quantity <= 0 {
            let err = ValidationError::NonPositiveQuantity(event.quantity);
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        } else if event.quantity > self.config.max_quantity {
            let err = ValidationError::QuantityTooLarge {
                value: event.quantity,
                max: self.config.max_quantity,
            };
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        // Timestamp validation
        let now = Utc::now();
        let max_future = now + Duration::hours(self.config.max_future_hours);
        let max_past = now - Duration::days(self.config.max_past_days);

        if event.timestamp > max_future {
            let err = ValidationError::TimestampTooFuture {
                timestamp: event.timestamp,
                max_hours: self.config.max_future_hours,
            };
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        if event.timestamp < max_past {
            let err = ValidationError::TimestampTooOld {
                timestamp: event.timestamp,
                max_days: self.config.max_past_days,
            };
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        // Metric code validation
        if event.code.is_empty() {
            let err = ValidationError::EmptyMetricCode;
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        } else if !is_valid_metric_code(&event.code) {
            let err = ValidationError::InvalidMetricCode(event.code.clone());
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        // Properties size validation
        let properties_size = event.properties.to_string().len();
        if properties_size > self.config.max_properties_bytes {
            let err = ValidationError::PropertiesTooLarge {
                size: properties_size,
                max_bytes: self.config.max_properties_bytes,
            };
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        // Delegation depth validation
        if event.delegation_depth > self.config.max_delegation_depth {
            let err = ValidationError::DelegationDepthTooDeep {
                depth: event.delegation_depth,
                max: self.config.max_delegation_depth,
            };
            if !self.config.collect_all_errors {
                return Err(err);
            }
            errors.push(err);
        }

        // External subscription ID validation
        if let Some(ref ext_id) = event.external_subscription_id {
            if ext_id.len() > self.config.max_external_subscription_id_length {
                let err = ValidationError::ExternalSubscriptionIdTooLong {
                    max: self.config.max_external_subscription_id_length,
                };
                if !self.config.collect_all_errors {
                    return Err(err);
                }
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.remove(0))
        } else {
            Err(ValidationError::Multiple(errors))
        }
    }

    /// Validate a batch of events, returning valid events and errors.
    pub fn validate_batch(&self, events: &[UsageEvent]) -> BatchValidationResult {
        let mut valid = Vec::with_capacity(events.len());
        let mut invalid = Vec::new();

        for (idx, event) in events.iter().enumerate() {
            match self.validate(event) {
                Ok(()) => valid.push(event.clone()),
                Err(e) => invalid.push((idx, e)),
            }
        }

        BatchValidationResult { valid, invalid }
    }
}

/// Result of batch validation.
#[derive(Debug)]
pub struct BatchValidationResult {
    /// Events that passed validation.
    pub valid: Vec<UsageEvent>,
    /// Index and error for events that failed validation.
    pub invalid: Vec<(usize, ValidationError)>,
}

impl BatchValidationResult {
    /// Check if all events passed validation.
    pub fn all_valid(&self) -> bool {
        self.invalid.is_empty()
    }

    /// Get the count of valid events.
    pub fn valid_count(&self) -> usize {
        self.valid.len()
    }

    /// Get the count of invalid events.
    pub fn invalid_count(&self) -> usize {
        self.invalid.len()
    }
}

/// Check if a metric code is valid (alphanumeric and underscores only).
fn is_valid_metric_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 64
        && code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{UsageEvent, UsageEventType};

    fn valid_event() -> UsageEvent {
        UsageEvent::builder()
            .event_type(UsageEventType::ApiCall)
            .quantity(1)
            .build()
    }

    #[test]
    fn test_valid_event_passes() {
        let validator = EventValidator::default_validator();
        let event = valid_event();
        assert!(validator.validate(&event).is_ok());
    }

    #[test]
    fn test_empty_transaction_id_fails() {
        let validator = EventValidator::default_validator();
        let mut event = valid_event();
        event.transaction_id = String::new();

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::EmptyTransactionId)));
    }

    #[test]
    fn test_non_positive_quantity_fails() {
        let validator = EventValidator::default_validator();
        let mut event = valid_event();
        event.quantity = 0;

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::NonPositiveQuantity(0))));
    }

    #[test]
    fn test_negative_quantity_fails() {
        let validator = EventValidator::default_validator();
        let mut event = valid_event();
        event.quantity = -5;

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::NonPositiveQuantity(-5))));
    }

    #[test]
    fn test_quantity_too_large_fails() {
        let validator = EventValidator::new(ValidationConfig {
            max_quantity: 100,
            ..Default::default()
        });
        let mut event = valid_event();
        event.quantity = 101;

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::QuantityTooLarge { .. })));
    }

    #[test]
    fn test_future_timestamp_fails() {
        let validator = EventValidator::new(ValidationConfig {
            max_future_hours: 0,
            ..Default::default()
        });
        let mut event = valid_event();
        event.timestamp = Utc::now() + Duration::hours(2);

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::TimestampTooFuture { .. })));
    }

    #[test]
    fn test_old_timestamp_fails() {
        let validator = EventValidator::new(ValidationConfig {
            max_past_days: 7,
            ..Default::default()
        });
        let mut event = valid_event();
        event.timestamp = Utc::now() - Duration::days(10);

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::TimestampTooOld { .. })));
    }

    #[test]
    fn test_invalid_metric_code_fails() {
        let validator = EventValidator::default_validator();
        let mut event = valid_event();
        event.code = "invalid-code!".to_string();

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::InvalidMetricCode(_))));
    }

    #[test]
    fn test_valid_metric_codes() {
        assert!(is_valid_metric_code("api_calls"));
        assert!(is_valid_metric_code("tokens_used"));
        assert!(is_valid_metric_code("CPU_MS"));
        assert!(is_valid_metric_code("event123"));
    }

    #[test]
    fn test_invalid_metric_codes() {
        assert!(!is_valid_metric_code(""));
        assert!(!is_valid_metric_code("api-calls"));
        assert!(!is_valid_metric_code("api.calls"));
        assert!(!is_valid_metric_code("api calls"));
    }

    #[test]
    fn test_batch_validation() {
        let validator = EventValidator::default_validator();

        let event1 = valid_event();
        let mut event2 = valid_event();
        event2.quantity = -1; // Invalid
        let event3 = valid_event();

        let result = validator.validate_batch(&[event1, event2, event3]);

        assert_eq!(result.valid_count(), 2);
        assert_eq!(result.invalid_count(), 1);
        assert_eq!(result.invalid[0].0, 1); // Index of invalid event
    }

    #[test]
    fn test_collect_all_errors() {
        let validator = EventValidator::new(ValidationConfig {
            collect_all_errors: true,
            max_quantity: 100,
            ..Default::default()
        });

        let mut event = valid_event();
        event.transaction_id = String::new();
        event.quantity = -1;
        event.code = String::new();

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::Multiple(_))));

        if let Err(ValidationError::Multiple(errors)) = result {
            assert_eq!(errors.len(), 3);
        }
    }

    #[test]
    fn test_delegation_depth_limit() {
        let validator = EventValidator::new(ValidationConfig {
            max_delegation_depth: 5,
            ..Default::default()
        });

        let mut event = valid_event();
        event.delegation_depth = 10;

        let result = validator.validate(&event);
        assert!(matches!(result, Err(ValidationError::DelegationDepthTooDeep { .. })));
    }
}
