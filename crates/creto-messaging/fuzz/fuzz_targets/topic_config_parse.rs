//! Fuzz test for TopicConfig name validation.
//!
//! This tests topic name parsing and validation with arbitrary strings.

#![no_main]

use libfuzzer_sys::fuzz_target;
use creto_messaging::topic::TopicConfig;
use creto_common::AgentId;

fuzz_target!(|data: &[u8]| {
    // Try to create a TopicConfig with arbitrary bytes as name
    if let Ok(name) = std::str::from_utf8(data) {
        let owner_id = AgentId::new();
        // This should handle any string gracefully
        let _ = TopicConfig::new(name.to_string(), owner_id);
    }
});
