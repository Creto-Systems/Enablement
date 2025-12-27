//! Notification channel adapters for routing approval requests.

use async_trait::async_trait;
use creto_common::{CretoError, CretoResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::request::OversightRequest;

/// Trait for notification channels.
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Send a notification about a new oversight request.
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult>;

    /// Send a reminder for a pending request.
    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult>;

    /// Get the channel type.
    fn channel_type(&self) -> ChannelType;
}

/// Type of notification channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Slack workspace.
    Slack,
    /// Email.
    Email,
    /// Microsoft Teams.
    Teams,
    /// SMS/text message.
    Sms,
    /// Webhook callback.
    Webhook,
    /// In-app notification.
    InApp,
}

/// Result of sending a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    /// Whether the notification was sent successfully.
    pub success: bool,
    /// Channel-specific message ID (for tracking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl NotificationResult {
    /// Create a success result.
    pub fn success(message_id: Option<String>) -> Self {
        Self {
            success: true,
            message_id,
            error: None,
        }
    }

    /// Create a failure result.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            message_id: None,
            error: Some(error.into()),
        }
    }
}

/// Configuration for the Slack channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Slack workspace token.
    pub token: String,
    /// Default channel for notifications.
    pub default_channel: String,
    /// Whether to use interactive buttons.
    #[serde(default = "default_true")]
    pub interactive_buttons: bool,
}

fn default_true() -> bool {
    true
}

/// Slack message payload with Block Kit formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    /// Target channel ID or name.
    pub channel: String,
    /// Text fallback for notifications.
    pub text: String,
    /// Rich Block Kit blocks for interactive UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<serde_json::Value>>,
    /// Message attachments (legacy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<serde_json::Value>>,
}

/// Slack button callback payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackCallback {
    /// Type of interaction.
    #[serde(rename = "type")]
    pub interaction_type: String,
    /// User who clicked the button.
    pub user: SlackUser,
    /// Action data.
    pub actions: Vec<SlackAction>,
    /// Original message context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<serde_json::Value>,
    /// Response URL for acknowledgment.
    pub response_url: String,
}

/// Slack user info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackUser {
    /// User ID.
    pub id: String,
    /// Username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Slack action from button click.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAction {
    /// Action ID.
    pub action_id: String,
    /// Block ID containing the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    /// Value associated with the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Parsed approval decision from Slack callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalDecision {
    /// Request approved.
    Approved,
    /// Request rejected.
    Rejected,
}

/// Slack notification channel (stub implementation).
///
/// Enable the `channels` feature for full HTTP client functionality.
pub struct SlackChannel {
    config: SlackConfig,
}

impl SlackChannel {
    /// Create a new Slack channel.
    pub fn new(config: SlackConfig) -> Self {
        Self { config }
    }

