//! Fuzz test for identity string parsing.
//!
//! This tests AgentId and UserId parsing from arbitrary strings.
//! Note: OrganizationId doesn't implement FromStr, only AgentId does.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str::FromStr;
use creto_common::AgentId;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as identity strings
    if let Ok(s) = std::str::from_utf8(data) {
        // Test AgentId::from_str - should handle "agent:" prefix
        // This parses UUID after stripping optional "agent:" prefix
        let _ = AgentId::from_str(s);
    }
});
