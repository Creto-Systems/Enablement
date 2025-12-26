//! Notification channel adapters for routing approval requests.

use async_trait::async_trait;
use creto_common::CretoResult;
use serde::{Deserialize, Serialize};

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

/// Slack notification channel.
pub struct SlackChannel {
    config: SlackConfig,
}

impl SlackChannel {
    /// Create a new Slack channel.
    pub fn new(config: SlackConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NotificationChannel for SlackChannel {
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        // TODO: Implement actual Slack API integration
        // 1. Format message with request details
        // 2. Add approve/reject buttons if interactive
        // 3. Send to appropriate channel
        // 4. Return message ID for tracking

        let _ = (&self.config, request);

        Ok(NotificationResult::success(Some("slack_msg_123".to_string())))
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        // TODO: Implement reminder logic

        let _ = request;

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
}

/// Email notification channel.
pub struct EmailChannel {
    config: EmailConfig,
}

impl EmailChannel {
    /// Create a new email channel.
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NotificationChannel for EmailChannel {
    async fn notify(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        // TODO: Implement actual email sending
        // 1. Generate HTML email with request details
        // 2. Include approve/reject links
        // 3. Send via SMTP
        // 4. Return message ID

        let _ = (&self.config, request);

        Ok(NotificationResult::success(Some("email_123".to_string())))
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        let _ = request;
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
        // TODO: Implement HTTP POST to webhook
        // 1. Serialize request to JSON
        // 2. POST to webhook URL
        // 3. Handle response

        let _ = (&self.url, &self.auth_header, request);

        Ok(NotificationResult::success(None))
    }

    async fn remind(&self, request: &OversightRequest) -> CretoResult<NotificationResult> {
        let _ = request;
        Ok(NotificationResult::success(None))
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Webhook
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
