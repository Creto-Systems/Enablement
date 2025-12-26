//! Core quota types and definitions.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use creto_common::{AgentId, OrganizationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A quota definition that limits usage of a specific metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    /// Unique quota ID.
    pub id: Uuid,
    /// Organization this quota belongs to.
    pub organization_id: OrganizationId,
    /// Optional: Specific agent this quota applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<AgentId>,
    /// Billable metric code this quota limits.
    pub metric_code: String,
    /// Maximum allowed usage per period.
    pub limit: i64,
    /// Reset period for the quota.
    pub period: QuotaPeriod,
    /// Current usage in this period.
    #[serde(default)]
    pub current_usage: i64,
    /// When the current period started.
    pub period_start: DateTime<Utc>,
    /// When the current period ends.
    pub period_end: DateTime<Utc>,
    /// Whether to allow overage (soft limit) or block (hard limit).
    #[serde(default)]
    pub allow_overage: bool,
    /// Optional: Budget in cents for cost-based quotas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_cents: Option<i64>,
}

impl Quota {
    /// Create a new quota with default values.
    pub fn new(
        organization_id: OrganizationId,
        metric_code: impl Into<String>,
        limit: i64,
        period: QuotaPeriod,
    ) -> Self {
        let now = Utc::now();
        let (period_start, period_end) = period.calculate_bounds(now);

        Self {
            id: Uuid::now_v7(),
            organization_id,
            agent_id: None,
            metric_code: metric_code.into(),
            limit,
            period,
            current_usage: 0,
            period_start,
            period_end,
            allow_overage: false,
            budget_cents: None,
        }
    }

    /// Check if this quota would be exceeded by adding `amount`.
    pub fn would_exceed(&self, amount: i64) -> bool {
        self.current_usage + amount > self.limit
    }

    /// Get remaining quota.
    pub fn remaining(&self) -> i64 {
        (self.limit - self.current_usage).max(0)
    }

    /// Get usage as a percentage (0.0 to 1.0+).
    pub fn usage_percentage(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        self.current_usage as f64 / self.limit as f64
    }

    /// Check if the current period has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.period_end
    }

    /// Reset the quota for a new period.
    pub fn reset(&mut self) {
        let now = Utc::now();
        let (period_start, period_end) = self.period.calculate_bounds(now);

        self.current_usage = 0;
        self.period_start = period_start;
        self.period_end = period_end;
    }
}

/// Time period for quota reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum QuotaPeriod {
    /// Reset every hour.
    Hourly,
    /// Reset every day at midnight UTC.
    #[default]
    Daily,
    /// Reset every week on Monday.
    Weekly,
    /// Reset on the 1st of each month.
    Monthly,
    /// Never reset (lifetime quota).
    Lifetime,
}


impl QuotaPeriod {
    /// Calculate the start and end bounds for a period containing the given timestamp.
    pub fn calculate_bounds(&self, timestamp: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        match self {
            QuotaPeriod::Hourly => {
                let start = Utc
                    .with_ymd_and_hms(
                        timestamp.year(),
                        timestamp.month(),
                        timestamp.day(),
                        timestamp.hour(),
                        0,
                        0,
                    )
                    .unwrap();
                let end = start + Duration::hours(1);
                (start, end)
            }
            QuotaPeriod::Daily => {
                let start = Utc
                    .with_ymd_and_hms(
                        timestamp.year(),
                        timestamp.month(),
                        timestamp.day(),
                        0,
                        0,
                        0,
                    )
                    .unwrap();
                let end = start + Duration::days(1);
                (start, end)
            }
            QuotaPeriod::Weekly => {
                let days_since_monday = timestamp.weekday().num_days_from_monday();
                let start = Utc
                    .with_ymd_and_hms(
                        timestamp.year(),
                        timestamp.month(),
                        timestamp.day(),
                        0,
                        0,
                        0,
                    )
                    .unwrap()
                    - Duration::days(days_since_monday as i64);
                let end = start + Duration::weeks(1);
                (start, end)
            }
            QuotaPeriod::Monthly => {
                let start = Utc
                    .with_ymd_and_hms(timestamp.year(), timestamp.month(), 1, 0, 0, 0)
                    .unwrap();
                let end = if timestamp.month() == 12 {
                    Utc.with_ymd_and_hms(timestamp.year() + 1, 1, 1, 0, 0, 0)
                        .unwrap()
                } else {
                    Utc.with_ymd_and_hms(timestamp.year(), timestamp.month() + 1, 1, 0, 0, 0)
                        .unwrap()
                };
                (start, end)
            }
            QuotaPeriod::Lifetime => {
                let start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
                let end = Utc.with_ymd_and_hms(2100, 1, 1, 0, 0, 0).unwrap();
                (start, end)
            }
        }
    }

    /// Get string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Lifetime => "lifetime",
        }
    }
}

/// Current status of a quota.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaStatus {
    /// Metric code being tracked.
    pub metric_code: String,
    /// Total limit for this period.
    pub limit: i64,
    /// Amount used so far.
    pub used: i64,
    /// Amount remaining.
    pub remaining: i64,
    /// Percentage used (0.0 to 1.0+).
    pub percentage_used: f64,
    /// Reset period.
    pub period: QuotaPeriod,
    /// When the quota resets.
    pub resets_at: DateTime<Utc>,
}

impl Default for QuotaStatus {
    fn default() -> Self {
        Self {
            metric_code: String::new(),
            limit: 0,
            used: 0,
            remaining: 0,
            percentage_used: 0.0,
            period: QuotaPeriod::default(),
            resets_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_would_exceed() {
        let mut quota = Quota::new(
            OrganizationId::new(),
            "api_calls",
            100,
            QuotaPeriod::Daily,
        );

        assert!(!quota.would_exceed(50));
        quota.current_usage = 90;
        assert!(!quota.would_exceed(10));
        assert!(quota.would_exceed(11));
    }

    #[test]
    fn test_quota_usage_percentage() {
        let mut quota = Quota::new(
            OrganizationId::new(),
            "tokens",
            1000,
            QuotaPeriod::Monthly,
        );

        assert_eq!(quota.usage_percentage(), 0.0);
        quota.current_usage = 500;
        assert_eq!(quota.usage_percentage(), 0.5);
        quota.current_usage = 1000;
        assert_eq!(quota.usage_percentage(), 1.0);
    }

    #[test]
    fn test_period_bounds_daily() {
        let timestamp = Utc::now();
        let (start, end) = QuotaPeriod::Daily.calculate_bounds(timestamp);

        assert!(start <= timestamp);
        assert!(timestamp < end);
        assert_eq!((end - start).num_hours(), 24);
    }

    #[test]
    fn test_period_as_str() {
        assert_eq!(QuotaPeriod::Hourly.as_str(), "hourly");
        assert_eq!(QuotaPeriod::Daily.as_str(), "daily");
        assert_eq!(QuotaPeriod::Weekly.as_str(), "weekly");
        assert_eq!(QuotaPeriod::Monthly.as_str(), "monthly");
        assert_eq!(QuotaPeriod::Lifetime.as_str(), "lifetime");
    }
}
