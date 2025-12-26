# API Structure - Trading Demo

## Overview
RESTful API for the Anthropic Trading Agent Platform with oversight and metering capabilities.

## Base URL
```
http://localhost:3000/api/v1
```

## Authentication
All endpoints require authentication via Bearer token:
```
Authorization: Bearer <token>
```

---

## Agent Management

### Create Agent
```http
POST /api/v1/agents
Content-Type: application/json

{
  "name": "HighFrequencyTrader",
  "type": "trading",
  "config": {
    "tradingPair": "BTC/USD",
    "maxPositionSize": 10000,
    "riskTolerance": "medium"
  },
  "userId": "user-123"
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
    "tradingPair": "BTC/USD",
    "maxPositionSize": 10000,
    "riskTolerance": "medium"
  },
  "userId": "user-123",
  "createdAt": "2024-01-26T10:00:00Z"
}
```

**Error Responses**:
- `400 Bad Request` - Invalid request body
- `401 Unauthorized` - Missing or invalid token

---

### Get Agent Details
```http
GET /api/v1/agents/:id
```

**Response**: `200 OK`
```json
{
  "id": "agent-abc123",
  "name": "HighFrequencyTrader",
  "type": "trading",
  "status": "active",
  "config": { ... },
  "userId": "user-123",
  "createdAt": "2024-01-26T10:00:00Z"
}
```

**Error Responses**:
- `404 Not Found` - Agent does not exist
- `401 Unauthorized` - Missing or invalid token

---

### Terminate Agent
```http
DELETE /api/v1/agents/:id
```

**Response**: `204 No Content`

**Error Responses**:
- `404 Not Found` - Agent does not exist
- `401 Unauthorized` - Missing or invalid token

---

## Trade Management

### Submit Trade
```http
POST /api/v1/agents/:id/trades
Content-Type: application/json

{
  "symbol": "BTC/USD",
  "side": "buy",
  "quantity": 1.5,
  "price": 45000,
  "type": "limit"
}
```

**Response**: `201 Created` (Autonomous execution)
```json
{
  "id": "trade-xyz789",
  "agentId": "agent-abc123",
  "symbol": "BTC/USD",
  "side": "buy",
  "quantity": 1.5,
  "price": 45000,
  "type": "limit",
  "status": "pending",
  "submittedAt": "2024-01-26T10:05:00Z"
}
```

**Response**: `202 Accepted` (Requires oversight)
```json
{
  "trade": {
    "id": "trade-xyz789",
    "agentId": "agent-abc123",
    "symbol": "BTC/USD",
    "side": "buy",
    "quantity": 100,
    "price": 45000,
    "type": "limit",
    "status": "pending_approval",
    "submittedAt": "2024-01-26T10:05:00Z"
  },
  "message": "Trade submitted for approval required",
  "reason": "Exceeds autonomous threshold"
}
```

**Error Responses**:
- `400 Bad Request` - Invalid trade parameters
- `403 Forbidden` - Quota exceeded
- `401 Unauthorized` - Missing or invalid token

---

### List Trades
```http
GET /api/v1/agents/:id/trades?page=1&limit=20&status=completed
```

**Query Parameters**:
- `page` (optional, default: 1)
- `limit` (optional, default: 20)
- `status` (optional, filter: pending|pending_approval|completed|cancelled|failed)

**Response**: `200 OK`
```json
{
  "trades": [
    {
      "id": "trade-xyz789",
      "agentId": "agent-abc123",
      "symbol": "BTC/USD",
      "side": "buy",
      "quantity": 1.5,
      "price": 45000,
      "type": "limit",
      "status": "completed",
      "submittedAt": "2024-01-26T10:05:00Z",
      "executedAt": "2024-01-26T10:05:15Z"
    }
  ],
  "total": 150,
  "page": 1,
  "limit": 20
}
```

---

## Oversight Management

### List Pending Requests
```http
GET /api/v1/oversight/requests?agentId=agent-abc123
```

