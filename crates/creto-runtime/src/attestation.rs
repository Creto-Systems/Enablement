//! Cryptographic attestation for sandbox security and integrity.
//!
//! This module provides attestation mechanisms to verify sandbox environments,
//! ensuring that code execution occurs in trusted, verified containers with
//! cryptographic proof of platform security features.

use chrono::{DateTime, Duration, Utc};
use creto_common::{AgentId, CretoResult};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::sandbox::SandboxId;

/// Platform security technologies for sandbox attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationPlatform {
    /// Google's gVisor user-space kernel.
    GVisor,
    /// Kata Containers with lightweight VMs.
    Kata,
    /// Intel SGX trusted execution environment.
    SGX,
    /// AMD SEV secure encrypted virtualization.
    SEV,
    /// No hardware security features (development only).
    None,
}

impl AttestationPlatform {
    /// Check if this platform provides hardware-level security.
    pub fn has_hardware_security(&self) -> bool {
        matches!(self, AttestationPlatform::SGX | AttestationPlatform::SEV)
    }

    /// Check if this platform is production-ready.
    pub fn is_production_ready(&self) -> bool {
        !matches!(self, AttestationPlatform::None)
    }
}

impl fmt::Display for AttestationPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AttestationPlatform::GVisor => "gVisor",
            AttestationPlatform::Kata => "Kata Containers",
            AttestationPlatform::SGX => "Intel SGX",
            AttestationPlatform::SEV => "AMD SEV",
            AttestationPlatform::None => "None",
        };
        write!(f, "{}", name)
    }
}

/// Cryptographic attestation proving sandbox security properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Sandbox being attested.
    pub sandbox_id: SandboxId,

    /// Agent using the sandbox.
    pub agent_id: AgentId,

    /// Hash of the container image (BLAKE3).
    pub image_hash: Vec<u8>,

    /// Hash of the sandbox configuration (BLAKE3).
    pub config_hash: Vec<u8>,

    /// Hash of initialization state (BLAKE3).
    pub init_hash: Vec<u8>,

    /// Security platform providing attestation.
    pub platform: AttestationPlatform,

    /// Platform-specific evidence (e.g., SGX quote, SEV measurement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_evidence: Option<Vec<u8>>,

    /// When this attestation was created.
    pub created_at: DateTime<Utc>,

    /// When this attestation expires.
    pub valid_until: DateTime<Utc>,

    /// Cryptographic signature over all fields (Ed25519).
    pub signature: Vec<u8>,
}

impl Attestation {
    /// Create a new attestation (without signature).
    pub fn new(
        sandbox_id: SandboxId,
        agent_id: AgentId,
        image_hash: Vec<u8>,
        config_hash: Vec<u8>,
        init_hash: Vec<u8>,
        platform: AttestationPlatform,
        validity_duration: Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            sandbox_id,
            agent_id,
            image_hash,
            config_hash,
            init_hash,
            platform,
            platform_evidence: None,
            created_at: now,
            valid_until: now + validity_duration,
            signature: Vec::new(),
        }
    }

    /// Check if this attestation is currently valid (not expired).
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.valid_until
    }

    /// Get time remaining until expiration.
    pub fn time_remaining(&self) -> Option<Duration> {
        let remaining = self.valid_until.signed_duration_since(Utc::now());
        if remaining.num_seconds() > 0 {
            Some(remaining)
        } else {
            None
        }
    }

    /// Get the canonical bytes for signing/verification.
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.sandbox_id.as_uuid().as_bytes()[..]);
        bytes.extend_from_slice(&self.agent_id.as_uuid().as_bytes()[..]);
        bytes.extend_from_slice(&self.image_hash);
        bytes.extend_from_slice(&self.config_hash);
        bytes.extend_from_slice(&self.init_hash);
        bytes.push(self.platform as u8);
        if let Some(ref evidence) = self.platform_evidence {
            bytes.extend_from_slice(evidence);
        }
        bytes.extend_from_slice(&self.created_at.timestamp().to_le_bytes());
        bytes.extend_from_slice(&self.valid_until.timestamp().to_le_bytes());
        bytes
    }
}

