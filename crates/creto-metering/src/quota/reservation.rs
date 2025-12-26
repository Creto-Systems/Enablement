//! Quota reservation system for pre-allocating quota.
//!
//! Enables agents to reserve quota before performing operations,
//! preventing overbooking under concurrent access.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

/// Reservation status state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReservationStatus {
    /// Reservation is active and holding quota.
    Active,
    /// Reservation was committed with actual usage.
    Committed,
    /// Reservation was explicitly released.
    Released,
    /// Reservation expired due to TTL.
    Expired,
}

impl ReservationStatus {
    /// Check if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Committed | Self::Released | Self::Expired)
    }
}

/// A quota reservation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    /// Unique reservation ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Agent ID.
    pub agent_id: String,
    /// Metric code being reserved.
    pub metric_code: String,
    /// Amount of quota reserved.
    pub reserved_amount: i64,
    /// Actual amount used (set on commit).
    pub actual_amount: Option<i64>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Expiry timestamp.
    pub expires_at: DateTime<Utc>,
    /// Current status.
    pub status: ReservationStatus,
    /// Optional metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl Reservation {
    /// Check if the reservation has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get time remaining until expiry.
    pub fn time_remaining(&self) -> Duration {
        let remaining = self.expires_at - Utc::now();
        if remaining < Duration::zero() {
            Duration::zero()
        } else {
            remaining
        }
    }

    /// Check if reservation can be committed.
    pub fn can_commit(&self) -> bool {
        self.status == ReservationStatus::Active && !self.is_expired()
    }

    /// Check if reservation can be released.
    pub fn can_release(&self) -> bool {
        self.status == ReservationStatus::Active
    }
}

/// Request to create a reservation.
#[derive(Debug, Clone)]
pub struct ReserveRequest {
    pub organization_id: Uuid,
    pub agent_id: String,
    pub metric_code: String,
    pub amount: i64,
    pub ttl_seconds: u64,
    pub metadata: Option<serde_json::Value>,
}

impl ReserveRequest {
    /// Create a new reserve request with default TTL (5 minutes).
    pub fn new(
        organization_id: Uuid,
        agent_id: impl Into<String>,
        metric_code: impl Into<String>,
        amount: i64,
    ) -> Self {
        Self {
            organization_id,
            agent_id: agent_id.into(),
            metric_code: metric_code.into(),
            amount,
            ttl_seconds: 300, // 5 minutes default
            metadata: None,
        }
    }

    /// Set custom TTL.
    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    /// Set metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Reservation error types.
#[derive(Debug, Error)]
pub enum ReservationError {
    #[error("Insufficient quota: requested {requested}, available {available}")]
    InsufficientQuota { requested: i64, available: i64 },

    #[error("Reservation not found: {0}")]
    NotFound(Uuid),

    #[error("Reservation already {0:?}")]
    InvalidStatus(ReservationStatus),

    #[error("Actual amount {actual} exceeds reserved {reserved}")]
    ExceedsReserved { actual: i64, reserved: i64 },

    #[error("Reservation expired at {0}")]
    Expired(DateTime<Utc>),

    #[error("Lock error: {0}")]
    LockError(String),
}

/// In-memory reservation store for local testing.
/// Production uses PostgreSQL + Redis.
pub struct ReservationStore {
    reservations: RwLock<HashMap<Uuid, Reservation>>,
    /// Total reserved per org+metric (for fast lookup).
    reserved_totals: RwLock<HashMap<String, i64>>,
}

impl ReservationStore {
    /// Create a new in-memory store.
    pub fn new() -> Self {
        Self {
            reservations: RwLock::new(HashMap::new()),
            reserved_totals: RwLock::new(HashMap::new()),
        }
    }

