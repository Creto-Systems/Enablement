# SPARC Architecture: Trading Demo Backend

**Version:** 1.0.0
**Date:** 2025-12-26
**Status:** Draft
**Author:** SPARC Backend Architecture Agent

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Layered Architecture](#3-layered-architecture)
4. [Directory Structure](#4-directory-structure)
5. [Service Layer Design](#5-service-layer-design)
6. [Repository Pattern](#6-repository-pattern)
7. [gRPC Client Integration](#7-grpc-client-integration)
8. [WebSocket Architecture](#8-websocket-architecture)
9. [Middleware Pipeline](#9-middleware-pipeline)
10. [Error Handling Strategy](#10-error-handling-strategy)
11. [Data Flow Diagrams](#11-data-flow-diagrams)
12. [Technology Stack](#12-technology-stack)
13. [Design Patterns](#13-design-patterns)
14. [Performance Considerations](#14-performance-considerations)
15. [Security Architecture](#15-security-architecture)
16. [Testing Strategy](#16-testing-strategy)

---

## 1. Executive Summary

### 1.1 Purpose

This document defines the backend architecture for the Autonomous Portfolio Manager Trading Demo. The backend serves as the orchestration layer between the React frontend, creto-metering/oversight gRPC services, and local SQLite persistence.

### 1.2 Architecture Goals

- **Clean Architecture**: Separation of concerns with distinct layers (Controller → Service → Repository)
- **High Performance**: < 200ms P95 latency for GET requests, < 500ms for trade operations
- **Real-Time Updates**: WebSocket integration for sub-second dashboard updates
- **Testability**: 90%+ code coverage with comprehensive unit, integration, and e2e tests
- **Maintainability**: Modular design, clear dependencies, comprehensive documentation

### 1.3 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Node.js + Express | Rapid development, excellent ecosystem, TypeScript support |
| SQLite | Lightweight, zero-config, sufficient for demo persistence |
| gRPC clients | Required for creto-metering and creto-oversight integration |
| Socket.IO | Mature WebSocket library with auto-reconnect, room support |
| TypeORM | Type-safe ORM with repository pattern support |

---

## 2. Architecture Overview

### 2.1 System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (React)                        │
│              WebSocket Connection + HTTP REST API                │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Backend (Node.js/Express)                  │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌──────────┐    │
│  │Controllers│→ │ Services  │→ │Repositories│→ │ SQLite   │    │
│  └───────────┘  └───────────┘  └───────────┘  └──────────┘    │
│                       │                                          │
│                       ├────────────┐                             │
│                       ▼            ▼                             │
│                  ┌─────────┐  ┌─────────┐                       │
│                  │ gRPC    │  │WebSocket│                       │
│                  │ Clients │  │   Hub   │                       │
│                  └─────────┘  └─────────┘                       │
└─────────────────────┬──────────────────┬─────────────────────────┘
                      │                  │
        ┌─────────────┴────────┐        │
        ▼                      ▼        ▼
┌──────────────┐      ┌──────────────┐  WebSocket Clients
│creto-metering│      │creto-oversight│
│  (port 50051)│      │  (port 50052)│
└──────────────┘      └──────────────┘
```

### 2.2 Architecture Principles

1. **Dependency Inversion**: High-level modules don't depend on low-level modules; both depend on abstractions
2. **Single Responsibility**: Each module has one reason to change
3. **Open/Closed**: Open for extension, closed for modification
4. **Interface Segregation**: Clients shouldn't depend on interfaces they don't use
5. **Don't Repeat Yourself (DRY)**: Eliminate code duplication through abstraction

---

## 3. Layered Architecture

### 3.1 Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     PRESENTATION LAYER                          │
│   - Controllers (HTTP route handlers)                           │
│   - WebSocket event handlers                                    │
│   - Request/Response DTOs                                       │
│   - Validation middleware                                       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      BUSINESS LOGIC LAYER                       │
│   - Services (orchestration, business rules)                    │
│   - Domain models                                               │
│   - Use case implementations                                    │
│   - Service interfaces                                          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      DATA ACCESS LAYER                          │
│   - Repositories (data persistence)                             │
│   - Database entities                                           │
│   - Query builders                                              │
│   - Transaction management                                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                     INFRASTRUCTURE LAYER                        │
│   - gRPC clients (metering, oversight)                          │
│   - WebSocket hub (real-time updates)                           │
│   - External APIs (market data, Slack)                          │
│   - Logging, monitoring, caching                                │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Layer Responsibilities

#### Presentation Layer
- HTTP request routing
- Request validation and sanitization
- Response formatting
- Error response handling
- WebSocket connection management

#### Business Logic Layer
- Business rule enforcement (e.g., budget validation, risk assessment)
- Service orchestration (e.g., trade execution → metering → oversight)
- Domain event publishing (e.g., trade executed, quota exceeded)
- State machine management (e.g., agent status transitions)

#### Data Access Layer
- CRUD operations on database entities
- Query optimization
- Transaction management
- Data integrity enforcement
- Database migrations

#### Infrastructure Layer
- External service integration
- Cross-cutting concerns (logging, monitoring, caching)
- Configuration management
- Health checks and diagnostics

---

## 4. Directory Structure

```
src/server/
├── index.ts                          # Application entry point
├── app.ts                            # Express app configuration
├── server.ts                         # HTTP server setup
│
├── config/                           # Configuration management
│   ├── database.ts                   # SQLite/TypeORM config
│   ├── grpc.ts                       # gRPC client config
│   ├── websocket.ts                  # Socket.IO config
│   ├── environment.ts                # Environment variables (dotenv)
│   └── constants.ts                  # Application constants
│
├── controllers/                      # HTTP route handlers
│   ├── agent.controller.ts           # Agent CRUD endpoints
│   ├── trade.controller.ts           # Trade execution endpoints
│   ├── portfolio.controller.ts       # Portfolio query endpoints
│   ├── oversight.controller.ts       # Oversight request endpoints
│   ├── metering.controller.ts        # Usage/quota endpoints
│   ├── health.controller.ts          # Health check endpoints
│   └── index.ts                      # Controller registry
│
├── services/                         # Business logic layer
│   ├── agent.service.ts              # Agent lifecycle management
│   ├── trade.service.ts              # Trade orchestration
│   ├── portfolio.service.ts          # Portfolio calculations
│   ├── oversight.service.ts          # Approval workflow orchestration
│   ├── metering.service.ts           # Quota enforcement & tracking
│   ├── market-data.service.ts        # Market data fetching (synthetic)
│   ├── notification.service.ts       # Alerts & notifications
│   └── index.ts                      # Service registry (DI container)
│
├── repositories/                     # Data access layer
│   ├── agent.repository.ts           # Agent persistence
│   ├── trade.repository.ts           # Trade persistence
│   ├── position.repository.ts        # Position persistence
│   ├── approval-request.repository.ts# Approval persistence
│   ├── usage-log.repository.ts       # Usage tracking persistence
│   └── index.ts                      # Repository registry
│
├── grpc/                             # gRPC client layer
│   ├── clients/
│   │   ├── metering.client.ts        # creto-metering gRPC client
│   │   ├── oversight.client.ts       # creto-oversight gRPC client
│   │   └── index.ts                  # Client factory
│   ├── proto/                        # Protocol buffer definitions
│   │   ├── metering.proto
│   │   ├── oversight.proto
│   │   └── common.proto
│   └── types/                        # Generated TypeScript types
│       ├── metering_pb.d.ts
│       └── oversight_pb.d.ts
│
├── websocket/                        # WebSocket layer
│   ├── hub.ts                        # Socket.IO server setup
│   ├── handlers/                     # Event handlers
│   │   ├── connection.handler.ts     # Connection/disconnection
│   │   ├── subscription.handler.ts   # Channel subscriptions
│   │   └── index.ts
│   ├── channels.ts                   # Channel definitions
│   ├── events.ts                     # Event types
│   └── middleware/                   # WebSocket middleware
│       ├── auth.ts                   # JWT authentication
│       └── rate-limit.ts             # Connection rate limiting
│
├── middleware/                       # HTTP middleware
│   ├── auth.ts                       # JWT authentication
│   ├── validation.ts                 # Request validation (Joi/Zod)
│   ├── error-handler.ts              # Global error handler
│   ├── rate-limiter.ts               # Rate limiting (express-rate-limit)
│   ├── cors.ts                       # CORS configuration
│   ├── helmet.ts                     # Security headers (helmet.js)
│   ├── logger.ts                     # Request logging (morgan)
│   └── index.ts                      # Middleware registry
│
├── entities/                         # Database entities (TypeORM)
│   ├── agent.entity.ts
│   ├── portfolio.entity.ts
│   ├── position.entity.ts
│   ├── trade.entity.ts
│   ├── quota-usage.entity.ts
│   ├── usage-log.entity.ts
│   ├── approval-request.entity.ts
│   ├── user.entity.ts
│   ├── audit-log.entity.ts
│   └── index.ts
│
├── dtos/                             # Data Transfer Objects
│   ├── requests/                     # Request DTOs
│   │   ├── create-agent.dto.ts
│   │   ├── update-agent.dto.ts
│   │   ├── execute-trade.dto.ts
│   │   └── configure-alerts.dto.ts
│   ├── responses/                    # Response DTOs
│   │   ├── agent.response.ts
│   │   ├── portfolio.response.ts
│   │   ├── trade.response.ts
│   │   └── usage.response.ts
│   └── index.ts
│
├── types/                            # TypeScript type definitions
│   ├── agent.types.ts                # Agent-related types
│   ├── trade.types.ts                # Trade-related types
│   ├── metering.types.ts             # Metering-related types
│   ├── oversight.types.ts            # Oversight-related types
│   ├── websocket.types.ts            # WebSocket event types
│   ├── express.d.ts                  # Express type extensions
│   └── index.ts
│
├── utils/                            # Utility functions
│   ├── logger.ts                     # Winston logger setup
│   ├── errors.ts                     # Custom error classes
│   ├── validators.ts                 # Validation helpers
│   ├── formatters.ts                 # Data formatters (currency, date)
│   ├── calculations.ts               # Financial calculations (ROI, Sharpe)
│   ├── retry.ts                      # Retry logic with backoff
│   ├── idempotency.ts                # Idempotency key handling
│   └── index.ts
│
├── routes/                           # Route definitions
│   ├── agent.routes.ts
│   ├── trade.routes.ts
│   ├── portfolio.routes.ts
│   ├── oversight.routes.ts
│   ├── metering.routes.ts
│   ├── health.routes.ts
│   └── index.ts
│
├── migrations/                       # Database migrations
│   ├── 1640000000000-CreateAgentTable.ts
│   ├── 1640000001000-CreateTradeTable.ts
│   └── ...
│
└── seeds/                            # Database seed data
    ├── demo-agents.seed.ts
    ├── demo-trades.seed.ts
    └── index.ts
```

---

## 5. Service Layer Design

### 5.1 Service Interface Pattern

All services implement well-defined interfaces for testability and dependency injection.

#### Example: AgentService Interface

```typescript
// services/agent.service.ts
import { Agent } from '../entities/agent.entity';
import { CreateAgentDTO, UpdateAgentDTO, AgentFilter } from '../dtos';

export interface IAgentService {
  /**
   * Create a new trading agent
   * @throws {ValidationError} if budget exceeds limit
   * @throws {QuotaExceededError} if user quota exceeded
   */
  createAgent(dto: CreateAgentDTO, userId: string): Promise<Agent>;

  /**
   * Find agent by ID
   * @throws {NotFoundError} if agent doesn't exist
   */
  findById(agentId: string, userId: string): Promise<Agent>;

  /**
   * List agents with optional filtering
   */
  findAll(filter: AgentFilter, userId: string): Promise<Agent[]>;

  /**
   * Update agent configuration
   * @throws {ValidationError} if update violates constraints
   */
  updateAgent(agentId: string, dto: UpdateAgentDTO, userId: string): Promise<Agent>;

  /**
   * Pause agent (stop trading, maintain positions)
   */
  pauseAgent(agentId: string, userId: string): Promise<Agent>;

  /**
   * Stop agent (liquidate positions)
   */
  stopAgent(agentId: string, userId: string): Promise<Agent>;

  /**
   * Delete agent (soft delete with 90-day retention)
   */
  deleteAgent(agentId: string, userId: string): Promise<void>;

  /**
   * Validate agent can trade (status, quota, budget checks)
   */
  canTrade(agentId: string): Promise<boolean>;
}
```

#### Example: AgentService Implementation

```typescript
// services/agent.service.ts
import { inject, injectable } from 'tsyringe';
import { IAgentRepository } from '../repositories/agent.repository';
import { IMeteringService } from './metering.service';
import { IPortfolioService } from './portfolio.service';
import { ValidationError, NotFoundError } from '../utils/errors';
import { Logger } from '../utils/logger';

@injectable()
export class AgentService implements IAgentService {
  constructor(
    @inject('AgentRepository') private agentRepo: IAgentRepository,
    @inject('MeteringService') private meteringService: IMeteringService,
    @inject('PortfolioService') private portfolioService: IPortfolioService,
    @inject('Logger') private logger: Logger
  ) {}

  async createAgent(dto: CreateAgentDTO, userId: string): Promise<Agent> {
    this.logger.info('Creating agent', { userId, dto });

    // Validation
    if (dto.initialBudget > 100000) {
      throw new ValidationError('Budget cannot exceed $100,000');
    }

    // Create agent entity
    const agent = await this.agentRepo.create({
      ...dto,
      userId,
      status: 'paused',
    });

    // Initialize portfolio
    await this.portfolioService.initializePortfolio(agent.id, dto.initialBudget);

    // Initialize quota tracking
    await this.meteringService.initializeQuota(agent.id, dto.quotaConfig);

    this.logger.info('Agent created successfully', { agentId: agent.id });
    return agent;
  }

  async findById(agentId: string, userId: string): Promise<Agent> {
    const agent = await this.agentRepo.findById(agentId);

    if (!agent || agent.userId !== userId) {
      throw new NotFoundError('Agent not found');
    }

    return agent;
  }

  async canTrade(agentId: string): Promise<boolean> {
    const agent = await this.agentRepo.findById(agentId);

    if (!agent || agent.status !== 'active') {
      return false;
    }

    // Check quota availability
    const hasQuota = await this.meteringService.checkQuota(agentId);
    if (!hasQuota) {
      this.logger.warn('Agent quota exhausted', { agentId });
      return false;
    }

    // Check budget availability
    const portfolio = await this.portfolioService.getPortfolio(agentId);
    if (portfolio.cash < 100) { // Minimum $100 for trading
      this.logger.warn('Agent has insufficient cash', { agentId });
      return false;
    }

    return true;
  }

  // ... other methods
}
```

### 5.2 Service Orchestration: TradeService

The TradeService orchestrates complex workflows involving multiple services.

```typescript
// services/trade.service.ts
import { inject, injectable } from 'tsyringe';
import { ITradeRepository } from '../repositories/trade.repository';
import { IMeteringService } from './metering.service';
import { IOversightService } from './oversight.service';
import { IPortfolioService } from './portfolio.service';
import { IMarketDataService } from './market-data.service';
import { INotificationService } from './notification.service';
import { Trade, TradeStatus } from '../entities/trade.entity';
import { ExecuteTradeDTO } from '../dtos';

@injectable()
export class TradeService implements ITradeService {
  constructor(
    @inject('TradeRepository') private tradeRepo: ITradeRepository,
    @inject('MeteringService') private meteringService: IMeteringService,
    @inject('OversightService') private oversightService: IOversightService,
    @inject('PortfolioService') private portfolioService: IPortfolioService,
    @inject('MarketDataService') private marketDataService: IMarketDataService,
    @inject('NotificationService') private notificationService: INotificationService
  ) {}

  /**
   * Execute trade workflow:
   * 1. Validate trade parameters
   * 2. Check quota (metering)
   * 3. Get current market price
   * 4. Check if oversight required
   * 5a. If oversight required: create approval request, pause
   * 5b. If no oversight: execute immediately
   * 6. Update portfolio
   * 7. Log usage event
   * 8. Notify via WebSocket
   */
  async executeTrade(agentId: string, dto: ExecuteTradeDTO): Promise<Trade> {
    // Step 1: Create pending trade
    const trade = await this.tradeRepo.create({
      agentId,
      ...dto,
      status: TradeStatus.PENDING,
      submittedAt: new Date(),
    });

    try {
      // Step 2: Check quota
      const quotaCheck = await this.meteringService.checkQuota(agentId);
      if (!quotaCheck.allowed) {
        trade.status = TradeStatus.FAILED;
        trade.failureReason = 'Quota exhausted';
        await this.tradeRepo.update(trade.id, trade);
        throw new QuotaExceededError('API quota exhausted');
      }

      // Step 3: Get market price
      const currentPrice = await this.marketDataService.getCurrentPrice(dto.symbol);
      const estimatedCost = currentPrice * dto.quantity;

      // Step 4: Check oversight triggers
      const requiresOversight = await this.oversightService.requiresApproval(
        agentId,
        estimatedCost
      );

      if (requiresOversight) {
        // Step 5a: Create approval request
        const approvalRequest = await this.oversightService.createApprovalRequest({
          agentId,
          tradeId: trade.id,
          estimatedValue: estimatedCost,
          tradeDetails: dto,
        });

        trade.status = TradeStatus.PENDING_APPROVAL;
        trade.approvalRequestId = approvalRequest.id;
        await this.tradeRepo.update(trade.id, trade);

        // Notify user
        await this.notificationService.sendOversightAlert(agentId, approvalRequest);

        return trade;
      }

      // Step 5b: Execute immediately
      return await this.executeTradeImmediate(trade, currentPrice);

    } catch (error) {
      trade.status = TradeStatus.FAILED;
      trade.failureReason = error.message;
      await this.tradeRepo.update(trade.id, trade);
      throw error;
    }
  }

  private async executeTradeImmediate(trade: Trade, price: number): Promise<Trade> {
    const executedAt = new Date();
    const totalCost = price * trade.quantity + this.calculateFees(price * trade.quantity);

    // Update trade
    trade.status = TradeStatus.COMPLETED;
    trade.executedAt = executedAt;
    trade.executedPrice = price;
    trade.totalCost = totalCost;
    await this.tradeRepo.update(trade.id, trade);

    // Update portfolio
    await this.portfolioService.applyTrade(trade);

    // Log metering event
    await this.meteringService.ingestEvent({
      agentId: trade.agentId,
      eventType: 'trade.executed',
      cost: 0.10, // $0.10 per trade
      metadata: { tradeId: trade.id, symbol: trade.symbol },
    });

    // Notify via WebSocket
    await this.notificationService.emitTradeExecuted(trade);

    return trade;
  }

  private calculateFees(amount: number): number {
    return Math.max(1.0, amount * 0.001); // Min $1, or 0.1%
  }

  // ... other methods
}
```

---

## 6. Repository Pattern

### 6.1 Repository Interface

Repositories abstract database operations and provide a clean API for data access.

```typescript
// repositories/agent.repository.ts
import { Agent } from '../entities/agent.entity';

export interface AgentFilter {
  status?: string[];
  strategy?: string[];
  createdAfter?: Date;
  createdBefore?: Date;
  search?: string; // Search by name
}

export interface IAgentRepository {
  /**
   * Create new agent
   */
  create(data: Partial<Agent>): Promise<Agent>;

  /**
   * Find agent by ID
   */
  findById(id: string): Promise<Agent | null>;

  /**
   * Find all agents matching filter
   */
  findAll(filter?: AgentFilter): Promise<Agent[]>;

  /**
   * Update agent
   */
  update(id: string, data: Partial<Agent>): Promise<Agent>;

  /**
   * Soft delete agent
   */
  delete(id: string): Promise<void>;

  /**
   * Count agents by user
   */
  countByUser(userId: string): Promise<number>;

  /**
   * Find agents requiring action (e.g., quota alerts)
   */
  findRequiringAction(): Promise<Agent[]>;
}
```

### 6.2 TypeORM Repository Implementation

```typescript
// repositories/agent.repository.ts
import { injectable } from 'tsyringe';
import { Repository, DataSource } from 'typeorm';
import { Agent } from '../entities/agent.entity';

@injectable()
export class AgentRepository implements IAgentRepository {
  private repository: Repository<Agent>;

  constructor(dataSource: DataSource) {
    this.repository = dataSource.getRepository(Agent);
  }

  async create(data: Partial<Agent>): Promise<Agent> {
    const agent = this.repository.create(data);
    return await this.repository.save(agent);
  }

  async findById(id: string): Promise<Agent | null> {
    return await this.repository.findOne({
      where: { id },
      relations: ['portfolio', 'quotaUsage'],
    });
  }

  async findAll(filter?: AgentFilter): Promise<Agent[]> {
    const query = this.repository.createQueryBuilder('agent')
      .leftJoinAndSelect('agent.portfolio', 'portfolio')
      .leftJoinAndSelect('agent.quotaUsage', 'quotaUsage');

    if (filter?.status) {
      query.andWhere('agent.status IN (:...status)', { status: filter.status });
    }

    if (filter?.strategy) {
      query.andWhere('agent.strategy IN (:...strategy)', { strategy: filter.strategy });
    }

    if (filter?.createdAfter) {
      query.andWhere('agent.createdAt >= :createdAfter', { createdAfter: filter.createdAfter });
    }

    if (filter?.createdBefore) {
      query.andWhere('agent.createdAt <= :createdBefore', { createdBefore: filter.createdBefore });
    }

    if (filter?.search) {
      query.andWhere('agent.name ILIKE :search', { search: `%${filter.search}%` });
    }

    query.orderBy('agent.createdAt', 'DESC');

    return await query.getMany();
  }

  async update(id: string, data: Partial<Agent>): Promise<Agent> {
    await this.repository.update(id, data);
    return await this.findById(id);
  }

  async delete(id: string): Promise<void> {
    await this.repository.softDelete(id); // Soft delete (sets deletedAt)
  }

  async countByUser(userId: string): Promise<number> {
    return await this.repository.count({
      where: { userId, deletedAt: null },
    });
  }

  async findRequiringAction(): Promise<Agent[]> {
    // Find agents with quota > 80% or budget near exhaustion
    return await this.repository.createQueryBuilder('agent')
      .leftJoinAndSelect('agent.quotaUsage', 'quotaUsage')
      .where('quotaUsage.apiCallsUsed / quotaUsage.apiCallsQuota >= 0.8')
      .orWhere('quotaUsage.budgetSpent / quotaUsage.budgetQuota >= 0.8')
      .getMany();
  }
}
```

---

## 7. gRPC Client Integration

### 7.1 Metering Client

```typescript
// grpc/clients/metering.client.ts
import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { injectable } from 'tsyringe';
import { Logger } from '../../utils/logger';
import { retry } from '../../utils/retry';

export interface BillableEvent {
  agentId: string;
  eventType: string;
  cost: number;
  timestamp?: Date;
  metadata?: Record<string, any>;
}

export interface QuotaStatus {
  allowed: boolean;
  remaining: {
    apiCalls: number;
    budget: number;
  };
  resetDate: Date;
}

export interface UsageMetrics {
  agentId: string;
  period: string;
  totalCost: number;
  totalCalls: number;
  breakdown: Record<string, number>;
}

@injectable()
export class MeteringClient {
  private client: any;
  private logger: Logger;

  constructor() {
    this.logger = new Logger('MeteringClient');
    this.initializeClient();
  }

  private initializeClient(): void {
    const packageDefinition = protoLoader.loadSync(
      './grpc/proto/metering.proto',
      {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
      }
    );

    const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
    const MeteringService = protoDescriptor.metering.MeteringService;

    this.client = new MeteringService(
      'localhost:50051',
      grpc.credentials.createInsecure(), // Use TLS in production
      {
        'grpc.keepalive_time_ms': 10000,
        'grpc.keepalive_timeout_ms': 5000,
      }
    );

    this.logger.info('Metering gRPC client initialized');
  }

  /**
   * Ingest billable event
   * Retries: 3 attempts with exponential backoff
   */
  async ingestEvent(event: BillableEvent): Promise<void> {
    return retry(async () => {
      return new Promise((resolve, reject) => {
        this.client.IngestEvent(
          {
            agent_id: event.agentId,
            event_type: event.eventType,
            cost: event.cost,
            timestamp: event.timestamp?.toISOString() || new Date().toISOString(),
            metadata: JSON.stringify(event.metadata || {}),
          },
          (error: any, response: any) => {
            if (error) {
              this.logger.error('Failed to ingest event', { error, event });
              reject(error);
            } else {
              this.logger.debug('Event ingested', { event, response });
              resolve();
            }
          }
        );
      });
    }, { maxAttempts: 3, backoffMs: 100 });
  }

  /**
   * Check quota availability
   */
  async checkQuota(agentId: string): Promise<QuotaStatus> {
    return new Promise((resolve, reject) => {
      this.client.CheckQuota(
        { agent_id: agentId },
        (error: any, response: any) => {
          if (error) {
            this.logger.error('Failed to check quota', { error, agentId });
            reject(error);
          } else {
            resolve({
              allowed: response.allowed,
              remaining: {
                apiCalls: response.remaining_api_calls,
                budget: response.remaining_budget,
              },
              resetDate: new Date(response.reset_date),
            });
          }
        }
      );
    });
  }

  /**
   * Get usage metrics for period
   */
  async getUsage(agentId: string, period: string): Promise<UsageMetrics> {
    return new Promise((resolve, reject) => {
      this.client.GetUsage(
        { agent_id: agentId, period },
        (error: any, response: any) => {
          if (error) {
            this.logger.error('Failed to get usage', { error, agentId, period });
            reject(error);
          } else {
            resolve({
              agentId: response.agent_id,
              period: response.period,
              totalCost: response.total_cost,
              totalCalls: response.total_calls,
              breakdown: JSON.parse(response.breakdown || '{}'),
            });
          }
        }
      );
    });
  }

  /**
   * Graceful shutdown
   */
  async close(): Promise<void> {
    return new Promise((resolve) => {
      this.client.close(() => {
        this.logger.info('Metering gRPC client closed');
        resolve();
      });
    });
  }
}
```

### 7.2 Oversight Client

```typescript
// grpc/clients/oversight.client.ts
import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { injectable } from 'tsyringe';
import { Logger } from '../../utils/logger';

export interface OversightRequest {
  agentId: string;
  tradeId: string;
  tradeDetails: any;
  reason: string;
  riskAssessment: any;
  expiresAt: Date;
}

export interface RequestId {
  id: string;
}

export interface Decision {
  result: 'approved' | 'rejected';
  approverId: string;
  approverName: string;
  reason?: string;
  timestamp: Date;
}

export interface RequestStatus {
  id: string;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  createdAt: Date;
  decidedAt?: Date;
  decision?: Decision;
}

@injectable()
export class OversightClient {
  private client: any;
  private logger: Logger;

  constructor() {
    this.logger = new Logger('OversightClient');
    this.initializeClient();
  }

  private initializeClient(): void {
    const packageDefinition = protoLoader.loadSync(
      './grpc/proto/oversight.proto',
      {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true,
      }
    );

    const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
    const OversightService = protoDescriptor.oversight.OversightService;

    this.client = new OversightService(
      'localhost:50052',
      grpc.credentials.createInsecure(),
      {
        'grpc.keepalive_time_ms': 10000,
        'grpc.keepalive_timeout_ms': 5000,
      }
    );

    this.logger.info('Oversight gRPC client initialized');
  }

  /**
   * Create approval request
   */
  async createRequest(request: OversightRequest): Promise<RequestId> {
    return new Promise((resolve, reject) => {
      this.client.CreateRequest(
        {
          agent_id: request.agentId,
          trade_id: request.tradeId,
          trade_details: JSON.stringify(request.tradeDetails),
          reason: request.reason,
          risk_assessment: JSON.stringify(request.riskAssessment),
          expires_at: request.expiresAt.toISOString(),
        },
        (error: any, response: any) => {
          if (error) {
            this.logger.error('Failed to create oversight request', { error, request });
            reject(error);
          } else {
            this.logger.info('Oversight request created', { requestId: response.id });
            resolve({ id: response.id });
          }
        }
      );
    });
  }

  /**
   * Record approval decision
   */
  async approveRequest(id: string, decision: Decision): Promise<void> {
    return new Promise((resolve, reject) => {
      this.client.ApproveRequest(
        {
          request_id: id,
          decision: {
            result: decision.result,
            approver_id: decision.approverId,
            approver_name: decision.approverName,
            reason: decision.reason,
            timestamp: decision.timestamp.toISOString(),
          },
        },
        (error: any, response: any) => {
          if (error) {
            this.logger.error('Failed to approve request', { error, id, decision });
            reject(error);
          } else {
            this.logger.info('Request decision recorded', { id, decision: decision.result });
            resolve();
          }
        }
      );
    });
  }

  /**
   * Get request status
   */
  async getRequestStatus(id: string): Promise<RequestStatus> {
    return new Promise((resolve, reject) => {
      this.client.GetRequestStatus(
        { request_id: id },
        (error: any, response: any) => {
          if (error) {
            this.logger.error('Failed to get request status', { error, id });
            reject(error);
          } else {
            resolve({
              id: response.id,
              status: response.status,
              createdAt: new Date(response.created_at),
              decidedAt: response.decided_at ? new Date(response.decided_at) : undefined,
              decision: response.decision ? {
                result: response.decision.result,
                approverId: response.decision.approver_id,
                approverName: response.decision.approver_name,
                reason: response.decision.reason,
                timestamp: new Date(response.decision.timestamp),
              } : undefined,
            });
          }
        }
      );
    });
  }

  async close(): Promise<void> {
    return new Promise((resolve) => {
      this.client.close(() => {
        this.logger.info('Oversight gRPC client closed');
        resolve();
      });
    });
  }
}
```

---

## 8. WebSocket Architecture

### 8.1 WebSocket Hub

```typescript
// websocket/hub.ts
import { Server as HttpServer } from 'http';
import { Server, Socket } from 'socket.io';
import { injectable } from 'tsyringe';
import { Logger } from '../utils/logger';
import { verifyJWT } from '../middleware/auth';

export interface SocketData {
  userId: string;
  agentId?: string;
}

@injectable()
export class WebSocketHub {
  private io: Server;
  private logger: Logger;

  constructor(httpServer: HttpServer) {
    this.logger = new Logger('WebSocketHub');
    this.io = new Server(httpServer, {
      cors: {
        origin: process.env.FRONTEND_URL || 'http://localhost:3000',
        credentials: true,
      },
      pingTimeout: 60000,
      pingInterval: 25000,
    });

    this.setupMiddleware();
    this.setupConnectionHandler();
  }

  private setupMiddleware(): void {
    // JWT authentication middleware
    this.io.use(async (socket: Socket, next) => {
      const token = socket.handshake.auth.token;

      if (!token) {
        return next(new Error('Authentication token required'));
      }

      try {
        const decoded = await verifyJWT(token);
        socket.data.userId = decoded.userId;
        this.logger.debug('WebSocket authenticated', { userId: decoded.userId });
        next();
      } catch (error) {
        this.logger.error('WebSocket authentication failed', { error });
        next(new Error('Invalid token'));
      }
    });

    // Rate limiting middleware
    this.io.use((socket: Socket, next) => {
      // Implement rate limiting logic
      // For example: max 100 connections per user
      next();
    });
  }

  private setupConnectionHandler(): void {
    this.io.on('connection', (socket: Socket) => {
      this.logger.info('Client connected', { socketId: socket.id, userId: socket.data.userId });

      // Subscribe to agent channel
      socket.on('subscribe:agent', (agentId: string) => {
        socket.join(`agent:${agentId}`);
        socket.data.agentId = agentId;
        this.logger.debug('Subscribed to agent channel', { socketId: socket.id, agentId });
      });

      // Unsubscribe from agent channel
      socket.on('unsubscribe:agent', (agentId: string) => {
        socket.leave(`agent:${agentId}`);
        this.logger.debug('Unsubscribed from agent channel', { socketId: socket.id, agentId });
      });

      // Subscribe to user-wide events
      socket.join(`user:${socket.data.userId}`);

      // Heartbeat
      socket.on('ping', () => {
        socket.emit('pong');
      });

      // Disconnection
      socket.on('disconnect', (reason) => {
        this.logger.info('Client disconnected', { socketId: socket.id, reason });
      });

      // Error handling
      socket.on('error', (error) => {
        this.logger.error('WebSocket error', { socketId: socket.id, error });
      });
    });
  }

  /**
   * Emit portfolio update to agent channel
   */
  emitPortfolioUpdate(agentId: string, data: any): void {
    this.io.to(`agent:${agentId}`).emit('portfolio:update', {
      type: 'portfolio.update',
      agentId,
      timestamp: new Date().toISOString(),
      data,
    });
  }

  /**
   * Emit trade executed event
   */
  emitTradeExecuted(agentId: string, trade: any): void {
    this.io.to(`agent:${agentId}`).emit('trade:executed', {
      type: 'trade.executed',
      agentId,
      timestamp: new Date().toISOString(),
      data: trade,
    });
  }

  /**
   * Emit quota alert
   */
  emitQuotaAlert(agentId: string, alert: any): void {
    this.io.to(`agent:${agentId}`).emit('usage:alert', {
      type: 'usage.alert',
      agentId,
      timestamp: new Date().toISOString(),
      data: alert,
    });
  }

  /**
   * Emit oversight request
   */
  emitOversightRequest(userId: string, request: any): void {
    this.io.to(`user:${userId}`).emit('oversight:request', {
      type: 'oversight.request',
      timestamp: new Date().toISOString(),
      data: request,
    });
  }

  /**
   * Get connection count for monitoring
   */
  getConnectionCount(): number {
    return this.io.sockets.sockets.size;
  }

  /**
   * Graceful shutdown
   */
  async close(): Promise<void> {
    return new Promise((resolve) => {
      this.io.close(() => {
        this.logger.info('WebSocket server closed');
        resolve();
      });
    });
  }
}
```

---

## 9. Middleware Pipeline

### 9.1 Request Flow

```
┌────────────────────────────────────────────────────────────┐
│                      HTTP Request                          │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  1. CORS Middleware (helmet.js)                            │
│     - Set CORS headers                                     │
│     - Security headers (X-Content-Type-Options, etc.)      │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  2. Request Logger (morgan)                                │
│     - Log: method, path, status, response time             │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  3. Rate Limiter (express-rate-limit)                      │
│     - Check: IP-based rate limit                           │
│     - Return 429 if exceeded                               │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  4. Authentication (JWT)                                   │
│     - Verify JWT from Authorization header                 │
│     - Attach user to req.user                              │
│     - Return 401 if invalid                                │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  5. Request Validation (Joi/Zod)                           │
│     - Validate: body, query, params                        │
│     - Return 400 if validation fails                       │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  6. Controller                                             │
│     - Call service layer                                   │
│     - Format response                                      │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│  7. Error Handler (global)                                 │
│     - Catch errors                                         │
│     - Format error response                                │
│     - Log error with correlation ID                        │
└────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────┐
│                      HTTP Response                         │
└────────────────────────────────────────────────────────────┘
```

### 9.2 Middleware Implementations

#### Authentication Middleware

```typescript
// middleware/auth.ts
import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { UnauthorizedError } from '../utils/errors';

export interface JWTPayload {
  userId: string;
  email: string;
  role: string;
}

declare global {
  namespace Express {
    interface Request {
      user?: JWTPayload;
    }
  }
}

export function authMiddleware(req: Request, res: Response, next: NextFunction): void {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw new UnauthorizedError('Missing or invalid Authorization header');
  }

  const token = authHeader.substring(7);

  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload;
    req.user = decoded;
    next();
  } catch (error) {
    throw new UnauthorizedError('Invalid or expired token');
  }
}

export function optionalAuth(req: Request, res: Response, next: NextFunction): void {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return next();
  }

  const token = authHeader.substring(7);

  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload;
    req.user = decoded;
  } catch (error) {
    // Ignore invalid tokens for optional auth
  }

  next();
}

export async function verifyJWT(token: string): Promise<JWTPayload> {
  return new Promise((resolve, reject) => {
    jwt.verify(token, process.env.JWT_SECRET!, (err, decoded) => {
      if (err) {
        reject(err);
      } else {
        resolve(decoded as JWTPayload);
      }
    });
  });
}
```

#### Validation Middleware

```typescript
// middleware/validation.ts
import { Request, Response, NextFunction } from 'express';
import Joi from 'joi';
import { ValidationError } from '../utils/errors';

export function validate(schema: Joi.ObjectSchema) {
  return (req: Request, res: Response, next: NextFunction) => {
    const { error, value } = schema.validate(
      {
        body: req.body,
        query: req.query,
        params: req.params,
      },
      { abortEarly: false }
    );

    if (error) {
      const details = error.details.map((detail) => ({
        field: detail.path.join('.'),
        message: detail.message,
      }));

      throw new ValidationError('Validation failed', details);
    }

    // Replace with validated values
    req.body = value.body || req.body;
    req.query = value.query || req.query;
    req.params = value.params || req.params;

    next();
  };
}

// Example schema
export const createAgentSchema = Joi.object({
  body: Joi.object({
    name: Joi.string().min(3).max(100).required(),
    strategy: Joi.string().valid('balanced', 'aggressive', 'conservative').required(),
    initialBudget: Joi.number().min(100).max(100000).required(),
    riskTolerance: Joi.number().integer().min(1).max(10).required(),
    quotaConfig: Joi.object({
      apiCalls: Joi.number().integer().min(1000).max(100000).default(10000),
      budgetPerMonth: Joi.number().min(1000).max(100000).default(10000),
    }).optional(),
  }),
});
```

#### Error Handler Middleware

```typescript
// middleware/error-handler.ts
import { Request, Response, NextFunction } from 'express';
import { Logger } from '../utils/logger';
import {
  AppError,
  ValidationError,
  NotFoundError,
  UnauthorizedError,
  ForbiddenError,
  ConflictError,
  QuotaExceededError,
} from '../utils/errors';

const logger = new Logger('ErrorHandler');

export function errorHandler(
  error: Error,
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Generate correlation ID for tracking
  const correlationId = req.headers['x-correlation-id'] || generateCorrelationId();

  // Log error with context
  logger.error('Request error', {
    correlationId,
    error: error.message,
    stack: error.stack,
    method: req.method,
    path: req.path,
    userId: req.user?.userId,
  });

  // Handle known error types
  if (error instanceof ValidationError) {
    return res.status(400).json({
      error: 'Validation Error',
      message: error.message,
      details: error.details,
      correlationId,
    });
  }

  if (error instanceof UnauthorizedError) {
    return res.status(401).json({
      error: 'Unauthorized',
      message: error.message,
      correlationId,
    });
  }

  if (error instanceof ForbiddenError) {
    return res.status(403).json({
      error: 'Forbidden',
      message: error.message,
      correlationId,
    });
  }

  if (error instanceof NotFoundError) {
    return res.status(404).json({
      error: 'Not Found',
      message: error.message,
      correlationId,
    });
  }

  if (error instanceof ConflictError) {
    return res.status(409).json({
      error: 'Conflict',
      message: error.message,
      correlationId,
    });
  }

  if (error instanceof QuotaExceededError) {
    return res.status(402).json({
      error: 'Quota Exceeded',
      message: error.message,
      correlationId,
    });
  }

  if (error instanceof AppError) {
    return res.status(error.statusCode).json({
      error: error.name,
      message: error.message,
      correlationId,
    });
  }

  // Unknown error - return generic 500
  return res.status(500).json({
    error: 'Internal Server Error',
    message: process.env.NODE_ENV === 'production'
      ? 'An unexpected error occurred'
      : error.message,
    correlationId,
  });
}

function generateCorrelationId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}
```

---

## 10. Error Handling Strategy

### 10.1 Custom Error Classes

```typescript
// utils/errors.ts

/**
 * Base application error
 */
export class AppError extends Error {
  constructor(
    public statusCode: number,
    public message: string,
    public details?: any
  ) {
    super(message);
    this.name = this.constructor.name;
    Error.captureStackTrace(this, this.constructor);
  }
}

/**
 * 400 Bad Request
 */
export class ValidationError extends AppError {
  constructor(message: string, details?: any) {
    super(400, message, details);
  }
}

/**
 * 401 Unauthorized
 */
export class UnauthorizedError extends AppError {
  constructor(message: string = 'Unauthorized') {
    super(401, message);
  }
}

/**
 * 403 Forbidden
 */
export class ForbiddenError extends AppError {
  constructor(message: string = 'Forbidden') {
    super(403, message);
  }
}

/**
 * 404 Not Found
 */
export class NotFoundError extends AppError {
  constructor(message: string = 'Resource not found') {
    super(404, message);
  }
}

/**
 * 409 Conflict
 */
export class ConflictError extends AppError {
  constructor(message: string) {
    super(409, message);
  }
}

/**
 * 402 Payment Required (Quota Exceeded)
 */
export class QuotaExceededError extends AppError {
  constructor(message: string = 'Quota exceeded') {
    super(402, message);
  }
}

/**
 * 500 Internal Server Error
 */
export class InternalError extends AppError {
  constructor(message: string = 'Internal server error') {
    super(500, message);
  }
}

/**
 * 503 Service Unavailable
 */
export class ServiceUnavailableError extends AppError {
  constructor(message: string = 'Service temporarily unavailable') {
    super(503, message);
  }
}
```

### 10.2 Retry Logic with Exponential Backoff

```typescript
// utils/retry.ts
import { Logger } from './logger';

export interface RetryOptions {
  maxAttempts?: number;
  backoffMs?: number;
  maxBackoffMs?: number;
  shouldRetry?: (error: any) => boolean;
}

const logger = new Logger('RetryUtil');

/**
 * Retry function with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {}
): Promise<T> {
  const {
    maxAttempts = 3,
    backoffMs = 100,
    maxBackoffMs = 5000,
    shouldRetry = () => true,
  } = options;

  let attempt = 0;
  let lastError: any;

  while (attempt < maxAttempts) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      attempt++;

      if (attempt >= maxAttempts || !shouldRetry(error)) {
        throw error;
      }

      const delay = Math.min(backoffMs * Math.pow(2, attempt - 1), maxBackoffMs);
      logger.warn(`Retry attempt ${attempt}/${maxAttempts} after ${delay}ms`, { error });

      await sleep(delay);
    }
  }

  throw lastError;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
```

---

## 11. Data Flow Diagrams

### 11.1 Trade Execution Flow

```
┌─────────┐                                                    ┌─────────┐
│ Client  │                                                    │Frontend │
└────┬────┘                                                    └────┬────┘
     │                                                              │
     │ POST /api/v1/agents/{agentId}/trades                        │
     │ {symbol: AAPL, action: buy, quantity: 10}                   │
     ▼                                                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          Express Backend                             │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 1. TradeController                                             │  │
│  │    - Validate request (Joi schema)                             │  │
│  │    - Extract user from JWT                                     │  │
│  │    - Call TradeService.executeTrade()                          │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 2. TradeService                                                │  │
│  │    - Create pending Trade entity                               │  │
│  │    - Call MeteringService.checkQuota()                         │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 3. MeteringService                                             │  │
│  │    - Call MeteringClient.checkQuota()                          │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 4. MeteringClient (gRPC)                                       │  │
│  │    - gRPC call to creto-metering:50051                         │  │
│  │    - Return: {allowed: true, remaining: 9234}                  │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 5. MarketDataService                                           │  │
│  │    - Get current price for AAPL: $150.25                       │  │
│  │    - Calculate estimated cost: 10 * $150.25 = $1502.50         │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 6. OversightService                                            │  │
│  │    - Check if trade requires approval ($1502.50 < $50K)        │  │
│  │    - Result: No approval needed                                │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 7. TradeService.executeTradeImmediate()                        │  │
│  │    - Update trade: status=COMPLETED, price=$150.25             │  │
│  │    - Save to database via TradeRepository                      │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 8. PortfolioService                                            │  │
│  │    - Update position: +10 AAPL @ $150.25                       │  │
│  │    - Deduct cash: $1502.50 + $1.50 fees                        │  │
│  │    - Save via PositionRepository                               │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 9. MeteringService.ingestEvent()                               │  │
│  │    - Log usage: {type: trade.executed, cost: $0.10}            │  │
│  │    - gRPC call to creto-metering                               │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│                               ▼                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 10. WebSocketHub                                               │  │
│  │    - Emit trade:executed event to agent:${agentId} channel     │  │
│  │    - Emit portfolio:update event with new balance              │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                               │                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 11. TradeController                                            │  │
│  │    - Format response DTO                                       │  │
│  │    - Return 200 OK with trade details                          │  │
│  └────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
     │                                                              │
     │ Response: {tradeId, status: completed, executedPrice...}     │
     ▼                                                              ▼
┌─────────┐                                                    ┌─────────┐
│ Client  │                                                    │Frontend │
│         │◄───────────────────────────────────────────────────┤ (WS)    │
│         │  WebSocket Event: trade:executed                   │         │
└─────────┘                                                    └─────────┘
```

### 11.2 Oversight Approval Flow

```
┌─────────┐
│ Agent   │ Wants to execute $75K trade
└────┬────┘
     │
     │ Trade Decision
     ▼
┌──────────────────────────────────────────────────────────────┐
│ TradeService                                                 │
│  - Check oversight triggers                                  │
│  - Trade value: $75K > $50K threshold → Requires approval    │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Create approval request
     ▼
┌──────────────────────────────────────────────────────────────┐
│ OversightService.createApprovalRequest()                     │
│  - Generate risk assessment                                  │
│  - Calculate portfolio impact                                │
│  - Set expiration: 10 minutes                                │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ gRPC call
     ▼
┌──────────────────────────────────────────────────────────────┐
│ OversightClient → creto-oversight:50052                      │
│  - CreateRequest RPC                                         │
│  - Returns: {requestId: ovr_123...}                          │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Store request
     ▼
┌──────────────────────────────────────────────────────────────┐
│ ApprovalRequestRepository                                    │
│  - Save to database: status=PENDING                          │
│  - Update trade: status=PENDING_APPROVAL                     │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Send notification
     ▼
┌──────────────────────────────────────────────────────────────┐
│ NotificationService                                          │
│  - Format Slack message with approval buttons               │
│  - Send to configured Slack channel                          │
│  - Emit WebSocket event to user                              │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Wait for decision (WebSocket real-time update)
     ▼
┌──────────────────────────────────────────────────────────────┐
│ Approver (via Slack or Dashboard)                           │
│  - Clicks "Approve" button                                   │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ POST /api/v1/oversight/requests/{id}/decision
     ▼
┌──────────────────────────────────────────────────────────────┐
│ OversightController                                          │
│  - Validate decision                                         │
│  - Call OversightService.processDecision()                   │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Update request
     ▼
┌──────────────────────────────────────────────────────────────┐
│ OversightService                                             │
│  - Update ApprovalRequest: status=APPROVED                   │
│  - Notify creto-oversight via gRPC                           │
│  - Trigger trade execution                                   │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Execute approved trade
     ▼
┌──────────────────────────────────────────────────────────────┐
│ TradeService.executeTradeImmediate()                         │
│  - Same flow as normal trade execution                       │
│  - Mark trade as approved in audit log                       │
└────┬─────────────────────────────────────────────────────────┘
     │
     │ Notify completion
     ▼
┌──────────────────────────────────────────────────────────────┐
│ WebSocketHub                                                 │
│  - Emit oversight:approved event                             │
│  - Emit trade:executed event                                 │
└──────────────────────────────────────────────────────────────┘
```

---

## 12. Technology Stack

### 12.1 Core Dependencies

```json
{
  "dependencies": {
    "express": "^4.18.2",
    "typescript": "^5.3.3",
    "typeorm": "^0.3.19",
    "sqlite3": "^5.1.7",
    "socket.io": "^4.6.1",
    "@grpc/grpc-js": "^1.9.13",
    "@grpc/proto-loader": "^0.7.10",
    "joi": "^17.11.0",
    "jsonwebtoken": "^9.0.2",
    "bcrypt": "^5.1.1",
    "winston": "^3.11.0",
    "morgan": "^1.10.0",
    "helmet": "^7.1.0",
    "cors": "^2.8.5",
    "express-rate-limit": "^7.1.5",
    "dotenv": "^16.3.1",
    "tsyringe": "^4.8.0",
    "reflect-metadata": "^0.1.14"
  },
  "devDependencies": {
    "@types/express": "^4.17.21",
    "@types/node": "^20.10.6",
    "@types/jsonwebtoken": "^9.0.5",
    "@types/bcrypt": "^5.0.2",
    "@types/morgan": "^1.9.9",
    "@types/cors": "^2.8.17",
    "jest": "^29.7.0",
    "ts-jest": "^29.1.1",
    "supertest": "^6.3.3",
    "@types/supertest": "^6.0.2",
    "ts-node": "^10.9.2",
    "nodemon": "^3.0.2",
    "eslint": "^8.56.0",
    "@typescript-eslint/eslint-plugin": "^6.16.0",
    "@typescript-eslint/parser": "^6.16.0",
    "prettier": "^3.1.1"
  }
}
```

### 12.2 Runtime Environment

- **Node.js**: 20.x LTS
- **TypeScript**: 5.x
- **Database**: SQLite 3.x (for demo; PostgreSQL for production)
- **Package Manager**: npm or pnpm

---

## 13. Design Patterns

### 13.1 Dependency Injection (tsyringe)

```typescript
// config/container.ts
import 'reflect-metadata';
import { container } from 'tsyringe';
import { DataSource } from 'typeorm';
import { AgentRepository } from '../repositories/agent.repository';
import { AgentService } from '../services/agent.service';
import { MeteringClient } from '../grpc/clients/metering.client';
import { Logger } from '../utils/logger';

export async function setupContainer(dataSource: DataSource): Promise<void> {
  // Register database
  container.register('DataSource', { useValue: dataSource });

  // Register repositories
  container.register('AgentRepository', { useClass: AgentRepository });
  container.register('TradeRepository', { useClass: TradeRepository });
  // ... other repositories

  // Register services
  container.register('AgentService', { useClass: AgentService });
  container.register('TradeService', { useClass: TradeService });
  // ... other services

  // Register gRPC clients
  container.register('MeteringClient', { useClass: MeteringClient });
  container.register('OversightClient', { useClass: OversightClient });

  // Register utilities
  container.register('Logger', { useValue: new Logger('App') });
}
```

### 13.2 Repository Pattern

Already demonstrated in [Section 6](#6-repository-pattern).

### 13.3 Service Layer Pattern

Already demonstrated in [Section 5](#5-service-layer-design).

### 13.4 DTO Pattern

```typescript
// dtos/requests/create-agent.dto.ts
export interface CreateAgentDTO {
  name: string;
  strategy: 'balanced' | 'aggressive' | 'conservative' | 'custom';
  initialBudget: number;
  riskTolerance: number; // 1-10
  quotaConfig?: {
    apiCalls?: number;
    budgetPerMonth?: number;
  };
  assetPreferences?: string[];
  tradingHours?: {
    start: string;
    end: string;
    timezone: string;
  };
}

// dtos/responses/agent.response.ts
export interface AgentResponseDTO {
  agentId: string;
  name: string;
  status: string;
  createdAt: string;
  portfolio: {
    value: number;
    cash: number;
    positions: any[];
  };
  quotaConfig: {
    apiCalls: number;
    budgetPerMonth: number;
    usage: {
      apiCalls: number;
      budgetSpent: number;
    };
  };
}
```

### 13.5 Factory Pattern

```typescript
// grpc/clients/index.ts
import { MeteringClient } from './metering.client';
import { OversightClient } from './oversight.client';

export class GRPCClientFactory {
  private static meteringClient: MeteringClient;
  private static oversightClient: OversightClient;

  static getMeteringClient(): MeteringClient {
    if (!this.meteringClient) {
      this.meteringClient = new MeteringClient();
    }
    return this.meteringClient;
  }

  static getOversightClient(): OversightClient {
    if (!this.oversightClient) {
      this.oversightClient = new OversightClient();
    }
    return this.oversightClient;
  }

  static async closeAll(): Promise<void> {
    await Promise.all([
      this.meteringClient?.close(),
      this.oversightClient?.close(),
    ]);
  }
}
```

---

## 14. Performance Considerations

### 14.1 Database Optimization

```typescript
// entities/trade.entity.ts
import { Entity, Column, PrimaryGeneratedColumn, Index, ManyToOne } from 'typeorm';
import { Agent } from './agent.entity';

@Entity('trades')
@Index(['agentId', 'submittedAt']) // Composite index for agent trade history
@Index(['status', 'submittedAt']) // Index for status queries
export class Trade {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  @Index() // Single index for agent lookups
  agentId: string;

  @ManyToOne(() => Agent, agent => agent.trades)
  agent: Agent;

  @Column()
  symbol: string;

  @Column({ type: 'varchar', length: 10 })
  action: 'buy' | 'sell';

  @Column({ type: 'int' })
  quantity: number;

  @Column({ type: 'varchar', length: 20 })
  status: string;

  @Column({ type: 'datetime' })
  submittedAt: Date;

  @Column({ type: 'datetime', nullable: true })
  executedAt: Date;

  @Column({ type: 'decimal', precision: 10, scale: 4, nullable: true })
  executedPrice: number;
}
```

### 14.2 Caching Strategy

```typescript
// utils/cache.ts
import { LRUCache } from 'lru-cache';

/**
 * In-memory cache for frequently accessed data
 */
export class CacheManager {
  private cache: LRUCache<string, any>;

  constructor() {
    this.cache = new LRUCache({
      max: 1000, // Max items
      ttl: 1000 * 60 * 5, // 5 minutes TTL
      updateAgeOnGet: true,
    });
  }

  get<T>(key: string): T | undefined {
    return this.cache.get(key) as T | undefined;
  }

  set<T>(key: string, value: T, ttl?: number): void {
    this.cache.set(key, value, { ttl });
  }

  delete(key: string): void {
    this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }
}

// Example usage in service
export class MarketDataService {
  constructor(private cache: CacheManager) {}

  async getCurrentPrice(symbol: string): Promise<number> {
    const cacheKey = `price:${symbol}`;

    // Check cache first
    const cachedPrice = this.cache.get<number>(cacheKey);
    if (cachedPrice !== undefined) {
      return cachedPrice;
    }

    // Fetch from API
    const price = await this.fetchPriceFromAPI(symbol);

    // Cache for 30 seconds
    this.cache.set(cacheKey, price, 30000);

    return price;
  }
}
```

### 14.3 Connection Pooling

```typescript
// config/database.ts
import { DataSource } from 'typeorm';

export const AppDataSource = new DataSource({
  type: 'sqlite',
  database: 'trading-demo.db',
  synchronize: process.env.NODE_ENV !== 'production', // Auto-sync in dev
  logging: process.env.NODE_ENV === 'development',
  entities: ['src/entities/**/*.entity.ts'],
  migrations: ['src/migrations/**/*.ts'],

  // Connection pooling (for PostgreSQL, not SQLite)
  // poolSize: 10,
  // extra: {
  //   max: 10,
  //   min: 2,
  //   idleTimeoutMillis: 30000,
  // },
});
```

---

## 15. Security Architecture

### 15.1 Authentication Flow

```
┌─────────┐                                       ┌──────────┐
│ Client  │                                       │ Backend  │
└────┬────┘                                       └────┬─────┘
     │                                                 │
     │ POST /api/v1/auth/login                        │
     │ {email, password}                              │
     ├────────────────────────────────────────────────►
     │                                                 │
     │                                                 │ Validate credentials
     │                                                 │ (bcrypt.compare)
     │                                                 │
     │                                                 │ Generate JWT
     │                                                 │ (exp: 1 hour)
     │                                                 │
     │◄────────────────────────────────────────────────┤
     │ Response: {token, refreshToken, user}          │
     │                                                 │
     │ Store token in memory/localStorage              │
     │                                                 │
     │ GET /api/v1/agents                             │
     │ Authorization: Bearer <token>                   │
     ├────────────────────────────────────────────────►
     │                                                 │
     │                                                 │ Verify JWT
     │                                                 │ (jwt.verify)
     │                                                 │
     │                                                 │ Extract user from token
     │                                                 │ Attach to req.user
     │                                                 │
     │◄────────────────────────────────────────────────┤
     │ Response: [agents...]                          │
     │                                                 │
```

### 15.2 Input Validation

All inputs validated using Joi schemas before processing.

### 15.3 SQL Injection Prevention

TypeORM parameterized queries prevent SQL injection.

### 15.4 CORS Configuration

```typescript
// middleware/cors.ts
import cors from 'cors';

export const corsOptions = cors({
  origin: process.env.FRONTEND_URL || 'http://localhost:3000',
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Correlation-ID'],
  exposedHeaders: ['X-Correlation-ID'],
  maxAge: 86400, // 24 hours
});
```

### 15.5 Security Headers (Helmet)

```typescript
// middleware/helmet.ts
import helmet from 'helmet';

export const helmetConfig = helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", 'data:', 'https:'],
    },
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true,
  },
});
```

---

## 16. Testing Strategy

### 16.1 Testing Pyramid

```
                  ┌────────────┐
                  │    E2E     │  5% (Full system tests)
                  │  Tests     │
                  └────────────┘
              ┌──────────────────┐
              │   Integration    │  15% (Service + Repository)
              │     Tests        │
              └──────────────────┘
          ┌────────────────────────┐
          │      Unit Tests        │  80% (Services, Repositories, Utils)
          │                        │
          └────────────────────────┘
```

### 16.2 Unit Test Example

```typescript
// __tests__/services/agent.service.test.ts
import { AgentService } from '../../src/services/agent.service';
import { AgentRepository } from '../../src/repositories/agent.repository';
import { MeteringService } from '../../src/services/metering.service';
import { ValidationError } from '../../src/utils/errors';

describe('AgentService', () => {
  let agentService: AgentService;
  let mockAgentRepo: jest.Mocked<AgentRepository>;
  let mockMeteringService: jest.Mocked<MeteringService>;

  beforeEach(() => {
    mockAgentRepo = {
      create: jest.fn(),
      findById: jest.fn(),
    } as any;

    mockMeteringService = {
      initializeQuota: jest.fn(),
    } as any;

    agentService = new AgentService(
      mockAgentRepo,
      mockMeteringService,
      {} as any, // portfolio service
      {} as any  // logger
    );
  });

  describe('createAgent', () => {
    it('should create agent successfully with valid data', async () => {
      const dto = {
        name: 'Test Agent',
        strategy: 'balanced' as const,
        initialBudget: 10000,
        riskTolerance: 5,
      };

      const mockAgent = {
        id: 'agt_123',
        ...dto,
        status: 'paused',
      };

      mockAgentRepo.create.mockResolvedValue(mockAgent as any);
      mockMeteringService.initializeQuota.mockResolvedValue();

      const result = await agentService.createAgent(dto, 'user_123');

      expect(result).toEqual(mockAgent);
      expect(mockAgentRepo.create).toHaveBeenCalledWith({
        ...dto,
        userId: 'user_123',
        status: 'paused',
      });
      expect(mockMeteringService.initializeQuota).toHaveBeenCalled();
    });

    it('should throw ValidationError if budget exceeds limit', async () => {
      const dto = {
        name: 'Test Agent',
        strategy: 'balanced' as const,
        initialBudget: 150000, // Exceeds $100K limit
        riskTolerance: 5,
      };

      await expect(
        agentService.createAgent(dto, 'user_123')
      ).rejects.toThrow(ValidationError);

      expect(mockAgentRepo.create).not.toHaveBeenCalled();
    });
  });
});
```

### 16.3 Integration Test Example

```typescript
// __tests__/integration/trade-flow.test.ts
import request from 'supertest';
import { app } from '../../src/app';
import { AppDataSource } from '../../src/config/database';
import { Agent } from '../../src/entities/agent.entity';

describe('Trade Execution Integration', () => {
  let authToken: string;
  let agentId: string;

  beforeAll(async () => {
    await AppDataSource.initialize();

    // Login and get token
    const loginRes = await request(app)
      .post('/api/v1/auth/login')
      .send({ email: 'test@example.com', password: 'password123' });

    authToken = loginRes.body.token;
  });

  beforeEach(async () => {
    // Create test agent
    const createRes = await request(app)
      .post('/api/v1/agents')
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        name: 'Integration Test Agent',
        strategy: 'balanced',
        initialBudget: 10000,
        riskTolerance: 5,
      });

    agentId = createRes.body.agentId;
  });

  afterEach(async () => {
    // Cleanup: delete test agent
    await request(app)
      .delete(`/api/v1/agents/${agentId}`)
      .set('Authorization', `Bearer ${authToken}`);
  });

  afterAll(async () => {
    await AppDataSource.destroy();
  });

  it('should execute trade and update portfolio', async () => {
    // Execute trade
    const tradeRes = await request(app)
      .post(`/api/v1/agents/${agentId}/trades`)
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        symbol: 'AAPL',
        action: 'buy',
        quantity: 10,
        orderType: 'market',
      });

    expect(tradeRes.status).toBe(200);
    expect(tradeRes.body.status).toBe('completed');
    expect(tradeRes.body.symbol).toBe('AAPL');

    // Verify portfolio updated
    const portfolioRes = await request(app)
      .get(`/api/v1/agents/${agentId}/portfolio`)
      .set('Authorization', `Bearer ${authToken}`);

    expect(portfolioRes.status).toBe(200);
    expect(portfolioRes.body.portfolio.positions).toHaveLength(1);
    expect(portfolioRes.body.portfolio.positions[0].symbol).toBe('AAPL');
    expect(portfolioRes.body.portfolio.positions[0].quantity).toBe(10);
  });
});
```

### 16.4 E2E Test Example

```typescript
// __tests__/e2e/demo-flow.test.ts
import request from 'supertest';
import { app } from '../../src/app';
import { AppDataSource } from '../../src/config/database';

/**
 * Full demo flow:
 * 1. Login
 * 2. Create agent
 * 3. Execute trade (no approval)
 * 4. Execute high-value trade (requires approval)
 * 5. Approve trade
 * 6. Check usage metrics
 */
describe('Demo Flow E2E', () => {
  let token: string;
  let agentId: string;

  beforeAll(async () => {
    await AppDataSource.initialize();
  });

  afterAll(async () => {
    await AppDataSource.destroy();
  });

  it('should complete full demo workflow', async () => {
    // 1. Login
    const loginRes = await request(app)
      .post('/api/v1/auth/login')
      .send({ email: 'demo@example.com', password: 'demo123' });

    expect(loginRes.status).toBe(200);
    token = loginRes.body.token;

    // 2. Create agent
    const createRes = await request(app)
      .post('/api/v1/agents')
      .set('Authorization', `Bearer ${token}`)
      .send({
        name: 'Demo Agent',
        strategy: 'balanced',
        initialBudget: 100000,
        riskTolerance: 5,
      });

    expect(createRes.status).toBe(201);
    agentId = createRes.body.agentId;

    // 3. Execute small trade (auto-approved)
    const smallTradeRes = await request(app)
      .post(`/api/v1/agents/${agentId}/trades`)
      .set('Authorization', `Bearer ${token}`)
      .send({
        symbol: 'AAPL',
        action: 'buy',
        quantity: 10,
        orderType: 'market',
      });

    expect(smallTradeRes.status).toBe(200);
    expect(smallTradeRes.body.status).toBe('completed');

    // 4. Execute high-value trade (requires approval)
    const largeTradeRes = await request(app)
      .post(`/api/v1/agents/${agentId}/trades`)
      .set('Authorization', `Bearer ${token}`)
      .send({
        symbol: 'TSLA',
        action: 'buy',
        quantity: 500, // $75K value
        orderType: 'market',
      });

    expect(largeTradeRes.status).toBe(200);
    expect(largeTradeRes.body.status).toBe('pending_approval');

    const approvalRequestId = largeTradeRes.body.approvalRequestId;

    // 5. Approve trade
    const approveRes = await request(app)
      .post(`/api/v1/oversight/requests/${approvalRequestId}/decision`)
      .set('Authorization', `Bearer ${token}`)
      .send({
        decision: 'approved',
        reason: 'Demo approval',
      });

    expect(approveRes.status).toBe(200);

    // Wait for trade execution (async)
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // 6. Check usage metrics
    const usageRes = await request(app)
      .get(`/api/v1/agents/${agentId}/usage`)
      .set('Authorization', `Bearer ${token}`);

    expect(usageRes.status).toBe(200);
    expect(usageRes.body.currentUsage.apiCalls).toBeGreaterThan(0);
  });
});
```

---

## Conclusion

This backend architecture provides a **robust, scalable, and maintainable** foundation for the Trading Demo. Key takeaways:

1. **Layered Architecture**: Clear separation between presentation, business logic, data access, and infrastructure layers
2. **Service-Oriented Design**: Services orchestrate complex workflows and enforce business rules
3. **Repository Pattern**: Clean data access abstraction with TypeORM
4. **gRPC Integration**: Efficient communication with creto-metering and creto-oversight services
5. **Real-Time Updates**: WebSocket hub for sub-second dashboard updates
6. **Comprehensive Error Handling**: Custom error classes, global error handler, retry logic
7. **Security-First**: JWT authentication, input validation, CORS, Helmet security headers
8. **High Testability**: 90%+ code coverage target with unit, integration, and e2e tests
9. **Performance Optimized**: Database indexing, caching, connection pooling
10. **Production-Ready**: Logging, monitoring, graceful shutdown, health checks

**Next Steps:**
1. Review this architecture document with the team
2. Implement core entities and repositories
3. Build service layer with business logic
4. Integrate gRPC clients
5. Add WebSocket real-time updates
6. Write comprehensive tests
7. Deploy to staging environment
8. Conduct load testing and performance tuning

---

**Document Status:** Draft
**Next Review Date:** 2025-12-27
**Approved By:** [Pending Review]
