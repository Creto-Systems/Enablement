//! Messaging service facade.

use std::collections::HashMap;
use std::sync::Arc;

use creto_common::{AgentId, CretoResult};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    channel::{Channel, ChannelRouter},
    envelope::{DeliveryReceipt, Envelope},
    keys::{KeyBundle, KeyStore},
    session::{Session, SessionState, SessionStore},
    topic::{Subscription, SubscriptionFilter, TopicConfig, TopicId, TopicManager},
    x3dh::X3DH,
};

/// Main entry point for the messaging system.
pub struct MessagingService {
    /// Key store for managing cryptographic keys.
    key_store: Option<Arc<dyn KeyStore>>,

    /// Session store for managing sessions.
    session_store: Option<Arc<dyn SessionStore>>,

    /// Channel router for message delivery.
    channel_router: Arc<RwLock<ChannelRouter>>,

    /// Active sessions cache.
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,

    /// Local agent's key bundle.
    local_bundle: Option<KeyBundle>,

    /// Topic manager for pub/sub.
    topic_manager: Arc<RwLock<TopicManager>>,
}

impl MessagingService {
    /// Create a new messaging service.
    pub fn new() -> Self {
        Self {
            key_store: None,
            session_store: None,
            channel_router: Arc::new(RwLock::new(ChannelRouter::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            local_bundle: None,
            topic_manager: Arc::new(RwLock::new(TopicManager::new())),
        }
    }

    /// Set the key store.
    pub fn with_key_store(mut self, store: Arc<dyn KeyStore>) -> Self {
        self.key_store = Some(store);
        self
    }

    /// Set the session store.
    pub fn with_session_store(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.session_store = Some(store);
        self
    }

    /// Add a delivery channel.
    pub async fn add_channel(&self, channel: Box<dyn Channel>) {
        let mut router = self.channel_router.write().await;
        router.add_channel(channel);
    }

    /// Initialize for a local agent.
    pub async fn initialize(&mut self, agent_id: AgentId) -> CretoResult<()> {
        // Generate key bundle
        let bundle = KeyBundle::new(agent_id);

        // Store keys if we have a key store
        if let Some(store) = &self.key_store {
            store.store_bundle(&bundle).await?;
        }

        self.local_bundle = Some(bundle);

        tracing::info!(%agent_id, "Messaging service initialized");

        Ok(())
    }

    /// Establish a session with another agent.
    pub async fn establish_session(&self, remote_agent: AgentId) -> CretoResult<Uuid> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        // Get recipient's key bundle
        let remote_bundle = if let Some(store) = &self.key_store {
            store.get_bundle(remote_agent).await?.ok_or_else(|| {
                creto_common::CretoError::SessionError(format!(
                    "No key bundle found for agent {}",
                    remote_agent
                ))
            })?
        } else {
            return Err(creto_common::CretoError::SessionError(
                "No key store configured".to_string(),
            ));
        };

        // Perform X3DH
        let x3dh_result = X3DH::initiate(local_bundle, &remote_bundle)?;

        // Create session
        let session = Session::new_initiator(
            local_bundle.agent_id,
            remote_agent,
            &x3dh_result,
        );

        let session_id = session.id;

        // Store session
        if let Some(store) = &self.session_store {
            store.store_session(&session).await?;
        }

        // Cache session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);

        tracing::info!(
            session_id = %session_id,
            remote_agent = %remote_agent,
            "Session established"
        );

        Ok(session_id)
    }

    /// Send a message to another agent.
    pub async fn send(
        &self,
        session_id: Uuid,
        message: &[u8],
    ) -> CretoResult<DeliveryReceipt> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or_else(|| {
            creto_common::CretoError::SessionError(format!(
                "Session {} not found",
                session_id
            ))
        })?;

        // Encrypt message
        let envelope = session.encrypt(message)?;

        // Deliver via channel
        let router = self.channel_router.read().await;
        let receipt = router.route(&envelope).await?;

        Ok(receipt)
    }

    /// Send a message to an agent, establishing session if needed.
    pub async fn send_to(
        &self,
        remote_agent: AgentId,
        message: &[u8],
    ) -> CretoResult<DeliveryReceipt> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        // Find existing session
        let session_id = {
            let sessions = self.sessions.read().await;
            sessions
                .iter()
                .find(|(_, s)| {
                    s.local_agent == local_bundle.agent_id
                        && s.remote_agent == remote_agent
                        && s.state == SessionState::Active
                })
                .map(|(id, _)| *id)
        };

