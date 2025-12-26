//! Integration tests for creto-messaging.
//!
//! These tests verify the messaging service functionality including
//! envelope creation, key management, and session handling.

use creto_integration_tests::common::{TestFixture, test_agent_id};
use creto_messaging::{
    Envelope, EncryptedPayload, ContentType,
    DeliveryReceipt, ReceiptType,
    IdentityKey, PreKey, KeyBundle, SessionState,
};
use uuid::Uuid;

#[test]
fn test_encrypted_payload() {
    let payload = EncryptedPayload::new(
        b"encrypted message content".to_vec(),
        vec![0u8; 16], // MAC
    );

    assert!(!payload.is_empty());
    assert_eq!(payload.len(), 25); // Length of ciphertext
    assert_eq!(payload.ciphertext, b"encrypted message content".to_vec());
}

#[test]
fn test_content_types() {
    // Verify all content types exist
    let _ = ContentType::Text;
    let _ = ContentType::Json;
    let _ = ContentType::Binary;
    let _ = ContentType::ToolRequest;
    let _ = ContentType::ToolResponse;
    let _ = ContentType::Status;
    let _ = ContentType::Control;

    // Default is Text
    assert_eq!(ContentType::default(), ContentType::Text);
}

#[test]
fn test_delivery_receipt_delivered() {
    let message_id = Uuid::now_v7();
    let receipt = DeliveryReceipt::delivered(message_id);

    assert_eq!(receipt.message_id, message_id);
    assert_eq!(receipt.receipt_type, ReceiptType::Delivered);
}

#[test]
fn test_delivery_receipt_read() {
    let message_id = Uuid::now_v7();
    let receipt = DeliveryReceipt::read(message_id);

    assert_eq!(receipt.message_id, message_id);
    assert_eq!(receipt.receipt_type, ReceiptType::Read);
}

#[test]
fn test_receipt_types() {
    // Verify all receipt types exist
    let _ = ReceiptType::Delivered;
    let _ = ReceiptType::Read;
    let _ = ReceiptType::Failed;
}

#[test]
fn test_identity_key_generation() {
    let fixture = TestFixture::new();
    let key = IdentityKey::generate(fixture.agent_id);

    assert_eq!(key.agent_id, fixture.agent_id);
    assert!(!key.public_key.is_empty());
    assert!(key.has_private_key());
}

#[test]
fn test_identity_key_from_public() {
    let fixture = TestFixture::new();
    let public_key = vec![0u8; 32];
    let key = IdentityKey::from_public(fixture.agent_id, public_key.clone());

    assert_eq!(key.agent_id, fixture.agent_id);
    assert_eq!(key.public_key, public_key);
    assert!(!key.has_private_key());
}

#[test]
fn test_prekey_generation() {
    let prekey = PreKey::generate(1);

    assert_eq!(prekey.id, 1);
    assert!(!prekey.public_key.is_empty());
    assert!(prekey.private_key.is_some());
}

#[test]
fn test_prekey_batch_generation() {
    let keys = PreKey::generate_batch(100, 10);

    assert_eq!(keys.len(), 10);
    assert_eq!(keys[0].id, 100);
    assert_eq!(keys[9].id, 109);
}

#[test]
fn test_session_states() {
    // Verify all session states exist
    let _ = SessionState::Establishing;
    let _ = SessionState::Active;
    let _ = SessionState::Suspended;
    let _ = SessionState::Closed;
    let _ = SessionState::Failed;
}

#[test]
fn test_key_bundle_creation() {
    let fixture = TestFixture::new();
    let bundle = KeyBundle::new(fixture.agent_id);

    assert_eq!(bundle.agent_id, fixture.agent_id);
    assert!(bundle.identity_key.has_private_key());
    assert!(bundle.one_time_pre_key.is_some());
}

#[test]
fn test_key_bundle_public() {
    let fixture = TestFixture::new();
    let bundle = KeyBundle::new(fixture.agent_id);
    let public = bundle.public_bundle();

    // Public bundle should not have private keys
    assert!(!public.identity_key.has_private_key());
    if let Some(pk) = &public.one_time_pre_key {
        assert!(pk.private_key.is_none());
    }
}

#[test]
fn test_envelope_serialization() {
    // Create a minimal envelope for serialization testing
    use creto_messaging::ratchet::MessageHeader;

    let sender = test_agent_id();
    let recipient = test_agent_id();

    let ratchet_header = MessageHeader {
        dh_public: vec![0u8; 32],
        prev_chain_length: 0,
        message_number: 1,
    };

    let envelope = Envelope::new(
        sender,
        recipient,
        ratchet_header,
        vec![1, 2, 3, 4],
    );

    // Test serialization roundtrip
    let bytes = envelope.to_bytes().unwrap();
    let decoded = Envelope::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.id, envelope.id);
    assert_eq!(decoded.payload.ciphertext, envelope.payload.ciphertext);
    assert_eq!(decoded.header.sender_id, sender);
    assert_eq!(decoded.header.recipient_id, recipient);
}

#[test]
fn test_envelope_with_content_type() {
    use creto_messaging::ratchet::MessageHeader;

    let sender = test_agent_id();
    let recipient = test_agent_id();

    let ratchet_header = MessageHeader {
        dh_public: vec![0u8; 32],
        prev_chain_length: 0,
        message_number: 1,
    };

    let envelope = Envelope::new(sender, recipient, ratchet_header, vec![1, 2, 3])
        .with_content_type(ContentType::Json);

    assert_eq!(envelope.header.content_type, ContentType::Json);
}

#[test]
fn test_envelope_with_reply_to() {
    use creto_messaging::ratchet::MessageHeader;

    let sender = test_agent_id();
    let recipient = test_agent_id();
    let reply_to_id = Uuid::now_v7();

    let ratchet_header = MessageHeader {
        dh_public: vec![0u8; 32],
        prev_chain_length: 0,
        message_number: 1,
    };

    let envelope = Envelope::new(sender, recipient, ratchet_header, vec![1, 2, 3])
        .with_reply_to(reply_to_id);

    assert_eq!(envelope.header.reply_to, Some(reply_to_id));
}
