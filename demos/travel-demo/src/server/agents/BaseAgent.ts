import type {
  Agent,
  AgentMessage,
  AgentType,
  AgentStatus,
  MessagePriority,
  TripConstraints,
} from '@shared/types';
import {
  generateKeyPair,
  generateCorrelationId,
  generateId,
} from '../utils/encryption.js';

/**
 * Base Agent class with secure messaging capabilities
 * All specialized agents extend this class
 */
export abstract class BaseAgent implements Agent {
  public id: string;
  public type: AgentType;
  public status: AgentStatus;
  public tripId: string;
  public publicKey: string;
  private privateKey: string;
  public lastActive: string;
  public metrics: {
    responseTime: number;
    successRate: number;
    messagesProcessed: number;
  };

  protected messageBroker: MessageBrokerInterface | null = null;

  constructor(type: AgentType, tripId: string) {
    this.id = generateId();
    this.type = type;
    this.status = 'idle';
    this.tripId = tripId;
    this.publicKey = '';
    this.privateKey = '';
    this.lastActive = new Date().toISOString();
    this.metrics = {
      responseTime: 0,
      successRate: 1.0,
      messagesProcessed: 0,
    };
  }

  /**
   * Initialize agent with encryption keys
   */
  async initialize(): Promise<void> {
    const keys = await generateKeyPair();
    this.publicKey = keys.publicKey;
    this.privateKey = keys.privateKey;
    this.updateStatus('idle');
  }

  /**
   * Set message broker for communication
   */
  setMessageBroker(broker: MessageBrokerInterface): void {
    this.messageBroker = broker;
  }

  /**
   * Update agent status
   */
  protected updateStatus(status: AgentStatus): void {
    this.status = status;
    this.lastActive = new Date().toISOString();
  }

  /**
   * Send encrypted message to another agent or coordinator
   */
  protected async sendMessage(
    to: string,
    type: AgentMessage['type'],
    action: string,
    data?: unknown,
    priority: MessagePriority = 'medium'
  ): Promise<void> {
    if (!this.messageBroker) {
      throw new Error('Message broker not set');
    }

    const message: AgentMessage = {
      id: generateId(),
      from: this.id,
      to,
      type,
      payload: {
        action,
        data,
      },
      encrypted: true,
      timestamp: Date.now(),
      correlationId: generateCorrelationId(),
    };

    await this.messageBroker.send(message, priority);
    this.metrics.messagesProcessed++;
  }

  /**
   * Handle incoming message
   */
  async handleMessage(message: AgentMessage): Promise<void> {
    const startTime = Date.now();

    try {
      await this.processMessage(message);

      // Update metrics
      const responseTime = Date.now() - startTime;
      this.metrics.responseTime =
        (this.metrics.responseTime + responseTime) / 2;
    } catch (error) {
      console.error(`Agent ${this.id} error processing message:`, error);
      this.metrics.successRate =
        (this.metrics.successRate * this.metrics.messagesProcessed) /
        (this.metrics.messagesProcessed + 1);

      await this.sendMessage(
        message.from,
        'error',
        'processingError',
        {
          originalMessage: message.id,
          error: error instanceof Error ? error.message : 'Unknown error',
        },
        'high'
      );
    }
  }

  /**
   * Abstract method - each agent implements its own message processing
   */
  protected abstract processMessage(message: AgentMessage): Promise<void>;

  /**
   * Abstract method - each agent implements its own search logic
   */
  abstract search(constraints: TripConstraints): Promise<unknown>;

  /**
   * Notify budget agent of costs
   */
  protected async notifyBudget(
    category: string,
    amount: number
  ): Promise<void> {
    await this.sendMessage(
      'budget',
      'notification',
      'costUpdate',
      { category, amount },
      'high'
    );
  }

  /**
   * Send results to coordinator
   */
  protected async sendResults(to: string, data: unknown): Promise<void> {
    await this.sendMessage(to, 'response', 'searchResults', data, 'high');
  }

  /**
   * Get agent info for status reporting
   */
  getInfo(): Agent {
    return {
      id: this.id,
      type: this.type,
      status: this.status,
      tripId: this.tripId,
      publicKey: this.publicKey,
      lastActive: this.lastActive,
      metrics: { ...this.metrics },
    };
  }

  /**
   * Cleanup agent resources
   */
  async destroy(): Promise<void> {
    this.updateStatus('idle');
    this.messageBroker = null;
  }
}

/**
 * Interface for message broker
 */
export interface MessageBrokerInterface {
  send(message: AgentMessage, priority: MessagePriority): Promise<void>;
  getPublicKey(agentId: string): string | undefined;
}