/// Policy controlling attestation requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationPolicy {
    /// Whether attestation is required for sandbox execution.
    pub require_attestation: bool,

    /// Allowed attestation platforms (empty = all allowed).
    #[serde(default)]
    pub allowed_platforms: Vec<AttestationPlatform>,

    /// Maximum age of attestation in seconds.
    #[serde(default = "default_max_attestation_age")]
    pub max_attestation_age_seconds: u64,
}

fn default_max_attestation_age() -> u64 {
    3600 // 1 hour
}

impl Default for AttestationPolicy {
    fn default() -> Self {
        Self {
            require_attestation: false,
            allowed_platforms: Vec::new(),
            max_attestation_age_seconds: default_max_attestation_age(),
        }
    }
}

impl AttestationPolicy {
    /// Create a strict policy requiring hardware security.
    pub fn strict() -> Self {
        Self {
            require_attestation: true,
            allowed_platforms: vec![AttestationPlatform::SGX, AttestationPlatform::SEV],
            max_attestation_age_seconds: 300, // 5 minutes
        }
    }

    /// Create a production policy allowing gVisor/Kata.
    pub fn production() -> Self {
        Self {
            require_attestation: true,
            allowed_platforms: vec![
                AttestationPlatform::GVisor,
                AttestationPlatform::Kata,
                AttestationPlatform::SGX,
                AttestationPlatform::SEV,
            ],
            max_attestation_age_seconds: 3600, // 1 hour
        }
    }

    /// Create a development policy (no attestation required).
    pub fn development() -> Self {
        Self {
            require_attestation: false,
            allowed_platforms: vec![AttestationPlatform::None],
            max_attestation_age_seconds: 86400, // 24 hours
        }
    }

    /// Validate an attestation against this policy.
    pub fn validate(&self, attestation: &Attestation) -> CretoResult<()> {
        if self.require_attestation {
            // Check platform is allowed
            if !self.allowed_platforms.is_empty()
                && !self.allowed_platforms.contains(&attestation.platform)
            {
                return Err(creto_common::CretoError::ValidationFailed(format!(
                    "Platform {} not allowed by policy",
                    attestation.platform
                )));
            }

            // Check expiration
            if !attestation.is_valid() {
                return Err(creto_common::CretoError::ValidationFailed(
                    "Attestation has expired".to_string(),
                ));
            }

            // Check age
            let age = Utc::now()
                .signed_duration_since(attestation.created_at)
                .num_seconds() as u64;
            if age > self.max_attestation_age_seconds {
                return Err(creto_common::CretoError::ValidationFailed(format!(
                    "Attestation too old: {} seconds (max: {})",
                    age, self.max_attestation_age_seconds
                )));
            }
        }

        Ok(())
    }
}

/// Trait for generating attestations.
#[async_trait::async_trait]
pub trait AttestationGenerator: Send + Sync {
    /// Generate an attestation for a sandbox.
    async fn generate(
        &self,
        sandbox_id: SandboxId,
        agent_id: AgentId,
        image_hash: Vec<u8>,
        config_hash: Vec<u8>,
        init_hash: Vec<u8>,
        platform: AttestationPlatform,
    ) -> CretoResult<Attestation>;
}

/// Trait for verifying attestations.
#[async_trait::async_trait]
pub trait AttestationVerifier: Send + Sync {
    /// Verify an attestation's signature and validity.
    async fn verify(&self, attestation: &Attestation) -> CretoResult<bool>;

    /// Verify an attestation against a policy.
    async fn verify_with_policy(
        &self,
        attestation: &Attestation,
        policy: &AttestationPolicy,
    ) -> CretoResult<bool> {
        // First check signature
        if !self.verify(attestation).await? {
            return Ok(false);
        }

        // Then check policy
        policy.validate(attestation)?;
        Ok(true)
    }
}

/// Mock attestation provider for testing.
pub struct MockAttestationProvider {
    signing_key: Vec<u8>,
    /// Kept for future verification implementation.
    #[allow(dead_code)]
    verification_key: Vec<u8>,
}

impl MockAttestationProvider {
    /// Create a new mock provider with generated keys.
    pub fn new() -> Self {
        // In production, this would use Ed25519 key generation
        // For testing, we use simple fixed keys
        Self {
            signing_key: vec![0x42; 32],
            verification_key: vec![0x43; 32],
        }
    }

    /// Create with specific keys.
    pub fn with_keys(signing_key: Vec<u8>, verification_key: Vec<u8>) -> Self {
        Self {
            signing_key,
            verification_key,
        }
    }

