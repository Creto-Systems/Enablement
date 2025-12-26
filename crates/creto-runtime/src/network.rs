//! Network policy enforcement for sandboxed agent execution.
//!
//! This module provides egress filtering, DNS policy enforcement, and
//! integration with the Authorization service for network access control.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Network policy defining egress rules and default behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Default action when no rules match.
    pub default_action: NetworkAction,
    /// Ordered list of egress rules (first match wins).
    pub egress_rules: Vec<EgressRule>,
    /// DNS resolution policy.
    pub dns_policy: DnsPolicy,
}

impl NetworkPolicy {
    /// Create a new network policy with default deny.
    pub fn new_default_deny() -> Self {
        Self {
            default_action: NetworkAction::Deny,
            egress_rules: Vec::new(),
            dns_policy: DnsPolicy::default(),
        }
    }

    /// Create a new network policy with default allow.
    pub fn new_default_allow() -> Self {
        Self {
            default_action: NetworkAction::Allow,
            egress_rules: Vec::new(),
            dns_policy: DnsPolicy::default(),
        }
    }

    /// Add an egress rule to the policy.
    pub fn add_rule(&mut self, rule: EgressRule) {
        self.egress_rules.push(rule);
    }
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self::new_default_deny()
    }
}

/// A single egress rule specifying destination and action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressRule {
    /// Destination to match.
    pub destination: EgressDestination,
    /// Action to take if matched.
    pub action: NetworkAction,
    /// Whether to check authorization before allowing.
    #[serde(default)]
    pub authorization_check: bool,
}

impl EgressRule {
    /// Create a new egress rule.
    pub fn new(destination: EgressDestination, action: NetworkAction) -> Self {
        Self {
            destination,
            action,
            authorization_check: false,
        }
    }

    /// Create a new rule that requires authorization.
    pub fn with_authz(destination: EgressDestination) -> Self {
        Self {
            destination,
            action: NetworkAction::RequireAuthz,
            authorization_check: true,
        }
    }
}

/// Destination specification for egress rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EgressDestination {
    /// Any destination (wildcard).
    Any,
    /// IP address or CIDR block (e.g., "10.0.0.0/8").
    CidrBlock(String),
    /// Domain with subdomain wildcard (e.g., "*.example.com").
    Domain(String),
    /// Exact domain match (e.g., "api.example.com").
    DomainExact(String),
    /// Named service (e.g., "s3", "dynamodb").
    Service(String),
}

impl EgressDestination {
    /// Check if this destination matches a given IP address.
    pub fn matches_ip(&self, ip: &IpAddr) -> bool {
        match self {
            EgressDestination::Any => true,
            EgressDestination::CidrBlock(cidr) => {
                // Parse CIDR and check if IP is in range
                Self::ip_in_cidr(ip, cidr)
            }
            _ => false,
        }
    }

    /// Check if this destination matches a given domain.
    pub fn matches_domain(&self, domain: &str) -> bool {
        match self {
            EgressDestination::Any => true,
            EgressDestination::Domain(pattern) => {
                // Wildcard matching (e.g., "*.example.com")
                if let Some(suffix) = pattern.strip_prefix("*.") {
                    // Match if domain equals suffix OR ends with ".suffix"
                    domain == suffix || domain.ends_with(&format!(".{}", suffix))
                } else {
                    domain == pattern
                }
            }
            EgressDestination::DomainExact(exact) => domain == exact,
            _ => false,
        }
    }

    /// Check if IP is in CIDR block (simplified implementation).
    fn ip_in_cidr(ip: &IpAddr, cidr: &str) -> bool {
        // Parse CIDR notation
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let network_addr: IpAddr = match parts[0].parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        let prefix_len: u8 = match parts[1].parse() {
            Ok(len) => len,
            Err(_) => return false,
        };

        // Only handle IPv4 for now
        match (ip, network_addr) {
            (IpAddr::V4(ip_v4), IpAddr::V4(net_v4)) => {
                let ip_bits = u32::from(*ip_v4);
                let net_bits = u32::from(net_v4);
                let mask = if prefix_len == 0 {
                    0
                } else {
                    !0u32 << (32 - prefix_len)
                };
                (ip_bits & mask) == (net_bits & mask)
            }
            _ => false,
        }
    }
}

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkAction {
    /// Allow the connection.
    Allow,
    /// Deny the connection.
    Deny,
    /// Require authorization check before allowing.
    RequireAuthz,
}

