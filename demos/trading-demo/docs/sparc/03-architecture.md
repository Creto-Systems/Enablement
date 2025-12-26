# Trading Demo - System Architecture

## Document Information
- **Phase**: Architecture (SPARC Phase 3)
- **Version**: 1.0.0
- **Date**: 2025-12-26
- **Status**: Design Complete

## Table of Contents
1. [High-Level Architecture](#high-level-architecture)
2. [Component Architecture](#component-architecture)
3. [Technology Stack](#technology-stack)
4. [Directory Structure](#directory-structure)
5. [Data Flow Diagrams](#data-flow-diagrams)
6. [Security Architecture](#security-architecture)
7. [Deployment Architecture](#deployment-architecture)
8. [Architecture Decision Records](#architecture-decision-records)

---

## 1. High-Level Architecture

### 1.1 System Context Diagram (C4 Level 1)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           TRADING DEMO SYSTEM                            │
│                                                                          │
│  ┌──────────────────┐         ┌──────────────────┐                     │
│  │                  │         │                  │                     │
│  │   React SPA      │◄───────►│   Express API    │                     │
│  │   (Frontend)     │  HTTPS  │   (Backend)      │                     │
│  │                  │  REST   │                  │                     │
│  └──────────────────┘         └──────────────────┘                     │
│         │                             │                                 │
│         │ WebSocket                   │                                 │
│         ▼                             │                                 │
│  ┌──────────────────┐                │                                 │
│  │   WebSocket      │◄───────────────┘                                 │
│  │   Server         │                                                   │
│  │   (Real-time)    │                                                   │
│  └──────────────────┘                                                   │
│         │                                                               │
│         │                                                               │
│  ┌──────┴────────┬─────────────┬──────────────┐                       │
│  ▼               ▼             ▼              ▼                       │
│ ┌────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐                  │
│ │SQLite  │  │ gRPC    │  │  gRPC    │  │  Slack   │                  │
│ │Database│  │Metering │  │Oversight │  │  Webhook │                  │
│ └────────┘  └─────────┘  └──────────┘  └──────────┘                  │
│                  :50051       :50052                                    │
└─────────────────────────────────────────────────────────────────────────┘
         │                    │              │              │
         ▼                    ▼              ▼              ▼
    ┌────────┐         ┌──────────┐   ┌──────────┐   ┌─────────┐
    │ Trader │         │  creto-  │   │  creto-  │   │ Slack   │
    │ (User) │         │ metering │   │ oversight│   │ Channel │
    └────────┘         └──────────┘   └──────────┘   └─────────┘
```

### 1.2 Container Diagram (C4 Level 2)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TRADING DEMO CONTAINERS                          │
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                      FRONTEND CONTAINER                           │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │  │
│  │  │ Trading  │  │Portfolio │  │ Settings │  │ Analytics│        │  │
│  │  │Dashboard │  │ View     │  │ Page     │  │ Page     │        │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │  │
│  │                                                                   │  │
│  │  ┌────────────────────────────────────────────────────┐         │  │
│  │  │         State Management (Context + Reducers)      │         │  │
│  │  └────────────────────────────────────────────────────┘         │  │
│  │                                                                   │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                       │  │
│  │  │REST API  │  │WebSocket │  │ Charting │                       │  │
│  │  │Client    │  │Client    │  │ Library  │                       │  │
│  │  └──────────┘  └──────────┘  └──────────┘                       │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │        │                                  │
│                         HTTP │        │ WebSocket                        │
│                              ▼        ▼                                  │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                      BACKEND CONTAINER                            │  │
│  │                                                                    │  │
│  │  ┌──────────────────────────────────────────────────────────────┐ │  │
│  │  │                     API LAYER                                 │ │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │ │  │
│  │  │  │  Trade   │  │Portfolio │  │ Metering │  │  Alert   │    │ │  │
│  │  │  │Controller│  │Controller│  │Controller│  │Controller│    │ │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │ │  │
│  │  └──────────────────────────────────────────────────────────────┘ │  │
│  │                                                                    │  │
│  │  ┌──────────────────────────────────────────────────────────────┐ │  │
│  │  │                   SERVICE LAYER                               │ │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │ │  │
│  │  │  │  Trade   │  │Portfolio │  │ Metering │  │  Slack   │    │ │  │
│  │  │  │ Service  │  │ Service  │  │ Service  │  │ Service  │    │ │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │ │  │
│  │  └──────────────────────────────────────────────────────────────┘ │  │
│  │                                                                    │  │
│  │  ┌──────────────────────────────────────────────────────────────┐ │  │
│  │  │                  REPOSITORY LAYER                             │ │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │ │  │
│  │  │  │  Trade   │  │Portfolio │  │ Metering │  │  Alert   │    │ │  │
│  │  │  │   Repo   │  │   Repo   │  │   Repo   │  │   Repo   │    │ │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │ │  │
│  │  └──────────────────────────────────────────────────────────────┘ │  │
│  │                                                                    │  │
│  │  ┌──────────────────────────────────────────────────────────────┐ │  │
│  │  │                 INFRASTRUCTURE LAYER                          │ │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │ │  │
│  │  │  │  SQLite  │  │  gRPC    │  │WebSocket │  │  Slack   │    │ │  │
│  │  │  │  Client  │  │ Clients  │  │  Server  │  │  Client  │    │ │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │ │  │
│  │  └──────────────────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                       │         │           │          │                │
└───────────────────────┼─────────┼───────────┼──────────┼────────────────┘
                        ▼         ▼           ▼          ▼
                   ┌────────┐ ┌──────┐  ┌──────┐  ┌──────────┐
                   │ SQLite │ │creto-│  │creto-│  │  Slack   │
                   │  File  │ │meter │  │sight │  │  Webhook │
                   └────────┘ └──────┘  └──────┘  └──────────┘
```

---

## 2. Component Architecture

### 2.1 Frontend Architecture

#### 2.1.1 Presentation Layer

```typescript
// Component Hierarchy
TradingApp/
├── Layout/
│   ├── Header (navigation, user info)
│   ├── Sidebar (portfolio summary)
│   └── Footer (status indicators)
│
├── Pages/
│   ├── DashboardPage (main trading interface)
│   ├── PortfolioPage (holdings, P&L, history)
│   ├── AnalyticsPage (charts, metrics)
│   └── SettingsPage (configuration)
│
└── Components/
    ├── Trading/
    │   ├── OrderForm (buy/sell interface)
    │   ├── OrderBook (live order display)
    │   ├── TradeHistory (executed trades)
    │   └── PriceChart (candlestick/line chart)
    │
    ├── Portfolio/
    │   ├── PositionTable (current holdings)
    │   ├── ProfitLossCard (P&L summary)
    │   └── AllocationChart (pie chart)
    │
    ├── Metering/
    │   ├── UsageDashboard (API call metrics)
    │   ├── CostEstimator (pricing calculator)
    │   └── QuotaIndicator (usage limits)
    │
    └── Common/
        ├── Button, Input, Select (form controls)
        ├── Card, Table, Modal (containers)
        └── LoadingSpinner, ErrorBoundary (utilities)
```

#### 2.1.2 State Management

```typescript
// Global State Structure
interface AppState {
  auth: {
    user: User | null;
    token: string | null;
    isAuthenticated: boolean;
  };

  trading: {
    activeOrders: Order[];
    tradeHistory: Trade[];
    orderBook: OrderBookEntry[];
    selectedSymbol: string;
  };

  portfolio: {
    positions: Position[];
    totalValue: number;
    profitLoss: ProfitLoss;
    performance: PerformanceMetrics;
  };

  metering: {
    usage: UsageMetrics;
    costs: CostBreakdown;
    quota: QuotaStatus;
  };

  websocket: {
    connected: boolean;
    lastUpdate: Date;
    subscriptions: string[];
  };

  ui: {
    theme: 'light' | 'dark';
    notifications: Notification[];
    loading: Record<string, boolean>;
    errors: Record<string, Error>;
  };
}

// State Management Pattern: Context + Reducers
// - AuthContext: authentication state
// - TradingContext: trading operations
// - PortfolioContext: portfolio data
// - MeteringContext: metering metrics
// - WebSocketContext: real-time updates
```

#### 2.1.3 Service Layer (Frontend)

```typescript
// API Client Services
class TradingApiClient {
  async submitOrder(order: OrderRequest): Promise<OrderResponse>;
  async getOrders(filters?: OrderFilters): Promise<Order[]>;
  async cancelOrder(orderId: string): Promise<void>;
  async getTradeHistory(): Promise<Trade[]>;
}

class PortfolioApiClient {
  async getPositions(): Promise<Position[]>;
  async getPerformance(): Promise<PerformanceMetrics>;
  async getProfitLoss(period: TimePeriod): Promise<ProfitLoss>;
}

class MeteringApiClient {
  async getUsageMetrics(): Promise<UsageMetrics>;
  async getCostBreakdown(): Promise<CostBreakdown>;
  async getQuotaStatus(): Promise<QuotaStatus>;
}

// WebSocket Client
class WebSocketClient {
  connect(): Promise<void>;
  subscribe(channel: string, handler: MessageHandler): void;
  unsubscribe(channel: string): void;
  send(message: WebSocketMessage): void;
}
```

### 2.2 Backend Architecture

#### 2.2.1 API Layer (Controllers)

```typescript
// Express Controllers (Route Handlers)

// TradeController
class TradeController {
  // POST /api/trades
  async createTrade(req: Request, res: Response): Promise<void>;

  // GET /api/trades
  async getTrades(req: Request, res: Response): Promise<void>;

  // GET /api/trades/:id
  async getTradeById(req: Request, res: Response): Promise<void>;

  // DELETE /api/trades/:id
  async cancelTrade(req: Request, res: Response): Promise<void>;
}

// PortfolioController
class PortfolioController {
  // GET /api/portfolio
  async getPortfolio(req: Request, res: Response): Promise<void>;

  // GET /api/portfolio/positions
  async getPositions(req: Request, res: Response): Promise<void>;

  // GET /api/portfolio/performance
  async getPerformance(req: Request, res: Response): Promise<void>;
}

// MeteringController
class MeteringController {
  // GET /api/metering/usage
  async getUsage(req: Request, res: Response): Promise<void>;

  // GET /api/metering/costs
  async getCosts(req: Request, res: Response): Promise<void>;

  // POST /api/metering/track
  async trackUsage(req: Request, res: Response): Promise<void>;
}

// AlertController
class AlertController {
  // GET /api/alerts
  async getAlerts(req: Request, res: Response): Promise<void>;

  // POST /api/alerts
  async createAlert(req: Request, res: Response): Promise<void>;

  // PUT /api/alerts/:id/acknowledge
  async acknowledgeAlert(req: Request, res: Response): Promise<void>;
}
```

#### 2.2.2 Service Layer (Business Logic)

```typescript
// Domain Services

class TradeService {
  constructor(
    private tradeRepo: TradeRepository,
    private meteringService: MeteringService,
    private oversightService: OversightService,
    private slackService: SlackService,
    private wsServer: WebSocketServer
  ) {}

  async executeTrade(order: TradeOrder): Promise<Trade> {
    // 1. Validate order
    this.validateOrder(order);

    // 2. Check risk limits (via oversight)
    await this.oversightService.checkRiskLimits(order);

    // 3. Track metering
    const meteringId = await this.meteringService.trackOperation('trade.execute');

    // 4. Execute trade
    const trade = await this.tradeRepo.create(order);

    // 5. Send notifications
    await this.slackService.notifyTrade(trade);
    this.wsServer.broadcast('trade:executed', trade);

    // 6. Complete metering
    await this.meteringService.completeOperation(meteringId);

    return trade;
  }

  async getTradeHistory(userId: string, filters: TradeFilters): Promise<Trade[]> {
    return this.tradeRepo.findByUserId(userId, filters);
  }

  async cancelTrade(tradeId: string): Promise<void> {
    // Track metering
    const meteringId = await this.meteringService.trackOperation('trade.cancel');

    // Cancel trade
    await this.tradeRepo.cancel(tradeId);

    // Notify
    this.wsServer.broadcast('trade:cancelled', { tradeId });
    await this.meteringService.completeOperation(meteringId);
  }

  private validateOrder(order: TradeOrder): void {
    if (order.quantity <= 0) throw new ValidationError('Invalid quantity');
    if (!order.symbol) throw new ValidationError('Symbol required');
    // ... more validation
  }
}

class PortfolioService {
  constructor(
    private portfolioRepo: PortfolioRepository,
    private tradeRepo: TradeRepository,
    private meteringService: MeteringService
  ) {}

  async getPortfolio(userId: string): Promise<Portfolio> {
    const meteringId = await this.meteringService.trackOperation('portfolio.get');

    const positions = await this.portfolioRepo.getPositions(userId);
    const performance = await this.calculatePerformance(positions);

    await this.meteringService.completeOperation(meteringId);

    return { positions, performance };
  }

  async calculatePerformance(positions: Position[]): Promise<PerformanceMetrics> {
    // Calculate total value, P&L, returns, etc.
    const totalValue = positions.reduce((sum, p) => sum + p.marketValue, 0);
    const totalCost = positions.reduce((sum, p) => sum + p.costBasis, 0);
    const unrealizedPL = totalValue - totalCost;

    return {
      totalValue,
      unrealizedPL,
      realizedPL: await this.calculateRealizedPL(),
      returnPct: (unrealizedPL / totalCost) * 100
    };
  }

  private async calculateRealizedPL(): Promise<number> {
    // Query closed positions from trades
    // ...
    return 0;
  }
}

class MeteringService {
  constructor(
    private meteringClient: MeteringGrpcClient,
    private meteringRepo: MeteringRepository
  ) {}

  async trackOperation(operationType: string): Promise<string> {
    // Call creto-metering via gRPC
    const response = await this.meteringClient.trackUsage({
      operationType,
      timestamp: new Date(),
      metadata: { demo: 'trading' }
    });

    // Store locally for dashboard
    await this.meteringRepo.create({
      id: response.id,
      operationType,
      cost: response.estimatedCost,
      timestamp: new Date()
    });

    return response.id;
  }

  async completeOperation(meteringId: string): Promise<void> {
    await this.meteringClient.completeUsage({ id: meteringId });
    await this.meteringRepo.updateStatus(meteringId, 'completed');
  }

  async getUsageMetrics(period: TimePeriod): Promise<UsageMetrics> {
    return this.meteringRepo.getMetrics(period);
  }
}

class OversightService {
  constructor(private oversightClient: OversightGrpcClient) {}

  async checkRiskLimits(order: TradeOrder): Promise<void> {
    const response = await this.oversightClient.evaluateRisk({
      orderSize: order.quantity * order.price,
      symbol: order.symbol,
      type: order.type
    });

    if (response.riskLevel === 'HIGH') {
      throw new RiskLimitError('Order exceeds risk limits');
    }
  }

  async getApprovalStatus(requestId: string): Promise<ApprovalStatus> {
    return this.oversightClient.getApprovalStatus({ requestId });
  }
}

class SlackService {
  constructor(private webhookUrl: string) {}

  async notifyTrade(trade: Trade): Promise<void> {
    await this.sendMessage({
      text: `Trade executed: ${trade.type} ${trade.quantity} ${trade.symbol} @ $${trade.price}`,
      blocks: [
        {
          type: 'section',
          text: { type: 'mrkdwn', text: `*Trade Executed*` }
        },
        {
          type: 'section',
          fields: [
            { type: 'mrkdwn', text: `*Symbol:*\n${trade.symbol}` },
            { type: 'mrkdwn', text: `*Type:*\n${trade.type}` },
            { type: 'mrkdwn', text: `*Quantity:*\n${trade.quantity}` },
            { type: 'mrkdwn', text: `*Price:*\n$${trade.price}` }
          ]
        }
      ]
    });
  }

  async notifyAlert(alert: Alert): Promise<void> {
    await this.sendMessage({
      text: `⚠️ Alert: ${alert.message}`,
      attachments: [{
        color: alert.severity === 'high' ? 'danger' : 'warning',
        fields: [
          { title: 'Severity', value: alert.severity, short: true },
          { title: 'Type', value: alert.type, short: true }
        ]
      }]
    });
  }

  private async sendMessage(message: SlackMessage): Promise<void> {
    await fetch(this.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(message)
    });
  }
}
```

#### 2.2.3 Repository Layer (Data Access)

```typescript
// Database Repositories

class TradeRepository {
  constructor(private db: Database) {}

  async create(trade: CreateTradeDto): Promise<Trade> {
    const stmt = this.db.prepare(`
      INSERT INTO trades (id, user_id, symbol, type, quantity, price, status, created_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `);

    const id = generateId();
    stmt.run(
      id,
      trade.userId,
      trade.symbol,
      trade.type,
      trade.quantity,
      trade.price,
      'executed',
      Date.now()
    );

    return this.findById(id);
  }

  async findById(id: string): Promise<Trade | null> {
    const stmt = this.db.prepare('SELECT * FROM trades WHERE id = ?');
    return stmt.get(id) as Trade | null;
  }

  async findByUserId(userId: string, filters: TradeFilters): Promise<Trade[]> {
    let query = 'SELECT * FROM trades WHERE user_id = ?';
    const params: any[] = [userId];

    if (filters.symbol) {
      query += ' AND symbol = ?';
      params.push(filters.symbol);
    }

    if (filters.startDate) {
      query += ' AND created_at >= ?';
      params.push(filters.startDate.getTime());
    }

    query += ' ORDER BY created_at DESC LIMIT ?';
    params.push(filters.limit || 100);

    const stmt = this.db.prepare(query);
    return stmt.all(...params) as Trade[];
  }

  async cancel(tradeId: string): Promise<void> {
    const stmt = this.db.prepare('UPDATE trades SET status = ? WHERE id = ?');
    stmt.run('cancelled', tradeId);
  }
}

class PortfolioRepository {
  constructor(private db: Database) {}

  async getPositions(userId: string): Promise<Position[]> {
    const stmt = this.db.prepare(`
      SELECT
        symbol,
        SUM(CASE WHEN type = 'BUY' THEN quantity ELSE -quantity END) as quantity,
        AVG(price) as avgPrice,
        MIN(created_at) as firstTradeDate
      FROM trades
      WHERE user_id = ? AND status = 'executed'
      GROUP BY symbol
      HAVING quantity > 0
    `);

    const positions = stmt.all(userId) as any[];

    // Enrich with current market prices (mock for demo)
    return positions.map(p => ({
      symbol: p.symbol,
      quantity: p.quantity,
      avgPrice: p.avgPrice,
      currentPrice: p.avgPrice * (1 + (Math.random() - 0.5) * 0.1), // Mock price movement
      marketValue: p.quantity * p.avgPrice,
      costBasis: p.quantity * p.avgPrice,
      unrealizedPL: 0, // Calculate based on current price
      firstTradeDate: new Date(p.firstTradeDate)
    }));
  }
}

class MeteringRepository {
  constructor(private db: Database) {}

  async create(record: MeteringRecord): Promise<void> {
    const stmt = this.db.prepare(`
      INSERT INTO metering_records (id, operation_type, cost, status, created_at)
      VALUES (?, ?, ?, ?, ?)
    `);

    stmt.run(
      record.id,
      record.operationType,
      record.cost,
      'pending',
      Date.now()
    );
  }

  async updateStatus(id: string, status: string): Promise<void> {
    const stmt = this.db.prepare('UPDATE metering_records SET status = ? WHERE id = ?');
    stmt.run(status, id);
  }

  async getMetrics(period: TimePeriod): Promise<UsageMetrics> {
    const startDate = this.getStartDate(period);
    const stmt = this.db.prepare(`
      SELECT
        COUNT(*) as totalOperations,
        SUM(cost) as totalCost,
        operation_type
      FROM metering_records
      WHERE created_at >= ?
      GROUP BY operation_type
    `);

    const results = stmt.all(startDate.getTime()) as any[];

    return {
      period,
      totalOperations: results.reduce((sum, r) => sum + r.totalOperations, 0),
      totalCost: results.reduce((sum, r) => sum + r.totalCost, 0),
      breakdown: results.map(r => ({
        type: r.operation_type,
        count: r.totalOperations,
        cost: r.totalCost
      }))
    };
  }

  private getStartDate(period: TimePeriod): Date {
    const now = new Date();
    switch (period) {
      case 'day': return new Date(now.getTime() - 24 * 60 * 60 * 1000);
      case 'week': return new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
      case 'month': return new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
    }
  }
}
```

#### 2.2.4 Infrastructure Layer

```typescript
// gRPC Clients

class MeteringGrpcClient {
  private client: any;

  constructor(address: string = 'localhost:50051') {
    const packageDef = protoLoader.loadSync('metering.proto');
    const proto = grpc.loadPackageDefinition(packageDef);
    this.client = new proto.metering.MeteringService(
      address,
      grpc.credentials.createInsecure()
    );
  }

  async trackUsage(request: TrackUsageRequest): Promise<TrackUsageResponse> {
    return new Promise((resolve, reject) => {
      this.client.TrackUsage(request, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async completeUsage(request: CompleteUsageRequest): Promise<void> {
    return new Promise((resolve, reject) => {
      this.client.CompleteUsage(request, (err: Error) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }
}

class OversightGrpcClient {
  private client: any;

  constructor(address: string = 'localhost:50052') {
    const packageDef = protoLoader.loadSync('oversight.proto');
    const proto = grpc.loadPackageDefinition(packageDef);
    this.client = new proto.oversight.OversightService(
      address,
      grpc.credentials.createInsecure()
    );
  }

  async evaluateRisk(request: RiskEvaluationRequest): Promise<RiskEvaluationResponse> {
    return new Promise((resolve, reject) => {
      this.client.EvaluateRisk(request, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async getApprovalStatus(request: ApprovalStatusRequest): Promise<ApprovalStatus> {
    return new Promise((resolve, reject) => {
      this.client.GetApprovalStatus(request, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }
}

// WebSocket Server

class WebSocketServer {
  private wss: WebSocket.Server;
  private clients: Map<string, WebSocket> = new Map();

  constructor(server: http.Server) {
    this.wss = new WebSocket.Server({ server });
    this.setupHandlers();
  }

  private setupHandlers(): void {
    this.wss.on('connection', (ws: WebSocket, req: http.IncomingMessage) => {
      const clientId = this.generateClientId();
      this.clients.set(clientId, ws);

      ws.on('message', (message: string) => {
        this.handleMessage(clientId, message);
      });

      ws.on('close', () => {
        this.clients.delete(clientId);
      });

      ws.send(JSON.stringify({ type: 'connected', clientId }));
    });
  }

  private handleMessage(clientId: string, message: string): void {
    try {
      const data = JSON.parse(message);

      switch (data.type) {
        case 'subscribe':
          this.handleSubscribe(clientId, data.channel);
          break;
        case 'unsubscribe':
          this.handleUnsubscribe(clientId, data.channel);
          break;
        case 'ping':
          this.send(clientId, { type: 'pong' });
          break;
      }
    } catch (err) {
      console.error('WebSocket message error:', err);
    }
  }

  private handleSubscribe(clientId: string, channel: string): void {
    // Track subscriptions (could use Redis for multi-instance)
    console.log(`Client ${clientId} subscribed to ${channel}`);
  }

  private handleUnsubscribe(clientId: string, channel: string): void {
    console.log(`Client ${clientId} unsubscribed from ${channel}`);
  }

  broadcast(event: string, data: any): void {
    const message = JSON.stringify({ event, data, timestamp: Date.now() });
    this.clients.forEach(ws => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(message);
      }
    });
  }

  send(clientId: string, data: any): void {
    const ws = this.clients.get(clientId);
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(data));
    }
  }

  private generateClientId(): string {
    return `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}
```

---

## 3. Technology Stack

### 3.1 Frontend Stack

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| **Framework** | React | 18.3.1 | UI component library |
| **Build Tool** | Vite | 6.0.5 | Fast build and HMR |
| **Language** | TypeScript | 5.7.2 | Type safety |
| **Styling** | TailwindCSS | 3.4.17 | Utility-first CSS |
| **Charting** | Recharts | 2.15.0 | Data visualization |
| **State** | React Context | Built-in | Global state |
| **Routing** | React Router | 7.1.3 | Client-side routing |
| **HTTP** | Fetch API | Built-in | REST API calls |
| **WebSocket** | WebSocket API | Built-in | Real-time updates |
| **Forms** | React Hook Form | 7.54.2 | Form validation |
| **Testing** | Vitest | 2.1.8 | Unit testing |
| **Testing** | Testing Library | 16.1.0 | Component testing |
| **E2E Testing** | Playwright | 1.49.1 | Integration tests |

### 3.2 Backend Stack

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| **Runtime** | Node.js | 20+ | JavaScript runtime |
| **Framework** | Express | 5.0.1 | Web server |
| **Language** | TypeScript | 5.7.2 | Type safety |
| **Database** | better-sqlite3 | 11.8.1 | SQLite driver |
| **gRPC** | @grpc/grpc-js | 1.12.4 | gRPC client |
| **gRPC Proto** | @grpc/proto-loader | 0.7.16 | Proto parsing |
| **WebSocket** | ws | 8.18.0 | WebSocket server |
| **Validation** | Zod | 3.24.1 | Schema validation |
| **Logging** | Winston | 3.17.0 | Structured logging |
| **Testing** | Vitest | 2.1.8 | Unit testing |
| **HTTP Client** | node-fetch | 3.3.2 | Slack webhooks |

### 3.3 Development Tools

| Category | Technology | Purpose |
|----------|-----------|---------|
| **Package Manager** | npm | Dependency management |
| **Linter** | ESLint | Code quality |
| **Formatter** | Prettier | Code formatting |
| **Git Hooks** | Husky | Pre-commit checks |
| **Type Checking** | tsc | TypeScript validation |

---

## 4. Directory Structure

```
demos/trading-demo/
├── src/
│   ├── client/                    # React frontend
│   │   ├── components/            # UI components
│   │   │   ├── common/            # Shared components
│   │   │   │   ├── Button.tsx
│   │   │   │   ├── Card.tsx
│   │   │   │   ├── Input.tsx
│   │   │   │   ├── Modal.tsx
│   │   │   │   ├── Table.tsx
│   │   │   │   ├── LoadingSpinner.tsx
│   │   │   │   └── ErrorBoundary.tsx
│   │   │   │
│   │   │   ├── trading/           # Trading components
│   │   │   │   ├── OrderForm.tsx
│   │   │   │   ├── OrderBook.tsx
│   │   │   │   ├── TradeHistory.tsx
│   │   │   │   └── PriceChart.tsx
│   │   │   │
│   │   │   ├── portfolio/         # Portfolio components
│   │   │   │   ├── PositionTable.tsx
│   │   │   │   ├── ProfitLossCard.tsx
│   │   │   │   └── AllocationChart.tsx
│   │   │   │
│   │   │   ├── metering/          # Metering components
│   │   │   │   ├── UsageDashboard.tsx
│   │   │   │   ├── CostEstimator.tsx
│   │   │   │   └── QuotaIndicator.tsx
│   │   │   │
│   │   │   └── layout/            # Layout components
│   │   │       ├── Header.tsx
│   │   │       ├── Sidebar.tsx
│   │   │       └── Footer.tsx
│   │   │
│   │   ├── pages/                 # Route pages
│   │   │   ├── DashboardPage.tsx
│   │   │   ├── PortfolioPage.tsx
│   │   │   ├── AnalyticsPage.tsx
│   │   │   └── SettingsPage.tsx
│   │   │
│   │   ├── hooks/                 # Custom React hooks
│   │   │   ├── useAuth.ts
│   │   │   ├── useTrades.ts
│   │   │   ├── usePortfolio.ts
│   │   │   ├── useMetering.ts
│   │   │   └── useWebSocket.ts
│   │   │
│   │   ├── services/              # API clients
│   │   │   ├── api.ts             # Base API client
│   │   │   ├── tradingApi.ts      # Trading endpoints
│   │   │   ├── portfolioApi.ts    # Portfolio endpoints
│   │   │   ├── meteringApi.ts     # Metering endpoints
│   │   │   └── websocket.ts       # WebSocket client
│   │   │
│   │   ├── store/                 # State management
│   │   │   ├── AuthContext.tsx
│   │   │   ├── TradingContext.tsx
│   │   │   ├── PortfolioContext.tsx
│   │   │   ├── MeteringContext.tsx
│   │   │   └── WebSocketContext.tsx
│   │   │
│   │   ├── utils/                 # Utilities
│   │   │   ├── formatters.ts
│   │   │   ├── validators.ts
│   │   │   └── constants.ts
│   │   │
│   │   ├── App.tsx                # Root component
│   │   ├── main.tsx               # Entry point
│   │   └── index.css              # Global styles
│   │
│   ├── server/                    # Express backend
│   │   ├── controllers/           # HTTP handlers
│   │   │   ├── tradeController.ts
│   │   │   ├── portfolioController.ts
│   │   │   ├── meteringController.ts
│   │   │   └── alertController.ts
│   │   │
│   │   ├── services/              # Business logic
│   │   │   ├── tradeService.ts
│   │   │   ├── portfolioService.ts
│   │   │   ├── meteringService.ts
│   │   │   ├── oversightService.ts
│   │   │   └── slackService.ts
│   │   │
│   │   ├── repositories/          # Data access
│   │   │   ├── tradeRepository.ts
│   │   │   ├── portfolioRepository.ts
│   │   │   ├── meteringRepository.ts
│   │   │   └── alertRepository.ts
│   │   │
│   │   ├── grpc/                  # gRPC clients
│   │   │   ├── meteringClient.ts
│   │   │   └── oversightClient.ts
│   │   │
│   │   ├── websocket/             # WebSocket server
│   │   │   └── websocketServer.ts
│   │   │
│   │   ├── middleware/            # Express middleware
│   │   │   ├── auth.ts
│   │   │   ├── validation.ts
│   │   │   ├── errorHandler.ts
│   │   │   └── logging.ts
│   │   │
│   │   ├── db/                    # Database
│   │   │   ├── database.ts        # SQLite connection
│   │   │   ├── schema.sql         # Database schema
│   │   │   └── migrations/        # Schema migrations
│   │   │
│   │   ├── routes/                # Express routes
│   │   │   ├── index.ts
│   │   │   ├── trades.ts
│   │   │   ├── portfolio.ts
│   │   │   ├── metering.ts
│   │   │   └── alerts.ts
│   │   │
│   │   ├── utils/                 # Backend utilities
│   │   │   ├── logger.ts
│   │   │   ├── errors.ts
│   │   │   └── validators.ts
│   │   │
│   │   └── index.ts               # Server entry point
│   │
│   └── shared/                    # Shared types
│       ├── types/
│       │   ├── trade.ts
│       │   ├── portfolio.ts
│       │   ├── metering.ts
│       │   └── api.ts
│       └── constants.ts
│
├── tests/                         # Tests
│   ├── unit/                      # Unit tests
│   │   ├── client/
│   │   └── server/
│   ├── integration/               # Integration tests
│   └── e2e/                       # End-to-end tests
│
├── public/                        # Static files
│   ├── index.html
│   └── favicon.ico
│
├── docs/                          # Documentation
│   ├── sparc/                     # SPARC documents
│   │   ├── 01-specification.md
│   │   ├── 02-pseudocode.md
│   │   ├── 03-architecture.md    # This document
│   │   ├── 04-refinement.md
│   │   └── 05-completion.md
│   ├── api/                       # API documentation
│   └── guides/                    # User guides
│
├── config/                        # Configuration
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── tailwind.config.js
│
├── scripts/                       # Build scripts
│   ├── dev.sh
│   └── build.sh
│
├── package.json
├── package-lock.json
└── README.md
```

---

## 5. Data Flow Diagrams

### 5.1 Trade Execution Flow

```
┌──────────┐
│  User    │
│ (Trader) │
└────┬─────┘
     │
     │ 1. Submit Order
     ▼
┌─────────────────┐
│  OrderForm      │
│  Component      │
└────┬────────────┘
     │
     │ 2. POST /api/trades
     ▼
┌─────────────────┐
│ TradeController │
└────┬────────────┘
     │
     │ 3. executeTrade()
     ▼
┌─────────────────┐
│  TradeService   │
└────┬────────────┘
     │
     ├─────────────────────────────┬──────────────────┬─────────────────┐
     │                             │                  │                 │
     │ 4a. Check Risk             │ 4b. Track        │ 4c. Store       │
     ▼                             ▼                  ▼                 │
┌──────────────┐           ┌──────────────┐   ┌──────────────┐        │
│  Oversight   │           │  Metering    │   │    Trade     │        │
│   Service    │           │   Service    │   │  Repository  │        │
└──────┬───────┘           └──────┬───────┘   └──────┬───────┘        │
       │                          │                   │                │
       │ 5a. gRPC Call           │ 5b. gRPC Call    │ 5c. SQL       │
       ▼                          ▼                   ▼                │
┌──────────────┐           ┌──────────────┐   ┌──────────────┐        │
│creto-oversight│          │creto-metering│   │    SQLite    │        │
│   :50052     │           │   :50051     │   │   Database   │        │
└──────────────┘           └──────────────┘   └──────────────┘        │
                                                                       │
     ┌─────────────────────────────────────────────────────────────────┘
     │
     │ 6. Notify
     │
     ├────────────────────┬─────────────────────┐
     │                    │                     │
     │ 6a. Broadcast      │ 6b. Send Slack     │
     ▼                    ▼                     │
┌──────────────┐   ┌──────────────┐            │
│  WebSocket   │   │    Slack     │            │
│   Server     │   │   Service    │            │
└──────┬───────┘   └──────┬───────┘            │
       │                  │                    │
       │ 7a. Event       │ 7b. Webhook        │
       ▼                  ▼                    │
┌──────────────┐   ┌──────────────┐            │
│   React      │   │    Slack     │            │
│   Client     │   │   Channel    │            │
└──────────────┘   └──────────────┘            │
                                               │
                                               │ 8. Return Response
                                               ▼
                                        ┌──────────────┐
                                        │     User     │
                                        │   (Trader)   │
                                        └──────────────┘
```

### 5.2 Metering Integration Flow

```
┌──────────┐
│   Any    │
│Operation │
└────┬─────┘
     │
     │ 1. Start Operation
     ▼
┌─────────────────┐
│   Service       │
│   Layer         │
└────┬────────────┘
     │
     │ 2. trackOperation()
     ▼
┌─────────────────┐
│ MeteringService │
└────┬────────────┘
     │
     ├────────────────────────┬─────────────────────┐
     │                        │                     │
     │ 3a. gRPC Call         │ 3b. Store Local    │
     ▼                        ▼                     │
┌──────────────┐      ┌──────────────┐             │
│creto-metering│      │  Metering    │             │
│   :50051     │      │  Repository  │             │
└──────┬───────┘      └──────┬───────┘             │
       │                     │                     │
       │ 4. Generate ID     │ 4. Insert Record   │
       ▼                     ▼                     │
┌──────────────┐      ┌──────────────┐             │
│  Metering    │      │    SQLite    │             │
│  Database    │      │   Database   │             │
└──────────────┘      └──────────────┘             │
                                                   │
     ┌─────────────────────────────────────────────┘
     │
     │ 5. Return Metering ID
     ▼
┌─────────────────┐
│   Service       │
│   Layer         │
└────┬────────────┘
     │
     │ 6. Execute Operation
     │ ...
     │ 7. completeOperation()
     ▼
┌─────────────────┐
│ MeteringService │
└────┬────────────┘
     │
     ├────────────────────────┬─────────────────────┐
     │                        │                     │
     │ 8a. Complete gRPC     │ 8b. Update Local   │
     ▼                        ▼                     │
┌──────────────┐      ┌──────────────┐             │
│creto-metering│      │  Metering    │             │
│   :50051     │      │  Repository  │             │
└──────────────┘      └──────┬───────┘             │
                             │                     │
                             │ 9. Update Status   │
                             ▼                     │
                      ┌──────────────┐             │
                      │    SQLite    │             │
                      │   Database   │             │
                      └──────────────┘             │
                                                   │
     ┌─────────────────────────────────────────────┘
     │
     │ 10. Operation Complete
     ▼
┌──────────────┐
│   Service    │
│   Returns    │
└──────────────┘
```

### 5.3 Real-Time Update Flow

```
┌──────────┐
│  React   │
│  Client  │
└────┬─────┘
     │
     │ 1. WebSocket Connect
     ▼
┌─────────────────┐
│  WebSocket      │
│   Server        │
└────┬────────────┘
     │
     │ 2. Subscribe to Channels
     │
     ├─── 'trades' ────┐
     ├─── 'portfolio' ─┤
     └─── 'alerts' ────┤
                        │
     ┌──────────────────┘
     │
     │ 3. Server Event (Trade Executed)
     ▼
┌─────────────────┐
│  TradeService   │
└────┬────────────┘
     │
     │ 4. wsServer.broadcast('trade:executed', trade)
     ▼
┌─────────────────┐
│  WebSocket      │
│   Server        │
└────┬────────────┘
     │
     │ 5. Send to All Connected Clients
     │
     ├────────────┬────────────┬────────────┐
     ▼            ▼            ▼            ▼
┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│Client 1 │  │Client 2 │  │Client 3 │  │Client N │
└────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘
     │            │            │            │
     │ 6. Update  │ 6. Update  │ 6. Update  │ 6. Update
     │    State   │    State   │    State   │    State
     ▼            ▼            ▼            ▼
┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│  React  │  │  React  │  │  React  │  │  React  │
│   UI    │  │   UI    │  │   UI    │  │   UI    │
└─────────┘  └─────────┘  └─────────┘  └─────────┘
```

### 5.4 Oversight Approval Flow

```
┌──────────┐
│   User   │
│  (Large  │
│  Order)  │
└────┬─────┘
     │
     │ 1. Submit Large Order
     ▼
┌─────────────────┐
│  TradeService   │
└────┬────────────┘
     │
     │ 2. checkRiskLimits()
     ▼
┌─────────────────┐
│ OversightService│
└────┬────────────┘
     │
     │ 3. evaluateRisk() gRPC
     ▼
┌─────────────────┐
│creto-oversight  │
│    :50052       │
└────┬────────────┘
     │
     │ 4. Risk Analysis
     │
     ├── Risk Level: LOW ──► 5a. Auto-Approve ──► Execute Trade
     │
     ├── Risk Level: MEDIUM ──► 5b. Flag for Review ──┐
     │                                                 │
     └── Risk Level: HIGH ──► 5c. Require Approval ───┤
                                                       │
     ┌─────────────────────────────────────────────────┘
     │
     │ 6. Send Approval Request
     ▼
┌─────────────────┐
│  SlackService   │
└────┬────────────┘
     │
     │ 7. Webhook to Slack
     ▼
┌─────────────────┐
│     Slack       │
│    Channel      │
└────┬────────────┘
     │
     │ 8. Human Review
     ▼
┌─────────────────┐
│   Approver      │
│   (Manager)     │
└────┬────────────┘
     │
     │ 9. Approve/Reject
     ▼
┌─────────────────┐
│creto-oversight  │
│    :50052       │
└────┬────────────┘
     │
     │ 10. Update Status
     ▼
┌─────────────────┐
│  TradeService   │
│ (polling/webhook)│
└────┬────────────┘
     │
     ├── Approved ──► Execute Trade
     │
     └── Rejected ──► Notify User & Cancel
```

---

## 6. Security Architecture

### 6.1 Security Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                     SECURITY ARCHITECTURE                       │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              1. TRANSPORT LAYER SECURITY                  │ │
│  │  - HTTPS (TLS 1.3) for all HTTP traffic                  │ │
│  │  - WSS (WebSocket Secure) for real-time connections      │ │
│  │  - gRPC with TLS for service communication                │ │
│  └───────────────────────────────────────────────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │           2. AUTHENTICATION & AUTHORIZATION               │ │
│  │  - JWT (JSON Web Tokens) for session management          │ │
│  │  - Role-Based Access Control (RBAC)                      │ │
│  │  - API Key authentication for service-to-service         │ │
│  └───────────────────────────────────────────────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              3. INPUT VALIDATION                          │ │
│  │  - Zod schema validation on all inputs                   │ │
│  │  - SQL injection prevention (parameterized queries)      │ │
│  │  - XSS prevention (sanitization, CSP headers)            │ │
│  │  - CSRF protection (CSRF tokens)                         │ │
│  └───────────────────────────────────────────────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              4. RATE LIMITING                             │ │
│  │  - API rate limits (100 req/min per user)                │ │
│  │  - WebSocket message throttling                          │ │
│  │  - DDoS protection                                        │ │
│  └───────────────────────────────────────────────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              5. DATA PROTECTION                           │ │
│  │  - Sensitive data encryption at rest                     │ │
│  │  - Secrets management (environment variables)            │ │
│  │  - PII data masking in logs                              │ │
│  └───────────────────────────────────────────────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              6. AUDIT & MONITORING                        │ │
│  │  - Structured logging (Winston)                          │ │
│  │  - Security event tracking                               │ │
│  │  - Anomaly detection                                      │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Authentication Flow

```typescript
// JWT-based Authentication

// 1. Login Request
POST /api/auth/login
{
  "email": "trader@example.com",
  "password": "secure_password"
}

// 2. Server validates credentials
// 3. Generate JWT token
const token = jwt.sign(
  { userId: user.id, email: user.email, role: user.role },
  process.env.JWT_SECRET,
  { expiresIn: '24h' }
);

// 4. Return token to client
Response: {
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { "id": "123", "email": "trader@example.com", "role": "trader" }
}

// 5. Client stores token (localStorage/sessionStorage)
// 6. Include token in subsequent requests
GET /api/trades
Headers: {
  "Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}

// 7. Server validates token on each request
const authMiddleware = (req, res, next) => {
  const token = req.headers.authorization?.split(' ')[1];
  if (!token) return res.status(401).json({ error: 'Unauthorized' });

  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    req.user = decoded;
    next();
  } catch (err) {
    res.status(401).json({ error: 'Invalid token' });
  }
};
```

### 6.3 Authorization (RBAC)

```typescript
// Role-Based Access Control

enum Role {
  ADMIN = 'admin',
  TRADER = 'trader',
  VIEWER = 'viewer'
}

interface Permission {
  resource: string;
  actions: string[];
}

const rolePermissions: Record<Role, Permission[]> = {
  [Role.ADMIN]: [
    { resource: 'trades', actions: ['create', 'read', 'update', 'delete'] },
    { resource: 'users', actions: ['create', 'read', 'update', 'delete'] },
    { resource: 'settings', actions: ['read', 'update'] }
  ],
  [Role.TRADER]: [
    { resource: 'trades', actions: ['create', 'read'] },
    { resource: 'portfolio', actions: ['read'] }
  ],
  [Role.VIEWER]: [
    { resource: 'trades', actions: ['read'] },
    { resource: 'portfolio', actions: ['read'] }
  ]
};

// Authorization middleware
const authorize = (resource: string, action: string) => {
  return (req: Request, res: Response, next: NextFunction) => {
    const userRole = req.user.role as Role;
    const permissions = rolePermissions[userRole];

    const hasPermission = permissions.some(
      p => p.resource === resource && p.actions.includes(action)
    );

    if (!hasPermission) {
      return res.status(403).json({ error: 'Forbidden' });
    }

    next();
  };
};

// Usage
router.post('/api/trades', authMiddleware, authorize('trades', 'create'), tradeController.create);
router.get('/api/trades', authMiddleware, authorize('trades', 'read'), tradeController.list);
router.delete('/api/trades/:id', authMiddleware, authorize('trades', 'delete'), tradeController.delete);
```

### 6.4 Input Validation

```typescript
// Zod Schema Validation

import { z } from 'zod';

// Trade order schema
const TradeOrderSchema = z.object({
  symbol: z.string().min(1).max(10).regex(/^[A-Z]+$/),
  type: z.enum(['BUY', 'SELL']),
  quantity: z.number().positive().int(),
  price: z.number().positive(),
  orderType: z.enum(['MARKET', 'LIMIT']),
  timeInForce: z.enum(['GTC', 'DAY', 'IOC']).optional()
});

// Validation middleware
const validate = (schema: z.ZodSchema) => {
  return (req: Request, res: Response, next: NextFunction) => {
    try {
      schema.parse(req.body);
      next();
    } catch (err) {
      if (err instanceof z.ZodError) {
        res.status(400).json({
          error: 'Validation failed',
          details: err.errors
        });
      } else {
        next(err);
      }
    }
  };
};

// Usage
router.post('/api/trades', validate(TradeOrderSchema), tradeController.create);
```

### 6.5 CORS Configuration

```typescript
// CORS setup for frontend-backend communication

import cors from 'cors';

const corsOptions = {
  origin: process.env.NODE_ENV === 'production'
    ? ['https://trading-demo.example.com']
    : ['http://localhost:5173', 'http://localhost:3000'],
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
  allowedHeaders: ['Content-Type', 'Authorization']
};

app.use(cors(corsOptions));
```

### 6.6 Security Headers

```typescript
// Security headers middleware

import helmet from 'helmet';

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'", "'unsafe-inline'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'", "ws://localhost:3000", "wss://trading-demo.example.com"]
    }
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true
  }
}));
```

---

## 7. Deployment Architecture

### 7.1 Development Environment

```
┌─────────────────────────────────────────────────────────────────┐
│                    DEVELOPMENT ENVIRONMENT                      │
│                                                                 │
│  Developer Machine (localhost)                                 │
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────┐              │
│  │  Vite Dev Server│         │ Express Server  │              │
│  │   :5173         │◄────────│   :3000         │              │
│  │  (Frontend)     │  Proxy  │  (Backend)      │              │
│  └─────────────────┘         └─────────────────┘              │
│                                      │                         │
│                                      │                         │
│  ┌──────────────────────────────────┼────────────────────┐    │
│  │                                   │                    │    │
│  ▼                                   ▼                    ▼    │
│ ┌────────────┐              ┌──────────────┐      ┌─────────┐ │
│ │  SQLite    │              │creto-metering│      │ creto-  │ │
│ │trading.db  │              │   :50051     │      │oversight│ │
│ └────────────┘              └──────────────┘      │ :50052  │ │
│                                                    └─────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 Production Environment (Docker)

```
┌─────────────────────────────────────────────────────────────────┐
│                    PRODUCTION ENVIRONMENT                       │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                      NGINX (Reverse Proxy)                │ │
│  │                           :80/:443                        │ │
│  └───────────────────────────┬───────────────────────────────┘ │
│                              │                                 │
│              ┌───────────────┼───────────────────┐             │
│              │               │                   │             │
│              ▼               ▼                   ▼             │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │  Static Files   │ │  API Requests   │ │  WebSocket      │ │
│  │   (React SPA)   │ │                 │ │  Connections    │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
│                              │                   │             │
│                              ▼                   ▼             │
│                      ┌─────────────────────────────────────┐   │
│                      │   Node.js Container                 │   │
│                      │   (Express + WebSocket)             │   │
│                      └─────────────────┬───────────────────┘   │
│                                        │                       │
│                      ┌─────────────────┼───────────────────┐   │
│                      │                 │                   │   │
│                      ▼                 ▼                   ▼   │
│              ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│              │    SQLite    │  │creto-metering│  │  creto-  │ │
│              │   Volume     │  │   :50051     │  │oversight │ │
│              └──────────────┘  └──────────────┘  │ :50052   │ │
│                                                   └──────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 7.3 Docker Compose Configuration

```yaml
# docker-compose.yml

version: '3.8'

services:
  # Frontend + Backend (Node.js)
  trading-demo:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - JWT_SECRET=${JWT_SECRET}
      - METERING_GRPC_URL=creto-metering:50051
      - OVERSIGHT_GRPC_URL=creto-oversight:50052
      - SLACK_WEBHOOK_URL=${SLACK_WEBHOOK_URL}
    volumes:
      - ./data:/app/data  # SQLite database
    depends_on:
      - creto-metering
      - creto-oversight
    networks:
      - trading-network

  # NGINX Reverse Proxy
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - trading-demo
    networks:
      - trading-network

  # creto-metering (assumed to be running)
  creto-metering:
    image: creto/metering:latest
    ports:
      - "50051:50051"
    networks:
      - trading-network

  # creto-oversight (assumed to be running)
  creto-oversight:
    image: creto/oversight:latest
    ports:
      - "50052:50052"
    networks:
      - trading-network

networks:
  trading-network:
    driver: bridge

volumes:
  trading-data:
```

### 7.4 Dockerfile

```dockerfile
# Multi-stage build for optimal image size

# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build:client

# Stage 2: Build backend
FROM node:20-alpine AS backend-builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY src/server ./src/server
COPY src/shared ./src/shared
RUN npm run build:server

# Stage 3: Production image
FROM node:20-alpine
WORKDIR /app

# Copy backend build
COPY --from=backend-builder /app/node_modules ./node_modules
COPY --from=backend-builder /app/dist ./dist

# Copy frontend build
COPY --from=frontend-builder /app/dist/client ./dist/client

# Create data directory
RUN mkdir -p /app/data

EXPOSE 3000

CMD ["node", "dist/server/index.js"]
```

---

## 8. Architecture Decision Records

### ADR-001: React for Frontend Framework

**Status**: Accepted

**Context**: Need to choose a frontend framework for building the trading demo UI.

**Decision**: Use React 18 with TypeScript and Vite.

**Rationale**:
- Large ecosystem and community support
- TypeScript integration for type safety
- Vite provides fast development experience
- Component reusability
- Strong testing support (Vitest, Testing Library)

**Consequences**:
- Positive: Fast development, excellent tooling, large talent pool
- Negative: Need to manage state carefully, potential bundle size concerns

**Alternatives Considered**:
- Vue.js: Simpler learning curve but smaller ecosystem
- Angular: Too heavyweight for this demo
- Svelte: Less mature ecosystem

---

### ADR-002: SQLite for Demo Database

**Status**: Accepted

**Context**: Need to choose a database for storing demo trading data.

**Decision**: Use SQLite with better-sqlite3 driver.

**Rationale**:
- Zero configuration (no separate database server)
- Sufficient for demo/prototype workloads
- Easy local development
- Simple deployment
- Supports transactions and ACID

**Consequences**:
- Positive: Simple setup, fast for read-heavy workloads, portable
- Negative: Limited scalability, no built-in replication

**Alternatives Considered**:
- PostgreSQL: Overkill for demo, requires separate server
- MongoDB: Unnecessary complexity for structured trading data
- In-memory: Data loss on restart

---

### ADR-003: gRPC for Service Integration

**Status**: Accepted

**Context**: Need to integrate with creto-metering and creto-oversight services.

**Decision**: Use gRPC with @grpc/grpc-js and @grpc/proto-loader.

**Rationale**:
- Services already expose gRPC interfaces (ports 50051, 50052)
- Efficient binary protocol
- Strong typing with Protocol Buffers
- Bi-directional streaming support

**Consequences**:
- Positive: Efficient, type-safe, compatible with existing services
- Negative: Requires .proto files, less human-readable than REST

**Alternatives Considered**:
- REST API: Services don't expose REST endpoints
- GraphQL: Not supported by existing services

---

### ADR-004: WebSocket for Real-Time Updates

**Status**: Accepted

**Context**: Need to push real-time trade updates to frontend.

**Decision**: Use WebSocket (ws library) for bidirectional real-time communication.

**Rationale**:
- Low latency for trade updates
- Efficient for frequent updates
- Browser support (WebSocket API)
- Simpler than Server-Sent Events for bidirectional communication

**Consequences**:
- Positive: Real-time updates, low overhead
- Negative: Need to handle reconnection logic, connection pooling

**Alternatives Considered**:
- Polling: High latency, inefficient
- Server-Sent Events: Unidirectional only
- Socket.io: Added complexity, not needed

---

### ADR-005: Monorepo Structure

**Status**: Accepted

**Context**: Need to organize frontend and backend code.

**Decision**: Use a monorepo with `src/client` and `src/server` directories.

**Rationale**:
- Shared TypeScript types between frontend and backend
- Single build process
- Simplified dependency management
- Easier to keep frontend and backend in sync

**Consequences**:
- Positive: Code sharing, simplified deployment
- Negative: Single repository can grow large

**Alternatives Considered**:
- Separate repositories: Added complexity for shared types
- Nx/Turborepo: Overkill for two packages

---

### ADR-006: TailwindCSS for Styling

**Status**: Accepted

**Context**: Need to choose a CSS approach for UI styling.

**Decision**: Use TailwindCSS utility-first framework.

**Rationale**:
- Rapid prototyping with utility classes
- Consistent design system
- Small production bundle (purging unused CSS)
- No CSS naming conflicts
- Good TypeScript integration

**Consequences**:
- Positive: Fast development, consistent UI, small bundle
- Negative: HTML can become verbose with many classes

**Alternatives Considered**:
- CSS Modules: More verbose, harder to maintain consistency
- Styled Components: Runtime overhead, complexity
- Bootstrap: Too opinionated, larger bundle

---

### ADR-007: Context API for State Management

**Status**: Accepted

**Context**: Need to manage global state (auth, trading, portfolio, metering).

**Decision**: Use React Context API with useReducer hooks.

**Rationale**:
- Built into React (no additional dependencies)
- Sufficient for moderate state complexity
- Good TypeScript support
- Easy to understand and maintain

**Consequences**:
- Positive: Simple, lightweight, no extra dependencies
- Negative: Can cause re-renders if not optimized, less tooling than Redux

**Alternatives Considered**:
- Redux Toolkit: Too complex for this demo
- Zustand: Additional dependency, overkill
- Jotai/Recoil: Experimental, smaller ecosystem

---

### ADR-008: Vitest for Testing

**Status**: Accepted

**Context**: Need to choose a testing framework for unit and integration tests.

**Decision**: Use Vitest with React Testing Library and Playwright for E2E.

**Rationale**:
- Vite-native (fast test execution)
- Jest-compatible API
- Built-in TypeScript support
- Excellent watch mode
- Playwright for comprehensive E2E testing

**Consequences**:
- Positive: Fast tests, good DX, unified tooling
- Negative: Smaller ecosystem than Jest

**Alternatives Considered**:
- Jest: Slower, requires additional configuration with Vite
- Mocha/Chai: More setup required, less integrated

---

## Summary

This architecture document defines a **comprehensive, production-ready system design** for the Trading Demo. Key architectural highlights:

### Strengths
1. **Layered Architecture**: Clear separation of concerns (presentation, application, domain, infrastructure)
2. **Real-Time Capabilities**: WebSocket integration for live updates
3. **Service Integration**: gRPC clients for creto-metering and creto-oversight
4. **Type Safety**: TypeScript throughout the stack
5. **Security**: Multi-layered security with JWT, RBAC, input validation
6. **Scalability**: Modular design allows for future enhancements
7. **Observability**: Structured logging and metering integration

### Technology Decisions
- **Frontend**: React 18, TypeScript, Vite, TailwindCSS
- **Backend**: Node.js, Express, TypeScript
- **Database**: SQLite (better-sqlite3)
- **Real-Time**: WebSocket (ws)
- **Integration**: gRPC (@grpc/grpc-js)
- **Testing**: Vitest, React Testing Library, Playwright

### Next Steps (Refinement Phase)
1. Implement database schema and migrations
2. Build repository layer with SQLite
3. Create gRPC client wrappers
4. Implement service layer with business logic
5. Build Express API controllers
6. Implement WebSocket server
7. Create React components and pages
8. Add authentication and authorization
9. Write comprehensive tests
10. Deploy with Docker

---

**Document Status**: ✅ Complete
**Ready for**: Phase 4 - Refinement (TDD Implementation)
