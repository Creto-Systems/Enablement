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

pub mod channel;
pub mod envelope;
pub mod keys;
pub mod ratchet;
pub mod repository;
pub mod service;
pub mod session;
pub mod topic;
pub mod x3dh;

pub use channel::{Channel, ChannelConfig, ChannelType};
pub use envelope::{
    ContentType, DeliveryReceipt, EncryptedPayload, Envelope, EnvelopeHeader, ReceiptType,
};
pub use keys::{IdentityKey, KeyBundle, PreKey, SignedPreKey};
pub use ratchet::{DoubleRatchet, RatchetState};
pub use repository::{
    ChannelRepository, EnvelopeRepository, KeyBundleRepository, PgChannelRepository,
    PgEnvelopeRepository, PgKeyBundleRepository, PgPreKeyRepository, PgSessionRepository,
    PreKeyRepository, SessionRepository,
};
pub use service::MessagingService;
pub use session::{Session, SessionState};
pub use topic::{
    Subscription, SubscriptionFilter, SubscriptionId, Topic, TopicConfig, TopicId, TopicManager,
    TopicPolicy,
};
pub use x3dh::{X3DHParams, X3DHResult};
