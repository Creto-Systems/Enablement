//! Fuzz test for Envelope deserialization.
//!
//! This tests the JSON deserialization of encrypted message envelopes
//! from potentially malicious input bytes.

#![no_main]

use libfuzzer_sys::fuzz_target;
use creto_messaging::Envelope;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize arbitrary bytes as an Envelope
    // This should never panic, only return errors gracefully
    let _ = Envelope::from_bytes(data);
});
