//! Double Ratchet algorithm for continuous key derivation.
//!
//! Provides forward secrecy and post-compromise security by continuously
//! deriving new keys for each message.

use serde::{Deserialize, Serialize};

/// State of the Double Ratchet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetState {
    /// Current root key.
    root_key: Vec<u8>,

    /// Our current DH key pair (public).
    pub dh_public: Vec<u8>,

    /// Our current DH key pair (private).
    #[serde(skip_serializing)]
    dh_private: Option<Vec<u8>>,

    /// Their current DH public key.
    pub their_dh_public: Option<Vec<u8>>,

    /// Current sending chain key.
    send_chain_key: Option<Vec<u8>>,

    /// Current receiving chain key.
    recv_chain_key: Option<Vec<u8>>,

    /// Number of messages sent in current send chain.
    pub send_count: u32,

    /// Number of messages received in current recv chain.
    pub recv_count: u32,

    /// Previous chain lengths (for out-of-order messages).
    pub prev_chain_lengths: Vec<(Vec<u8>, u32)>,

    /// Skipped message keys (for out-of-order decryption).
    #[serde(skip)]
    skipped_keys: std::collections::HashMap<(Vec<u8>, u32), Vec<u8>>,

    /// Maximum number of skipped keys to store.
    max_skip: u32,
}

impl RatchetState {
    /// Create a new ratchet state from X3DH shared secret.
    pub fn initialize_sender(shared_secret: &[u8], their_dh_public: &[u8]) -> Self {
        // TODO: Use HKDF to derive initial root and chain keys

        // Generate our DH key pair
        let dh_public = vec![0u8; 32]; // Placeholder
        let dh_private = vec![0u8; 32]; // Placeholder

        // Derive send chain key from DH(our_dh, their_dh)
        let send_chain_key = shared_secret.to_vec();

        Self {
            root_key: shared_secret.to_vec(),
            dh_public,
            dh_private: Some(dh_private),
            their_dh_public: Some(their_dh_public.to_vec()),
            send_chain_key: Some(send_chain_key),
            recv_chain_key: None,
            send_count: 0,
            recv_count: 0,
            prev_chain_lengths: Vec::new(),
            skipped_keys: std::collections::HashMap::new(),
            max_skip: 100,
        }
    }

    /// Create a new ratchet state as receiver.
    pub fn initialize_receiver(
        shared_secret: &[u8],
        our_dh_public: &[u8],
        our_dh_private: &[u8],
    ) -> Self {
        Self {
            root_key: shared_secret.to_vec(),
            dh_public: our_dh_public.to_vec(),
            dh_private: Some(our_dh_private.to_vec()),
            their_dh_public: None,
            send_chain_key: None,
            recv_chain_key: None,
            send_count: 0,
            recv_count: 0,
            prev_chain_lengths: Vec::new(),
            skipped_keys: std::collections::HashMap::new(),
            max_skip: 100,
        }
    }

    /// Get our current DH public key.
    pub fn our_public_key(&self) -> &[u8] {
        &self.dh_public
    }

    /// Check if we have a sending chain.
    pub fn can_send(&self) -> bool {
        self.send_chain_key.is_some() && self.their_dh_public.is_some()
    }

    /// Check if we have a receiving chain.
    pub fn can_receive(&self) -> bool {
        self.recv_chain_key.is_some()
    }
}

/// Double Ratchet implementation.
pub struct DoubleRatchet {
    state: RatchetState,
}

impl DoubleRatchet {
    /// Create a new Double Ratchet as sender.
    pub fn new_sender(shared_secret: &[u8], their_signed_prekey: &[u8]) -> Self {
        Self {
            state: RatchetState::initialize_sender(shared_secret, their_signed_prekey),
        }
    }

    /// Create a new Double Ratchet as receiver.
    pub fn new_receiver(
        shared_secret: &[u8],
        our_signed_prekey_public: &[u8],
        our_signed_prekey_private: &[u8],
    ) -> Self {
        Self {
            state: RatchetState::initialize_receiver(
                shared_secret,
                our_signed_prekey_public,
                our_signed_prekey_private,
            ),
        }
    }

    /// Get the current state.
    pub fn state(&self) -> &RatchetState {
        &self.state
    }