    /// Build a Slack message with Block Kit interactive buttons.
    pub fn build_approval_message(&self, request: &OversightRequest) -> SlackMessage {
        let channel = self.config.default_channel.clone();
        let request_id = request.id.to_string();
        let agent_id = request.agent_id.to_string();
        let description = &request.description;

        // Extract context from the request's context field
        let context_str = if request.context.is_null() {
            "N/A".to_string()
        } else {
            request.context.to_string()
        };

        let text = format!("Approval Required: {} by {}", description, agent_id);

        let blocks = if self.config.interactive_buttons {
            Some(vec![
                // Header section
                json!({
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": "🔔 Approval Request",
                        "emoji": true
                    }
                }),
                // Request details
                json!({
                    "type": "section",
                    "fields": [
                        {
                            "type": "mrkdwn",
                            "text": format!("*Request ID:*\n{}", request_id)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Agent:*\n{}", agent_id)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Description:*\n{}", description)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Context:*\n{}", context_str)
                        }
                    ]
                }),
                // Divider
                json!({
                    "type": "divider"
                }),
                // Action buttons
                json!({
                    "type": "actions",
                    "elements": [
                        {
                            "type": "button",
                            "text": {
                                "type": "plain_text",
                                "text": "✅ Approve",
                                "emoji": true
                            },
                            "style": "primary",
                            "action_id": format!("approve_{}", request_id),
                            "value": request_id.clone()
                        },
                        {
                            "type": "button",
                            "text": {
                                "type": "plain_text",
                                "text": "❌ Reject",
                                "emoji": true
                            },
                            "style": "danger",
                            "action_id": format!("reject_{}", request_id),
                            "value": request_id
                        }
                    ]
                }),
            ])
        } else {
            None
        };

        SlackMessage {
            channel,
            text,
            blocks,
            attachments: None,
        }
    }

    /// Parse a Slack callback payload to extract approval decision.
    pub fn parse_callback(&self, payload: &str) -> CretoResult<(String, ApprovalDecision, String)> {
        let callback: SlackCallback = serde_json::from_str(payload).map_err(|e| {
            CretoError::SerializationError(format!("Invalid Slack callback: {}", e))
        })?;

        // Extract the first action
        let action = callback
            .actions
            .first()
            .ok_or_else(|| CretoError::Internal("No actions in callback".to_string()))?;

        // Parse action_id to get request_id and decision
        // Expected format: "approve_<uuid>" or "reject_<uuid>"
        let parts: Vec<&str> = action.action_id.splitn(2, '_').collect();
        if parts.len() < 2 {
            return Err(CretoError::Internal(format!(
                "Invalid action_id format: {}",
                action.action_id
            )));
        }

        let decision = match parts[0] {
            "approve" => ApprovalDecision::Approved,
            "reject" => ApprovalDecision::Rejected,
            other => {
                return Err(CretoError::Internal(format!(
                    "Unknown action type: {}",
                    other
                )))
            }
        };

        let request_id = parts[1].to_string();

        Ok((request_id, decision, callback.user.id))
    }

    /// Send notification (stub - logs and returns success).
    pub async fn send_approval_request(
        &self,
        request: &OversightRequest,
    ) -> CretoResult<NotificationResult> {
        let message = self.build_approval_message(request);

        // Stub implementation - in production, use reqwest with `channels` feature
        tracing::info!(
            channel = message.channel,
            request_id = %request.id,
            "Simulated Slack notification"
        );

        let message_id = format!("slack_{}_{}", request.id, chrono::Utc::now().timestamp());
        Ok(NotificationResult::success(Some(message_id)))
    }
}

#[async_trait]
impl NotificationChannel for SlackChannel {
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        self.send_approval_request(request).await
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        let request_id = request.id.to_string();
        let _agent_id = request.agent_id.to_string();

        tracing::info!(
            channel = self.config.default_channel,
            request_id = %request_id,
            "Simulated Slack reminder"
        );

        Ok(NotificationResult::success(None))
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Slack
    }
}

/// Configuration for the email channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server host.
    pub smtp_host: String,
    /// SMTP port.
    pub smtp_port: u16,
    /// Sender email address.
    pub from_address: String,
    /// Reply-to address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    /// Base URL for approval dashboard.
    pub dashboard_base_url: String,
    /// HMAC secret for token generation.
    pub token_secret: String,
}

/// Email approval token for secure verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalToken {
    /// Request ID.
    pub request_id: String,
    /// Approver email address.
    pub approver_email: String,
    /// Expiration timestamp (Unix seconds).
    pub expires_at: i64,
    /// HMAC signature.
    pub signature: String,
}

impl ApprovalToken {
    /// Generate a new approval token with HMAC signature.
    pub fn generate(
        request_id: impl Into<String>,
        approver_email: impl Into<String>,
        ttl_seconds: i64,
        secret: &str,
    ) -> String {
        let request_id = request_id.into();
        let approver_email = approver_email.into();
        let expires_at = chrono::Utc::now().timestamp() + ttl_seconds;

        // Create simple hash signature (in production, use proper HMAC)
        let payload = format!("{}:{}:{}", request_id, approver_email, expires_at);
        let signature = format!("{:x}", md5_hash(&format!("{}:{}", secret, payload)));

        let token = ApprovalToken {
            request_id,
            approver_email,
            expires_at,
            signature,
        };

        // Encode as base64
        let json = serde_json::to_string(&token).unwrap_or_default();
        base64_encode(&json)
    }

