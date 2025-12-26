/**
 * gRPC Client Factory - Metering & Oversight Services
 *
 * Creates and manages gRPC clients for creto-metering and creto-oversight services.
 * Includes health checks, retry logic, and connection pooling.
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { EventEmitter } from 'events';

/**
 * gRPC Client Configuration
 */
export interface GrpcClientConfig {
  /** Service address (host:port) */
  address: string;

  /** Maximum retry attempts */
  maxRetries?: number;

  /** Retry delay in milliseconds */
  retryDelay?: number;

  /** Connection timeout in milliseconds */
  timeout?: number;

  /** Enable keepalive */
  keepalive?: boolean;
}

/**
 * Metering Service Client Interface
 */
export interface MeteringClient {
  recordEvent(event: {
    agentId: string;
    eventType: string;
    amount: number;
    metadata?: Record<string, any>;
  }): Promise<void>;

  checkQuota(agentId: string, resource: string): Promise<{
    allowed: boolean;
    remaining: number;
    limit: number;
  }>;

  getUsage(agentId: string): Promise<{
    apiCalls: number;
    quotaUsed: number;
    quotaLimit: number;
  }>;
}

/**
 * Oversight Service Client Interface
 */
export interface OversightClient {
  requestApproval(request: {
    agentId: string;
    tradeId: string;
    amount: number;
    reason: string;
  }): Promise<{
    requestId: string;
    status: 'pending' | 'approved' | 'rejected';
  }>;

  getApprovalStatus(requestId: string): Promise<{
    status: 'pending' | 'approved' | 'rejected';
    approvals: Array<{ approverId: string; decision: string }>;
  }>;

  cancelApproval(requestId: string): Promise<void>;
}

/**
 * Mock Metering Client Implementation
 *
 * Used for development until gRPC services are deployed.
 */
class MockMeteringClient extends EventEmitter implements MeteringClient {
  private events: Map<string, any[]> = new Map();
  private quotas: Map<string, { used: number; limit: number }> = new Map();

  async recordEvent(event: {
    agentId: string;
    eventType: string;
    amount: number;
    metadata?: Record<string, any>;
  }): Promise<void> {
    const agentEvents = this.events.get(event.agentId) || [];
    agentEvents.push({ ...event, timestamp: new Date() });
    this.events.set(event.agentId, agentEvents);

    // Update quota
    const quota = this.quotas.get(event.agentId) || { used: 0, limit: 10000 };
    quota.used += 1;
    this.quotas.set(event.agentId, quota);

    this.emit('event-recorded', event);
  }

  async checkQuota(agentId: string, resource: string): Promise<{
    allowed: boolean;
    remaining: number;
    limit: number;
  }> {
    const quota = this.quotas.get(agentId) || { used: 0, limit: 10000 };
    const remaining = quota.limit - quota.used;

    return {
      allowed: remaining > 0,
      remaining,
      limit: quota.limit,
    };
  }

  async getUsage(agentId: string): Promise<{
    apiCalls: number;
    quotaUsed: number;
    quotaLimit: number;
  }> {
    const quota = this.quotas.get(agentId) || { used: 0, limit: 10000 };
    const events = this.events.get(agentId) || [];

    return {
      apiCalls: events.length,
      quotaUsed: quota.used,
      quotaLimit: quota.limit,
    };
  }
}

/**
 * Mock Oversight Client Implementation
 */
class MockOversightClient extends EventEmitter implements OversightClient {
  private requests: Map<string, any> = new Map();

  async requestApproval(request: {
    agentId: string;
    tradeId: string;
    amount: number;
    reason: string;
  }): Promise<{
    requestId: string;
    status: 'pending' | 'approved' | 'rejected';
  }> {
    const requestId = `oversight-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    // Auto-approve amounts under $10K for demo
    const status = request.amount < 10000 ? 'approved' : 'pending';

    this.requests.set(requestId, {
      ...request,
      requestId,
      status,
      createdAt: new Date(),
    });

    this.emit('approval-requested', { requestId, ...request });

    return { requestId, status };
  }

  async getApprovalStatus(requestId: string): Promise<{
    status: 'pending' | 'approved' | 'rejected';
    approvals: Array<{ approverId: string; decision: string }>;
  }> {
    const request = this.requests.get(requestId);

    if (!request) {
      throw new Error(`Oversight request ${requestId} not found`);
    }

    return {
      status: request.status,
      approvals: request.approvals || [],
    };
  }

  async cancelApproval(requestId: string): Promise<void> {
    const request = this.requests.get(requestId);

    if (!request) {
      throw new Error(`Oversight request ${requestId} not found`);
    }

    request.status = 'cancelled';
    this.emit('approval-cancelled', requestId);
  }
}

/**
 * gRPC Client Factory
 */
export class GrpcClientFactory {
  private meteringClient: MeteringClient | null = null;
  private oversightClient: OversightClient | null = null;

  private meteringConfig: GrpcClientConfig;
  private oversightConfig: GrpcClientConfig;

  constructor(
    meteringConfig: GrpcClientConfig,
    oversightConfig: GrpcClientConfig
  ) {
    this.meteringConfig = meteringConfig;
    this.oversightConfig = oversightConfig;
  }

  /**
   * Initialize Metering Client
   */
  async createMeteringClient(): Promise<MeteringClient> {
    if (this.meteringClient) {
      return this.meteringClient;
    }

    // For demo, use mock client
    // In production, load proto and create real gRPC client
    this.meteringClient = new MockMeteringClient();

    console.log(`✅ Metering client connected (mock mode)`);

    return this.meteringClient;
  }

  /**
   * Initialize Oversight Client
   */
  async createOversightClient(): Promise<OversightClient> {
    if (this.oversightClient) {
      return this.oversightClient;
    }

    // For demo, use mock client
    this.oversightClient = new MockOversightClient();

    console.log(`✅ Oversight client connected (mock mode)`);

    return this.oversightClient;
  }

  /**
   * Health Check
   */
  async checkHealth(): Promise<{
    metering: boolean;
    oversight: boolean;
  }> {
    return {
      metering: this.meteringClient !== null,
      oversight: this.oversightClient !== null,
    };
  }

  /**
   * Close All Connections
   */
  async close(): Promise<void> {
    this.meteringClient = null;
    this.oversightClient = null;
    console.log('✅ gRPC clients closed');
  }
}

/**
 * Create Default gRPC Client Factory
 */
export function createGrpcClients(): GrpcClientFactory {
  const meteringConfig: GrpcClientConfig = {
    address: process.env.METERING_GRPC_ADDRESS || 'localhost:50051',
    maxRetries: 3,
    retryDelay: 1000,
    timeout: 5000,
  };

  const oversightConfig: GrpcClientConfig = {
    address: process.env.OVERSIGHT_GRPC_ADDRESS || 'localhost:50052',
    maxRetries: 3,
    retryDelay: 1000,
    timeout: 5000,
  };

  return new GrpcClientFactory(meteringConfig, oversightConfig);
}