    /// Sign data using mock Ed25519-style signature.
    fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Mock signature: BLAKE3 hash of (signing_key || data)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.signing_key);
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Verify mock signature.
    fn verify_signature(&self, data: &[u8], signature: &[u8]) -> bool {
        // Mock verification: recompute and compare
        let expected = self.sign(data);
        expected == signature
    }
}

impl Default for MockAttestationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AttestationGenerator for MockAttestationProvider {
    async fn generate(
        &self,
        sandbox_id: SandboxId,
        agent_id: AgentId,
        image_hash: Vec<u8>,
        config_hash: Vec<u8>,
        init_hash: Vec<u8>,
        platform: AttestationPlatform,
    ) -> CretoResult<Attestation> {
        let mut attestation = Attestation::new(
            sandbox_id,
            agent_id,
            image_hash,
            config_hash,
            init_hash,
            platform,
            Duration::hours(1),
        );

        // Generate platform evidence for hardware platforms
        if platform.has_hardware_security() {
            attestation.platform_evidence = Some(vec![0xAB; 64]);
        }

        // Sign the attestation
        let canonical = attestation.canonical_bytes();
        attestation.signature = self.sign(&canonical);

        Ok(attestation)
    }
}

#[async_trait::async_trait]
impl AttestationVerifier for MockAttestationProvider {
    async fn verify(&self, attestation: &Attestation) -> CretoResult<bool> {
        let canonical = attestation.canonical_bytes();
        Ok(self.verify_signature(&canonical, &attestation.signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hashes() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut image_hasher = blake3::Hasher::new();
        image_hasher.update(b"image-data");
        let image_hash = image_hasher.finalize().as_bytes().to_vec();

        let mut config_hasher = blake3::Hasher::new();
        config_hasher.update(b"config-data");
        let config_hash = config_hasher.finalize().as_bytes().to_vec();

        let mut init_hasher = blake3::Hasher::new();
        init_hasher.update(b"init-data");
        let init_hash = init_hasher.finalize().as_bytes().to_vec();

        (image_hash, config_hash, init_hash)
    }

    #[tokio::test]
    async fn test_attestation_generation() {
        let provider = MockAttestationProvider::new();
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        let attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash,
                config_hash,
                init_hash,
                AttestationPlatform::GVisor,
            )
            .await
            .expect("Failed to generate attestation");

        assert_eq!(attestation.sandbox_id, sandbox_id);
        assert_eq!(attestation.agent_id, agent_id);
        assert_eq!(attestation.platform, AttestationPlatform::GVisor);
        assert!(!attestation.signature.is_empty());
        assert!(attestation.is_valid());
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let provider = MockAttestationProvider::new();
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        let attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash.clone(),
                config_hash.clone(),
                init_hash.clone(),
                AttestationPlatform::SGX,
            )
            .await
            .expect("Failed to generate attestation");

        // Valid signature should verify
        let valid = provider
            .verify(&attestation)
            .await
            .expect("Verification failed");
        assert!(valid, "Valid attestation should verify");

        // Tampered attestation should fail
        let mut tampered = attestation.clone();
        tampered.image_hash[0] ^= 0xFF;
        let invalid = provider
            .verify(&tampered)
            .await
            .expect("Verification failed");
        assert!(!invalid, "Tampered attestation should not verify");
    }

    #[tokio::test]
    async fn test_platform_filtering() {
        let policy = AttestationPolicy {
            require_attestation: true,
            allowed_platforms: vec![AttestationPlatform::SGX, AttestationPlatform::SEV],
            max_attestation_age_seconds: 3600,
        };

        let provider = MockAttestationProvider::new();
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        // SGX should be allowed
        let sgx_attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash.clone(),
                config_hash.clone(),
                init_hash.clone(),
                AttestationPlatform::SGX,
            )
            .await
            .expect("Failed to generate attestation");

        let result = provider
            .verify_with_policy(&sgx_attestation, &policy)
            .await;
        assert!(result.is_ok(), "SGX should be allowed");