    /// Verify a token's signature and expiration.
    pub fn verify(token_str: &str, secret: &str) -> CretoResult<ApprovalToken> {
        // Decode from base64
        let json = base64_decode(token_str)
            .map_err(|e| CretoError::Internal(format!("Invalid token encoding: {}", e)))?;

        let token: ApprovalToken = serde_json::from_slice(&json)
            .map_err(|e| CretoError::SerializationError(format!("Invalid token format: {}", e)))?;

        // Check expiration
        let now = chrono::Utc::now().timestamp();
        if token.expires_at < now {
            return Err(CretoError::ApprovalTimeout { seconds: 0 });
        }

        // Verify signature
        let payload = format!(
            "{}:{}:{}",
            token.request_id, token.approver_email, token.expires_at
        );
        let expected_signature = format!("{:x}", md5_hash(&format!("{}:{}", secret, payload)));

        if token.signature != expected_signature {
            return Err(CretoError::UnauthorizedApprover(
                "Invalid token signature".to_string(),
            ));
        }

        Ok(token)
    }
}

// Simple hash function (for development - use proper crypto in production)
fn md5_hash(input: &str) -> u128 {
    let mut hash: u128 = 0;
    for (i, byte) in input.bytes().enumerate() {
        hash = hash.wrapping_add((byte as u128).wrapping_mul((i as u128).wrapping_add(1)));
        hash = hash.wrapping_mul(31);
    }
    hash
}

// Simple base64 encoding (for development)
fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).map(|&b| b as u32).unwrap_or(0);
        let b2 = chunk.get(2).map(|&b| b as u32).unwrap_or(0);

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[(n >> 18) as usize & 0x3F] as char);
        result.push(ALPHABET[(n >> 12) as usize & 0x3F] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[(n >> 6) as usize & 0x3F] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[n as usize & 0x3F] as char);
        } else {
            result.push('=');
        }
    }

    result
}

// Simple base64 decoding (for development)
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn char_to_val(c: char) -> Result<u8, String> {
        ALPHABET
            .iter()
            .position(|&b| b == c as u8)
            .map(|p| p as u8)
            .ok_or_else(|| format!("Invalid base64 character: {}", c))
    }

    let input = input.trim_end_matches('=');
    let mut result = Vec::new();
    let chars: Vec<char> = input.chars().collect();

    for chunk in chars.chunks(4) {
        if chunk.is_empty() {
            break;
        }

        let v0 = char_to_val(chunk[0])? as u32;
        let v1 = chunk
            .get(1)
            .map(|&c| char_to_val(c))
            .transpose()?
            .unwrap_or(0) as u32;
        let v2 = chunk
            .get(2)
            .map(|&c| char_to_val(c))
            .transpose()?
            .unwrap_or(0) as u32;
        let v3 = chunk
            .get(3)
            .map(|&c| char_to_val(c))
            .transpose()?
            .unwrap_or(0) as u32;

        let n = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;

        result.push((n >> 16) as u8);
        if chunk.len() > 2 {
            result.push((n >> 8) as u8);
        }
        if chunk.len() > 3 {
            result.push(n as u8);
        }
    }

    Ok(result)
}

/// Email notification channel (stub implementation).
pub struct EmailChannel {
    config: EmailConfig,
}

impl EmailChannel {
    /// Create a new email channel.
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Generate approval URL with secure token.
    pub fn generate_approval_url(&self, request_id: &str, approver_email: &str) -> String {
        let token = ApprovalToken::generate(
            request_id,
            approver_email,
            86400, // 24 hours
            &self.config.token_secret,
        );

        format!(
            "{}/approval?token={}",
            self.config.dashboard_base_url.trim_end_matches('/'),
            urlencoding_encode(&token)
        )
    }

