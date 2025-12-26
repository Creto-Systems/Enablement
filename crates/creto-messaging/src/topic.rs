//! Topic-based publish/subscribe messaging module.
//!
//! This module provides pub/sub capabilities for agent messaging, allowing:
//! - Topic creation with configurable policies
//! - Subscription management with filtering
//! - Message publishing to multiple subscribers
//! - Access control via topic policies

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoError, CretoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a topic.
pub type TopicId = Uuid;

/// Unique identifier for a subscription.
pub type SubscriptionId = Uuid;

/// Topic access policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopicPolicy {
    /// Anyone can publish and subscribe.
    Open,

    /// Only owner can manage subscriptions.
    Private,

    /// Requires authorization check via creto-authz.
    AuthzRequired,

    /// Explicit allowlist of agents.
    Allowlist,
}

/// Topic retention policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum number of messages to retain.
    pub max_messages: Option<u32>,

    /// Time-to-live for messages in seconds.
    pub ttl_seconds: Option<u64>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_messages: Some(1000),
            ttl_seconds: Some(86400), // 24 hours
        }
    }
}

/// Topic configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    /// Unique topic identifier.
    pub id: TopicId,

    /// Human-readable topic name.
    pub name: String,

    /// Agent that owns this topic.
    pub owner_agent_id: AgentId,

    /// Publish access policy.
    pub publish_policy: TopicPolicy,

    /// Subscribe access policy.
    pub subscribe_policy: TopicPolicy,

    /// Message retention configuration.
    pub retention: RetentionPolicy,

    /// Maximum message size in bytes.
    pub max_message_size: usize,

    /// Maximum number of subscribers.
    pub max_subscribers: Option<u32>,

    /// Allowlist of agents (used with Allowlist policy).
    pub allowed_agents: Vec<AgentId>,

    /// When the topic was created.
    pub created_at: DateTime<Utc>,

    /// When the topic was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Topic {
    /// Create a new topic.
    pub fn new(name: String, owner_agent_id: AgentId) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            name,
            owner_agent_id,
            publish_policy: TopicPolicy::Private,
            subscribe_policy: TopicPolicy::Open,
            retention: RetentionPolicy::default(),
            max_message_size: 1024 * 1024, // 1MB default
            max_subscribers: Some(1000),
            allowed_agents: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if an agent can publish to this topic.
    pub fn can_publish(&self, agent_id: AgentId) -> bool {
        if agent_id == self.owner_agent_id {
            return true;
        }

        match self.publish_policy {
            TopicPolicy::Open => true,
            TopicPolicy::Private => false,
            TopicPolicy::Allowlist => self.allowed_agents.contains(&agent_id),
            TopicPolicy::AuthzRequired => {
                // In production, this would call creto-authz
                // For now, deny by default
                false
            }
        }
    }

    /// Check if an agent can subscribe to this topic.
    pub fn can_subscribe(&self, agent_id: AgentId) -> bool {
        if agent_id == self.owner_agent_id {
            return true;
        }

        match self.subscribe_policy {
            TopicPolicy::Open => true,
            TopicPolicy::Private => false,
            TopicPolicy::Allowlist => self.allowed_agents.contains(&agent_id),
            TopicPolicy::AuthzRequired => {
                // In production, this would call creto-authz
                // For now, deny by default
                false
            }
        }
    }
}

/// Subscription filter for selective message receiving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    /// Filter by message metadata key-value pairs.
    pub metadata: HashMap<String, String>,
}

impl SubscriptionFilter {
    /// Create an empty filter (receives all messages).
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    /// Add a metadata filter.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if a message matches this filter.
    pub fn matches(&self, message_metadata: &HashMap<String, String>) -> bool {
        // Empty filter matches all messages
        if self.metadata.is_empty() {
            return true;
        }

        // All filter criteria must match
        self.metadata.iter().all(|(k, v)| {
            message_metadata.get(k).map(|mv| mv == v).unwrap_or(false)
        })
    }
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Subscription to a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Unique subscription identifier.
    pub id: SubscriptionId,

    /// Topic being subscribed to.
    pub topic_id: TopicId,

    /// Agent that owns this subscription.
    pub subscriber_agent_id: AgentId,

    /// Optional filter for selective receiving.
    pub filter: Option<SubscriptionFilter>,

    /// When the subscription was created.
    pub subscribed_at: DateTime<Utc>,
}