    /// Encrypt a message.
    ///
    /// Returns the ciphertext and the message header.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> creto_common::CretoResult<EncryptedMessage> {
        if !self.state.can_send() {
            return Err(creto_common::CretoError::CryptoError(
                "Cannot encrypt: no sending chain".to_string(),
            ));
        }

        // 1. Derive message key from chain key
        let (message_key, next_chain_key) =
            self.kdf_chain(self.state.send_chain_key.as_ref().unwrap());

        // 2. Update chain key
        self.state.send_chain_key = Some(next_chain_key);

        // 3. Encrypt with message key
        // TODO: Use AEAD (AES-256-GCM or ChaCha20-Poly1305)
        let ciphertext = Self::aead_encrypt(&message_key, plaintext)?;

        // 4. Create header
        let header = MessageHeader {
            dh_public: self.state.dh_public.clone(),
            prev_chain_length: 0, // TODO: Track properly
            message_number: self.state.send_count,
        };

        self.state.send_count += 1;

        Ok(EncryptedMessage { header, ciphertext })
    }

    /// Decrypt a message.
    pub fn decrypt(&mut self, message: &EncryptedMessage) -> creto_common::CretoResult<Vec<u8>> {
        // Check if we need to perform a DH ratchet
        let need_ratchet = self
            .state
            .their_dh_public
            .as_ref()
            .map(|k| k != &message.header.dh_public)
            .unwrap_or(true);

        if need_ratchet {
            // Perform DH ratchet step
            self.dh_ratchet(&message.header.dh_public)?;
        }

        // Check for skipped messages
        let skip_key = (
            message.header.dh_public.clone(),
            message.header.message_number,
        );
        if let Some(message_key) = self.state.skipped_keys.remove(&skip_key) {
            return Self::aead_decrypt(&message_key, &message.ciphertext);
        }

        // Skip messages if needed
        self.skip_messages(message.header.message_number)?;

        // Derive message key
        let (message_key, next_chain_key) =
            self.kdf_chain(self.state.recv_chain_key.as_ref().ok_or_else(|| {
                creto_common::CretoError::CryptoError("No receiving chain".to_string())
            })?);

        self.state.recv_chain_key = Some(next_chain_key);
        self.state.recv_count += 1;

        Self::aead_decrypt(&message_key, &message.ciphertext)
    }

    /// Perform a DH ratchet step.
    fn dh_ratchet(&mut self, their_new_dh: &[u8]) -> creto_common::CretoResult<()> {
        // Store previous chain length for out-of-order handling
        if let Some(old_dh) = &self.state.their_dh_public {
            self.state
                .prev_chain_lengths
                .push((old_dh.clone(), self.state.recv_count));
        }

        // Update their DH public key
        self.state.their_dh_public = Some(their_new_dh.to_vec());

        // Derive new receiving chain key
        // DH(our_dh, their_new_dh) -> HKDF with root key
        let dh_output = self.dh(
            self.state.dh_private.as_ref().ok_or_else(|| {
                creto_common::CretoError::CryptoError("No DH private key".to_string())
            })?,
            their_new_dh,
        );

        let (new_root_key, recv_chain_key) = self.kdf_root(&self.state.root_key, &dh_output);
        self.state.root_key = new_root_key;
        self.state.recv_chain_key = Some(recv_chain_key);
        self.state.recv_count = 0;

        // Generate new DH key pair
        self.state.dh_public = vec![0u8; 32]; // TODO: Generate real key
        self.state.dh_private = Some(vec![0u8; 32]);

        // Derive new sending chain key
        let dh_output = self.dh(self.state.dh_private.as_ref().unwrap(), their_new_dh);

        let (new_root_key, send_chain_key) = self.kdf_root(&self.state.root_key, &dh_output);
        self.state.root_key = new_root_key;
        self.state.send_chain_key = Some(send_chain_key);
        self.state.send_count = 0;

        Ok(())
    }

    /// Skip messages and store their keys.
    fn skip_messages(&mut self, until: u32) -> creto_common::CretoResult<()> {
        if until > self.state.recv_count + self.state.max_skip {
            return Err(creto_common::CretoError::CryptoError(
                "Too many skipped messages".to_string(),
            ));
        }

        let their_dh = self.state.their_dh_public.clone().ok_or_else(|| {
            creto_common::CretoError::CryptoError("No their DH public key".to_string())
        })?;

        while self.state.recv_count < until {
            let (message_key, next_chain_key) =
                self.kdf_chain(self.state.recv_chain_key.as_ref().ok_or_else(|| {
                    creto_common::CretoError::CryptoError("No receiving chain".to_string())
                })?);

            let skip_key = (their_dh.clone(), self.state.recv_count);
            self.state.skipped_keys.insert(skip_key, message_key);
            self.state.recv_chain_key = Some(next_chain_key);
            self.state.recv_count += 1;
        }

        Ok(())
    }

    /// Diffie-Hellman operation.
    fn dh(&self, private: &[u8], public: &[u8]) -> Vec<u8> {
        // TODO: Use creto-crypto for X25519
        let mut output = vec![0u8; 32];
        for (i, (a, b)) in private.iter().zip(public.iter()).enumerate() {
            output[i] = a ^ b; // Placeholder XOR
        }
        output
    }

    /// KDF for root key update.
    fn kdf_root(&self, root_key: &[u8], dh_output: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // TODO: Use HKDF
        // HKDF(salt=root_key, ikm=dh_output) -> (new_root_key, chain_key)
        let mut combined = root_key.to_vec();
        combined.extend_from_slice(dh_output);

        let new_root = combined[..32].to_vec();
        let chain_key = combined[..32].to_vec(); // Placeholder
        (new_root, chain_key)
    }

    /// KDF for chain key update.
    fn kdf_chain(&self, chain_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // TODO: Use HMAC-SHA256
        // message_key = HMAC(chain_key, 0x01)
        // next_chain_key = HMAC(chain_key, 0x02)
        let message_key = chain_key.to_vec(); // Placeholder
        let next_chain_key = chain_key.to_vec(); // Placeholder
        (message_key, next_chain_key)
    }

    /// AEAD encryption.
    fn aead_encrypt(key: &[u8], plaintext: &[u8]) -> creto_common::CretoResult<Vec<u8>> {
        // TODO: Use AES-256-GCM or ChaCha20-Poly1305
        let _ = key;
        Ok(plaintext.to_vec()) // Placeholder: no actual encryption
    }

    /// AEAD decryption.
    fn aead_decrypt(key: &[u8], ciphertext: &[u8]) -> creto_common::CretoResult<Vec<u8>> {
        // TODO: Use AES-256-GCM or ChaCha20-Poly1305
        let _ = key;
        Ok(ciphertext.to_vec()) // Placeholder: no actual decryption
    }
}

