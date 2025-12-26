//! # creto-metering
//!
//! Usage-based billing and quota enforcement for AI agents.
//!
//! ## Overview
//!
//! This crate provides the metering infrastructure for the Creto Enablement Layer,
//! enabling organizations to:
//!
//! - **Track Usage**: Ingest usage events (API calls, tokens, compute time)
//! - **Enforce Quotas**: Block operations when limits are exceeded
//! - **Generate Invoices**: Aggregate usage into billable line items
//! - **Attribute Costs**: Allocate costs across agents, teams, and projects
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      creto-metering                             │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │   Ingestion  │  │  Aggregation │  │    Quota     │          │
//! │  │    Engine    │→ │    Engine    │→ │   Enforcer   │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! │         ↓                  ↓                  ↓                 │
//! │  ┌──────────────────────────────────────────────────┐          │
//! │  │              PostgreSQL + Redis                   │          │
//! │  └──────────────────────────────────────────────────┘          │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Week 3 Features
//!
//! - **Validation**: Comprehensive event validation with configurable rules
//! - **Deduplication**: Redis-backed idempotency with local fallback
//! - **gRPC Service**: High-performance ingestion service
//! - **Benchmarks**: Target >10K events/sec throughput
//!
//! ## Pattern Source
//!
//! Inspired by [Lago](https://github.com/getlago/lago) event ingestion patterns,
//! rebuilt with Creto Sovereign primitives (NHI, Cedar authorization, audit logging).

pub mod aggregation;
pub mod dedup;
pub mod events;
pub mod grpc;
pub mod invoice;
pub mod pricing;
pub mod quota;
pub mod repository;
pub mod service;
pub mod validation;

pub use dedup::{DedupConfig, DedupResult, Deduplicator};
pub use events::{UsageEvent, UsageEventType};
pub use grpc::{MeteringGrpcService, MeteringServiceConfig};
pub use quota::{Quota, QuotaEnforcer, QuotaPeriod};
pub use repository::{
    EventRepository, PgEventRepository,
    QuotaRepository, PgQuotaRepository,
    InvoiceRepository, PgInvoiceRepository, InvoiceRecord,
};
pub use service::MeteringService;
pub use validation::{BatchValidationResult, EventValidator, ValidationConfig, ValidationError};