/// DNS resolution policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsPolicy {
    /// Whether to allow DNS resolution.
    pub allow_dns: bool,
    /// Allowed DNS servers (empty = system default).
    #[serde(default)]
    pub allowed_nameservers: Vec<String>,
    /// Maximum DNS cache TTL in seconds.
    #[serde(default = "default_dns_ttl")]
    pub max_ttl_seconds: u32,
    /// Whether to log all DNS queries.
    #[serde(default)]
    pub log_queries: bool,
}

fn default_dns_ttl() -> u32 {
    300 // 5 minutes
}

impl Default for DnsPolicy {
    fn default() -> Self {
        Self {
            allow_dns: true,
            allowed_nameservers: Vec::new(),
            max_ttl_seconds: default_dns_ttl(),
            log_queries: false,
        }
    }
}

/// Network policy enforcer that checks egress requests.
pub struct NetworkPolicyEnforcer {
    policy: NetworkPolicy,
}

impl NetworkPolicyEnforcer {
    /// Create a new enforcer with the given policy.
    pub fn new(policy: NetworkPolicy) -> Self {
        Self { policy }
    }

    /// Check if an IP address is allowed by the policy.
    pub fn check_ip(&self, ip: &IpAddr) -> EgressDecision {
        for rule in &self.policy.egress_rules {
            if rule.destination.matches_ip(ip) {
                return EgressDecision {
                    action: rule.action,
                    requires_authz: rule.authorization_check,
                    matched_rule: Some(format!("{:?}", rule.destination)),
                };
            }
        }

        EgressDecision {
            action: self.policy.default_action,
            requires_authz: false,
            matched_rule: None,
        }
    }

    /// Check if a domain is allowed by the policy.
    pub fn check_domain(&self, domain: &str) -> EgressDecision {
        for rule in &self.policy.egress_rules {
            if rule.destination.matches_domain(domain) {
                return EgressDecision {
                    action: rule.action,
                    requires_authz: rule.authorization_check,
                    matched_rule: Some(format!("{:?}", rule.destination)),
                };
            }
        }

        EgressDecision {
            action: self.policy.default_action,
            requires_authz: false,
            matched_rule: None,
        }
    }

    /// Check if a DNS query is allowed.
    pub fn check_dns(&self, query: &str) -> bool {
        if !self.policy.dns_policy.allow_dns {
            return false;
        }

        if self.policy.dns_policy.log_queries {
            tracing::debug!("DNS query: {}", query);
        }

        true
    }

    /// Get the DNS policy.
    pub fn dns_policy(&self) -> &DnsPolicy {
        &self.policy.dns_policy
    }
}

/// Result of an egress check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressDecision {
    /// Action to take.
    pub action: NetworkAction,
    /// Whether authorization is required.
    pub requires_authz: bool,
    /// Which rule matched (if any).
    pub matched_rule: Option<String>,
}