    /// Reserve quota.
    pub fn reserve(
        &self,
        request: ReserveRequest,
        available_quota: i64,
    ) -> Result<Reservation, ReservationError> {
        let total_key = format!("{}:{}", request.organization_id, request.metric_code);

        // Check available quota including existing reservations
        let current_reserved = {
            let totals = self.reserved_totals.read()
                .map_err(|e| ReservationError::LockError(e.to_string()))?;
            *totals.get(&total_key).unwrap_or(&0)
        };

        let truly_available = available_quota - current_reserved;
        if truly_available < request.amount {
            return Err(ReservationError::InsufficientQuota {
                requested: request.amount,
                available: truly_available,
            });
        }

        // Create reservation
        let now = Utc::now();
        let reservation = Reservation {
            id: Uuid::now_v7(),
            organization_id: request.organization_id,
            agent_id: request.agent_id,
            metric_code: request.metric_code,
            reserved_amount: request.amount,
            actual_amount: None,
            created_at: now,
            expires_at: now + Duration::seconds(request.ttl_seconds as i64),
            status: ReservationStatus::Active,
            metadata: request.metadata,
        };

        // Store reservation
        {
            let mut reservations = self.reservations.write()
                .map_err(|e| ReservationError::LockError(e.to_string()))?;
            reservations.insert(reservation.id, reservation.clone());
        }

        // Update reserved total
        {
            let mut totals = self.reserved_totals.write()
                .map_err(|e| ReservationError::LockError(e.to_string()))?;
            *totals.entry(total_key).or_insert(0) += request.amount;
        }

        Ok(reservation)
    }

    /// Commit a reservation with actual usage.
    pub fn commit(
        &self,
        reservation_id: Uuid,
        actual_amount: i64,
    ) -> Result<Reservation, ReservationError> {
        let mut reservations = self.reservations.write()
            .map_err(|e| ReservationError::LockError(e.to_string()))?;

        let reservation = reservations.get_mut(&reservation_id)
            .ok_or(ReservationError::NotFound(reservation_id))?;

        if reservation.status != ReservationStatus::Active {
            return Err(ReservationError::InvalidStatus(reservation.status));
        }

        if reservation.is_expired() {
            reservation.status = ReservationStatus::Expired;
            return Err(ReservationError::Expired(reservation.expires_at));
        }

        if actual_amount > reservation.reserved_amount {
            return Err(ReservationError::ExceedsReserved {
                actual: actual_amount,
                reserved: reservation.reserved_amount,
            });
        }

        reservation.actual_amount = Some(actual_amount);
        reservation.status = ReservationStatus::Committed;

        // Update reserved total (release the reservation)
        let total_key = format!("{}:{}", reservation.organization_id, reservation.metric_code);
        {
            let mut totals = self.reserved_totals.write()
                .map_err(|e| ReservationError::LockError(e.to_string()))?;
            if let Some(total) = totals.get_mut(&total_key) {
                *total = (*total - reservation.reserved_amount).max(0);
            }
        }

        Ok(reservation.clone())
    }

    /// Release a reservation without using quota.
    pub fn release(&self, reservation_id: Uuid) -> Result<Reservation, ReservationError> {
        let mut reservations = self.reservations.write()
            .map_err(|e| ReservationError::LockError(e.to_string()))?;

        let reservation = reservations.get_mut(&reservation_id)
            .ok_or(ReservationError::NotFound(reservation_id))?;

        if reservation.status != ReservationStatus::Active {
            return Err(ReservationError::InvalidStatus(reservation.status));
        }

        reservation.status = ReservationStatus::Released;

        // Update reserved total
        let total_key = format!("{}:{}", reservation.organization_id, reservation.metric_code);
        {
            let mut totals = self.reserved_totals.write()
                .map_err(|e| ReservationError::LockError(e.to_string()))?;
            if let Some(total) = totals.get_mut(&total_key) {
                *total = (*total - reservation.reserved_amount).max(0);
            }
        }

        Ok(reservation.clone())
    }

    /// Get a reservation by ID.
    pub fn get(&self, reservation_id: Uuid) -> Option<Reservation> {
        let reservations = self.reservations.read().ok()?;
        reservations.get(&reservation_id).cloned()
    }

    /// Get total reserved for org+metric.
    pub fn get_total_reserved(&self, organization_id: Uuid, metric_code: &str) -> i64 {
        let total_key = format!("{}:{}", organization_id, metric_code);
        let totals = self.reserved_totals.read().ok();
        totals.map(|t| *t.get(&total_key).unwrap_or(&0)).unwrap_or(0)
    }