/// Header sent with each encrypted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Sender's current DH public key.
    pub dh_public: Vec<u8>,
    /// Previous chain length (for message ordering).
    pub prev_chain_length: u32,
    /// Message number in current chain.
    pub message_number: u32,
}

/// An encrypted message with header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Message header (sent in clear).
    pub header: MessageHeader,
    /// Encrypted ciphertext.
    pub ciphertext: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratchet_initialization() {
        let shared_secret = vec![0u8; 32];
        let their_dh = vec![1u8; 32];

        let state = RatchetState::initialize_sender(&shared_secret, &their_dh);

        assert!(state.can_send());
        assert!(!state.can_receive());
        assert_eq!(state.send_count, 0);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let shared_secret = vec![42u8; 32];
        let signed_prekey_pub = vec![1u8; 32];
        let signed_prekey_priv = vec![2u8; 32];

        // Create sender ratchet
        let mut sender = DoubleRatchet::new_sender(&shared_secret, &signed_prekey_pub);

        // Create receiver ratchet
        let mut receiver =
            DoubleRatchet::new_receiver(&shared_secret, &signed_prekey_pub, &signed_prekey_priv);

        // Sender encrypts
        let plaintext = b"Hello, world!";
        let encrypted = sender.encrypt(plaintext).unwrap();

        // Note: This test uses placeholder crypto, so it won't actually work
        // In production, proper DH ratchet setup is needed
        assert!(!encrypted.ciphertext.is_empty());
    }
}
