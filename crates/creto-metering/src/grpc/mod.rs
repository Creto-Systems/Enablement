//! gRPC service for event ingestion.
//!
//! This module provides a high-performance gRPC service for ingesting usage events.
//! It includes validation, deduplication, and batching for optimal throughput.

pub mod service;
mod types;

pub use service::{MeteringGrpcService, MeteringServiceConfig, ServiceMetrics};
pub use types::*;
