//! Secret injection for sandbox execution.

use async_trait::async_trait;
use creto_common::{AgentId, CretoResult, OrganizationId};
use serde::{Deserialize, Serialize};

/// A secret to be injected into a sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMount {
    /// Name of the secret (used as env var or file name).
    pub name: String,

    /// How to inject the secret.
    pub mount_type: SecretMountType,

    /// Source of the secret.
    pub source: SecretSource,
}

impl SecretMount {
    /// Create a new environment variable secret.
    pub fn env_var(name: impl Into<String>, source: SecretSource) -> Self {
        Self {
            name: name.into(),
            mount_type: SecretMountType::EnvironmentVariable,
            source,
        }
    }

    /// Create a new file mount secret.
    pub fn file(name: impl Into<String>, path: impl Into<String>, source: SecretSource) -> Self {
        Self {
            name: name.into(),
            mount_type: SecretMountType::File {
                path: path.into(),
                mode: 0o600,
            },
            source,
        }
    }
}

/// How to mount a secret in the sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretMountType {
    /// Inject as environment variable.
    EnvironmentVariable,
    /// Mount as file.
    File {
        /// Path inside sandbox.
        path: String,
        /// File permissions (octal).
        mode: u32,
    },
    /// Mount as directory (for multiple files).
    Directory {
        /// Directory path inside sandbox.
        path: String,
    },
}

/// Source of a secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretSource {
    /// From a vault (e.g., HashiCorp Vault, AWS Secrets Manager).
    Vault {
        /// Vault path.
        path: String,
        /// Key within the secret.
        key: String,
    },
    /// From organization's stored secrets.
    OrganizationSecret {
        /// Secret name in organization's secret store.
        name: String,
    },
    /// From agent's delegated credentials.
    AgentCredential {
        /// Credential name.
        name: String,
    },
    /// Inline value (for non-sensitive config).
    Inline {
        /// The value.
        value: String,
    },
}

/// Trait for secret providers.
#[async_trait]
pub trait SecretProvider: Send + Sync {
    /// Resolve a secret to its value.
    async fn resolve(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        source: &SecretSource,
    ) -> CretoResult<SecretValue>;

    /// Check if agent is authorized to access a secret.
    async fn authorize(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        source: &SecretSource,
    ) -> CretoResult<bool>;
}

/// A resolved secret value.
#[derive(Clone)]
pub struct SecretValue {
    /// The secret data.
    value: Vec<u8>,
    /// Whether this is binary data.
    binary: bool,
}

impl SecretValue {
    /// Create a new text secret.
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            value: value.into().into_bytes(),
            binary: false,
        }
    }

    /// Create a new binary secret.
    pub fn binary(value: Vec<u8>) -> Self {
        Self {
            value,
            binary: true,
        }
    }

    /// Get the value as string (if text).
    pub fn as_str(&self) -> Option<&str> {
        if self.binary {
            None
        } else {
            std::str::from_utf8(&self.value).ok()
        }
    }

    /// Get the raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.value
    }

    /// Check if this is binary data.
    pub fn is_binary(&self) -> bool {
        self.binary
    }
}

// Implement Debug manually to avoid leaking secrets
impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretValue")
            .field("binary", &self.binary)
            .field("length", &self.value.len())
            .finish()
    }
}

/// Composite secret provider that chains multiple providers.
pub struct ChainedSecretProvider {
    providers: Vec<Box<dyn SecretProvider>>,
}

impl ChainedSecretProvider {
    /// Create a new chained provider.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a provider to the chain.
    pub fn add_provider(&mut self, provider: Box<dyn SecretProvider>) {
        self.providers.push(provider);
    }
}

impl Default for ChainedSecretProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretProvider for ChainedSecretProvider {
    async fn resolve(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        source: &SecretSource,
    ) -> CretoResult<SecretValue> {
        // Try each provider in order
        for provider in &self.providers {
            match provider.resolve(organization_id, agent_id, source).await {
                Ok(value) => return Ok(value),
                Err(_) => continue,
            }
        }

        Err(creto_common::CretoError::SecretResolutionFailed {
            secret_name: format!("{:?}", source),
            source: None,
        })
    }

    async fn authorize(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        source: &SecretSource,
    ) -> CretoResult<bool> {
        // All providers must authorize
        for provider in &self.providers {
            if !provider
                .authorize(organization_id, agent_id, source)
                .await?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Mock secret provider for testing.
pub struct MockSecretProvider {
    secrets: std::collections::HashMap<String, SecretValue>,
}

impl MockSecretProvider {
    /// Create a new mock provider.
    pub fn new() -> Self {
        Self {
            secrets: std::collections::HashMap::new(),
        }
    }

    /// Add a mock secret.
    pub fn add_secret(&mut self, key: impl Into<String>, value: SecretValue) {
        self.secrets.insert(key.into(), value);
    }
}

impl Default for MockSecretProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretProvider for MockSecretProvider {
    async fn resolve(
        &self,
        _organization_id: OrganizationId,
        _agent_id: AgentId,
        source: &SecretSource,
    ) -> CretoResult<SecretValue> {
        let key = match source {
            SecretSource::Vault { path, key } => format!("{}:{}", path, key),
            SecretSource::OrganizationSecret { name } => name.clone(),
            SecretSource::AgentCredential { name } => name.clone(),
            SecretSource::Inline { value } => return Ok(SecretValue::text(value)),
        };

        self.secrets.get(&key).cloned().ok_or_else(|| {
            creto_common::CretoError::SecretResolutionFailed {
                secret_name: key,
                source: None,
            }
        })
    }

    async fn authorize(
        &self,
        _organization_id: OrganizationId,
        _agent_id: AgentId,
        _source: &SecretSource,
    ) -> CretoResult<bool> {
        // Mock always authorizes
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_mount_env() {
        let mount = SecretMount::env_var(
            "API_KEY",
            SecretSource::OrganizationSecret {
                name: "openai_key".to_string(),
            },
        );

        assert_eq!(mount.name, "API_KEY");
        assert!(matches!(
            mount.mount_type,
            SecretMountType::EnvironmentVariable
        ));
    }

    #[test]
    fn test_secret_mount_file() {
        let mount = SecretMount::file(
            "credentials",
            "/etc/secrets/creds.json",
            SecretSource::Vault {
                path: "secret/data/myapp".to_string(),
                key: "credentials".to_string(),
            },
        );

        assert_eq!(mount.name, "credentials");
        if let SecretMountType::File { path, mode } = mount.mount_type {
            assert_eq!(path, "/etc/secrets/creds.json");
            assert_eq!(mode, 0o600);
        } else {
            panic!("Expected File mount type");
        }
    }

    #[test]
    fn test_secret_value_text() {
        let value = SecretValue::text("my-secret");
        assert_eq!(value.as_str(), Some("my-secret"));
        assert!(!value.is_binary());
    }

    #[test]
    fn test_secret_value_binary() {
        let value = SecretValue::binary(vec![0x00, 0x01, 0x02]);
        assert!(value.is_binary());
        assert!(value.as_str().is_none());
        assert_eq!(value.as_bytes(), &[0x00, 0x01, 0x02]);
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let mut provider = MockSecretProvider::new();
        provider.add_secret("test_key", SecretValue::text("test_value"));

        let value = provider
            .resolve(
                OrganizationId::new(),
                AgentId::new(),
                &SecretSource::OrganizationSecret {
                    name: "test_key".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(value.as_str(), Some("test_value"));
    }
}