impl Subscription {
    /// Create a new subscription.
    pub fn new(topic_id: TopicId, subscriber_agent_id: AgentId) -> Self {
        Self {
            id: Uuid::new_v4(),
            topic_id,
            subscriber_agent_id,
            filter: None,
            subscribed_at: Utc::now(),
        }
    }

    /// Create a subscription with a filter.
    pub fn with_filter(
        topic_id: TopicId,
        subscriber_agent_id: AgentId,
        filter: SubscriptionFilter,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            topic_id,
            subscriber_agent_id,
            filter: Some(filter),
            subscribed_at: Utc::now(),
        }
    }
}

/// Configuration for creating a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    /// Topic name.
    pub name: String,

    /// Owner agent ID.
    pub owner_agent_id: AgentId,

    /// Publish policy.
    pub publish_policy: TopicPolicy,

    /// Subscribe policy.
    pub subscribe_policy: TopicPolicy,

    /// Retention policy.
    pub retention: RetentionPolicy,

    /// Maximum message size.
    pub max_message_size: usize,

    /// Maximum subscribers.
    pub max_subscribers: Option<u32>,

    /// Allowed agents (for Allowlist policy).
    pub allowed_agents: Vec<AgentId>,
}

impl TopicConfig {
    /// Create a new topic configuration.
    pub fn new(name: String, owner_agent_id: AgentId) -> Self {
        Self {
            name,
            owner_agent_id,
            publish_policy: TopicPolicy::Private,
            subscribe_policy: TopicPolicy::Open,
            retention: RetentionPolicy::default(),
            max_message_size: 1024 * 1024,
            max_subscribers: Some(1000),
            allowed_agents: Vec::new(),
        }
    }
}

/// Message published to a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMessage {
    /// Message ID.
    pub id: Uuid,

    /// Topic ID.
    pub topic_id: TopicId,

    /// Publisher agent ID.
    pub publisher_id: AgentId,

    /// Message payload.
    pub payload: Vec<u8>,

    /// Message metadata for filtering.
    pub metadata: HashMap<String, String>,

    /// When the message was published.
    pub published_at: DateTime<Utc>,
}

/// Manager for topics and subscriptions.
pub struct TopicManager {
    /// All topics by ID.
    topics: HashMap<TopicId, Topic>,

    /// All subscriptions by ID.
    subscriptions: HashMap<SubscriptionId, Subscription>,

    /// Subscriptions indexed by topic.
    topic_subscriptions: HashMap<TopicId, Vec<SubscriptionId>>,

    /// Messages by topic (for retention).
    topic_messages: HashMap<TopicId, Vec<TopicMessage>>,
}

