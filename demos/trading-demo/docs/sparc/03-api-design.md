# Trading Demo - API Design Specification

**SPARC Phase:** Architecture
**Component:** API Design
**Version:** 1.0.0
**Last Updated:** 2025-12-26

## Overview

This document defines the complete API specification for the Trading Demo, including REST endpoints, WebSocket protocol, authentication, and rate limiting.

## Table of Contents

1. [API Architecture](#api-architecture)
2. [REST API Endpoints](#rest-api-endpoints)
3. [WebSocket Protocol](#websocket-protocol)
4. [Authentication & Authorization](#authentication--authorization)
5. [Rate Limiting](#rate-limiting)
6. [Error Handling](#error-handling)
7. [OpenAPI Specification](#openapi-specification)

---

## API Architecture

### Base URL
- **Production:** `https://api.trading-demo.example.com`
- **Development:** `http://localhost:3001`
- **API Version:** `/api/v1`

### Communication Protocols
- **REST API:** Request/response operations
- **WebSocket:** Real-time updates and streaming data
- **HTTP/2:** Enabled for improved performance

### Content Types
- Request: `application/json`
- Response: `application/json`
- Error: `application/problem+json` (RFC 7807)

---

## REST API Endpoints

### Agent Management

#### Create Agent
```http
POST /api/v1/agents
Content-Type: application/json
Authorization: Bearer <token>

Request Body:
{
  "name": "string (required, 1-100 chars)",
  "strategy": "conservative | moderate | aggressive (required)",
  "initialCapital": "number (required, min: 1000, max: 1000000)",
  "riskTolerance": "number (required, 0.01-0.50)",
  "tradingPairs": ["string"] (required, 1-10 pairs),
  "config": {
    "maxPositionSize": "number (optional)",
    "stopLossPercent": "number (optional)",
    "takeProfitPercent": "number (optional)"
  }
}

Response 201 Created:
{
  "id": "agent_abc123",
  "name": "Conservative Trader",
  "strategy": "conservative",
  "status": "inactive",
  "portfolio": {
    "totalValue": 10000.00,
    "cash": 10000.00,
    "positions": []
  },
  "createdAt": "2025-12-26T10:00:00Z",
  "updatedAt": "2025-12-26T10:00:00Z"
}
```

#### List Agents
```http
GET /api/v1/agents?status=active&strategy=conservative&limit=20&offset=0
Authorization: Bearer <token>

Response 200 OK:
{
  "agents": [
    {
      "id": "agent_abc123",
      "name": "Conservative Trader",
      "strategy": "conservative",
      "status": "active",
      "portfolio": {
        "totalValue": 10500.00,
        "cash": 5000.00,
        "positionCount": 3
      },
      "performance": {
        "totalReturn": 5.00,
        "dailyReturn": 0.5
      },
      "createdAt": "2025-12-26T10:00:00Z"
    }
  ],
  "pagination": {
    "total": 45,
    "limit": 20,
    "offset": 0,
    "hasMore": true
  }
}
```

#### Get Agent Details
```http
GET /api/v1/agents/:id
Authorization: Bearer <token>

Response 200 OK:
{
  "id": "agent_abc123",
  "name": "Conservative Trader",
  "strategy": "conservative",
  "status": "active",
  "portfolio": {
    "totalValue": 10500.00,
    "cash": 5000.00,
    "positions": [
      {
        "symbol": "BTC/USD",
        "quantity": 0.1,
        "averagePrice": 50000.00,
        "currentPrice": 51000.00,
        "unrealizedPnL": 100.00,
        "unrealizedPnLPercent": 2.00
      }
    ]
  },
  "config": {
    "initialCapital": 10000.00,
    "riskTolerance": 0.15,
    "maxPositionSize": 2000.00,
    "stopLossPercent": 5.00,
    "takeProfitPercent": 10.00
  },
  "usage": {
    "totalCalls": 1250,
    "totalTokens": 125000,
    "estimatedCost": 2.50
  },
  "createdAt": "2025-12-26T10:00:00Z",
  "updatedAt": "2025-12-26T12:30:00Z",
  "lastActiveAt": "2025-12-26T12:30:00Z"
}
```

#### Update Agent
```http
PATCH /api/v1/agents/:id
Content-Type: application/json
Authorization: Bearer <token>

Request Body:
{
  "name": "string (optional)",
  "riskTolerance": "number (optional)",
  "config": {
    "maxPositionSize": "number (optional)",
    "stopLossPercent": "number (optional)"
  }
}

Response 200 OK:
{
  "id": "agent_abc123",
  "name": "Updated Name",
  "updatedAt": "2025-12-26T13:00:00Z"
}
```

#### Activate Agent
```http
POST /api/v1/agents/:id/activate
Authorization: Bearer <token>

Response 200 OK:
{
  "id": "agent_abc123",
  "status": "active",
  "activatedAt": "2025-12-26T13:00:00Z"
}
```

#### Pause Agent
```http
POST /api/v1/agents/:id/pause
Authorization: Bearer <token>

Response 200 OK:
{
  "id": "agent_abc123",
  "status": "paused",
  "pausedAt": "2025-12-26T13:00:00Z"
}
```

#### Delete Agent
```http
DELETE /api/v1/agents/:id
Authorization: Bearer <token>

Response 204 No Content
```

### Portfolio Operations

#### Get Portfolio
```http
GET /api/v1/agents/:id/portfolio
Authorization: Bearer <token>

Response 200 OK:
{
  "agentId": "agent_abc123",
  "totalValue": 10500.00,
  "cash": 5000.00,
  "investedValue": 5500.00,
  "positions": [
    {
      "symbol": "BTC/USD",
      "quantity": 0.1,
      "averagePrice": 50000.00,
      "currentPrice": 51000.00,
      "marketValue": 5100.00,
      "costBasis": 5000.00,
      "unrealizedPnL": 100.00,
      "unrealizedPnLPercent": 2.00,
      "allocation": 48.57
    }
  ],
  "updatedAt": "2025-12-26T13:00:00Z"
}
```

#### List Positions
```http
GET /api/v1/agents/:id/positions?symbol=BTC/USD
Authorization: Bearer <token>

Response 200 OK:
{
  "positions": [
    {
      "id": "pos_xyz789",
      "symbol": "BTC/USD",
      "quantity": 0.1,
      "side": "long",
      "entryPrice": 50000.00,
      "currentPrice": 51000.00,
      "unrealizedPnL": 100.00,
      "duration": "2 days",
      "openedAt": "2025-12-24T10:00:00Z"
    }
  ]
}
```

#### Get Performance Metrics
```http
GET /api/v1/agents/:id/performance?period=30d
Authorization: Bearer <token>

Response 200 OK:
{
  "agentId": "agent_abc123",
  "period": "30d",
  "metrics": {
    "totalReturn": 5.00,
    "totalReturnAmount": 500.00,
    "dailyReturn": 0.16,
    "sharpeRatio": 1.25,
    "maxDrawdown": -2.5,
    "winRate": 65.00,
    "totalTrades": 42,
    "profitableTrades": 27,
    "averageWin": 75.00,
    "averageLoss": -45.00,
    "profitFactor": 1.67
  },
  "equity": [
    {
      "timestamp": "2025-11-26T00:00:00Z",
      "value": 10000.00
    },
    {
      "timestamp": "2025-12-26T00:00:00Z",
      "value": 10500.00
    }
  ]
}
```

### Trade Operations

#### Submit Trade
```http
POST /api/v1/agents/:id/trades
Content-Type: application/json
Authorization: Bearer <token>

Request Body:
{
  "symbol": "BTC/USD (required)",
  "side": "buy | sell (required)",
  "type": "market | limit (required)",
  "quantity": "number (required, > 0)",
  "price": "number (required for limit orders)",
  "stopLoss": "number (optional)",
  "takeProfit": "number (optional)",
  "rationale": "string (optional, max 500 chars)"
}

Response 201 Created:
{
  "id": "trade_def456",
  "agentId": "agent_abc123",
  "symbol": "BTC/USD",
  "side": "buy",
  "type": "market",
  "quantity": 0.05,
  "status": "pending_oversight",
  "estimatedPrice": 51000.00,
  "estimatedValue": 2550.00,
  "rationale": "Bullish momentum detected",
  "oversightRequired": true,
  "createdAt": "2025-12-26T13:00:00Z"
}
```

#### List Trades
```http
GET /api/v1/agents/:id/trades?status=executed&limit=50&offset=0
Authorization: Bearer <token>

Response 200 OK:
{
  "trades": [
    {
      "id": "trade_def456",
      "symbol": "BTC/USD",
      "side": "buy",
      "type": "market",
      "quantity": 0.05,
      "price": 51000.00,
      "value": 2550.00,
      "status": "executed",
      "pnl": null,
      "executedAt": "2025-12-26T13:05:00Z"
    }
  ],
  "pagination": {
    "total": 127,
    "limit": 50,
    "offset": 0
  }
}
```

#### Get Trade Details
```http
GET /api/v1/agents/:id/trades/:tradeId
Authorization: Bearer <token>

Response 200 OK:
{
  "id": "trade_def456",
  "agentId": "agent_abc123",
  "symbol": "BTC/USD",
  "side": "buy",
  "type": "market",
  "quantity": 0.05,
  "requestedPrice": null,
  "executedPrice": 51000.00,
  "value": 2550.00,
  "fees": 2.55,
  "status": "executed",
  "rationale": "Bullish momentum detected",
  "oversightRequest": {
    "id": "req_ghi789",
    "status": "approved",
    "approvedAt": "2025-12-26T13:03:00Z",
    "approvedBy": "user_123"
  },
  "createdAt": "2025-12-26T13:00:00Z",
  "executedAt": "2025-12-26T13:05:00Z"
}
```

#### Cancel Trade
```http
DELETE /api/v1/agents/:id/trades/:tradeId
Authorization: Bearer <token>

Response 200 OK:
{
  "id": "trade_def456",
  "status": "cancelled",
  "cancelledAt": "2025-12-26T13:10:00Z"
}
```

### Metering Operations

#### Get Usage Summary
```http
GET /api/v1/agents/:id/usage?period=7d
Authorization: Bearer <token>

Response 200 OK:
{
  "agentId": "agent_abc123",
  "period": "7d",
  "summary": {
    "totalCalls": 1250,
    "totalTokens": 125000,
    "inputTokens": 75000,
    "outputTokens": 50000,
    "estimatedCost": 2.50,
    "averageCallsPerDay": 178.57
  },
  "quotaStatus": {
    "dailyLimit": 500,
    "dailyUsed": 178,
    "dailyRemaining": 322,
    "monthlyLimit": 15000,
    "monthlyUsed": 4250,
    "monthlyRemaining": 10750
  }
}
```

#### Get Usage Breakdown
```http
GET /api/v1/agents/:id/usage/breakdown?period=24h&granularity=1h
Authorization: Bearer <token>

Response 200 OK:
{
  "agentId": "agent_abc123",
  "period": "24h",
  "granularity": "1h",
  "breakdown": [
    {
      "timestamp": "2025-12-26T00:00:00Z",
      "calls": 15,
      "tokens": 1500,
      "cost": 0.03
    },
    {
      "timestamp": "2025-12-26T01:00:00Z",
      "calls": 12,
      "tokens": 1200,
      "cost": 0.024
    }
  ],
  "byOperation": {
    "marketAnalysis": {
      "calls": 450,
      "tokens": 45000,
      "cost": 0.90
    },
    "tradeDecision": {
      "calls": 180,
      "tokens": 36000,
      "cost": 0.72
    }
  }
}
```

#### Get Quota Status
```http
GET /api/v1/agents/:id/quota
Authorization: Bearer <token>

Response 200 OK:
{
  "agentId": "agent_abc123",
  "quotas": {
    "calls": {
      "daily": { "limit": 500, "used": 178, "remaining": 322 },
      "monthly": { "limit": 15000, "used": 4250, "remaining": 10750 }
    },
    "tokens": {
      "daily": { "limit": 50000, "used": 17800, "remaining": 32200 },
      "monthly": { "limit": 1500000, "used": 425000, "remaining": 1075000 }
    },
    "cost": {
      "daily": { "limit": 10.00, "used": 3.56, "remaining": 6.44 },
      "monthly": { "limit": 300.00, "used": 85.00, "remaining": 215.00 }
    },
    "trades": {
      "daily": { "limit": 50, "used": 12, "remaining": 38 },
      "monthly": { "limit": 1000, "used": 127, "remaining": 873 }
    }
  },
  "alerts": [
    {
      "type": "warning",
      "metric": "daily_calls",
      "message": "Usage at 35% of daily limit"
    }
  ]
}
```

### Oversight Operations

#### List Oversight Requests
```http
GET /api/v1/oversight/requests?status=pending&agentId=agent_abc123&limit=20
Authorization: Bearer <token>

Response 200 OK:
{
  "requests": [
    {
      "id": "req_ghi789",
      "agentId": "agent_abc123",
      "agentName": "Conservative Trader",
      "type": "trade_approval",
      "status": "pending",
      "priority": "normal",
      "trade": {
        "symbol": "BTC/USD",
        "side": "buy",
        "quantity": 0.05,
        "estimatedValue": 2550.00
      },
      "rationale": "Bullish momentum detected",
      "createdAt": "2025-12-26T13:00:00Z",
      "expiresAt": "2025-12-26T13:15:00Z"
    }
  ],
  "pagination": {
    "total": 8,
    "limit": 20,
    "offset": 0
  }
}
```

#### Get Oversight Request Details
```http
GET /api/v1/oversight/requests/:id
Authorization: Bearer <token>

Response 200 OK:
{
  "id": "req_ghi789",
  "agentId": "agent_abc123",
  "agentName": "Conservative Trader",
  "type": "trade_approval",
  "status": "pending",
  "priority": "normal",
  "trade": {
    "id": "trade_def456",
    "symbol": "BTC/USD",
    "side": "buy",
    "type": "market",
    "quantity": 0.05,
    "estimatedPrice": 51000.00,
    "estimatedValue": 2550.00,
    "stopLoss": 48450.00,
    "takeProfit": 56100.00
  },
  "context": {
    "currentPortfolioValue": 10500.00,
    "availableCash": 5000.00,
    "currentPositions": 3,
    "recentPerformance": 5.00
  },
  "rationale": "Technical analysis shows strong buy signal with RSI at 35 and MACD crossover. Risk/reward ratio of 1:2.2",
  "analysis": {
    "riskAmount": 127.50,
    "riskPercent": 1.21,
    "potentialProfit": 280.50,
    "riskRewardRatio": 2.20
  },
  "createdAt": "2025-12-26T13:00:00Z",
  "expiresAt": "2025-12-26T13:15:00Z"
}
```

#### Approve Oversight Request
```http
POST /api/v1/oversight/requests/:id/approve
Content-Type: application/json
Authorization: Bearer <token>

Request Body:
{
  "comment": "string (optional, max 500 chars)"
}

Response 200 OK:
{
  "id": "req_ghi789",
  "status": "approved",
  "approvedBy": "user_123",
  "approvedAt": "2025-12-26T13:03:00Z",
  "comment": "Approved - risk parameters acceptable"
}
```

#### Reject Oversight Request
```http
POST /api/v1/oversight/requests/:id/reject
Content-Type: application/json
Authorization: Bearer <token>

Request Body:
{
  "reason": "string (required, max 500 chars)"
}

Response 200 OK:
{
  "id": "req_ghi789",
  "status": "rejected",
  "rejectedBy": "user_123",
  "rejectedAt": "2025-12-26T13:03:00Z",
  "reason": "Position size too large for current market conditions"
}
```

---

## WebSocket Protocol

### Connection

```javascript
// Connect with JWT token
const ws = new WebSocket('ws://localhost:3001/ws?token=<jwt_token>');

ws.onopen = () => {
  console.log('Connected to Trading Demo WebSocket');
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = (event) => {
  console.log('Disconnected:', event.code, event.reason);
};
```

### Message Format

All WebSocket messages follow this structure:

```json
{
  "type": "string (required)",
  "channel": "string (optional)",
  "data": "object (optional)",
  "timestamp": "ISO8601 string",
  "messageId": "string (uuid)"
}
```

### Client → Server Messages

#### Subscribe to Channel
```json
{
  "type": "subscribe",
  "channel": "portfolio:agent_abc123"
}
```

Supported channels:
- `portfolio:<agentId>` - Portfolio updates
- `trades:<agentId>` - Trade executions
- `oversight:all` - All oversight requests (admin only)
- `oversight:<agentId>` - Agent-specific oversight
- `usage:<agentId>` - Usage metrics
- `alerts:<agentId>` - Alerts and notifications
- `market:<symbol>` - Market data for symbol

#### Unsubscribe from Channel
```json
{
  "type": "unsubscribe",
  "channel": "portfolio:agent_abc123"
}
```

#### Ping (Keepalive)
```json
{
  "type": "ping"
}
```

### Server → Client Messages

#### Subscription Confirmed
```json
{
  "type": "subscribed",
  "channel": "portfolio:agent_abc123",
  "timestamp": "2025-12-26T13:00:00Z",
  "messageId": "msg_123"
}
```

#### Pong Response
```json
{
  "type": "pong",
  "timestamp": "2025-12-26T13:00:00Z"
}
```

#### Portfolio Update
```json
{
  "type": "portfolio:update",
  "channel": "portfolio:agent_abc123",
  "data": {
    "agentId": "agent_abc123",
    "totalValue": 10550.00,
    "cash": 5000.00,
    "positions": [
      {
        "symbol": "BTC/USD",
        "quantity": 0.1,
        "currentPrice": 51500.00,
        "unrealizedPnL": 150.00
      }
    ]
  },
  "timestamp": "2025-12-26T13:00:00Z",
  "messageId": "msg_124"
}
```

#### Trade Executed
```json
{
  "type": "trade:executed",
  "channel": "trades:agent_abc123",
  "data": {
    "tradeId": "trade_def456",
    "agentId": "agent_abc123",
    "symbol": "BTC/USD",
    "side": "buy",
    "quantity": 0.05,
    "price": 51000.00,
    "value": 2550.00,
    "fees": 2.55,
    "executedAt": "2025-12-26T13:05:00Z"
  },
  "timestamp": "2025-12-26T13:05:00Z",
  "messageId": "msg_125"
}
```

#### Trade Pending Oversight
```json
{
  "type": "trade:pending",
  "channel": "trades:agent_abc123",
  "data": {
    "tradeId": "trade_def456",
    "agentId": "agent_abc123",
    "symbol": "BTC/USD",
    "side": "buy",
    "quantity": 0.05,
    "status": "pending_oversight",
    "oversightRequestId": "req_ghi789"
  },
  "timestamp": "2025-12-26T13:00:00Z",
  "messageId": "msg_126"
}
```

#### Oversight Request Created
```json
{
  "type": "oversight:created",
  "channel": "oversight:all",
  "data": {
    "requestId": "req_ghi789",
    "agentId": "agent_abc123",
    "type": "trade_approval",
    "priority": "normal",
    "trade": {
      "symbol": "BTC/USD",
      "side": "buy",
      "quantity": 0.05
    },
    "expiresAt": "2025-12-26T13:15:00Z"
  },
  "timestamp": "2025-12-26T13:00:00Z",
  "messageId": "msg_127"
}
```

#### Oversight Request Resolved
```json
{
  "type": "oversight:resolved",
  "channel": "oversight:agent_abc123",
  "data": {
    "requestId": "req_ghi789",
    "agentId": "agent_abc123",
    "decision": "approved",
    "resolvedBy": "user_123",
    "resolvedAt": "2025-12-26T13:03:00Z"
  },
  "timestamp": "2025-12-26T13:03:00Z",
  "messageId": "msg_128"
}
```

#### Usage Updated
```json
{
  "type": "usage:updated",
  "channel": "usage:agent_abc123",
  "data": {
    "agentId": "agent_abc123",
    "totalCalls": 1251,
    "totalTokens": 125100,
    "estimatedCost": 2.502,
    "quotaStatus": {
      "dailyRemaining": 321,
      "monthlyRemaining": 10749
    }
  },
  "timestamp": "2025-12-26T13:05:00Z",
  "messageId": "msg_129"
}
```

#### Alert Triggered
```json
{
  "type": "alert:triggered",
  "channel": "alerts:agent_abc123",
  "data": {
    "agentId": "agent_abc123",
    "alertType": "quota_warning",
    "severity": "warning",
    "message": "Daily token usage at 80% of limit",
    "metric": "daily_tokens",
    "currentValue": 40000,
    "threshold": 50000
  },
  "timestamp": "2025-12-26T13:00:00Z",
  "messageId": "msg_130"
}
```

#### Error Message
```json
{
  "type": "error",
  "data": {
    "code": "SUBSCRIPTION_FAILED",
    "message": "Insufficient permissions to subscribe to channel",
    "channel": "oversight:all"
  },
  "timestamp": "2025-12-26T13:00:00Z",
  "messageId": "msg_131"
}
```

---

## Authentication & Authorization

### JWT Token Structure

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "user_123",
    "email": "user@example.com",
    "role": "trader",
    "permissions": ["agent:read", "agent:write", "trade:execute"],
    "iat": 1703592000,
    "exp": 1703595600
  }
}
```

### Authentication Flow

1. **Login Request**
```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}

Response 200 OK:
{
  "accessToken": "eyJhbGci...",
  "refreshToken": "eyJhbGci...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

2. **Token Refresh**
```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refreshToken": "eyJhbGci..."
}

Response 200 OK:
{
  "accessToken": "eyJhbGci...",
  "expiresIn": 3600
}
```

### Authorization Roles

#### Roles & Permissions

| Role | Permissions |
|------|-------------|
| `admin` | All permissions |
| `trader` | agent:*, trade:*, portfolio:read, oversight:approve |
| `viewer` | agent:read, portfolio:read, trade:read |
| `demo` | Limited agent:read, portfolio:read |

#### Permission Matrix

| Resource | Action | Admin | Trader | Viewer | Demo |
|----------|--------|-------|--------|--------|------|
| Agents | Create | ✅ | ✅ | ❌ | ❌ |
| Agents | Read | ✅ | ✅ | ✅ | ✅* |
| Agents | Update | ✅ | ✅ | ❌ | ❌ |
| Agents | Delete | ✅ | ✅ | ❌ | ❌ |
| Trades | Execute | ✅ | ✅ | ❌ | ❌ |
| Oversight | Approve | ✅ | ✅ | ❌ | ❌ |
| Oversight | View | ✅ | ✅ | ✅ | ❌ |

*Demo role limited to own agents only

### Request Authorization

```http
GET /api/v1/agents
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

Response:
- 200 OK if authorized
- 401 Unauthorized if token missing/invalid
- 403 Forbidden if insufficient permissions
```

---

## Rate Limiting

### Rate Limit Tiers

| Tier | Requests/min | Trades/min | Burst Allowance |
|------|--------------|------------|-----------------|
| Demo | 50 | 5 | 10 |
| Basic | 100 | 10 | 20 |
| Pro | 500 | 50 | 100 |
| Enterprise | Unlimited | Unlimited | N/A |

### Rate Limit Headers

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 85
X-RateLimit-Reset: 1703592060
X-RateLimit-Policy: per-user
```

### Rate Limit Exceeded Response

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/problem+json
Retry-After: 45

{
  "type": "https://api.trading-demo.example.com/errors/rate-limit-exceeded",
  "title": "Rate Limit Exceeded",
  "status": 429,
  "detail": "You have exceeded the rate limit of 100 requests per minute",
  "instance": "/api/v1/agents",
  "retryAfter": 45,
  "limit": 100,
  "remaining": 0,
  "resetAt": "2025-12-26T13:01:00Z"
}
```

### Rate Limiting by Resource

| Endpoint Pattern | Additional Limit |
|------------------|------------------|
| `POST /api/v1/agents/:id/trades` | 10/min per agent |
| `POST /api/v1/oversight/*/approve` | 50/min per user |
| `GET /api/v1/agents/:id/performance` | 20/min per agent |
| WebSocket messages | 100/min per connection |

---

## Error Handling

### Error Response Format (RFC 7807)

```json
{
  "type": "string (URI reference)",
  "title": "string (short description)",
  "status": "number (HTTP status code)",
  "detail": "string (detailed explanation)",
  "instance": "string (request URI)",
  "errors": [
    {
      "field": "string (optional)",
      "message": "string",
      "code": "string (optional)"
    }
  ]
}
```

### Standard Error Types

#### 400 Bad Request - Validation Error
```json
{
  "type": "https://api.trading-demo.example.com/errors/validation-error",
  "title": "Validation Error",
  "status": 400,
  "detail": "Request validation failed",
  "instance": "/api/v1/agents",
  "errors": [
    {
      "field": "initialCapital",
      "message": "Must be between 1000 and 1000000",
      "code": "OUT_OF_RANGE"
    },
    {
      "field": "strategy",
      "message": "Must be one of: conservative, moderate, aggressive",
      "code": "INVALID_VALUE"
    }
  ]
}
```

#### 401 Unauthorized
```json
{
  "type": "https://api.trading-demo.example.com/errors/unauthorized",
  "title": "Unauthorized",
  "status": 401,
  "detail": "Authentication credentials are missing or invalid",
  "instance": "/api/v1/agents"
}
```

#### 403 Forbidden
```json
{
  "type": "https://api.trading-demo.example.com/errors/forbidden",
  "title": "Forbidden",
  "status": 403,
  "detail": "You do not have permission to perform this action",
  "instance": "/api/v1/agents/agent_abc123/delete",
  "requiredPermission": "agent:delete"
}
```

#### 404 Not Found
```json
{
  "type": "https://api.trading-demo.example.com/errors/not-found",
  "title": "Resource Not Found",
  "status": 404,
  "detail": "Agent with ID 'agent_invalid' does not exist",
  "instance": "/api/v1/agents/agent_invalid",
  "resourceType": "agent",
  "resourceId": "agent_invalid"
}
```

#### 409 Conflict
```json
{
  "type": "https://api.trading-demo.example.com/errors/conflict",
  "title": "Resource Conflict",
  "status": 409,
  "detail": "Cannot delete agent with active positions",
  "instance": "/api/v1/agents/agent_abc123",
  "conflictReason": "Agent has 3 active positions totaling $5,500"
}
```

#### 422 Unprocessable Entity - Business Logic Error
```json
{
  "type": "https://api.trading-demo.example.com/errors/business-logic-error",
  "title": "Business Logic Error",
  "status": 422,
  "detail": "Insufficient funds for trade",
  "instance": "/api/v1/agents/agent_abc123/trades",
  "businessRule": "INSUFFICIENT_FUNDS",
  "context": {
    "requiredAmount": 2550.00,
    "availableFunds": 2000.00,
    "shortfall": 550.00
  }
}
```

#### 500 Internal Server Error
```json
{
  "type": "https://api.trading-demo.example.com/errors/internal-error",
  "title": "Internal Server Error",
  "status": 500,
  "detail": "An unexpected error occurred while processing your request",
  "instance": "/api/v1/agents/agent_abc123/trades",
  "errorId": "err_xyz789",
  "message": "Please contact support with this error ID if the problem persists"
}
```

#### 503 Service Unavailable
```json
{
  "type": "https://api.trading-demo.example.com/errors/service-unavailable",
  "title": "Service Unavailable",
  "status": 503,
  "detail": "Market data service is temporarily unavailable",
  "instance": "/api/v1/agents/agent_abc123/trades",
  "retryAfter": 60,
  "affectedServices": ["market-data"]
}
```

---

## OpenAPI Specification

```yaml
openapi: 3.0.0
info:
  title: Trading Demo API
  description: |
    REST API for AI-powered trading agent management, portfolio operations,
    trade execution, metering, and oversight workflows.

    ## Features
    - Multi-agent trading system
    - Real-time portfolio tracking
    - Trade oversight and approval
    - Comprehensive usage metering
    - WebSocket real-time updates

    ## Authentication
    All endpoints require JWT Bearer token authentication.

    ## Rate Limiting
    - 100 requests/minute per user (Basic tier)
    - 10 trades/minute per agent
    - Burst allowance: 20 requests

  version: 1.0.0
  contact:
    name: Trading Demo Support
    email: support@trading-demo.example.com
  license:
    name: MIT
    url: https://opensource.org/licenses/MIT

servers:
  - url: https://api.trading-demo.example.com/api/v1
    description: Production server
  - url: http://localhost:3001/api/v1
    description: Development server

security:
  - bearerAuth: []

tags:
  - name: Agents
    description: Agent lifecycle and management
  - name: Portfolio
    description: Portfolio and position operations
  - name: Trades
    description: Trade execution and history
  - name: Metering
    description: Usage tracking and quotas
  - name: Oversight
    description: Trade approval workflows
  - name: Authentication
    description: User authentication and authorization

paths:
  # Agent Management
  /agents:
    post:
      tags: [Agents]
      summary: Create new trading agent
      description: Creates a new AI trading agent with specified strategy and configuration
      operationId: createAgent
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateAgentRequest'
            examples:
              conservative:
                summary: Conservative trading agent
                value:
                  name: Conservative Trader
                  strategy: conservative
                  initialCapital: 10000
                  riskTolerance: 0.15
                  tradingPairs: ["BTC/USD", "ETH/USD"]
                  config:
                    maxPositionSize: 2000
                    stopLossPercent: 5
                    takeProfitPercent: 10
      responses:
        '201':
          description: Agent created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Agent'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '403':
          $ref: '#/components/responses/Forbidden'
        '422':
          $ref: '#/components/responses/UnprocessableEntity'
        '429':
          $ref: '#/components/responses/TooManyRequests'
        '500':
          $ref: '#/components/responses/InternalServerError'

    get:
      tags: [Agents]
      summary: List all agents
      description: Retrieve paginated list of trading agents with optional filtering
      operationId: listAgents
      parameters:
        - name: status
          in: query
          description: Filter by agent status
          schema:
            type: string
            enum: [active, inactive, paused]
        - name: strategy
          in: query
          description: Filter by trading strategy
          schema:
            type: string
            enum: [conservative, moderate, aggressive]
        - name: limit
          in: query
          description: Maximum number of results
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
        - name: offset
          in: query
          description: Number of results to skip
          schema:
            type: integer
            minimum: 0
            default: 0
      responses:
        '200':
          description: List of agents
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentList'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '429':
          $ref: '#/components/responses/TooManyRequests'

  /agents/{agentId}:
    parameters:
      - $ref: '#/components/parameters/AgentId'

    get:
      tags: [Agents]
      summary: Get agent details
      description: Retrieve detailed information about a specific agent
      operationId: getAgent
      responses:
        '200':
          description: Agent details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentDetails'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

    patch:
      tags: [Agents]
      summary: Update agent
      description: Update agent configuration and settings
      operationId: updateAgent
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateAgentRequest'
      responses:
        '200':
          description: Agent updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Agent'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

    delete:
      tags: [Agents]
      summary: Delete agent
      description: Permanently delete an agent (must have no active positions)
      operationId: deleteAgent
      responses:
        '204':
          description: Agent deleted successfully
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'
        '409':
          $ref: '#/components/responses/Conflict'

  /agents/{agentId}/activate:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    post:
      tags: [Agents]
      summary: Activate agent
      description: Activate a trading agent to begin automated trading
      operationId: activateAgent
      responses:
        '200':
          description: Agent activated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentStatusChange'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /agents/{agentId}/pause:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    post:
      tags: [Agents]
      summary: Pause agent
      description: Pause trading agent activity
      operationId: pauseAgent
      responses:
        '200':
          description: Agent paused
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentStatusChange'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  # Portfolio Operations
  /agents/{agentId}/portfolio:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    get:
      tags: [Portfolio]
      summary: Get portfolio
      description: Retrieve current portfolio state including positions and cash
      operationId: getPortfolio
      responses:
        '200':
          description: Portfolio details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Portfolio'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /agents/{agentId}/positions:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    get:
      tags: [Portfolio]
      summary: List positions
      description: Get all current trading positions
      operationId: listPositions
      parameters:
        - name: symbol
          in: query
          description: Filter by trading pair symbol
          schema:
            type: string
      responses:
        '200':
          description: List of positions
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/PositionList'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /agents/{agentId}/performance:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    get:
      tags: [Portfolio]
      summary: Get performance metrics
      description: Retrieve performance analytics and statistics
      operationId: getPerformance
      parameters:
        - name: period
          in: query
          description: Time period for metrics
          schema:
            type: string
            enum: [1d, 7d, 30d, 90d, 1y, all]
            default: 30d
      responses:
        '200':
          description: Performance metrics
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Performance'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  # Trade Operations
  /agents/{agentId}/trades:
    parameters:
      - $ref: '#/components/parameters/AgentId'

    post:
      tags: [Trades]
      summary: Submit trade
      description: Submit a new trade order for execution
      operationId: submitTrade
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TradeRequest'
            examples:
              marketBuy:
                summary: Market buy order
                value:
                  symbol: BTC/USD
                  side: buy
                  type: market
                  quantity: 0.05
                  rationale: Bullish momentum detected
      responses:
        '201':
          description: Trade submitted
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Trade'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'
        '422':
          $ref: '#/components/responses/UnprocessableEntity'
        '429':
          $ref: '#/components/responses/TooManyRequests'

    get:
      tags: [Trades]
      summary: List trades
      description: Get trade history with filtering and pagination
      operationId: listTrades
      parameters:
        - name: status
          in: query
          description: Filter by trade status
          schema:
            type: string
            enum: [pending, pending_oversight, approved, rejected, executed, cancelled]
        - name: symbol
          in: query
          description: Filter by trading pair
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 50
        - name: offset
          in: query
          schema:
            type: integer
            minimum: 0
            default: 0
      responses:
        '200':
          description: List of trades
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TradeList'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /agents/{agentId}/trades/{tradeId}:
    parameters:
      - $ref: '#/components/parameters/AgentId'
      - $ref: '#/components/parameters/TradeId'

    get:
      tags: [Trades]
      summary: Get trade details
      description: Retrieve detailed information about a specific trade
      operationId: getTrade
      responses:
        '200':
          description: Trade details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TradeDetails'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

    delete:
      tags: [Trades]
      summary: Cancel trade
      description: Cancel a pending trade order
      operationId: cancelTrade
      responses:
        '200':
          description: Trade cancelled
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TradeCancellation'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'
        '409':
          $ref: '#/components/responses/Conflict'

  # Metering Operations
  /agents/{agentId}/usage:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    get:
      tags: [Metering]
      summary: Get usage summary
      description: Retrieve usage statistics and quota status
      operationId: getUsage
      parameters:
        - name: period
          in: query
          description: Time period for usage data
          schema:
            type: string
            enum: [1h, 24h, 7d, 30d]
            default: 7d
      responses:
        '200':
          description: Usage summary
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UsageSummary'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /agents/{agentId}/usage/breakdown:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    get:
      tags: [Metering]
      summary: Get usage breakdown
      description: Detailed breakdown of usage by operation and time
      operationId: getUsageBreakdown
      parameters:
        - name: period
          in: query
          schema:
            type: string
            enum: [1h, 24h, 7d, 30d]
            default: 24h
        - name: granularity
          in: query
          description: Time bucket size for breakdown
          schema:
            type: string
            enum: [5m, 15m, 1h, 1d]
            default: 1h
      responses:
        '200':
          description: Detailed usage breakdown
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UsageBreakdown'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /agents/{agentId}/quota:
    parameters:
      - $ref: '#/components/parameters/AgentId'
    get:
      tags: [Metering]
      summary: Get quota status
      description: Check current quota limits and remaining allowances
      operationId: getQuota
      responses:
        '200':
          description: Quota status
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/QuotaStatus'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  # Oversight Operations
  /oversight/requests:
    get:
      tags: [Oversight]
      summary: List oversight requests
      description: Get all pending and historical oversight requests
      operationId: listOversightRequests
      parameters:
        - name: status
          in: query
          description: Filter by request status
          schema:
            type: string
            enum: [pending, approved, rejected, expired]
        - name: agentId
          in: query
          description: Filter by agent ID
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
        - name: offset
          in: query
          schema:
            type: integer
            minimum: 0
            default: 0
      responses:
        '200':
          description: List of oversight requests
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightRequestList'
        '401':
          $ref: '#/components/responses/Unauthorized'

  /oversight/requests/{requestId}:
    parameters:
      - $ref: '#/components/parameters/RequestId'

    get:
      tags: [Oversight]
      summary: Get oversight request details
      description: Retrieve detailed information about an oversight request
      operationId: getOversightRequest
      responses:
        '200':
          description: Oversight request details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightRequestDetails'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          $ref: '#/components/responses/NotFound'

  /oversight/requests/{requestId}/approve:
    parameters:
      - $ref: '#/components/parameters/RequestId'
    post:
      tags: [Oversight]
      summary: Approve oversight request
      description: Approve a pending trade or action
      operationId: approveOversightRequest
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                comment:
                  type: string
                  maxLength: 500
                  description: Optional approval comment
      responses:
        '200':
          description: Request approved
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightResolution'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '403':
          $ref: '#/components/responses/Forbidden'
        '404':
          $ref: '#/components/responses/NotFound'
        '409':
          $ref: '#/components/responses/Conflict'

  /oversight/requests/{requestId}/reject:
    parameters:
      - $ref: '#/components/parameters/RequestId'
    post:
      tags: [Oversight]
      summary: Reject oversight request
      description: Reject a pending trade or action
      operationId: rejectOversightRequest
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [reason]
              properties:
                reason:
                  type: string
                  maxLength: 500
                  description: Reason for rejection
      responses:
        '200':
          description: Request rejected
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightResolution'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '403':
          $ref: '#/components/responses/Forbidden'
        '404':
          $ref: '#/components/responses/NotFound'

  # Authentication
  /auth/login:
    post:
      tags: [Authentication]
      summary: User login
      description: Authenticate user and receive JWT tokens
      operationId: login
      security: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/LoginRequest'
      responses:
        '200':
          description: Login successful
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/LoginResponse'
        '401':
          $ref: '#/components/responses/Unauthorized'

  /auth/refresh:
    post:
      tags: [Authentication]
      summary: Refresh access token
      description: Get new access token using refresh token
      operationId: refreshToken
      security: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [refreshToken]
              properties:
                refreshToken:
                  type: string
      responses:
        '200':
          description: Token refreshed
          content:
            application/json:
              schema:
                type: object
                properties:
                  accessToken:
                    type: string
                  expiresIn:
                    type: integer
        '401':
          $ref: '#/components/responses/Unauthorized'

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

  parameters:
    AgentId:
      name: agentId
      in: path
      required: true
      description: Agent identifier
      schema:
        type: string
        pattern: '^agent_[a-z0-9]+$'

    TradeId:
      name: tradeId
      in: path
      required: true
      description: Trade identifier
      schema:
        type: string
        pattern: '^trade_[a-z0-9]+$'

    RequestId:
      name: requestId
      in: path
      required: true
      description: Oversight request identifier
      schema:
        type: string
        pattern: '^req_[a-z0-9]+$'

  schemas:
    # Agent Schemas
    CreateAgentRequest:
      type: object
      required: [name, strategy, initialCapital, riskTolerance, tradingPairs]
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 100
          description: Agent display name
        strategy:
          type: string
          enum: [conservative, moderate, aggressive]
          description: Trading strategy type
        initialCapital:
          type: number
          minimum: 1000
          maximum: 1000000
          description: Starting capital in USD
        riskTolerance:
          type: number
          minimum: 0.01
          maximum: 0.50
          description: Risk tolerance (0.01-0.50)
        tradingPairs:
          type: array
          minItems: 1
          maxItems: 10
          items:
            type: string
          description: Trading pairs to trade
        config:
          type: object
          properties:
            maxPositionSize:
              type: number
              description: Maximum position size in USD
            stopLossPercent:
              type: number
              minimum: 1
              maximum: 50
              description: Stop loss percentage
            takeProfitPercent:
              type: number
              minimum: 1
              maximum: 100
              description: Take profit percentage

    Agent:
      type: object
      properties:
        id:
          type: string
          description: Agent unique identifier
        name:
          type: string
          description: Agent display name
        strategy:
          type: string
          enum: [conservative, moderate, aggressive]
        status:
          type: string
          enum: [active, inactive, paused]
        portfolio:
          type: object
          properties:
            totalValue:
              type: number
            cash:
              type: number
            positions:
              type: array
              items:
                $ref: '#/components/schemas/Position'
        createdAt:
          type: string
          format: date-time
        updatedAt:
          type: string
          format: date-time

    AgentDetails:
      allOf:
        - $ref: '#/components/schemas/Agent'
        - type: object
          properties:
            config:
              type: object
            usage:
              type: object
              properties:
                totalCalls:
                  type: integer
                totalTokens:
                  type: integer
                estimatedCost:
                  type: number
            lastActiveAt:
              type: string
              format: date-time

    AgentList:
      type: object
      properties:
        agents:
          type: array
          items:
            $ref: '#/components/schemas/Agent'
        pagination:
          $ref: '#/components/schemas/Pagination'

    UpdateAgentRequest:
      type: object
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 100
        riskTolerance:
          type: number
          minimum: 0.01
          maximum: 0.50
        config:
          type: object

    AgentStatusChange:
      type: object
      properties:
        id:
          type: string
        status:
          type: string
          enum: [active, inactive, paused]
        activatedAt:
          type: string
          format: date-time
        pausedAt:
          type: string
          format: date-time

    # Portfolio Schemas
    Portfolio:
      type: object
      properties:
        agentId:
          type: string
        totalValue:
          type: number
          description: Total portfolio value in USD
        cash:
          type: number
          description: Available cash
        investedValue:
          type: number
          description: Value in positions
        positions:
          type: array
          items:
            $ref: '#/components/schemas/Position'
        updatedAt:
          type: string
          format: date-time

    Position:
      type: object
      properties:
        id:
          type: string
        symbol:
          type: string
          description: Trading pair symbol
        quantity:
          type: number
          description: Position size
        side:
          type: string
          enum: [long, short]
        averagePrice:
          type: number
          description: Average entry price
        currentPrice:
          type: number
          description: Current market price
        marketValue:
          type: number
          description: Current position value
        costBasis:
          type: number
          description: Total cost of position
        unrealizedPnL:
          type: number
          description: Unrealized profit/loss
        unrealizedPnLPercent:
          type: number
          description: Unrealized P&L percentage
        allocation:
          type: number
          description: Percentage of portfolio
        openedAt:
          type: string
          format: date-time

    PositionList:
      type: object
      properties:
        positions:
          type: array
          items:
            $ref: '#/components/schemas/Position'

    Performance:
      type: object
      properties:
        agentId:
          type: string
        period:
          type: string
        metrics:
          type: object
          properties:
            totalReturn:
              type: number
              description: Total return percentage
            totalReturnAmount:
              type: number
              description: Total return in USD
            dailyReturn:
              type: number
              description: Average daily return
            sharpeRatio:
              type: number
              description: Risk-adjusted return metric
            maxDrawdown:
              type: number
              description: Maximum drawdown percentage
            winRate:
              type: number
              description: Percentage of profitable trades
            totalTrades:
              type: integer
            profitableTrades:
              type: integer
            averageWin:
              type: number
            averageLoss:
              type: number
            profitFactor:
              type: number
        equity:
          type: array
          items:
            type: object
            properties:
              timestamp:
                type: string
                format: date-time
              value:
                type: number

    # Trade Schemas
    TradeRequest:
      type: object
      required: [symbol, side, type, quantity]
      properties:
        symbol:
          type: string
          description: Trading pair (e.g., BTC/USD)
        side:
          type: string
          enum: [buy, sell]
        type:
          type: string
          enum: [market, limit]
        quantity:
          type: number
          minimum: 0
          exclusiveMinimum: true
        price:
          type: number
          description: Limit price (required for limit orders)
        stopLoss:
          type: number
          description: Stop loss price
        takeProfit:
          type: number
          description: Take profit price
        rationale:
          type: string
          maxLength: 500
          description: Trade rationale

    Trade:
      type: object
      properties:
        id:
          type: string
        agentId:
          type: string
        symbol:
          type: string
        side:
          type: string
          enum: [buy, sell]
        type:
          type: string
          enum: [market, limit]
        quantity:
          type: number
        status:
          type: string
          enum: [pending, pending_oversight, approved, rejected, executed, cancelled]
        estimatedPrice:
          type: number
        estimatedValue:
          type: number
        rationale:
          type: string
        oversightRequired:
          type: boolean
        createdAt:
          type: string
          format: date-time

    TradeDetails:
      allOf:
        - $ref: '#/components/schemas/Trade'
        - type: object
          properties:
            executedPrice:
              type: number
            value:
              type: number
            fees:
              type: number
            oversightRequest:
              type: object
              properties:
                id:
                  type: string
                status:
                  type: string
                approvedAt:
                  type: string
                  format: date-time
                approvedBy:
                  type: string
            executedAt:
              type: string
              format: date-time

    TradeList:
      type: object
      properties:
        trades:
          type: array
          items:
            $ref: '#/components/schemas/Trade'
        pagination:
          $ref: '#/components/schemas/Pagination'

    TradeCancellation:
      type: object
      properties:
        id:
          type: string
        status:
          type: string
          enum: [cancelled]
        cancelledAt:
          type: string
          format: date-time

    # Metering Schemas
    UsageSummary:
      type: object
      properties:
        agentId:
          type: string
        period:
          type: string
        summary:
          type: object
          properties:
            totalCalls:
              type: integer
            totalTokens:
              type: integer
            inputTokens:
              type: integer
            outputTokens:
              type: integer
            estimatedCost:
              type: number
            averageCallsPerDay:
              type: number
        quotaStatus:
          type: object
          properties:
            dailyLimit:
              type: integer
            dailyUsed:
              type: integer
            dailyRemaining:
              type: integer
            monthlyLimit:
              type: integer
            monthlyUsed:
              type: integer
            monthlyRemaining:
              type: integer

    UsageBreakdown:
      type: object
      properties:
        agentId:
          type: string
        period:
          type: string
        granularity:
          type: string
        breakdown:
          type: array
          items:
            type: object
            properties:
              timestamp:
                type: string
                format: date-time
              calls:
                type: integer
              tokens:
                type: integer
              cost:
                type: number
        byOperation:
          type: object
          additionalProperties:
            type: object
            properties:
              calls:
                type: integer
              tokens:
                type: integer
              cost:
                type: number

    QuotaStatus:
      type: object
      properties:
        agentId:
          type: string
        quotas:
          type: object
          properties:
            calls:
              $ref: '#/components/schemas/QuotaMetric'
            tokens:
              $ref: '#/components/schemas/QuotaMetric'
            cost:
              $ref: '#/components/schemas/QuotaMetric'
            trades:
              $ref: '#/components/schemas/QuotaMetric'
        alerts:
          type: array
          items:
            type: object
            properties:
              type:
                type: string
                enum: [warning, critical]
              metric:
                type: string
              message:
                type: string

    QuotaMetric:
      type: object
      properties:
        daily:
          $ref: '#/components/schemas/QuotaLimit'
        monthly:
          $ref: '#/components/schemas/QuotaLimit'

    QuotaLimit:
      type: object
      properties:
        limit:
          type: number
        used:
          type: number
        remaining:
          type: number

    # Oversight Schemas
    OversightRequestList:
      type: object
      properties:
        requests:
          type: array
          items:
            $ref: '#/components/schemas/OversightRequest'
        pagination:
          $ref: '#/components/schemas/Pagination'

    OversightRequest:
      type: object
      properties:
        id:
          type: string
        agentId:
          type: string
        agentName:
          type: string
        type:
          type: string
          enum: [trade_approval]
        status:
          type: string
          enum: [pending, approved, rejected, expired]
        priority:
          type: string
          enum: [low, normal, high, critical]
        trade:
          type: object
        rationale:
          type: string
        createdAt:
          type: string
          format: date-time
        expiresAt:
          type: string
          format: date-time

    OversightRequestDetails:
      allOf:
        - $ref: '#/components/schemas/OversightRequest'
        - type: object
          properties:
            context:
              type: object
              properties:
                currentPortfolioValue:
                  type: number
                availableCash:
                  type: number
                currentPositions:
                  type: integer
                recentPerformance:
                  type: number
            analysis:
              type: object
              properties:
                riskAmount:
                  type: number
                riskPercent:
                  type: number
                potentialProfit:
                  type: number
                riskRewardRatio:
                  type: number

    OversightResolution:
      type: object
      properties:
        id:
          type: string
        status:
          type: string
          enum: [approved, rejected]
        approvedBy:
          type: string
        rejectedBy:
          type: string
        approvedAt:
          type: string
          format: date-time
        rejectedAt:
          type: string
          format: date-time
        comment:
          type: string
        reason:
          type: string

    # Authentication Schemas
    LoginRequest:
      type: object
      required: [email, password]
      properties:
        email:
          type: string
          format: email
        password:
          type: string
          format: password

    LoginResponse:
      type: object
      properties:
        accessToken:
          type: string
        refreshToken:
          type: string
        expiresIn:
          type: integer
          description: Token expiration in seconds
        tokenType:
          type: string
          default: Bearer

    # Common Schemas
    Pagination:
      type: object
      properties:
        total:
          type: integer
          description: Total number of items
        limit:
          type: integer
          description: Maximum items per page
        offset:
          type: integer
          description: Number of items skipped
        hasMore:
          type: boolean
          description: Whether more results exist

    Error:
      type: object
      required: [type, title, status, detail, instance]
      properties:
        type:
          type: string
          format: uri
          description: URI reference identifying the error type
        title:
          type: string
          description: Short, human-readable error summary
        status:
          type: integer
          description: HTTP status code
        detail:
          type: string
          description: Detailed error explanation
        instance:
          type: string
          format: uri
          description: Request URI that generated the error
        errors:
          type: array
          items:
            type: object
            properties:
              field:
                type: string
              message:
                type: string
              code:
                type: string

  responses:
    BadRequest:
      description: Bad Request - Invalid input parameters
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/validation-error
            title: Validation Error
            status: 400
            detail: Request validation failed
            instance: /api/v1/agents
            errors:
              - field: initialCapital
                message: Must be between 1000 and 1000000
                code: OUT_OF_RANGE

    Unauthorized:
      description: Unauthorized - Missing or invalid authentication
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/unauthorized
            title: Unauthorized
            status: 401
            detail: Authentication credentials are missing or invalid
            instance: /api/v1/agents

    Forbidden:
      description: Forbidden - Insufficient permissions
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/forbidden
            title: Forbidden
            status: 403
            detail: You do not have permission to perform this action
            instance: /api/v1/agents/agent_abc123/delete

    NotFound:
      description: Not Found - Resource does not exist
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/not-found
            title: Resource Not Found
            status: 404
            detail: Agent with ID 'agent_invalid' does not exist
            instance: /api/v1/agents/agent_invalid

    Conflict:
      description: Conflict - Resource state conflict
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/conflict
            title: Resource Conflict
            status: 409
            detail: Cannot delete agent with active positions
            instance: /api/v1/agents/agent_abc123

    UnprocessableEntity:
      description: Unprocessable Entity - Business logic error
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/business-logic-error
            title: Business Logic Error
            status: 422
            detail: Insufficient funds for trade
            instance: /api/v1/agents/agent_abc123/trades

    TooManyRequests:
      description: Too Many Requests - Rate limit exceeded
      headers:
        X-RateLimit-Limit:
          schema:
            type: integer
          description: Request limit per window
        X-RateLimit-Remaining:
          schema:
            type: integer
          description: Remaining requests in current window
        X-RateLimit-Reset:
          schema:
            type: integer
          description: Unix timestamp when limit resets
        Retry-After:
          schema:
            type: integer
          description: Seconds until retry is allowed
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/rate-limit-exceeded
            title: Rate Limit Exceeded
            status: 429
            detail: You have exceeded the rate limit of 100 requests per minute
            instance: /api/v1/agents

    InternalServerError:
      description: Internal Server Error - Unexpected server error
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/Error'
          example:
            type: https://api.trading-demo.example.com/errors/internal-error
            title: Internal Server Error
            status: 500
            detail: An unexpected error occurred while processing your request
            instance: /api/v1/agents/agent_abc123/trades
            errorId: err_xyz789
