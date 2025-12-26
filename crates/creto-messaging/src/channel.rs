//! Message delivery channels.

use async_trait::async_trait;
use creto_common::{AgentId, CretoResult};
use serde::{Deserialize, Serialize};

use crate::envelope::{DeliveryReceipt, Envelope, EnvelopeBatch};

/// A message delivery channel.
#[async_trait]
pub trait Channel: Send + Sync {
    /// Get the channel type.
    fn channel_type(&self) -> ChannelType;

    /// Send a single envelope.
    async fn send(&self, envelope: &Envelope) -> CretoResult<DeliveryReceipt>;

    /// Send a batch of envelopes.
    async fn send_batch(&self, batch: &EnvelopeBatch) -> CretoResult<Vec<DeliveryReceipt>>;

    /// Receive pending envelopes for an agent.
    async fn receive(&self, agent_id: AgentId, limit: u32) -> CretoResult<Vec<Envelope>>;

    /// Acknowledge receipt of envelopes.
    async fn acknowledge(&self, envelope_ids: &[uuid::Uuid]) -> CretoResult<()>;

    /// Check if channel is connected.
    async fn is_connected(&self) -> bool;
}

/// Type of delivery channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Direct connection (WebSocket, gRPC stream).
    Direct,
    /// Message queue (Redis, RabbitMQ).
    Queue,
    /// Pub/sub (NATS, Kafka).
    PubSub,
    /// Store-and-forward (database).
    StoreForward,
    /// Webhook delivery.
    Webhook,
}

/// Channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Channel type.
    pub channel_type: ChannelType,

    /// Connection URL.
    pub url: String,

    /// Retry policy.
    #[serde(default)]
    pub retry: RetryPolicy,

    /// Timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Maximum batch size.
    #[serde(default = "default_batch_size")]
    pub max_batch_size: u32,
}

fn default_timeout() -> u64 {
    30_000 // 30 seconds
}

fn default_batch_size() -> u32 {
    100
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            channel_type: ChannelType::Queue,
            url: String::new(),
            retry: RetryPolicy::default(),
            timeout_ms: default_timeout(),
            max_batch_size: default_batch_size(),
        }
    }
}

/// Retry policy for message delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retry attempts.
    pub max_attempts: u32,

    /// Initial backoff in milliseconds.
    pub initial_backoff_ms: u64,

    /// Maximum backoff in milliseconds.
    pub max_backoff_ms: u64,

    /// Backoff multiplier.
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10_000,
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Calculate backoff for a given attempt.
    pub fn backoff_ms(&self, attempt: u32) -> u64 {
        if attempt == 0 {
            return 0;
        }

        let backoff = self.initial_backoff_ms as f64 * self.multiplier.powi(attempt as i32 - 1);
        backoff.min(self.max_backoff_ms as f64) as u64
    }
}

/// In-memory channel for testing.
pub struct InMemoryChannel {
    messages: std::sync::Arc<tokio::sync::RwLock<Vec<Envelope>>>,
}

impl InMemoryChannel {
    /// Create a new in-memory channel.
    pub fn new() -> Self {
        Self {
            messages: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for InMemoryChannel {
    fn channel_type(&self) -> ChannelType {
        ChannelType::StoreForward
    }

    async fn send(&self, envelope: &Envelope) -> CretoResult<DeliveryReceipt> {
        let mut messages = self.messages.write().await;
        messages.push(envelope.clone());
        Ok(DeliveryReceipt::delivered(envelope.id))
    }

    async fn send_batch(&self, batch: &EnvelopeBatch) -> CretoResult<Vec<DeliveryReceipt>> {
        let mut messages = self.messages.write().await;
        let mut receipts = Vec::with_capacity(batch.envelopes.len());

        for envelope in &batch.envelopes {
            messages.push(envelope.clone());
            receipts.push(DeliveryReceipt::delivered(envelope.id));
        }

        Ok(receipts)
    }

    async fn receive(&self, agent_id: AgentId, limit: u32) -> CretoResult<Vec<Envelope>> {
        let messages = self.messages.read().await;
        let received: Vec<Envelope> = messages
            .iter()
            .filter(|e| e.header.recipient_id == agent_id)
            .take(limit as usize)
            .cloned()
            .collect();

        Ok(received)
    }

    async fn acknowledge(&self, envelope_ids: &[uuid::Uuid]) -> CretoResult<()> {
        let mut messages = self.messages.write().await;
        messages.retain(|e| !envelope_ids.contains(&e.id));
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        true
    }
}

/// Channel router for multi-channel delivery.
pub struct ChannelRouter {
    channels: Vec<Box<dyn Channel>>,
    default_channel: usize,
}

impl ChannelRouter {
    /// Create a new channel router.
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            default_channel: 0,
        }
    }

    /// Add a channel.
    pub fn add_channel(&mut self, channel: Box<dyn Channel>) {
        self.channels.push(channel);
    }

    /// Set the default channel index.
    pub fn set_default(&mut self, index: usize) {
        if index < self.channels.len() {
            self.default_channel = index;
        }
    }

    /// Route an envelope to the appropriate channel.
    pub async fn route(&self, envelope: &Envelope) -> CretoResult<DeliveryReceipt> {
        // TODO: Implement routing logic based on agent preferences
        if self.channels.is_empty() {
            return Err(creto_common::CretoError::ChannelError(
                "No channels configured".to_string(),
            ));
        }

        self.channels[self.default_channel].send(envelope).await
    }
}

impl Default for ChannelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ratchet::MessageHeader;

    #[tokio::test]
    async fn test_in_memory_channel() {
        let channel = InMemoryChannel::new();

        let sender = AgentId::new();
        let recipient = AgentId::new();

        let header = MessageHeader {
            dh_public: vec![0u8; 32],
            prev_chain_length: 0,
            message_number: 1,
        };

        let envelope = Envelope::new(sender, recipient, header, vec![1, 2, 3]);

        // Send
        let receipt = channel.send(&envelope).await.unwrap();
        assert_eq!(receipt.message_id, envelope.id);

        // Receive
        let received = channel.receive(recipient, 10).await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, envelope.id);

        // Acknowledge
        channel.acknowledge(&[envelope.id]).await.unwrap();

        // Should be empty now
        let received = channel.receive(recipient, 10).await.unwrap();
        assert!(received.is_empty());
    }

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.backoff_ms(0), 0);
        assert_eq!(policy.backoff_ms(1), 100);
        assert_eq!(policy.backoff_ms(2), 200);
        assert_eq!(policy.backoff_ms(3), 400);
    }
}
