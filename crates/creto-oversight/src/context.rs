//! Context enrichment for oversight requests.

use serde::{Deserialize, Serialize};

/// Context attached to an oversight request for reviewer decision-making.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Summary of what the agent wants to do.
    pub summary: String,

    /// Detailed explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    /// Relevant conversation history.
    #[serde(default)]
    pub conversation: Vec<ConversationMessage>,

    /// Related data items.
    #[serde(default)]
    pub data_items: Vec<DataItem>,

    /// Risk assessment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_assessment: Option<RiskAssessment>,

    /// Previous similar requests for reference.
    #[serde(default)]
    pub precedents: Vec<Precedent>,
}

impl RequestContext {
    /// Create a new context with a summary.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            details: None,
            conversation: Vec::new(),
            data_items: Vec::new(),
            risk_assessment: None,
            precedents: Vec::new(),
        }
    }

    /// Add details.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add a conversation message.
    pub fn add_message(&mut self, message: ConversationMessage) {
        self.conversation.push(message);
    }

    /// Add a data item.
    pub fn add_data(&mut self, item: DataItem) {
        self.data_items.push(item);
    }

    /// Set risk assessment.
    pub fn with_risk(mut self, risk: RiskAssessment) -> Self {
        self.risk_assessment = Some(risk);
        self
    }
}

/// A message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Role of the speaker.
    pub role: MessageRole,
    /// Message content.
    pub content: String,
    /// Timestamp (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Role of a conversation participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// Human user.
    User,
    /// AI assistant.
    Assistant,
    /// System message.
    System,
    /// Tool/function result.
    Tool,
}

/// A data item attached to the context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    /// Type of data.
    pub data_type: String,
    /// Human-readable label.
    pub label: String,
    /// The data value.
    pub value: serde_json::Value,
    /// Whether this data is sensitive.
    #[serde(default)]
    pub sensitive: bool,
}

/// Risk assessment for the requested action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level.
    pub level: RiskLevel,
    /// Risk score (0-100).
    pub score: u8,
    /// Factors contributing to the risk.
    pub factors: Vec<RiskFactor>,
    /// Recommended mitigations.
    #[serde(default)]
    pub mitigations: Vec<String>,
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Minimal risk.
    Low,
    /// Moderate risk.
    Medium,
    /// Significant risk.
    High,
    /// Critical risk requiring extra scrutiny.
    Critical,
}

/// A factor contributing to the risk assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor name.
    pub name: String,
    /// Description of why this is a risk.
    pub description: String,
    /// Impact on risk score.
    pub impact: i8,
}

/// A precedent (similar past request) for reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precedent {
    /// ID of the past request.
    pub request_id: String,
    /// Summary of the past request.
    pub summary: String,
    /// What decision was made.
    pub decision: String,
    /// How similar this precedent is (0-100).
    pub similarity_score: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let context = RequestContext::new("Transfer $1000 to external account")
            .with_details("The user requested a wire transfer to account XYZ")
            .with_risk(RiskAssessment {
                level: RiskLevel::Medium,
                score: 45,
                factors: vec![RiskFactor {
                    name: "Amount".to_string(),
                    description: "Medium-sized transaction".to_string(),
                    impact: 20,
                }],
                mitigations: vec!["Verify account ownership".to_string()],
            });

        assert_eq!(context.summary, "Transfer $1000 to external account");
        assert!(context.risk_assessment.is_some());
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
    }
}