    /// Build HTML email template for approval request.
    fn build_email_template(
        &self,
        request: &OversightRequest,
        approver_email: &str,
    ) -> (String, String, String) {
        let request_id = request.id.to_string();
        let agent_id = request.agent_id.to_string();
        let description = &request.description;
        let context_str = if request.context.is_null() {
            "N/A".to_string()
        } else {
            request.context.to_string()
        };

        let approval_url = self.generate_approval_url(&request_id, approver_email);

        let subject = format!("Approval Required: {} by {}", description, agent_id);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #667eea; color: white; padding: 20px; border-radius: 8px 8px 0 0; text-align: center; }}
        .content {{ background: #f9fafb; padding: 20px; border-radius: 0 0 8px 8px; }}
        .details {{ background: white; padding: 15px; border-radius: 8px; margin: 15px 0; border-left: 4px solid #667eea; }}
        .button {{ display: inline-block; background: #667eea; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 15px 0; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🔔 Approval Request</h1>
    </div>
    <div class="content">
        <p>An agent operation requires your approval:</p>
        <div class="details">
            <p><strong>Request ID:</strong> {}</p>
            <p><strong>Agent:</strong> {}</p>
            <p><strong>Description:</strong> {}</p>
            <p><strong>Context:</strong> {}</p>
        </div>
        <a href="{}" class="button">Review & Approve/Reject</a>
        <p style="color: #6b7280; font-size: 14px;">This link expires in 24 hours.</p>
    </div>
</body>
</html>"#,
            request_id, agent_id, description, context_str, approval_url
        );

        let text_body = format!(
            r#"Approval Request

Request ID: {}
Agent: {}
Description: {}
Context: {}

Click here to review: {}

This link expires in 24 hours."#,
            request_id, agent_id, description, context_str, approval_url
        );

        (subject, html_body, text_body)
    }
}

// Simple URL encoding
fn urlencoding_encode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[async_trait]
impl NotificationChannel for EmailChannel {
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        // Extract approver email from metadata or use default
        let approver_email = request
            .metadata
            .get("approver_email")
            .and_then(|v| v.as_str())
            .unwrap_or("approver@example.com");

        let (subject, _html_body, _text_body) = self.build_email_template(request, approver_email);

        // Stub implementation - log instead of sending
        tracing::info!(
            to = approver_email,
            subject = subject,
            smtp_host = self.config.smtp_host,
            "Simulated email notification"
        );

        let message_id = format!("email_{}_{}", request.id, chrono::Utc::now().timestamp());
        Ok(NotificationResult::success(Some(message_id)))
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        let approver_email = request
            .metadata
            .get("approver_email")
            .and_then(|v| v.as_str())
            .unwrap_or("approver@example.com");

        tracing::info!(
            to = approver_email,
            request_id = %request.id,
            "Simulated email reminder"
        );

        Ok(NotificationResult::success(None))
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Email
    }
}

/// Webhook notification channel.
pub struct WebhookChannel {
    /// Webhook URL.
    url: String,
    /// Optional authentication header.
    auth_header: Option<String>,
}

impl WebhookChannel {
    /// Create a new webhook channel.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            auth_header: None,
        }
    }

    /// Set authentication header.
    pub fn with_auth(mut self, auth: impl Into<String>) -> Self {
        self.auth_header = Some(auth.into());
        self
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        tracing::info!(
            url = self.url,
            request_id = %request.id,
            "Simulated webhook notification"
        );
        Ok(NotificationResult::success(None))
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        tracing::info!(
            url = self.url,
            request_id = %request.id,
            "Simulated webhook reminder"
        );
        Ok(NotificationResult::success(None))
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Webhook
    }
}

/// Mock notification channel for testing.
pub struct MockChannel {
    /// Stored notifications for verification.
    notifications: Arc<RwLock<Vec<OversightRequest>>>,
    /// Stored reminders.
    reminders: Arc<RwLock<Vec<OversightRequest>>>,
    /// Whether to simulate failure.
    should_fail: Arc<RwLock<bool>>,
    /// Custom failure message.
    failure_message: Arc<RwLock<Option<String>>>,
}

impl MockChannel {
    /// Create a new mock channel.
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(RwLock::new(Vec::new())),
            reminders: Arc::new(RwLock::new(Vec::new())),
            should_fail: Arc::new(RwLock::new(false)),
            failure_message: Arc::new(RwLock::new(None)),
        }
    }

    /// Configure the mock to fail on next operation.
    pub async fn set_should_fail(&self, fail: bool) {
        *self.should_fail.write().await = fail;
    }

