//! Identity types for agents, users, and organizations.
//!
//! These types integrate with creto-authz for authorization decisions
//! and creto-nhi for non-human identity management.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an AI agent (NHI - Non-Human Identity).
///
/// # Example
/// ```
/// use creto_common::AgentId;
///
/// let agent = AgentId::new();
/// println!("Agent ID: {}", agent);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(Uuid);

impl AgentId {
    /// Create a new random agent ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create an agent ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "agent:{}", self.0)
    }
}

impl std::str::FromStr for AgentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid_str = s.strip_prefix("agent:").unwrap_or(s);
        Ok(Self(Uuid::parse_str(uuid_str)?))
    }
}

/// Unique identifier for a human user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(Uuid);

impl UserId {
    /// Create a new random user ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create a user ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

/// Unique identifier for an organization (tenant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrganizationId(Uuid);

impl OrganizationId {
    /// Create a new random organization ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create an organization ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for OrganizationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OrganizationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "org:{}", self.0)
    }
}

/// A delegation chain representing the trust path from human to agent.
///
/// Used by creto-authz to evaluate authorization based on delegation depth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationChain {
    /// The original human principal who initiated the delegation.
    pub root: UserId,
    /// Ordered list of agents in the delegation chain.
    pub agents: Vec<AgentId>,
    /// Maximum allowed delegation depth (from Cedar policy).
    pub max_depth: u8,
}

impl DelegationChain {
    /// Create a new delegation chain starting from a user.
    pub fn new(root: UserId) -> Self {
        Self {
            root,
            agents: Vec::new(),
            max_depth: 3, // Default from SDD
        }
    }

    /// Add an agent to the delegation chain.
    ///
    /// Returns `Err` if adding would exceed max depth.
    pub fn delegate(&mut self, agent: AgentId) -> Result<(), &'static str> {
        if self.agents.len() >= self.max_depth as usize {
            return Err("Delegation depth exceeded");
        }
        self.agents.push(agent);
        Ok(())
    }

    /// Get the current delegation depth.
    pub fn depth(&self) -> usize {
        self.agents.len()
    }

    /// Get the leaf agent (most recent in chain).
    pub fn leaf(&self) -> Option<&AgentId> {
        self.agents.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_roundtrip() {
        let agent = AgentId::new();
        let s = agent.to_string();
        let parsed: AgentId = s.parse().unwrap();
        assert_eq!(agent, parsed);
    }

    #[test]
    fn test_delegation_chain_max_depth() {
        let mut chain = DelegationChain::new(UserId::new());
        chain.max_depth = 2;

        assert!(chain.delegate(AgentId::new()).is_ok());
        assert!(chain.delegate(AgentId::new()).is_ok());
        assert!(chain.delegate(AgentId::new()).is_err());
    }
}
