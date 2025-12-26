//! Creto Messaging - Secure End-to-End Encrypted Agent Communication
//!
//! This crate provides secure messaging for AI agents using patterns from the
//! Signal protocol, including X3DH key agreement and Double Ratchet encryption.
//!
//! # Architecture
//!
//! The messaging system follows Signal protocol patterns:
//! - **X3DH**: Extended Triple Diffie-Hellman for initial key agreement
//! - **Double Ratchet**: Continuous key derivation for forward secrecy
//! - **Envelope**: Encrypted payload with wrapped key and signature
//!
//! # Security Properties
//!
//! - **Forward Secrecy**: Past messages cannot be decrypted if keys are compromised
//! - **Post-Compromise Security**: Security is restored after key compromise
//! - **Deniability**: Messages cannot be cryptographically attributed
//!
//! # Example
//!
//! ```rust,ignore
//! use creto_messaging::{MessagingService, KeyBundle};
//!
//! let service = MessagingService::new();
//! let session = service.establish_session(sender_id, recipient_id).await?;
//! let encrypted = session.encrypt(b"Hello, agent!").await?;
//! ```

pub mod keys;
pub mod x3dh;
pub mod ratchet;
pub mod envelope;
pub mod repository;
pub mod session;
pub mod channel;
pub mod service;
pub mod topic;

pub use keys::{KeyBundle, IdentityKey, PreKey, SignedPreKey};
pub use x3dh::{X3DHParams, X3DHResult};
pub use ratchet::{DoubleRatchet, RatchetState};
pub use envelope::{Envelope, EnvelopeHeader, EncryptedPayload, ContentType, DeliveryReceipt, ReceiptType};
pub use repository::{
    KeyBundleRepository, PgKeyBundleRepository,
    PreKeyRepository, PgPreKeyRepository,
    SessionRepository, PgSessionRepository,
    EnvelopeRepository, PgEnvelopeRepository,
    ChannelRepository, PgChannelRepository,
};
pub use session::{Session, SessionState};
pub use channel::{Channel, ChannelType, ChannelConfig};
pub use service::MessagingService;
pub use topic::{
    Topic, TopicId, TopicPolicy, TopicConfig, TopicManager,
    Subscription, SubscriptionId, SubscriptionFilter,
};