impl TopicManager {
    /// Create a new topic manager.
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            subscriptions: HashMap::new(),
            topic_subscriptions: HashMap::new(),
            topic_messages: HashMap::new(),
        }
    }

    /// Create a new topic.
    pub fn create_topic(&mut self, config: TopicConfig) -> CretoResult<TopicId> {
        let mut topic = Topic::new(config.name, config.owner_agent_id);
        topic.publish_policy = config.publish_policy;
        topic.subscribe_policy = config.subscribe_policy;
        topic.retention = config.retention;
        topic.max_message_size = config.max_message_size;
        topic.max_subscribers = config.max_subscribers;
        topic.allowed_agents = config.allowed_agents;

        let topic_id = topic.id;
        self.topics.insert(topic_id, topic);
        self.topic_subscriptions.insert(topic_id, Vec::new());
        self.topic_messages.insert(topic_id, Vec::new());

        tracing::info!(topic_id = %topic_id, "Topic created");

        Ok(topic_id)
    }

    /// Delete a topic.
    pub fn delete_topic(&mut self, topic_id: TopicId, agent_id: AgentId) -> CretoResult<()> {
        let topic = self.topics.get(&topic_id).ok_or_else(|| {
            CretoError::NotFound(format!("Topic {} not found", topic_id))
        })?;

        // Only owner can delete
        if topic.owner_agent_id != agent_id {
            return Err(CretoError::Unauthorized(
                "Only topic owner can delete topic".to_string(),
            ));
        }

        // Remove all subscriptions
        if let Some(sub_ids) = self.topic_subscriptions.remove(&topic_id) {
            for sub_id in sub_ids {
                self.subscriptions.remove(&sub_id);
            }
        }

        // Remove messages
        self.topic_messages.remove(&topic_id);

        // Remove topic
        self.topics.remove(&topic_id);

        tracing::info!(topic_id = %topic_id, "Topic deleted");

        Ok(())
    }

    /// Subscribe to a topic.
    pub fn subscribe(
        &mut self,
        topic_id: TopicId,
        subscriber_agent_id: AgentId,
        filter: Option<SubscriptionFilter>,
    ) -> CretoResult<Subscription> {
        let topic = self.topics.get(&topic_id).ok_or_else(|| {
            CretoError::NotFound(format!("Topic {} not found", topic_id))
        })?;

        // Check subscription permission
        if !topic.can_subscribe(subscriber_agent_id) {
            return Err(CretoError::Unauthorized(
                "Not authorized to subscribe to this topic".to_string(),
            ));
        }

        // Check max subscribers
        if let Some(max) = topic.max_subscribers {
            let current_count = self.topic_subscriptions
                .get(&topic_id)
                .map(|s| s.len())
                .unwrap_or(0);

            if current_count >= max as usize {
                return Err(CretoError::LimitExceeded(
                    format!("Topic has reached maximum of {} subscribers", max),
                ));
            }
        }

        let subscription = match filter {
            Some(f) => Subscription::with_filter(topic_id, subscriber_agent_id, f),
            None => Subscription::new(topic_id, subscriber_agent_id),
        };

        let sub_id = subscription.id;

        self.subscriptions.insert(sub_id, subscription.clone());
        self.topic_subscriptions
            .entry(topic_id)
            .or_insert_with(Vec::new)
            .push(sub_id);

        tracing::info!(
            topic_id = %topic_id,
            subscriber = %subscriber_agent_id,
            subscription_id = %sub_id,
            "Subscription created"
        );

        Ok(subscription)
    }

    /// Unsubscribe from a topic.
    pub fn unsubscribe(
        &mut self,
        subscription_id: SubscriptionId,
        agent_id: AgentId,
    ) -> CretoResult<()> {
        let subscription = self.subscriptions.get(&subscription_id).ok_or_else(|| {
            CretoError::NotFound(format!("Subscription {} not found", subscription_id))
        })?;

        // Only subscriber can unsubscribe
        if subscription.subscriber_agent_id != agent_id {
            return Err(CretoError::Unauthorized(
                "Only subscriber can unsubscribe".to_string(),
            ));
        }

        let topic_id = subscription.topic_id;

        // Remove from topic subscriptions
        if let Some(subs) = self.topic_subscriptions.get_mut(&topic_id) {
            subs.retain(|&id| id != subscription_id);
        }

        // Remove subscription
        self.subscriptions.remove(&subscription_id);

        tracing::info!(
            topic_id = %topic_id,
            subscription_id = %subscription_id,
            "Subscription removed"
        );

        Ok(())
    }

    /// Publish a message to a topic.
    pub fn publish(
        &mut self,
        topic_id: TopicId,
        publisher_id: AgentId,
        payload: &[u8],
        metadata: HashMap<String, String>,
    ) -> CretoResult<Vec<AgentId>> {
        let topic = self.topics.get(&topic_id).ok_or_else(|| {
            CretoError::NotFound(format!("Topic {} not found", topic_id))
        })?;

        // Check publish permission
        if !topic.can_publish(publisher_id) {
            return Err(CretoError::Unauthorized(
                "Not authorized to publish to this topic".to_string(),
            ));
        }

        // Check message size
        if payload.len() > topic.max_message_size {
            return Err(CretoError::ValidationFailed(format!(
                "Message size {} exceeds maximum of {}",
                payload.len(),
                topic.max_message_size
            )));
        }

        // Create message
        let message = TopicMessage {
            id: Uuid::new_v4(),
            topic_id,
            publisher_id,
            payload: payload.to_vec(),
            metadata: metadata.clone(),
            published_at: Utc::now(),
        };

        // Store message (apply retention)
        let messages = self.topic_messages.entry(topic_id).or_insert_with(Vec::new);
        messages.push(message);

        // Apply retention policy
        if let Some(max_messages) = topic.retention.max_messages {
            if messages.len() > max_messages as usize {
                messages.drain(0..messages.len() - max_messages as usize);
            }
        }

        // Get matching subscribers
        let subscriber_ids: Vec<AgentId> = self.topic_subscriptions
            .get(&topic_id)
            .map(|subs| {
                subs.iter()
                    .filter_map(|&sub_id| {
                        self.subscriptions.get(&sub_id).and_then(|sub| {
                            // Check if message matches filter
                            match &sub.filter {
                                Some(filter) if !filter.matches(&metadata) => None,
                                _ => Some(sub.subscriber_agent_id),
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        tracing::info!(
            topic_id = %topic_id,
            publisher = %publisher_id,
            subscribers = subscriber_ids.len(),
            "Message published"
        );

        Ok(subscriber_ids)
    }

    /// List all subscribers to a topic.
    pub fn list_subscribers(&self, topic_id: TopicId) -> CretoResult<Vec<Subscription>> {
        if !self.topics.contains_key(&topic_id) {
            return Err(CretoError::NotFound(format!(
                "Topic {} not found",
                topic_id
            )));
        }

        let subscriptions = self.topic_subscriptions
            .get(&topic_id)
            .map(|subs| {
                subs.iter()
                    .filter_map(|&sub_id| self.subscriptions.get(&sub_id).cloned())
                    .collect()
            })
            .unwrap_or_default();

        Ok(subscriptions)
    }

    /// Get topic by ID.
    pub fn get_topic(&self, topic_id: TopicId) -> Option<&Topic> {
        self.topics.get(&topic_id)
    }

    /// List all topics.
    pub fn list_topics(&self) -> Vec<&Topic> {
        self.topics.values().collect()
    }
}

impl Default for TopicManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent() -> AgentId {
        AgentId::new()
    }

    #[test]
    fn test_topic_creation() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();

        let config = TopicConfig::new("test-topic".to_string(), owner);
        let topic_id = manager.create_topic(config).unwrap();

        let topic = manager.get_topic(topic_id).unwrap();
        assert_eq!(topic.name, "test-topic");
        assert_eq!(topic.owner_agent_id, owner);
        assert_eq!(topic.publish_policy, TopicPolicy::Private);
        assert_eq!(topic.subscribe_policy, TopicPolicy::Open);
    }

    #[test]
    fn test_subscription_management() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();
        let subscriber = create_test_agent();

        // Create open topic
        let mut config = TopicConfig::new("open-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Open;
        let topic_id = manager.create_topic(config).unwrap();

        // Subscribe
        let subscription = manager.subscribe(topic_id, subscriber, None).unwrap();
        assert_eq!(subscription.topic_id, topic_id);
        assert_eq!(subscription.subscriber_agent_id, subscriber);

        // List subscribers
        let subscribers = manager.list_subscribers(topic_id).unwrap();
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0].subscriber_agent_id, subscriber);

        // Unsubscribe
        manager.unsubscribe(subscription.id, subscriber).unwrap();
        let subscribers = manager.list_subscribers(topic_id).unwrap();
        assert_eq!(subscribers.len(), 0);
    }

    #[test]
    fn test_open_vs_private_policy() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();
        let other = create_test_agent();

        // Create private topic (private subscribe policy)
        let mut config = TopicConfig::new("private-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Private;
        let private_topic_id = manager.create_topic(config).unwrap();

        // Non-owner cannot subscribe to private topic
        let result = manager.subscribe(private_topic_id, other, None);
        assert!(result.is_err());

        // Owner can subscribe to their own private topic
        let owner_sub = manager.subscribe(private_topic_id, owner, None).unwrap();
        assert_eq!(owner_sub.subscriber_agent_id, owner);

        // Create open topic
        let mut open_config = TopicConfig::new("open-topic".to_string(), owner);
        open_config.subscribe_policy = TopicPolicy::Open;
        open_config.publish_policy = TopicPolicy::Open;
        let open_topic_id = manager.create_topic(open_config).unwrap();

        // Anyone can subscribe to open topic
        let subscription = manager.subscribe(open_topic_id, other, None).unwrap();
        assert_eq!(subscription.subscriber_agent_id, other);

        // Anyone can publish to open topic
        let subscribers = manager
            .publish(open_topic_id, other, b"test message", HashMap::new())
            .unwrap();
        assert_eq!(subscribers.len(), 1);
    }

    #[test]
    fn test_publishing_to_topic() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();
        let sub1 = create_test_agent();
        let sub2 = create_test_agent();

        // Create topic
        let mut config = TopicConfig::new("pub-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Open;
        let topic_id = manager.create_topic(config).unwrap();

        // Add subscribers
        manager.subscribe(topic_id, sub1, None).unwrap();
        manager.subscribe(topic_id, sub2, None).unwrap();

        // Owner publishes (private publish policy, only owner can publish)
        let subscribers = manager
            .publish(topic_id, owner, b"hello", HashMap::new())
            .unwrap();

        assert_eq!(subscribers.len(), 2);
        assert!(subscribers.contains(&sub1));
        assert!(subscribers.contains(&sub2));
    }

    #[test]
    fn test_subscriber_filtering() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();
        let sub1 = create_test_agent();
        let sub2 = create_test_agent();

        // Create topic
        let mut config = TopicConfig::new("filtered-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Open;
        let topic_id = manager.create_topic(config).unwrap();

        // Subscribe with filter
        let filter = SubscriptionFilter::new()
            .with_metadata("type".to_string(), "alert".to_string());
        manager.subscribe(topic_id, sub1, Some(filter)).unwrap();

        // Subscribe without filter
        manager.subscribe(topic_id, sub2, None).unwrap();

        // Publish with matching metadata
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "alert".to_string());
        let subscribers = manager
            .publish(topic_id, owner, b"alert!", metadata)
            .unwrap();

        // Both should receive
        assert_eq!(subscribers.len(), 2);

        // Publish without matching metadata
        let metadata = HashMap::new();
        let subscribers = manager
            .publish(topic_id, owner, b"info", metadata)
            .unwrap();

        // Only sub2 (no filter) should receive
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], sub2);
    }

    #[test]
    fn test_topic_deletion_with_cleanup() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();
        let subscriber = create_test_agent();

        // Create topic with subscription
        let mut config = TopicConfig::new("temp-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Open;
        let topic_id = manager.create_topic(config).unwrap();

        let sub = manager.subscribe(topic_id, subscriber, None).unwrap();

        // Publish message
        manager
            .publish(topic_id, owner, b"test", HashMap::new())
            .unwrap();

        // Delete topic
        manager.delete_topic(topic_id, owner).unwrap();

        // Topic should be gone
        assert!(manager.get_topic(topic_id).is_none());

        // Subscription should be gone
        assert!(!manager.subscriptions.contains_key(&sub.id));

        // Messages should be gone
        assert!(!manager.topic_messages.contains_key(&topic_id));
    }

    #[test]
    fn test_max_subscribers_limit() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();

        // Create topic with max 2 subscribers
        let mut config = TopicConfig::new("limited-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Open;
        config.max_subscribers = Some(2);
        let topic_id = manager.create_topic(config).unwrap();

        // Add 2 subscribers - should succeed
        let sub1 = create_test_agent();
        let sub2 = create_test_agent();
        manager.subscribe(topic_id, sub1, None).unwrap();
        manager.subscribe(topic_id, sub2, None).unwrap();

        // Try to add 3rd subscriber - should fail
        let sub3 = create_test_agent();
        let result = manager.subscribe(topic_id, sub3, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_size_limit() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();

        // Create topic with small max message size
        let mut config = TopicConfig::new("small-msg-topic".to_string(), owner);
        config.max_message_size = 100;
        let topic_id = manager.create_topic(config).unwrap();

        // Try to publish oversized message
        let large_message = vec![0u8; 200];
        let result = manager.publish(topic_id, owner, &large_message, HashMap::new());
        assert!(result.is_err());

        // Publish normal-sized message
        let small_message = vec![0u8; 50];
        let result = manager.publish(topic_id, owner, &small_message, HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_allowlist_policy() {
        let mut manager = TopicManager::new();
        let owner = create_test_agent();
        let allowed_agent = create_test_agent();
        let denied_agent = create_test_agent();

        // Create topic with allowlist policy
        let mut config = TopicConfig::new("allowlist-topic".to_string(), owner);
        config.subscribe_policy = TopicPolicy::Allowlist;
        config.publish_policy = TopicPolicy::Allowlist;
        config.allowed_agents = vec![allowed_agent];
        let topic_id = manager.create_topic(config).unwrap();

        // Allowed agent can subscribe
        let result = manager.subscribe(topic_id, allowed_agent, None);
        assert!(result.is_ok());

        // Denied agent cannot subscribe
        let result = manager.subscribe(topic_id, denied_agent, None);
        assert!(result.is_err());

        // Allowed agent can publish
        let result = manager.publish(topic_id, allowed_agent, b"test", HashMap::new());
        assert!(result.is_ok());

        // Denied agent cannot publish
        let result = manager.publish(topic_id, denied_agent, b"test", HashMap::new());
        assert!(result.is_err());
    }
}