impl EgressDecision {
    /// Check if the egress should be allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self.action, NetworkAction::Allow)
    }

    /// Check if authorization is required.
    pub fn needs_authorization(&self) -> bool {
        self.requires_authz || matches!(self.action, NetworkAction::RequireAuthz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_default_deny_policy() {
        let policy = NetworkPolicy::new_default_deny();
        let enforcer = NetworkPolicyEnforcer::new(policy);

        let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let decision = enforcer.check_ip(&ip);

        assert_eq!(decision.action, NetworkAction::Deny);
        assert!(!decision.is_allowed());
    }

    #[test]
    fn test_allow_specific_domain() {
        let mut policy = NetworkPolicy::new_default_deny();
        policy.add_rule(EgressRule::new(
            EgressDestination::DomainExact("api.example.com".to_string()),
            NetworkAction::Allow,
        ));

        let enforcer = NetworkPolicyEnforcer::new(policy);

        let decision = enforcer.check_domain("api.example.com");
        assert_eq!(decision.action, NetworkAction::Allow);
        assert!(decision.is_allowed());

        let decision = enforcer.check_domain("other.example.com");
        assert_eq!(decision.action, NetworkAction::Deny);
        assert!(!decision.is_allowed());
    }

    #[test]
    fn test_cidr_block_matching() {
        let mut policy = NetworkPolicy::new_default_deny();
        policy.add_rule(EgressRule::new(
            EgressDestination::CidrBlock("10.0.0.0/8".to_string()),
            NetworkAction::Allow,
        ));

        let enforcer = NetworkPolicyEnforcer::new(policy);

        // IP in range
        let ip = IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3));
        let decision = enforcer.check_ip(&ip);
        assert_eq!(decision.action, NetworkAction::Allow);
        assert!(decision.is_allowed());

        // IP out of range
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let decision = enforcer.check_ip(&ip);
        assert_eq!(decision.action, NetworkAction::Deny);
        assert!(!decision.is_allowed());
    }

    #[test]
    fn test_require_authz_flow() {
        let mut policy = NetworkPolicy::new_default_deny();
        policy.add_rule(EgressRule::with_authz(
            EgressDestination::Domain("*.secure.example.com".to_string()),
        ));

        let enforcer = NetworkPolicyEnforcer::new(policy);

        let decision = enforcer.check_domain("api.secure.example.com");
        assert_eq!(decision.action, NetworkAction::RequireAuthz);
        assert!(decision.needs_authorization());
    }

    #[test]
    fn test_dns_policy_enforcement() {
        let mut policy = NetworkPolicy::new_default_deny();
        policy.dns_policy.allow_dns = false;

        let enforcer = NetworkPolicyEnforcer::new(policy);
        assert!(!enforcer.check_dns("example.com"));

        let mut policy = NetworkPolicy::new_default_deny();
        policy.dns_policy.allow_dns = true;
        policy.dns_policy.log_queries = true;

        let enforcer = NetworkPolicyEnforcer::new(policy);
        assert!(enforcer.check_dns("example.com"));
    }

    #[test]
    fn test_wildcard_domain_matching() {
        let dest = EgressDestination::Domain("*.example.com".to_string());

        assert!(dest.matches_domain("api.example.com"));
        assert!(dest.matches_domain("www.example.com"));
        assert!(dest.matches_domain("example.com"));
        assert!(!dest.matches_domain("example.org"));
        assert!(!dest.matches_domain("notexample.com"));
    }

    #[test]
    fn test_first_match_wins() {
        let mut policy = NetworkPolicy::new_default_allow();

        // More specific deny rule first
        policy.add_rule(EgressRule::new(
            EgressDestination::DomainExact("blocked.example.com".to_string()),
            NetworkAction::Deny,
        ));

        // Broader allow rule second
        policy.add_rule(EgressRule::new(
            EgressDestination::Domain("*.example.com".to_string()),
            NetworkAction::Allow,
        ));

        let enforcer = NetworkPolicyEnforcer::new(policy);

        // Should match first rule (deny)
        let decision = enforcer.check_domain("blocked.example.com");
        assert_eq!(decision.action, NetworkAction::Deny);

        // Should match second rule (allow)
        let decision = enforcer.check_domain("api.example.com");
        assert_eq!(decision.action, NetworkAction::Allow);
    }

    #[test]
    fn test_cidr_parsing() {
        // Valid /24 network
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        assert!(EgressDestination::ip_in_cidr(&ip, "192.168.1.0/24"));
        assert!(!EgressDestination::ip_in_cidr(&ip, "192.168.2.0/24"));

        // Valid /16 network
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 5, 10));
        assert!(EgressDestination::ip_in_cidr(&ip, "172.16.0.0/16"));
        assert!(!EgressDestination::ip_in_cidr(&ip, "172.17.0.0/16"));

        // Edge case: /32 (single IP)
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        assert!(EgressDestination::ip_in_cidr(&ip, "10.0.0.1/32"));
        assert!(!EgressDestination::ip_in_cidr(&ip, "10.0.0.2/32"));
    }

    #[test]
    fn test_service_destination() {
        let dest = EgressDestination::Service("s3".to_string());

        // Services don't match IPs or domains directly
        assert!(!dest.matches_ip(&IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))));
        assert!(!dest.matches_domain("s3.amazonaws.com"));

        // Service matching would be done by a higher-level resolver
    }

    #[test]
    fn test_any_destination() {
        let dest = EgressDestination::Any;

        assert!(dest.matches_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
        assert!(dest.matches_domain("example.com"));
    }

    #[test]
    fn test_dns_policy_configuration() {
        let policy = DnsPolicy {
            allow_dns: true,
            allowed_nameservers: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()],
            max_ttl_seconds: 600,
            log_queries: true,
        };

        assert!(policy.allow_dns);
        assert_eq!(policy.allowed_nameservers.len(), 2);
        assert_eq!(policy.max_ttl_seconds, 600);
        assert!(policy.log_queries);
    }
}
