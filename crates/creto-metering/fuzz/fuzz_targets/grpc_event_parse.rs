//! Fuzz test for GrpcUsageEvent to UsageEvent conversion.
//!
//! This tests UUID parsing from potentially malicious gRPC input strings.

#![no_main]

use libfuzzer_sys::fuzz_target;
use creto_metering::grpc::{GrpcUsageEvent, GrpcUsageEventType};

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as UTF-8 strings for UUIDs
    if let Ok(s) = std::str::from_utf8(data) {
        // Split the input into multiple fields
        let parts: Vec<&str> = s.splitn(4, '\n').collect();

        if parts.len() >= 3 {
            let event = GrpcUsageEvent {
                transaction_id: parts.get(0).unwrap_or(&"").to_string(),
                organization_id: parts.get(1).unwrap_or(&"").to_string(),
                agent_id: parts.get(2).unwrap_or(&"").to_string(),
                external_subscription_id: parts.get(3).map(|s| s.to_string()),
                event_type: GrpcUsageEventType::ApiCall,
                code: "fuzz_test".to_string(),
                quantity: 1,
                timestamp: None,
                properties: None,
                delegation_depth: 0,
            };

            // This should never panic, only return errors
            let _ = event.to_usage_event();
        }
    }
});
