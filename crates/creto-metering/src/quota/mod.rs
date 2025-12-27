//! Quota enforcement for AI agent metering.
//!
//! This module provides high-performance quota checking with:
//! - Bloom filter for fast negative lookups (<1µs)
//! - Local LRU cache for recent checks (~5µs)
//! - Redis fallback for cache misses (~100µs)
//! - Reservation system for pre-allocation
//!
//! ## Performance Targets
//!
//! | Operation | Target Latency |
//! |-----------|---------------|
//! | Bloom filter check | <1µs |
//! | Cache hit | <5µs |
//! | Redis fallback | <100µs |
//! | Total p99 | <10µs |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use creto_metering::quota::{QuotaEnforcer, Quota, QuotaPeriod};
//!
//! let enforcer = QuotaEnforcer::with_defaults();
//!
//! // Register a quota
//! let quota = Quota::new(org_id, "api_calls", 1000, QuotaPeriod::Daily);
//! enforcer.register_quota(&quota);
//!
//! // Check quota before operation
//! let result = enforcer.check(&org_id, &agent_id, "api_calls", 1)?;
//! if result.allowed {
//!     // Perform operation...
//!     enforcer.record_usage(&org_id, &agent_id, "api_calls", 1)?;
//! }
//! ```

mod bloom;
mod enforcer;
mod reservation;
mod types;

pub use bloom::{BloomConfig, QuotaBloomFilter, QuotaKey};
pub use enforcer::{CheckSource, EnforcerConfig, EnforcerError, QuotaCheckResult, QuotaEnforcer};
pub use reservation::{
    Reservation, ReservationError, ReservationStatus, ReservationStore, ReserveRequest,
};
pub use types::{Quota, QuotaPeriod, QuotaStatus};