    /// Set custom failure message.
    pub async fn set_failure_message(&self, message: impl Into<String>) {
        *self.failure_message.write().await = Some(message.into());
    }

    /// Get all stored notifications.
    pub async fn get_notifications(&self) -> Vec<OversightRequest> {
        self.notifications.read().await.clone()
    }

    /// Get all stored reminders.
    pub async fn get_reminders(&self) -> Vec<OversightRequest> {
        self.reminders.read().await.clone()
    }

    /// Clear all stored notifications and reminders.
    pub async fn clear(&self) {
        self.notifications.write().await.clear();
        self.reminders.write().await.clear();
        *self.should_fail.write().await = false;
        *self.failure_message.write().await = None;
    }

    /// Get count of notifications sent.
    pub async fn notification_count(&self) -> usize {
        self.notifications.read().await.len()
    }

    /// Get count of reminders sent.
    pub async fn reminder_count(&self) -> usize {
        self.reminders.read().await.len()
    }

    /// Verify a specific notification was sent.
    pub async fn verify_notification(&self, request_id: &str) -> bool {
        self.notifications
            .read()
            .await
            .iter()
            .any(|r| r.id.to_string() == request_id)
    }
}

impl Default for MockChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationChannel for MockChannel {
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        // Check if should fail
        if *self.should_fail.read().await {
            let message = self
                .failure_message
                .read()
                .await
                .clone()
                .unwrap_or_else(|| "Mock channel failure".to_string());
            return Ok(NotificationResult::failure(message));
        }

        // Store notification
        self.notifications.write().await.push(request.clone());

        // Return success with mock message ID
        let message_id = format!("mock_msg_{}", uuid::Uuid::new_v4());
        Ok(NotificationResult::success(Some(message_id)))
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        // Check if should fail
        if *self.should_fail.read().await {
            let message = self
                .failure_message
                .read()
                .await
                .clone()
                .unwrap_or_else(|| "Mock channel failure".to_string());
            return Ok(NotificationResult::failure(message));
        }

        // Store reminder
        self.reminders.write().await.push(request.clone());

        // Return success
        Ok(NotificationResult::success(None))
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::InApp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::ActionType;
    use creto_common::{AgentId, OrganizationId};

    fn create_test_request() -> OversightRequest {
        OversightRequest::new(
            OrganizationId::new(),
            AgentId::new(),
            ActionType::Custom {
                type_id: "test_op".to_string(),
            },
            "Test operation",
        )
    }

