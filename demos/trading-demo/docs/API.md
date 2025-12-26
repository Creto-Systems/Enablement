# Trading Demo - Complete API Documentation

## Overview

RESTful API for the Creto Trading Agent Platform with oversight and metering capabilities.

**Base URL**: `http://localhost:3000/api/v1`

**Authentication**: All endpoints require Bearer token authentication.

```http
Authorization: Bearer <token>
```

---

## Table of Contents

1. [Agent Management](#agent-management)
2. [Trade Management](#trade-management)
3. [Oversight Management](#oversight-management)
4. [Metering & Quotas](#metering--quotas)
5. [WebSocket Protocol](#websocket-protocol)
6. [Error Responses](#error-responses)
7. [Rate Limiting](#rate-limiting)

---

## Agent Management

### Create Agent

Create a new trading agent with specified configuration.

**Endpoint**: `POST /api/v1/agents`

**Headers**:
```http
Content-Type: application/json
Authorization: Bearer <token>
```

**Request Body**:
```json
{
  "name": "HighFrequencyTrader",
  "type": "trading",
  "config": {
    "budget": 100000,
    "strategy": "balanced",
    "riskTolerance": "medium",
    "maxPositionSize": 10000
  },
  "userId": "user-123"
}
```

**Request Schema**:
```typescript
{
  name: string;           // Required, non-empty
  type: "trading";        // Required
  config: {
    budget: number;       // Required, 1000-1000000
    strategy: "conservative" | "balanced" | "aggressive";
    riskTolerance: "low" | "medium" | "high";
    maxPositionSize: number; // Optional
  };
  userId: string;         // Required
}
```

**Response**: `201 Created`
```json
{
  "id": "agent-abc123",
  "name": "HighFrequencyTrader",
  "type": "trading",
  "status": "active",
  "config": {
    "budget": 100000,
    "strategy": "balanced",
    "riskTolerance": "medium",
    "maxPositionSize": 10000
  },
  "userId": "user-123",
  "createdAt": "2024-12-26T10:00:00Z",
  "updatedAt": "2024-12-26T10:00:00Z"
}
```

**Error Responses**:
```json
// 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": [
      {
        "field": "config.budget",
        "message": "Budget must be between $1,000 and $1,000,000"
      }
    ]
  }
}

// 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Missing or invalid authentication token"
  }
}
```

---

### Get Agent Details

Retrieve detailed information about a specific agent.

**Endpoint**: `GET /api/v1/agents/:id`

**Path Parameters**:
- `id` (string, required): Agent identifier

**Response**: `200 OK`
```json
{
  "id": "agent-abc123",
  "name": "HighFrequencyTrader",
  "type": "trading",
  "status": "active",
  "config": {
    "budget": 100000,
    "strategy": "balanced",
    "riskTolerance": "medium",
    "maxPositionSize": 10000
  },
  "userId": "user-123",
  "createdAt": "2024-12-26T10:00:00Z",
  "updatedAt": "2024-12-26T10:00:00Z",
  "portfolio": {
    "totalValue": 102500,
    "positions": [
      {
        "symbol": "AAPL",
        "quantity": 100,
        "averagePrice": 150,
        "currentPrice": 155,
        "unrealizedPnL": 500
      }
    ]
  }
}
```

**Error Responses**:
```json
// 404 Not Found
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Agent with id 'agent-abc123' not found"
  }
}
```

---

### Terminate Agent

Permanently terminate an agent and close all positions.

**Endpoint**: `DELETE /api/v1/agents/:id`

**Path Parameters**:
- `id` (string, required): Agent identifier

**Response**: `204 No Content`

**Error Responses**:
```json
// 404 Not Found
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Agent with id 'agent-abc123' not found"
  }
}

// 409 Conflict
{
  "error": {
    "code": "AGENT_HAS_PENDING_TRADES",
    "message": "Cannot terminate agent with pending trades",
    "details": {
      "pendingTrades": 3
    }
  }
}
```

---

### List Agents

Retrieve all agents for the authenticated user.

**Endpoint**: `GET /api/v1/agents`

**Query Parameters**:
- `status` (optional): Filter by status (`active`, `terminated`)
- `page` (optional, default: 1): Page number
- `limit` (optional, default: 20): Results per page

**Response**: `200 OK`
```json
{
  "agents": [
    {
      "id": "agent-abc123",
      "name": "HighFrequencyTrader",
      "type": "trading",
      "status": "active",
      "createdAt": "2024-12-26T10:00:00Z"
    }
  ],
  "total": 5,
  "page": 1,
  "limit": 20
}
```

---

## Trade Management

### Submit Trade

Submit a trade order for execution.

**Endpoint**: `POST /api/v1/agents/:id/trades`

**Path Parameters**:
- `id` (string, required): Agent identifier

**Request Body**:
```json
{
  "symbol": "AAPL",
  "side": "buy",
  "quantity": 100,
  "price": 150,
  "type": "limit"
}
```

**Request Schema**:
```typescript
{
  symbol: string;              // Required, e.g., "AAPL", "BTC/USD"
  side: "buy" | "sell";        // Required
  quantity: number;            // Required, > 0
  price: number;               // Required, > 0
  type: "market" | "limit";    // Required
}
```

**Response (Autonomous Execution)**: `201 Created`
```json
{
  "id": "trade-xyz789",
  "agentId": "agent-abc123",
  "symbol": "AAPL",
  "side": "buy",
  "quantity": 100,
  "price": 150,
  "type": "limit",
  "status": "executed",
  "amount": 15000,
  "submittedAt": "2024-12-26T10:05:00Z",
  "executedAt": "2024-12-26T10:05:02Z"
}
```

**Response (Requires Oversight)**: `202 Accepted`
```json
{
  "trade": {
    "id": "trade-xyz789",
    "agentId": "agent-abc123",
    "symbol": "NVDA",
    "side": "buy",
    "quantity": 500,
    "price": 500,
    "type": "limit",
    "status": "pending_approval",
    "amount": 250000,
    "submittedAt": "2024-12-26T10:05:00Z"
  },
  "oversightRequest": {
    "id": "req-def456",
    "reason": "Trade amount ($250,000) exceeds autonomous threshold ($50,000)",
    "createdAt": "2024-12-26T10:05:00Z"
  },
  "message": "Trade submitted for approval"
}
```

**Error Responses**:
```json
// 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid trade parameters",
    "details": [
      {
        "field": "quantity",
        "message": "Quantity must be greater than 0"
      }
    ]
  }
}

// 403 Forbidden - Quota Exceeded
{
  "error": {
    "code": "QUOTA_EXCEEDED",
    "message": "Trade would exceed daily quota",
    "details": {
      "requestedAmount": 15000,
      "remainingQuota": 5000,
      "quotaLimit": 100000,
      "resetAt": "2024-12-27T00:00:00Z"
    }
  }
}

// 404 Not Found
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Agent with id 'agent-abc123' not found"
  }
}
```

---

### List Trades

Retrieve paginated list of trades for an agent.

**Endpoint**: `GET /api/v1/agents/:id/trades`

**Path Parameters**:
- `id` (string, required): Agent identifier

**Query Parameters**:
- `page` (optional, default: 1): Page number
- `limit` (optional, default: 20): Results per page (max 100)
- `status` (optional): Filter by status
  - `pending` - Awaiting execution
  - `pending_approval` - Awaiting oversight approval
  - `executed` - Successfully executed
  - `cancelled` - Cancelled by user or system
  - `failed` - Execution failed
- `symbol` (optional): Filter by trading symbol
- `startDate` (optional): ISO 8601 date, filter trades after this date
- `endDate` (optional): ISO 8601 date, filter trades before this date

**Response**: `200 OK`
```json
{
  "trades": [
    {
      "id": "trade-xyz789",
      "agentId": "agent-abc123",
      "symbol": "AAPL",
      "side": "buy",
      "quantity": 100,
      "price": 150,
      "type": "limit",
      "status": "executed",
      "amount": 15000,
      "submittedAt": "2024-12-26T10:05:00Z",
      "executedAt": "2024-12-26T10:05:02Z"
    }
  ],
  "total": 150,
  "page": 1,
  "limit": 20,
  "hasMore": true
}
```

---

### Cancel Trade

Cancel a pending trade order.

**Endpoint**: `DELETE /api/v1/agents/:agentId/trades/:tradeId`

**Path Parameters**:
- `agentId` (string, required): Agent identifier
- `tradeId` (string, required): Trade identifier

**Response**: `200 OK`
```json
{
  "id": "trade-xyz789",
  "status": "cancelled",
  "cancelledAt": "2024-12-26T10:10:00Z"
}
```

**Error Responses**:
```json
// 409 Conflict
{
  "error": {
    "code": "CANNOT_CANCEL_EXECUTED_TRADE",
    "message": "Cannot cancel trade that has already been executed"
  }
}
```

---

## Oversight Management

### List Pending Requests

Retrieve all pending oversight approval requests.

**Endpoint**: `GET /api/v1/oversight/requests`

**Query Parameters**:
- `agentId` (optional): Filter by specific agent
- `status` (optional): Filter by status (`pending`, `approved`, `rejected`)
- `page` (optional, default: 1): Page number
- `limit` (optional, default: 20): Results per page

**Response**: `200 OK`
```json
{
  "requests": [
    {
      "id": "req-def456",
      "agentId": "agent-abc123",
      "tradeId": "trade-xyz789",
      "trade": {
        "symbol": "NVDA",
        "side": "buy",
        "quantity": 500,
        "price": 500,
        "amount": 250000
      },
      "reason": "Trade amount ($250,000) exceeds autonomous threshold ($50,000)",
      "status": "pending",
      "createdAt": "2024-12-26T10:05:00Z",
      "priority": "high"
    }
  ],
  "total": 3,
  "page": 1,
  "limit": 20
}
```

---

### Approve Request

Approve a pending oversight request and execute the trade.

**Endpoint**: `POST /api/v1/oversight/requests/:id/approve`

**Path Parameters**:
- `id` (string, required): Oversight request identifier

**Request Body** (optional):
```json
{
  "comment": "Approved - within risk parameters"
}
```

**Response**: `200 OK`
```json
{
  "id": "req-def456",
  "agentId": "agent-abc123",
  "tradeId": "trade-xyz789",
  "status": "approved",
  "approvedBy": "supervisor-789",
  "approvedAt": "2024-12-26T10:10:00Z",
  "comment": "Approved - within risk parameters",
  "executedTrade": {
    "id": "trade-xyz789",
    "status": "executed",
    "executedAt": "2024-12-26T10:10:01Z"
  }
}
```

**Error Responses**:
```json
// 404 Not Found
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Oversight request with id 'req-def456' not found"
  }
}

// 409 Conflict
{
  "error": {
    "code": "REQUEST_ALREADY_PROCESSED",
    "message": "Request has already been approved or rejected",
    "details": {
      "currentStatus": "approved",
      "processedAt": "2024-12-26T10:10:00Z"
    }
  }
}

// 403 Forbidden
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "User does not have permission to approve requests"
  }
}
```

---

### Reject Request

Reject a pending oversight request and cancel the trade.

**Endpoint**: `POST /api/v1/oversight/requests/:id/reject`

**Path Parameters**:
- `id` (string, required): Oversight request identifier

**Request Body**:
```json
{
  "reason": "Trade parameters outside acceptable risk range"
}
```

**Request Schema**:
```typescript
{
  reason: string; // Required, min 10 characters
}
```

**Response**: `200 OK`
```json
{
  "id": "req-def456",
  "agentId": "agent-abc123",
  "tradeId": "trade-xyz789",
  "status": "rejected",
  "rejectedBy": "supervisor-789",
  "rejectedAt": "2024-12-26T10:10:00Z",
  "rejectionReason": "Trade parameters outside acceptable risk range",
  "cancelledTrade": {
    "id": "trade-xyz789",
    "status": "cancelled",
    "cancelledAt": "2024-12-26T10:10:00Z"
  }
}
```

**Error Responses**:
```json
// 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Rejection reason is required",
    "details": [
      {
        "field": "reason",
        "message": "Reason must be at least 10 characters"
      }
    ]
  }
}
```

---

## Metering & Quotas

### Get Usage Summary

Retrieve usage metrics for an agent.

**Endpoint**: `GET /api/v1/agents/:id/usage`

**Path Parameters**:
- `id` (string, required): Agent identifier

**Query Parameters**:
- `period` (optional, default: `daily`): Time period
  - `hourly` - Last hour
  - `daily` - Last 24 hours
  - `weekly` - Last 7 days
  - `monthly` - Last 30 days
  - `yearly` - Last 365 days
- `includeBreakdown` (optional, default: `false`): Include detailed breakdown

**Response**: `200 OK`
```json
{
  "agentId": "agent-abc123",
  "period": "daily",
  "startDate": "2024-12-26T00:00:00Z",
  "endDate": "2024-12-26T23:59:59Z",
  "summary": {
    "totalTrades": 45,
    "totalVolume": 125000,
    "totalFees": 125,
    "apiCalls": 320,
    "computeTime": 1800
  },
  "breakdown": {
    "trades": {
      "buy": 25,
      "sell": 20
    },
    "bySymbol": {
      "BTC/USD": 30,
      "ETH/USD": 15
    },
    "byType": {
      "market": 35,
      "limit": 10
    },
    "byStatus": {
      "executed": 42,
      "cancelled": 2,
      "failed": 1
    }
  }
}
```

---

### Get Quota Status

Retrieve current quota status and limits.

**Endpoint**: `GET /api/v1/agents/:id/quota`

**Path Parameters**:
- `id` (string, required): Agent identifier

**Response**: `200 OK`
```json
{
  "agentId": "agent-abc123",
  "quotas": {
    "daily": {
      "trades": {
        "limit": 100,
        "used": 45,
        "remaining": 55,
        "percentUsed": 45,
        "resetAt": "2024-12-27T00:00:00Z"
      },
      "volume": {
        "limit": 500000,
        "used": 125000,
        "remaining": 375000,
        "percentUsed": 25,
        "currency": "USD",
        "resetAt": "2024-12-27T00:00:00Z"
      }
    },
    "monthly": {
      "budget": {
        "limit": 100000,
        "used": 25000,
        "remaining": 75000,
        "percentUsed": 25,
        "currency": "USD",
        "resetAt": "2025-01-01T00:00:00Z"
      }
    }
  },
  "warnings": [
    {
      "type": "approaching_limit",
      "quota": "daily.trades",
      "threshold": 80,
      "message": "Daily trade quota is 45% used"
    }
  ]
}
```

**Quota Warning Thresholds**:
- **80%**: Warning notification
- **90%**: Critical warning
- **100%**: Quota exceeded, requests blocked

---

## WebSocket Protocol

Real-time updates via WebSocket connection.

**Connection**: `ws://localhost:3000/ws`

**Authentication**:
```javascript
const ws = new WebSocket('ws://localhost:3000/ws');
ws.send(JSON.stringify({
  type: 'auth',
  token: 'bearer-token-here'
}));
```

### Server-to-Client Events

**Trade Update**:
```json
{
  "type": "trade:update",
  "data": {
    "tradeId": "trade-xyz789",
    "agentId": "agent-abc123",
    "status": "executed",
    "executedAt": "2024-12-26T10:05:02Z"
  }
}
```

**Oversight Request**:
```json
{
  "type": "oversight:request_created",
  "data": {
    "requestId": "req-def456",
    "agentId": "agent-abc123",
    "tradeId": "trade-xyz789",
    "reason": "Exceeds autonomous threshold"
  }
}
```

**Quota Warning**:
```json
{
  "type": "quota:warning",
  "data": {
    "agentId": "agent-abc123",
    "quota": "daily.trades",
    "percentUsed": 85,
    "threshold": 80,
    "message": "Daily trade quota is 85% used"
  }
}
```

**Portfolio Update**:
```json
{
  "type": "portfolio:update",
  "data": {
    "agentId": "agent-abc123",
    "totalValue": 102500,
    "unrealizedPnL": 2500
  }
}
```

### Client-to-Server Events

**Subscribe to Agent**:
```json
{
  "type": "subscribe",
  "agentId": "agent-abc123"
}
```

**Unsubscribe from Agent**:
```json
{
  "type": "unsubscribe",
  "agentId": "agent-abc123"
}
```

---

## Error Responses

All error responses follow this structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      // Additional context (optional)
    }
  }
}
```

### HTTP Status Codes

| Code | Meaning | Use Case |
|------|---------|----------|
| 200 | OK | Request succeeded |
| 201 | Created | Resource created successfully |
| 202 | Accepted | Request accepted, requires approval |
| 204 | No Content | Success with no response body |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions or quota exceeded |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Request conflicts with current state |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | External service unavailable |

### Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Request validation failed |
| `UNAUTHORIZED` | Missing or invalid auth token |
| `INSUFFICIENT_PERMISSIONS` | User lacks required permissions |
| `NOT_FOUND` | Requested resource not found |
| `QUOTA_EXCEEDED` | Agent quota exceeded |
| `RATE_LIMIT_EXCEEDED` | API rate limit exceeded |
| `REQUEST_ALREADY_PROCESSED` | Oversight request already decided |
| `CANNOT_CANCEL_EXECUTED_TRADE` | Trade already executed |
| `AGENT_HAS_PENDING_TRADES` | Agent has pending trades |
| `EXTERNAL_SERVICE_ERROR` | External service failure |

---

## Rate Limiting

API requests are rate-limited to prevent abuse.

**Limits**:
- **Global**: 1000 requests/hour per user
- **Per Agent**: 100 requests/minute per agent
- **Per Endpoint**: Varies by endpoint

**Rate Limit Headers**:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1735219200
```

**Rate Limit Exceeded Response**: `429 Too Many Requests`
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests, please try again later",
    "details": {
      "limit": 100,
      "remaining": 0,
      "resetAt": "2024-12-26T11:00:00Z",
      "retryAfter": 60
    }
  }
}
```

---

## Pagination

All list endpoints support pagination.

**Query Parameters**:
- `page` (number, default: 1): Page number (1-indexed)
- `limit` (number, default: 20, max: 100): Items per page

**Response Structure**:
```json
{
  "data": [...],
  "total": 150,
  "page": 1,
  "limit": 20,
  "hasMore": true,
  "nextPage": 2
}
```

---

## Authentication Guide

### Obtaining a Token

```bash
# Development: Use test token
export TOKEN="test-bearer-token"

# Production: OAuth 2.0 flow
curl -X POST https://auth.creto.ai/oauth/token \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type": "client_credentials",
    "client_id": "your-client-id",
    "client_secret": "your-client-secret"
  }'
```

### Using the Token

```bash
curl -X GET http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $TOKEN"
```

---

## Examples

### Complete Trade Flow

```bash
# 1. Create agent
AGENT_ID=$(curl -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Demo Trader",
    "type": "trading",
    "config": {"budget": 100000, "strategy": "balanced"},
    "userId": "user-123"
  }' | jq -r '.id')

# 2. Submit small trade (autonomous)
curl -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/trades \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "AAPL",
    "side": "buy",
    "quantity": 10,
    "price": 150,
    "type": "limit"
  }'

# 3. Submit large trade (requires approval)
TRADE_RESPONSE=$(curl -X POST http://localhost:3000/api/v1/agents/$AGENT_ID/trades \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "NVDA",
    "side": "buy",
    "quantity": 500,
    "price": 500,
    "type": "limit"
  }')

REQUEST_ID=$(echo $TRADE_RESPONSE | jq -r '.oversightRequest.id')

# 4. Approve the trade
curl -X POST http://localhost:3000/api/v1/oversight/requests/$REQUEST_ID/approve \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"comment": "Approved"}'

# 5. Check quota status
curl -X GET http://localhost:3000/api/v1/agents/$AGENT_ID/quota \
  -H "Authorization: Bearer $TOKEN"
```

---

**Last Updated**: 2024-12-26
**API Version**: 1.0.0
**Status**: Production Ready
