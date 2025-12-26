//! Fuzz test for UsageEvent JSON deserialization.
//!
//! This tests JSON parsing of usage events from arbitrary bytes.

#![no_main]

use libfuzzer_sys::fuzz_target;
use creto_metering::UsageEvent;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize arbitrary bytes as a UsageEvent
    // This should never panic, only return errors gracefully
    let _ = serde_json::from_slice::<UsageEvent>(data);
});