        let session_id = match session_id {
            Some(id) => id,
            None => self.establish_session(remote_agent).await?,
        };

        self.send(session_id, message).await
    }

    /// Receive messages for the local agent.
    pub async fn receive(&self, limit: u32) -> CretoResult<Vec<(Uuid, Vec<u8>)>> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        // Get pending envelopes from channel
        let router = self.channel_router.read().await;

        // TODO: Properly iterate channels
        // For now, just return empty
        let _ = (local_bundle, limit, router);

        Ok(Vec::new())
    }

    /// Process a received envelope.
    pub async fn process_envelope(&self, envelope: &Envelope) -> CretoResult<Vec<u8>> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        // Verify we are the recipient
        if envelope.header.recipient_id != local_bundle.agent_id {
            return Err(creto_common::CretoError::SessionError(
                "Not the intended recipient".to_string(),
            ));
        }

        // Find session for sender
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .values_mut()
            .find(|s| {
                s.remote_agent == envelope.header.sender_id
                    && s.local_agent == local_bundle.agent_id
            })
            .ok_or_else(|| {
                creto_common::CretoError::SessionError(format!(
                    "No session found for sender {}",
                    envelope.header.sender_id
                ))
            })?;

        // Decrypt
        session.decrypt(envelope)
    }

    /// Close a session.
    pub async fn close_session(&self, session_id: Uuid) -> CretoResult<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(&session_id) {
            session.close();

            // Update in store
            if let Some(store) = &self.session_store {
                store.store_session(session).await?;
            }
        }

        sessions.remove(&session_id);

        Ok(())
    }

    /// Get session status.
    pub async fn session_status(&self, session_id: Uuid) -> Option<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).map(|s| s.state)
    }

    /// List active sessions.
    pub async fn list_sessions(&self) -> Vec<Uuid> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .filter(|(_, s)| s.is_active())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Create a new topic.
    pub async fn create_topic(&self, config: TopicConfig) -> CretoResult<TopicId> {
        let mut manager = self.topic_manager.write().await;
        manager.create_topic(config)
    }

    /// Subscribe to a topic.
    pub async fn subscribe(
        &self,
        topic_id: TopicId,
        filter: Option<SubscriptionFilter>,
    ) -> CretoResult<Subscription> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        let mut manager = self.topic_manager.write().await;
        manager.subscribe(topic_id, local_bundle.agent_id, filter)
    }

    /// Publish a message to a topic.
    pub async fn publish(
        &self,
        topic_id: TopicId,
        message: &[u8],
        metadata: std::collections::HashMap<String, String>,
    ) -> CretoResult<Vec<AgentId>> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        let mut manager = self.topic_manager.write().await;
        manager.publish(topic_id, local_bundle.agent_id, message, metadata)
    }

    /// Unsubscribe from a topic.
    pub async fn unsubscribe(&self, subscription_id: Uuid) -> CretoResult<()> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        let mut manager = self.topic_manager.write().await;
        manager.unsubscribe(subscription_id, local_bundle.agent_id)
    }

    /// Delete a topic.
    pub async fn delete_topic(&self, topic_id: TopicId) -> CretoResult<()> {
        let local_bundle = self.local_bundle.as_ref().ok_or_else(|| {
            creto_common::CretoError::SessionError("Service not initialized".to_string())
        })?;

        let mut manager = self.topic_manager.write().await;
        manager.delete_topic(topic_id, local_bundle.agent_id)
    }

    /// List subscribers to a topic.
    pub async fn list_topic_subscribers(
        &self,
        topic_id: TopicId,
    ) -> CretoResult<Vec<Subscription>> {
        let manager = self.topic_manager.read().await;
        manager.list_subscribers(topic_id)
    }
}

impl Default for MessagingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = MessagingService::new();
        let sessions = service.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_service_initialization() {
        let mut service = MessagingService::new();
        let agent_id = AgentId::new();

        // Note: This will succeed but session establishment requires key store
        service.initialize(agent_id).await.unwrap();
    }
}