    #[test]
    fn test_notification_result_success() {
        let result = NotificationResult::success(Some("msg_123".to_string()));
        assert!(result.success);
        assert_eq!(result.message_id, Some("msg_123".to_string()));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_notification_result_failure() {
        let result = NotificationResult::failure("Connection failed");
        assert!(!result.success);
        assert!(result.message_id.is_none());
        assert_eq!(result.error, Some("Connection failed".to_string()));
    }

    #[tokio::test]
    async fn test_mock_channel_notify() {
        let mock = MockChannel::new();
        let request = create_test_request();
        let request_id = request.id.to_string();

        let result = mock.notify(&request).await.unwrap();
        assert!(result.success);
        assert!(result.message_id.is_some());

        let notifications = mock.get_notifications().await;
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].id.to_string(), request_id);
    }

    #[tokio::test]
    async fn test_mock_channel_remind() {
        let mock = MockChannel::new();
        let request = create_test_request();
        let request_id = request.id.to_string();

        let result = mock.remind(&request).await.unwrap();
        assert!(result.success);

        let reminders = mock.get_reminders().await;
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].id.to_string(), request_id);
    }

    #[tokio::test]
    async fn test_mock_channel_failure() {
        let mock = MockChannel::new();
        mock.set_should_fail(true).await;
        mock.set_failure_message("Test failure").await;

        let request = create_test_request();

        let result = mock.notify(&request).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.error, Some("Test failure".to_string()));

        let notifications = mock.get_notifications().await;
        assert_eq!(notifications.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_channel_clear() {
        let mock = MockChannel::new();
        let request = create_test_request();

        mock.notify(&request).await.unwrap();
        mock.remind(&request).await.unwrap();

        assert_eq!(mock.notification_count().await, 1);
        assert_eq!(mock.reminder_count().await, 1);

        mock.clear().await;

        assert_eq!(mock.notification_count().await, 0);
        assert_eq!(mock.reminder_count().await, 0);
    }

    #[test]
    fn test_approval_token_generate_and_verify() {
        let secret = "test_secret_key_12345";
        let token_str = ApprovalToken::generate("req_123", "user@example.com", 3600, secret);

        assert!(!token_str.is_empty());

        let verified = ApprovalToken::verify(&token_str, secret).unwrap();
        assert_eq!(verified.request_id, "req_123");
        assert_eq!(verified.approver_email, "user@example.com");
    }

    #[test]
    fn test_approval_token_invalid_signature() {
        let secret = "test_secret";
        let wrong_secret = "wrong_secret";

        let token_str = ApprovalToken::generate("req_456", "user@example.com", 3600, secret);

        let result = ApprovalToken::verify(&token_str, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_approval_token_expired() {
        let secret = "test_secret";
        // Create token that expires immediately
        let token_str = ApprovalToken::generate("req_789", "user@example.com", -1, secret);

        // Wait a moment to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = ApprovalToken::verify(&token_str, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_slack_callback_parse_approve() {
        let slack_channel = SlackChannel::new(SlackConfig {
            token: "xoxb-test".to_string(),
            default_channel: "#test".to_string(),
            interactive_buttons: true,
        });

        let payload = r#"{
            "type": "block_actions",
            "user": {"id": "U123456", "username": "john"},
            "actions": [{"action_id": "approve_req_123", "value": "req_123"}],
            "response_url": "https://hooks.slack.com/actions/test"
        }"#;

        let (request_id, decision, user_id) = slack_channel.parse_callback(payload).unwrap();
        assert_eq!(request_id, "req_123");
        assert_eq!(decision, ApprovalDecision::Approved);
        assert_eq!(user_id, "U123456");
    }

    #[test]
    fn test_slack_callback_parse_reject() {
        let slack_channel = SlackChannel::new(SlackConfig {
            token: "xoxb-test".to_string(),
            default_channel: "#test".to_string(),
            interactive_buttons: true,
        });

        let payload = r#"{
            "type": "block_actions",
            "user": {"id": "U789", "username": "jane"},
            "actions": [{"action_id": "reject_some-uuid-here", "value": "some-uuid-here"}],
            "response_url": "https://hooks.slack.com/actions/test"
        }"#;

        let (request_id, decision, user_id) = slack_channel.parse_callback(payload).unwrap();
        assert_eq!(request_id, "some-uuid-here");
        assert_eq!(decision, ApprovalDecision::Rejected);
        assert_eq!(user_id, "U789");
    }

    #[test]
    fn test_email_channel_generate_approval_url() {
        let email_channel = EmailChannel::new(EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            from_address: "noreply@example.com".to_string(),
            reply_to: None,
            dashboard_base_url: "https://approval.example.com".to_string(),
            token_secret: "secret123".to_string(),
        });

        let url = email_channel.generate_approval_url("req_123", "approver@example.com");
        assert!(url.starts_with("https://approval.example.com/approval?token="));
    }

    #[test]
    fn test_slack_message_building() {
        let slack_channel = SlackChannel::new(SlackConfig {
            token: "xoxb-test".to_string(),
            default_channel: "#approvals".to_string(),
            interactive_buttons: true,
        });

        let request = create_test_request();
        let message = slack_channel.build_approval_message(&request);

        assert_eq!(message.channel, "#approvals");
        assert!(message.text.contains("Test operation"));
        assert!(message.blocks.is_some());

        let blocks = message.blocks.unwrap();
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = "Hello, World! This is a test.";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        let result = String::from_utf8(decoded).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_urlencoding() {
        let input = "hello world=test&foo";
        let encoded = urlencoding_encode(input);
        assert!(encoded.contains("%20"));
        assert!(encoded.contains("%3D"));
        assert!(encoded.contains("%26"));
    }
}