        // GVisor should be rejected
        let gvisor_attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash.clone(),
                config_hash.clone(),
                init_hash.clone(),
                AttestationPlatform::GVisor,
            )
            .await
            .expect("Failed to generate attestation");

        let result = provider
            .verify_with_policy(&gvisor_attestation, &policy)
            .await;
        assert!(result.is_err(), "GVisor should be rejected by policy");
    }

    #[tokio::test]
    async fn test_expiration_checking() {
        let provider = MockAttestationProvider::new();
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        // Create attestation with very short validity
        let mut attestation = Attestation::new(
            sandbox_id,
            agent_id,
            image_hash,
            config_hash,
            init_hash,
            AttestationPlatform::Kata,
            Duration::seconds(0), // Already expired
        );

        let canonical = attestation.canonical_bytes();
        attestation.signature = provider.sign(&canonical);

        // Should be expired
        assert!(!attestation.is_valid(), "Attestation should be expired");
        assert_eq!(attestation.time_remaining(), None);

        // Policy should reject expired attestation
        let policy = AttestationPolicy::production();
        let result = policy.validate(&attestation);
        assert!(result.is_err(), "Policy should reject expired attestation");
    }

    #[tokio::test]
    async fn test_policy_enforcement() {
        let provider = MockAttestationProvider::new();
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        // Test strict policy (hardware only)
        let strict_policy = AttestationPolicy::strict();
        let gvisor_attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash.clone(),
                config_hash.clone(),
                init_hash.clone(),
                AttestationPlatform::GVisor,
            )
            .await
            .expect("Failed to generate attestation");

        let result = provider
            .verify_with_policy(&gvisor_attestation, &strict_policy)
            .await;
        assert!(
            result.is_err(),
            "Strict policy should reject non-hardware platforms"
        );

        // Test production policy (allows gVisor)
        let prod_policy = AttestationPolicy::production();
        let result = provider
            .verify_with_policy(&gvisor_attestation, &prod_policy)
            .await;
        assert!(
            result.is_ok(),
            "Production policy should allow gVisor: {:?}",
            result
        );

        // Test development policy (allows None)
        let dev_policy = AttestationPolicy::development();
        let none_attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash,
                config_hash,
                init_hash,
                AttestationPlatform::None,
            )
            .await
            .expect("Failed to generate attestation");

        let result = provider
            .verify_with_policy(&none_attestation, &dev_policy)
            .await;
        assert!(
            result.is_ok(),
            "Development policy should allow None platform"
        );
    }

    #[test]
    fn test_platform_properties() {
        assert!(AttestationPlatform::SGX.has_hardware_security());
        assert!(AttestationPlatform::SEV.has_hardware_security());
        assert!(!AttestationPlatform::GVisor.has_hardware_security());
        assert!(!AttestationPlatform::Kata.has_hardware_security());
        assert!(!AttestationPlatform::None.has_hardware_security());

        assert!(AttestationPlatform::SGX.is_production_ready());
        assert!(AttestationPlatform::GVisor.is_production_ready());
        assert!(!AttestationPlatform::None.is_production_ready());
    }

    #[test]
    fn test_attestation_validity_time() {
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        // Create attestation valid for 1 hour
        let attestation = Attestation::new(
            sandbox_id,
            agent_id,
            image_hash,
            config_hash,
            init_hash,
            AttestationPlatform::Kata,
            Duration::hours(1),
        );

        assert!(attestation.is_valid());
        let remaining = attestation.time_remaining().expect("Should have time remaining");
        assert!(remaining.num_seconds() > 3500); // ~1 hour minus a bit
        assert!(remaining.num_seconds() <= 3600);
    }

    #[tokio::test]
    async fn test_hardware_platform_evidence() {
        let provider = MockAttestationProvider::new();
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let (image_hash, config_hash, init_hash) = create_test_hashes();

        // SGX should have platform evidence
        let sgx_attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash.clone(),
                config_hash.clone(),
                init_hash.clone(),
                AttestationPlatform::SGX,
            )
            .await
            .expect("Failed to generate attestation");

        assert!(
            sgx_attestation.platform_evidence.is_some(),
            "SGX attestation should include platform evidence"
        );

        // GVisor should not have platform evidence
        let gvisor_attestation = provider
            .generate(
                sandbox_id,
                agent_id,
                image_hash,
                config_hash,
                init_hash,
                AttestationPlatform::GVisor,
            )
            .await
            .expect("Failed to generate attestation");

        assert!(
            gvisor_attestation.platform_evidence.is_none(),
            "GVisor attestation should not include platform evidence"
        );
    }
}