**Query Parameters**:
- `agentId` (optional, filter by specific agent)

**Response**: `200 OK`
```json
{
  "requests": [
    {
      "id": "req-def456",
      "agentId": "agent-abc123",
      "tradeId": "trade-xyz789",
      "reason": "Exceeds autonomous threshold",
      "status": "pending",
      "createdAt": "2024-01-26T10:05:00Z"
    }
  ],
  "total": 1
}
```

---

### Approve Request
```http
POST /api/v1/oversight/requests/:id/approve
```

**Response**: `200 OK`
```json
{
  "id": "req-def456",
  "agentId": "agent-abc123",
  "tradeId": "trade-xyz789",
  "status": "approved",
  "approvedBy": "supervisor-789",
  "approvedAt": "2024-01-26T10:10:00Z"
}
```

**Error Responses**:
- `404 Not Found` - Request does not exist
- `403 Forbidden` - Insufficient permissions
- `401 Unauthorized` - Missing or invalid token

---

### Reject Request
```http
POST /api/v1/oversight/requests/:id/reject
Content-Type: application/json

{
  "reason": "Trade parameters outside acceptable risk range"
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
  "rejectedAt": "2024-01-26T10:10:00Z",
  "rejectionReason": "Trade parameters outside acceptable risk range"
}
```

**Error Responses**:
- `400 Bad Request` - Missing rejection reason
- `404 Not Found` - Request does not exist
- `401 Unauthorized` - Missing or invalid token

---

## Metering & Quotas

### Get Usage Summary
```http
GET /api/v1/agents/:id/usage?period=daily&includeBreakdown=true
```

**Query Parameters**:
- `period` (optional, default: daily): daily|weekly|monthly|yearly
- `includeBreakdown` (optional, default: false): Include detailed breakdown

**Response**: `200 OK`
```json
{
  "agentId": "agent-abc123",
  "period": "daily",
  "totalTrades": 45,
  "totalVolume": 125000,
  "apiCalls": 320,
  "computeTime": 1800,
  "startDate": "2024-01-26T00:00:00Z",
  "endDate": "2024-01-26T23:59:59Z",
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
    }
  }
}
```

---

### Get Quota Status
```http
GET /api/v1/agents/:id/quota
```

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
        "resetAt": "2024-01-27T00:00:00Z"
      },
      "volume": {
        "limit": 500000,
        "used": 125000,
        "remaining": 375000,
        "resetAt": "2024-01-27T00:00:00Z"
      }
    },
    "monthly": {
      "budget": {
        "limit": 10000,
        "used": 2500,
        "remaining": 7500,
        "currency": "USD",
        "resetAt": "2024-02-01T00:00:00Z"
      }
    }
  }
}
```

**Error Responses**:
- `404 Not Found` - Agent does not exist
- `401 Unauthorized` - Missing or invalid token

---

## Status Codes

| Code | Meaning |
|------|---------|
| 200 | OK - Request succeeded |
| 201 | Created - Resource created successfully |
| 202 | Accepted - Request accepted, requires approval |
| 204 | No Content - Success with no response body |
| 400 | Bad Request - Invalid request parameters |
| 401 | Unauthorized - Missing or invalid authentication |
| 403 | Forbidden - Insufficient permissions or quota exceeded |
| 404 | Not Found - Resource does not exist |
| 500 | Internal Server Error - Server-side error |

---

## Error Response Format

All errors follow this structure:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": [
      {
        "field": "quantity",
        "message": "Quantity must be greater than 0"
      }
    ]
  }
}
```

---

## Rate Limiting

API requests are rate-limited per agent:

- **Global**: 1000 requests/hour
- **Per Agent**: 100 requests/minute

Rate limit headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1706270400
```

---

## Webhooks (Future)

Planned webhook events:
- `trade.submitted`
- `trade.executed`
- `trade.failed`
- `oversight.required`
- `oversight.approved`
- `oversight.rejected`
- `quota.warning`
- `quota.exceeded`
