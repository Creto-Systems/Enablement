import { ed25519 } from '@noble/ed25519';
import { randomBytes, createCipheriv, createDecipheriv } from 'crypto';
import type { AgentMessage, EncryptedMessage } from '@shared/types';

/**
 * Encryption utilities using Ed25519 for signatures and AES-256-GCM for encryption
 * Following creto-messaging patterns for secure agent communication
 */

const ALGORITHM = 'aes-256-gcm';
const IV_LENGTH = 16;
const AUTH_TAG_LENGTH = 16;
const KEY_LENGTH = 32;

export interface KeyPair {
  publicKey: string;
  privateKey: string;
}

/**
 * Generate Ed25519 keypair for agent
 */
export async function generateKeyPair(): Promise<KeyPair> {
  const privateKey = ed25519.utils.randomPrivateKey();
  const publicKey = await ed25519.getPublicKey(privateKey);

  return {
    publicKey: Buffer.from(publicKey).toString('hex'),
    privateKey: Buffer.from(privateKey).toString('hex'),
  };
}

/**
 * Sign message with Ed25519 private key
 */
export async function signMessage(
  message: string,
  privateKeyHex: string
): Promise<string> {
  const privateKey = Buffer.from(privateKeyHex, 'hex');
  const messageBytes = new TextEncoder().encode(message);
  const signature = await ed25519.sign(messageBytes, privateKey);

  return Buffer.from(signature).toString('hex');
}

/**
 * Verify Ed25519 signature
 */
export async function verifySignature(
  message: string,
  signatureHex: string,
  publicKeyHex: string
): Promise<boolean> {
  try {
    const publicKey = Buffer.from(publicKeyHex, 'hex');
    const signature = Buffer.from(signatureHex, 'hex');
    const messageBytes = new TextEncoder().encode(message);

    return await ed25519.verify(signature, messageBytes, publicKey);
  } catch {
    return false;
  }
}

/**
 * Derive AES key from Ed25519 public key
 * In production, use proper key derivation (ECDH)
 * For demo, we hash the public key
 */
function deriveAESKey(publicKeyHex: string): Buffer {
  const crypto = await import('crypto');
  const hash = crypto.createHash('sha256');
  hash.update(Buffer.from(publicKeyHex, 'hex'));
  return hash.digest();
}

/**
 * Encrypt message with AES-256-GCM
 */
export async function encryptMessage(
  message: AgentMessage,
  recipientPublicKeyHex: string,
  senderPrivateKeyHex: string
): Promise<EncryptedMessage> {
  // Serialize message
  const payload = JSON.stringify(message.payload);

  // Sign with sender's private key
  const signature = await signMessage(payload, senderPrivateKeyHex);

  // Generate IV
  const iv = randomBytes(IV_LENGTH);

  // Derive encryption key from recipient's public key
  const key = deriveAESKey(recipientPublicKeyHex);

  // Encrypt
  const cipher = createCipheriv(ALGORITHM, key, iv);
  let encrypted = cipher.update(payload, 'utf8', 'hex');
  encrypted += cipher.final('hex');

  // Get auth tag
  const authTag = cipher.getAuthTag();

  // Combine encrypted data with auth tag
  const combined = encrypted + authTag.toString('hex');

  return {
    envelope: {
      from: message.from,
      to: message.to,
      timestamp: message.timestamp,
      correlationId: message.correlationId,
    },
    payload: combined,
    signature,
    nonce: iv.toString('hex'),
  };
}

/**
 * Decrypt message with AES-256-GCM
 */
export async function decryptMessage(
  encrypted: EncryptedMessage,
  recipientPrivateKeyHex: string,
  senderPublicKeyHex: string
): Promise<AgentMessage> {
  // Extract auth tag
  const authTag = Buffer.from(
    encrypted.payload.slice(-AUTH_TAG_LENGTH * 2),
    'hex'
  );
  const ciphertext = encrypted.payload.slice(0, -AUTH_TAG_LENGTH * 2);

  // Derive decryption key (same as encryption key for demo)
  // In production, use ECDH with recipient's private key
  const publicKey = await ed25519.getPublicKey(
    Buffer.from(recipientPrivateKeyHex, 'hex')
  );
  const key = deriveAESKey(Buffer.from(publicKey).toString('hex'));

  // Decrypt
  const iv = Buffer.from(encrypted.nonce, 'hex');
  const decipher = createDecipheriv(ALGORITHM, key, iv);
  decipher.setAuthTag(authTag);

  let decrypted = decipher.update(ciphertext, 'hex', 'utf8');
  decrypted += decipher.final('utf8');

  // Verify signature
  const valid = await verifySignature(
    decrypted,
    encrypted.signature,
    senderPublicKeyHex
  );

  if (!valid) {
    throw new Error('Invalid message signature');
  }

  // Parse payload
  const payload = JSON.parse(decrypted);

  return {
    id: encrypted.envelope.correlationId,
    from: encrypted.envelope.from,
    to: encrypted.envelope.to,
    type: payload.type,
    payload,
    encrypted: true,
    signature: encrypted.signature,
    timestamp: encrypted.envelope.timestamp,
    correlationId: encrypted.envelope.correlationId,
  };
}

/**
 * Generate correlation ID for message tracking
 */
export function generateCorrelationId(): string {
  return randomBytes(16).toString('hex');
}

/**
 * Generate unique ID
 */
export function generateId(): string {
  return randomBytes(12).toString('hex');
}
