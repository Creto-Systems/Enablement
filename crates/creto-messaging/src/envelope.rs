//! Message envelope format for encrypted messages.

use chrono::{DateTime, Utc};
use creto_common::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ratchet::MessageHeader;

/// A complete message envelope.
///
/// Contains all information needed to deliver and decrypt a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Unique message ID.
    pub id: Uuid,

    /// Envelope version.
    pub version: u8,

    /// Message header.
    pub header: EnvelopeHeader,

    /// Encrypted payload.
    pub payload: EncryptedPayload,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl Envelope {
    /// Create a new envelope.
    pub fn new(
        sender_id: AgentId,
        recipient_id: AgentId,
        ratchet_header: MessageHeader,
        ciphertext: Vec<u8>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            version: 1,
            header: EnvelopeHeader {
                sender_id,
                recipient_id,
                ratchet_header,
                content_type: ContentType::Text,
                reply_to: None,
            },
            payload: EncryptedPayload {
                ciphertext,
                mac: vec![0u8; 16], // Placeholder
            },
            timestamp: Utc::now(),
        }
    }

    /// Set content type.
    pub fn with_content_type(mut self, content_type: ContentType) -> Self {
        self.header.content_type = content_type;
        self
    }

    /// Set reply-to reference.
    pub fn with_reply_to(mut self, reply_to: Uuid) -> Self {
        self.header.reply_to = Some(reply_to);
        self
    }

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> creto_common::CretoResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| creto_common::CretoError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> creto_common::CretoResult<Self> {
        serde_json::from_slice(bytes).map_err(|e| creto_common::CretoError::SerializationError(e.to_string()))
    }
}

/// Envelope header (sent in clear, needed for routing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeHeader {
    /// Sender agent ID.
    pub sender_id: AgentId,

    /// Recipient agent ID.
    pub recipient_id: AgentId,

    /// Double Ratchet header.
    pub ratchet_header: MessageHeader,

    /// Content type hint.
    pub content_type: ContentType,

    /// Reference to message being replied to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Uuid>,
}

/// Encrypted payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// Ciphertext.
    pub ciphertext: Vec<u8>,

    /// Message authentication code.
    pub mac: Vec<u8>,
}

impl EncryptedPayload {
    /// Create a new payload.
    pub fn new(ciphertext: Vec<u8>, mac: Vec<u8>) -> Self {
        Self { ciphertext, mac }
    }

    /// Get ciphertext length.
    pub fn len(&self) -> usize {
        self.ciphertext.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.ciphertext.is_empty()
    }
}

/// Content type for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ContentType {
    /// Plain text message.
    #[default]
    Text,
    /// JSON structured data.
    Json,
    /// Binary data.
    Binary,
    /// Tool invocation request.
    ToolRequest,
    /// Tool invocation response.
    ToolResponse,
    /// Status update.
    Status,
    /// Control message (session management).
    Control,
}


/// Delivery receipt for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    /// Message ID being acknowledged.
    pub message_id: Uuid,

    /// Receipt type.
    pub receipt_type: ReceiptType,

    /// Timestamp of receipt.
    pub timestamp: DateTime<Utc>,

    /// Signature over the receipt.
    pub signature: Vec<u8>,
}

impl DeliveryReceipt {
    /// Create a delivery receipt.
    pub fn delivered(message_id: Uuid) -> Self {
        Self {
            message_id,
            receipt_type: ReceiptType::Delivered,
            timestamp: Utc::now(),
            signature: Vec::new(), // TODO: Sign
        }
    }

    /// Create a read receipt.
    pub fn read(message_id: Uuid) -> Self {
        Self {
            message_id,
            receipt_type: ReceiptType::Read,
            timestamp: Utc::now(),
            signature: Vec::new(),
        }
    }
}

/// Type of delivery receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptType {
    /// Message delivered to recipient's device.
    Delivered,
    /// Message has been read/processed.
    Read,
    /// Message delivery failed.
    Failed,
}

/// A batch of envelopes for efficient delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeBatch {
    /// Batch ID.
    pub id: Uuid,

    /// Envelopes in this batch.
    pub envelopes: Vec<Envelope>,

    /// Batch timestamp.
    pub timestamp: DateTime<Utc>,
}

impl EnvelopeBatch {
    /// Create a new batch.
    pub fn new(envelopes: Vec<Envelope>) -> Self {
        Self {
            id: Uuid::now_v7(),
            envelopes,
            timestamp: Utc::now(),
        }
    }

    /// Number of envelopes in batch.
    pub fn len(&self) -> usize {
        self.envelopes.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.envelopes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ratchet::MessageHeader;

    #[test]
    fn test_envelope_creation() {
        let sender = AgentId::new();
        let recipient = AgentId::new();

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

        assert_eq!(envelope.header.sender_id, sender);
        assert_eq!(envelope.header.recipient_id, recipient);
        assert_eq!(envelope.payload.ciphertext, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_envelope_serialization() {
        let sender = AgentId::new();
        let recipient = AgentId::new();

        let ratchet_header = MessageHeader {
            dh_public: vec![0u8; 32],
            prev_chain_length: 0,
            message_number: 1,
        };

        let envelope = Envelope::new(sender, recipient, ratchet_header, vec![1, 2, 3]);

        let bytes = envelope.to_bytes().unwrap();
        let decoded = Envelope::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.id, envelope.id);
        assert_eq!(decoded.payload.ciphertext, envelope.payload.ciphertext);
    }

    #[test]
    fn test_delivery_receipt() {
        let message_id = Uuid::now_v7();
        let receipt = DeliveryReceipt::delivered(message_id);

        assert_eq!(receipt.message_id, message_id);
        assert_eq!(receipt.receipt_type, ReceiptType::Delivered);
    }
}
