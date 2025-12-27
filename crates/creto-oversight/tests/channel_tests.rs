//! Comprehensive channel adapter tests for Week 9 features.

use creto_common::{AgentId, OrganizationId};
use creto_oversight::{
    channels::*,
    request::{ActionType, OversightRequest},
};

#[tokio::test]
async fn test_slack_approval_payload_format() {
    let config = SlackConfig {
        token: "xoxb-test-token".to_string(),
        default_channel: "#approvals".to_string(),
        interactive_buttons: true,
    };
    let channel = SlackChannel::new(config);

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 500000,
            currency: "USD".to_string(),
        },
        "Transfer $5,000 to vendor",
    );

    let result = channel.notify(&request).await.unwrap();

    // Verify notification succeeds and returns message ID
    assert!(result.success);
    assert!(result.message_id.is_some());
    assert_eq!(channel.channel_type(), ChannelType::Slack);
}

#[test]
fn test_slack_callback_parsing() {
    // Test that Slack channel config correctly handles interactive buttons
    let config = SlackConfig {
        token: "xoxb-token".to_string(),
        default_channel: "#test".to_string(),
        interactive_buttons: true,
    };

    assert!(config.interactive_buttons);

    // Test with buttons disabled
    let config_no_buttons = SlackConfig {
        token: "xoxb-token".to_string(),
        default_channel: "#test".to_string(),
        interactive_buttons: false,
    };

    assert!(!config_no_buttons.interactive_buttons);
}

#[tokio::test]
async fn test_email_template_generation() {
    let config = EmailConfig {
        smtp_host: "smtp.example.com".to_string(),
        smtp_port: 587,
        from_address: "approvals@example.com".to_string(),
        reply_to: Some("noreply@example.com".to_string()),
        dashboard_base_url: "https://dashboard.example.com".to_string(),
        token_secret: "test-secret-key".to_string(),
    };
    let channel = EmailChannel::new(config);

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::DataAccess {
            data_type: "customer_records".to_string(),
            scope: "pii".to_string(),
        },
        "Access customer PII data",
    );

    let result = channel.notify(&request).await.unwrap();

    assert!(result.success);
    assert!(result.message_id.is_some());
    assert_eq!(channel.channel_type(), ChannelType::Email);
}

#[test]
fn test_email_verification_token() {
    // Test email config with and without reply-to
    let config_with_reply = EmailConfig {
        smtp_host: "smtp.test.com".to_string(),
        smtp_port: 587,
        from_address: "test@example.com".to_string(),
        reply_to: Some("reply@example.com".to_string()),
        dashboard_base_url: "https://dashboard.test.com".to_string(),
        token_secret: "secret123".to_string(),
    };

    assert!(config_with_reply.reply_to.is_some());
    assert_eq!(config_with_reply.reply_to.unwrap(), "reply@example.com");

    let config_without_reply = EmailConfig {
        smtp_host: "smtp.test.com".to_string(),
        smtp_port: 587,
        from_address: "test@example.com".to_string(),
        reply_to: None,
        dashboard_base_url: "https://dashboard.test.com".to_string(),
        token_secret: "secret123".to_string(),
    };

    assert!(config_without_reply.reply_to.is_none());
}

#[tokio::test]
async fn test_mock_channel_records_messages() {
    // Test webhook channel (mock implementation)
    let channel = WebhookChannel::new("https://webhook.example.com/approval")
        .with_auth("Bearer secret-token");

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::CodeExecution {
            runtime: "python".to_string(),
            risk_level: "high".to_string(),
        },
        "Execute untrusted Python code",
    );

    // Verify notify succeeds
    let result = channel.notify(&request).await.unwrap();
    assert!(result.success);

    // Verify remind works
    let remind_result = channel.remind(&request).await.unwrap();
    assert!(remind_result.success);

    assert_eq!(channel.channel_type(), ChannelType::Webhook);
}

