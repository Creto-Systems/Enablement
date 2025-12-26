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
//! ## Pattern Source
//!
//! Inspired by [Lago](https://github.com/getlago/lago) event ingestion patterns,
//! rebuilt with Creto Sovereign primitives (NHI, Cedar authorization, audit logging).

pub mod aggregation;
pub mod events;
pub mod invoice;
pub mod pricing;
pub mod quota;
pub mod repository;
pub mod service;

pub use events::{UsageEvent, UsageEventType};
pub use quota::{Quota, QuotaEnforcer, QuotaPeriod};
pub use repository::{
    EventRepository, PgEventRepository,
    QuotaRepository, PgQuotaRepository,
    InvoiceRepository, PgInvoiceRepository, InvoiceRecord,
};
pub use service::MeteringService;