    /// Expire stale reservations (background task).
    pub fn expire_stale(&self) -> Vec<Uuid> {
        let mut expired_ids = Vec::new();
        let now = Utc::now();

        // Find expired reservations
        if let Ok(mut reservations) = self.reservations.write() {
            for (id, reservation) in reservations.iter_mut() {
                if reservation.status == ReservationStatus::Active && reservation.expires_at < now {
                    reservation.status = ReservationStatus::Expired;
                    expired_ids.push(*id);
                }
            }
        }

        // Update reserved totals for expired reservations
        if let Ok(reservations) = self.reservations.read() {
            if let Ok(mut totals) = self.reserved_totals.write() {
                for id in &expired_ids {
                    if let Some(reservation) = reservations.get(id) {
                        let total_key = format!("{}:{}", reservation.organization_id, reservation.metric_code);
                        if let Some(total) = totals.get_mut(&total_key) {
                            *total = (*total - reservation.reserved_amount).max(0);
                        }
                    }
                }
            }
        }

        expired_ids
    }

    /// Get count of active reservations.
    pub fn active_count(&self) -> usize {
        self.reservations.read()
            .map(|r| r.values().filter(|r| r.status == ReservationStatus::Active).count())
            .unwrap_or(0)
    }
}

impl Default for ReservationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserve_quota() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let request = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let reservation = store.reserve(request, 1000).unwrap();

        assert_eq!(reservation.status, ReservationStatus::Active);
        assert_eq!(reservation.reserved_amount, 100);
        assert!(reservation.actual_amount.is_none());
    }

    #[test]
    fn test_insufficient_quota() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let request = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let result = store.reserve(request, 50); // Only 50 available

        assert!(matches!(result, Err(ReservationError::InsufficientQuota { .. })));
    }

    #[test]
    fn test_commit_reservation() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let request = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let reservation = store.reserve(request, 1000).unwrap();

        let committed = store.commit(reservation.id, 75).unwrap();

        assert_eq!(committed.status, ReservationStatus::Committed);
        assert_eq!(committed.actual_amount, Some(75));
    }

    #[test]
    fn test_commit_exceeds_reserved() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let request = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let reservation = store.reserve(request, 1000).unwrap();

        let result = store.commit(reservation.id, 150); // More than reserved

        assert!(matches!(result, Err(ReservationError::ExceedsReserved { .. })));
    }

    #[test]
    fn test_release_reservation() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let request = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let reservation = store.reserve(request, 1000).unwrap();

        let released = store.release(reservation.id).unwrap();

        assert_eq!(released.status, ReservationStatus::Released);
    }

    #[test]
    fn test_concurrent_reservations() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        // Available: 1000, Reserve 600
        let req1 = ReserveRequest::new(org_id, "agent1", "api_calls", 600);
        store.reserve(req1, 1000).unwrap();

        // Now only 400 available, try to reserve 600 more
        let req2 = ReserveRequest::new(org_id, "agent2", "api_calls", 600);
        let result = store.reserve(req2, 1000);

        assert!(matches!(result, Err(ReservationError::InsufficientQuota { available: 400, .. })));
    }

    #[test]
    fn test_get_total_reserved() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let req1 = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let req2 = ReserveRequest::new(org_id, "agent2", "api_calls", 200);

        store.reserve(req1, 1000).unwrap();
        store.reserve(req2, 1000).unwrap();

        assert_eq!(store.get_total_reserved(org_id, "api_calls"), 300);
    }

    #[test]
    fn test_reservation_status_transitions() {
        assert!(!ReservationStatus::Active.is_terminal());
        assert!(ReservationStatus::Committed.is_terminal());
        assert!(ReservationStatus::Released.is_terminal());
        assert!(ReservationStatus::Expired.is_terminal());
    }

    #[test]
    fn test_double_commit_fails() {
        let store = ReservationStore::new();
        let org_id = Uuid::new_v4();

        let request = ReserveRequest::new(org_id, "agent1", "api_calls", 100);
        let reservation = store.reserve(request, 1000).unwrap();

        store.commit(reservation.id, 50).unwrap();
        let result = store.commit(reservation.id, 50);

        assert!(matches!(result, Err(ReservationError::InvalidStatus(ReservationStatus::Committed))));
    }
}