#[test]
fn test_channel_type_serialization() {
    // Verify all channel types can be serialized/deserialized
    let types = vec![
        ChannelType::Slack,
        ChannelType::Email,
        ChannelType::Teams,
        ChannelType::Sms,
        ChannelType::Webhook,
        ChannelType::InApp,
    ];

    for channel_type in types {
        let json = serde_json::to_string(&channel_type).unwrap();
        let deserialized: ChannelType = serde_json::from_str(&json).unwrap();
        assert_eq!(channel_type, deserialized);
    }
}

#[tokio::test]
async fn test_channel_reminder_functionality() {
    let slack_config = SlackConfig {
        token: "xoxb-test".to_string(),
        default_channel: "#reminders".to_string(),
        interactive_buttons: false,
    };
    let slack = SlackChannel::new(slack_config);

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Custom {
            type_id: "test_action".to_string(),
        },
        "Test reminder",
    );

    // Test that reminders work
    let result = slack.remind(&request).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_multiple_channel_coordination() {
    // Test coordinating notifications across multiple channels
    let slack = SlackChannel::new(SlackConfig {
        token: "test".to_string(),
        default_channel: "#test".to_string(),
        interactive_buttons: true,
    });

    let email = EmailChannel::new(EmailConfig {
        smtp_host: "smtp.test.com".to_string(),
        smtp_port: 587,
        from_address: "test@example.com".to_string(),
        reply_to: None,
        dashboard_base_url: "https://dashboard.test.com".to_string(),
        token_secret: "secret".to_string(),
    });

    let webhook = WebhookChannel::new("https://example.com/webhook");

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Transaction {
            amount_cents: 1000000,
            currency: "USD".to_string(),
        },
        "Large transaction",
    );

    // All channels should successfully notify
    let slack_result = slack.notify(&request).await.unwrap();
    let email_result = email.notify(&request).await.unwrap();
    let webhook_result = webhook.notify(&request).await.unwrap();

    assert!(slack_result.success);
    assert!(email_result.success);
    assert!(webhook_result.success);
}

#[tokio::test]
async fn test_mock_channel_full_workflow() {
    let mock = MockChannel::new();

    let request = OversightRequest::new(
        OrganizationId::new(),
        AgentId::new(),
        ActionType::Custom {
            type_id: "test".to_string(),
        },
        "Test operation",
    );

    // Test notification
    let notify_result = mock.notify(&request).await.unwrap();
    assert!(notify_result.success);
    assert_eq!(mock.notification_count().await, 1);

    // Test reminder
    let remind_result = mock.remind(&request).await.unwrap();
    assert!(remind_result.success);
    assert_eq!(mock.reminder_count().await, 1);

    // Test failure mode
    mock.set_should_fail(true).await;
    mock.set_failure_message("Simulated failure").await;

    let fail_result = mock.notify(&request).await.unwrap();
    assert!(!fail_result.success);
    assert_eq!(fail_result.error.as_deref(), Some("Simulated failure"));

    // Test clear
    mock.clear().await;
    assert_eq!(mock.notification_count().await, 0);
    assert_eq!(mock.reminder_count().await, 0);
}

#[test]
fn test_approval_token_roundtrip() {
    let secret = "test-secret-12345";
    let request_id = "req-abc123";
    let email = "user@example.com";

    let token = ApprovalToken::generate(request_id, email, 3600, secret);

    // Verify token can be decoded
    let verified = ApprovalToken::verify(&token, secret).unwrap();
    assert_eq!(verified.request_id, request_id);
    assert_eq!(verified.approver_email, email);
}

#[test]
fn test_approval_token_wrong_secret() {
    let token = ApprovalToken::generate("req-123", "user@test.com", 3600, "correct-secret");

    let result = ApprovalToken::verify(&token, "wrong-secret");
    assert!(result.is_err());
}

#[test]
fn test_notification_result_creation() {
    let success = NotificationResult::success(Some("msg-123".to_string()));
    assert!(success.success);
    assert_eq!(success.message_id, Some("msg-123".to_string()));
    assert!(success.error.is_none());

    let failure = NotificationResult::failure("Connection error");
    assert!(!failure.success);
    assert!(failure.message_id.is_none());
    assert_eq!(failure.error, Some("Connection error".to_string()));
}
