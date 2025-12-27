//! Usage aggregation engine.
//!
//! Aggregates raw usage events into summarized metrics for billing and reporting.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, OrganizationId};
use serde::{Deserialize, Serialize};

use crate::events::UsageEventType;

/// Aggregation function to apply to usage events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    /// Count number of events.
    Count,
    /// Sum of quantities.
    Sum,
    /// Maximum quantity.
    Max,
    /// Minimum quantity.
    Min,
    /// Average quantity.
    Average,
    /// Count of unique values (for a property field).
    UniqueCount,
    /// Latest value.
    Latest,
}

/// Result of an aggregation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    /// Organization ID.
    pub organization_id: OrganizationId,

    /// Optional agent ID (if aggregating for specific agent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<AgentId>,

    /// Metric code that was aggregated.
    pub metric_code: String,

    /// Event type that was aggregated.
    pub event_type: UsageEventType,

    /// Aggregation function used.
    pub aggregation_type: AggregationType,

    /// Resulting value.
    pub value: AggregationValue,

    /// Start of the aggregation period.
    pub period_start: DateTime<Utc>,

    /// End of the aggregation period.
    pub period_end: DateTime<Utc>,

    /// Number of events included in this aggregation.
    pub event_count: u64,
}

/// Value types that can result from aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AggregationValue {
    /// Integer value (count, sum, etc.).
    Integer(i64),
    /// Floating point value (average).
    Float(f64),
    /// String value (latest).
    String(String),
}

impl AggregationValue {
    /// Get as integer, returning 0 if not an integer.
    pub fn as_i64(&self) -> i64 {
        match self {
            AggregationValue::Integer(v) => *v,
            AggregationValue::Float(v) => *v as i64,
            AggregationValue::String(_) => 0,
        }
    }

    /// Get as float.
    pub fn as_f64(&self) -> f64 {
        match self {
            AggregationValue::Integer(v) => *v as f64,
            AggregationValue::Float(v) => *v,
            AggregationValue::String(_) => 0.0,
        }
    }
}

/// Engine for computing usage aggregations.
pub struct AggregationEngine {
    // TODO: Add database connection, cache
    _private: (),
}

impl AggregationEngine {
    /// Create a new aggregation engine.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Compute an aggregation for a given period.
    ///
    /// # Performance
    ///
    /// Uses pre-computed materialized views for common aggregations.
    /// Real-time aggregations for custom periods may be slower.
    pub async fn aggregate(
        &self,
        organization_id: OrganizationId,
        agent_id: Option<AgentId>,
        metric_code: &str,
        aggregation_type: AggregationType,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Aggregation, creto_common::CretoError> {
        // TODO: Implement actual aggregation
        // 1. Check materialized view cache
        // 2. If not available, query usage_events table
        // 3. Apply aggregation function

        let _ = (
            organization_id,
            agent_id,
            metric_code,
            aggregation_type,
            period_start,
            period_end,
        );

        todo!("Aggregation not yet implemented")
    }

    /// Get pre-computed aggregations for a billing period.
    ///
    /// These are computed nightly and stored for fast retrieval.
    pub async fn get_billing_aggregations(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Vec<Aggregation>, creto_common::CretoError> {
        // TODO: Implement billing aggregation retrieval

        let _ = (organization_id, period_start, period_end);

        todo!("Billing aggregations not yet implemented")
    }
}

impl Default for AggregationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Specification for a billable metric that can be aggregated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillableMetric {
    /// Unique metric code.
    pub code: String,

    /// Human-readable name.
    pub name: String,

    /// Description of what this metric measures.
    pub description: String,

    /// Event type this metric is derived from.
    pub event_type: UsageEventType,

    /// How to aggregate events into this metric.
    pub aggregation_type: AggregationType,

    /// Optional property field for UNIQUE_COUNT aggregations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_count_field: Option<String>,
}

impl BillableMetric {
    /// Create a standard API calls metric.
    pub fn api_calls() -> Self {
        Self {
            code: "api_calls".to_string(),
            name: "API Calls".to_string(),
            description: "Number of API calls made".to_string(),
            event_type: UsageEventType::ApiCall,
            aggregation_type: AggregationType::Count,
            unique_count_field: None,
        }
    }

    /// Create a standard input tokens metric.
    pub fn input_tokens() -> Self {
        Self {
            code: "input_tokens".to_string(),
            name: "Input Tokens".to_string(),
            description: "Number of input tokens processed".to_string(),
            event_type: UsageEventType::InputTokens,
            aggregation_type: AggregationType::Sum,
            unique_count_field: None,
        }
    }

    /// Create a standard output tokens metric.
    pub fn output_tokens() -> Self {
        Self {
            code: "output_tokens".to_string(),
            name: "Output Tokens".to_string(),
            description: "Number of output tokens generated".to_string(),
            event_type: UsageEventType::OutputTokens,
            aggregation_type: AggregationType::Sum,
            unique_count_field: None,
        }
    }

    /// Create a unique agents metric.
    pub fn unique_agents() -> Self {
        Self {
            code: "unique_agents".to_string(),
            name: "Unique Agents".to_string(),
            description: "Number of unique agents active".to_string(),
            event_type: UsageEventType::ApiCall,
            aggregation_type: AggregationType::UniqueCount,
            unique_count_field: Some("agent_id".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_value_conversions() {
        let int_val = AggregationValue::Integer(42);
        assert_eq!(int_val.as_i64(), 42);
        assert_eq!(int_val.as_f64(), 42.0);

        let float_val = AggregationValue::Float(3.14);
        assert_eq!(float_val.as_i64(), 3);
        assert_eq!(float_val.as_f64(), 3.14);
    }

    #[test]
    fn test_billable_metric_presets() {
        let api_calls = BillableMetric::api_calls();
        assert_eq!(api_calls.code, "api_calls");
        assert_eq!(api_calls.aggregation_type, AggregationType::Count);

        let input_tokens = BillableMetric::input_tokens();
        assert_eq!(input_tokens.aggregation_type, AggregationType::Sum);
    }
}
