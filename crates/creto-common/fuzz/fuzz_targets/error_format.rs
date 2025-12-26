//! Fuzz test for error message formatting.
//!
//! This tests that error types handle arbitrary input in their
//! Display implementations without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use creto_common::CretoError;

fuzz_target!(|data: &[u8]| {
    // Test that error types handle arbitrary strings in Display
    if let Ok(s) = std::str::from_utf8(data) {
        // Create various error types with fuzzed input
        let errors = [
            CretoError::InvalidUsageEvent(s.to_string()),
            CretoError::SerializationError(s.to_string()),
            CretoError::NotFound(s.to_string()),
            CretoError::Unauthorized(s.to_string()),
            CretoError::Internal(s.to_string()),
            CretoError::Database(s.to_string()),
            CretoError::EncryptionFailed(s.to_string()),
            CretoError::DecryptionFailed(s.to_string()),
        ];

        // Formatting should never panic
        for err in &errors {
            let _ = format!("{}", err);
            let _ = format!("{:?}", err);
        }
    }
});
