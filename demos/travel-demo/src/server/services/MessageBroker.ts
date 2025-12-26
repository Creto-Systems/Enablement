import type {
  AgentMessage,
  MessagePriority,
  EncryptedMessage,
} from '@shared/types';
import {
  encryptMessage,
  decryptMessage,
  generateId,
} from '../utils/encryption.js';
import type { MessageBrokerInterface } from '../agents/BaseAgent.js';

/**
 * Message Broker - Handles encrypted message routing between agents
 * Implements priority queue and dead letter queue patterns
 */
export class MessageBroker implements MessageBrokerInterface {
  private queues: Record<MessagePriority, QueuedMessage[]> = {
    critical: [],
    high: [],
    medium: [],
    low: [],
  };

  private deadLetterQueue: QueuedMessage[] = [];
  private processing = false;
  private keyRegistry: Map<string, { public: string; private: string }> =
    new Map();
  private messageHandlers: Map<string, MessageHandler> = new Map();

  /**
   * Register agent keys for encryption
   */
  registerAgent(
    agentId: string,
    publicKey: string,
    privateKey: string
  ): void {
    this.keyRegistry.set(agentId, { public: publicKey, private: privateKey });
  }

  /**
   * Register message handler for agent
   */
  registerHandler(agentId: string, handler: MessageHandler): void {
    this.messageHandlers.set(agentId, handler);
  }

  /**
   * Get public key for agent
   */
  getPublicKey(agentId: string): string | undefined {
    return this.keyRegistry.get(agentId)?.public;
  }

  /**
   * Send message with priority
   */
  async send(message: AgentMessage, priority: MessagePriority): Promise<void> {
    const queuedMessage: QueuedMessage = {
      message,
      priority,
      enqueuedAt: Date.now(),
      retries: 0,
    };

    // Add to appropriate queue
    this.queues[priority].push(queuedMessage);

    // Start processing if not already running
    if (!this.processing) {
      void this.processQueue();
    }
  }

  /**
   * Process message queue by priority
   */
  private async processQueue(): Promise<void> {
    this.processing = true;

    while (this.hasMessages()) {
      // Process by priority order
      for (const priority of ['critical', 'high', 'medium', 'low'] as const) {
        const queue = this.queues[priority];

        if (queue.length > 0) {
          const item = queue.shift()!;

          try {
            await this.routeMessage(item.message);
          } catch (error) {
            await this.handleMessageError(item, error as Error, priority);
          }

          break; // Process one message per iteration
        }
      }

      // Small delay to prevent CPU saturation
      await new Promise((resolve) => setTimeout(resolve, 10));
    }

    this.processing = false;
  }

  /**
   * Route message to recipient with encryption
   */
  private async routeMessage(message: AgentMessage): Promise<void> {
    const recipient = message.to;

    // Get recipient's handler
    const handler = this.messageHandlers.get(recipient);
    if (!handler) {
      // Special handling for coordinator messages
      if (recipient === 'coordinator') {
        // Store for coordinator to retrieve
        return;
      }
      throw new Error(`No handler registered for ${recipient}`);
    }

    // Get keys for encryption
    const senderKeys = this.keyRegistry.get(message.from);
    const recipientKeys = this.keyRegistry.get(recipient);

    if (!senderKeys || !recipientKeys) {
      throw new Error('Encryption keys not found');
    }

    // Encrypt message
    const encrypted = await encryptMessage(
      message,
      recipientKeys.public,
      senderKeys.private
    );

    // Decrypt for recipient (simulating secure channel)
    const decrypted = await decryptMessage(
      encrypted,
      recipientKeys.private,
      senderKeys.public
    );

    // Deliver to handler
    await handler(decrypted);
  }

  /**
   * Handle message error with retry logic
   */
  private async handleMessageError(
    item: QueuedMessage,
    error: Error,
    priority: MessagePriority
  ): Promise<void> {
    item.retries++;

    if (item.retries < 3) {
      // Re-queue with exponential backoff
      const delay = Math.pow(2, item.retries) * 1000;

      setTimeout(() => {
        this.queues[priority].push(item);
      }, delay);
    } else {
      // Move to dead letter queue
      this.deadLetterQueue.push({
        ...item,
        error: error.message,
        failedAt: Date.now(),
      });

      console.error('Message failed after retries:', {
        messageId: item.message.id,
        error: error.message,
      });
    }
  }

  /**
   * Check if any messages in queues
   */
  private hasMessages(): boolean {
    return Object.values(this.queues).some((queue) => queue.length > 0);
  }

  /**
   * Get queue stats for monitoring
   */
  getStats(): QueueStats {
    return {
      queues: {
        critical: this.queues.critical.length,
        high: this.queues.high.length,
        medium: this.queues.medium.length,
        low: this.queues.low.length,
      },
      deadLetter: this.deadLetterQueue.length,
      processing: this.processing,
    };
  }

  /**
   * Clear all queues (for testing)
   */
  clear(): void {
    this.queues = {
      critical: [],
      high: [],
      medium: [],
      low: [],
    };
    this.deadLetterQueue = [];
  }
}

// Types
interface QueuedMessage {
  message: AgentMessage;
  priority: MessagePriority;
  enqueuedAt: number;
  retries: number;
  error?: string;
  failedAt?: number;
}

interface QueueStats {
  queues: Record<MessagePriority, number>;
  deadLetter: number;
  processing: boolean;
}

type MessageHandler = (message: AgentMessage) => Promise<void>;
