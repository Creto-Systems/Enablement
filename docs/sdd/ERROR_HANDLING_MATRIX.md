# Error Handling Matrix - Enablement Layer

**Document Version**: 1.0.0
**Last Updated**: 2025-12-26
**Status**: Draft

## Table of Contents

1. [Error Code Registry](#error-code-registry)
2. [Error Categories](#error-categories)
3. [Retry Strategies](#retry-strategies)
4. [Error Response Format](#error-response-format)
5. [Cross-Product Error Propagation](#cross-product-error-propagation)
6. [Client Handling Guidelines](#client-handling-guidelines)
7. [Monitoring & Alerting](#monitoring--alerting)

---

## 1. Error Code Registry

### 1.1 Prefix Scheme

Each product uses a unique 3-letter prefix followed by a 3-digit code:

| Product | Prefix | Range | Example |
|---------|--------|-------|---------|
| **Runtime** | `RTM` | 000-999 | `RTM-001` |
| **Metering** | `MTR` | 000-999 | `MTR-001` |
| **Oversight** | `OVS` | 000-999 | `OVS-001` |
| **Messaging** | `MSG` | 000-999 | `MSG-001` |

### 1.2 Severity Levels

| Level | Description | Action Required |
|-------|-------------|-----------------|
| **CRITICAL** | Service unavailable, data loss risk | Immediate escalation, PagerDuty alert |
| **ERROR** | Request failed, user action blocked | Investigation required, error logging |
| **WARNING** | Degraded performance, partial failure | Monitoring, may require action |
| **INFO** | Informational, expected behavior | Logging only, no action required |

---

### 1.3 Runtime (RTM) Error Codes

| Code | Severity | Category | Message | HTTP | gRPC | Retryable |
|------|----------|----------|---------|------|------|-----------|
| `RTM-001` | CRITICAL | System | Runtime service unavailable | 503 | UNAVAILABLE | Yes |
| `RTM-002` | ERROR | Validation | Invalid function execution request | 400 | INVALID_ARGUMENT | No |
| `RTM-003` | ERROR | Validation | Missing required execution parameters | 400 | INVALID_ARGUMENT | No |
| `RTM-004` | ERROR | Authentication | Invalid or expired API key | 401 | UNAUTHENTICATED | No |
| `RTM-005` | ERROR | Authentication | Missing authentication credentials | 401 | UNAUTHENTICATED | No |
| `RTM-006` | ERROR | Authorization | Insufficient permissions for function execution | 403 | PERMISSION_DENIED | No |
| `RTM-007` | ERROR | Authorization | Function execution not allowed for tenant | 403 | PERMISSION_DENIED | No |
| `RTM-008` | ERROR | Resource | Function not found | 404 | NOT_FOUND | No |
| `RTM-009` | ERROR | Resource | Execution context not found | 404 | NOT_FOUND | No |
| `RTM-010` | ERROR | Resource | Sandbox environment unavailable | 503 | UNAVAILABLE | Yes |
| `RTM-011` | ERROR | Timeout | Function execution timeout exceeded | 504 | DEADLINE_EXCEEDED | No |
| `RTM-012` | ERROR | Timeout | Sandbox initialization timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `RTM-013` | ERROR | Resource | Memory limit exceeded during execution | 413 | RESOURCE_EXHAUSTED | No |
| `RTM-014` | ERROR | Resource | CPU quota exceeded | 429 | RESOURCE_EXHAUSTED | No |
| `RTM-015` | ERROR | Resource | Disk space quota exceeded | 507 | RESOURCE_EXHAUSTED | No |
| `RTM-016` | ERROR | Execution | Function runtime error | 500 | INTERNAL | No |
| `RTM-017` | ERROR | Execution | Sandbox initialization failed | 500 | INTERNAL | Yes |
| `RTM-018` | ERROR | Execution | Function code compilation error | 422 | FAILED_PRECONDITION | No |
| `RTM-019` | ERROR | Dependency | Required dependency unavailable | 424 | FAILED_PRECONDITION | Yes |
| `RTM-020` | ERROR | Dependency | External service timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `RTM-021` | WARNING | Quota | Approaching execution quota limit | 200 | OK | N/A |
| `RTM-022` | ERROR | Quota | Execution quota exhausted | 429 | RESOURCE_EXHAUSTED | No |
| `RTM-023` | ERROR | Network | Network connection timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `RTM-024` | ERROR | Network | Network connection refused | 503 | UNAVAILABLE | Yes |
| `RTM-025` | ERROR | Crypto | Encryption operation failed | 500 | INTERNAL | Yes |
| `RTM-026` | ERROR | Crypto | Decryption operation failed | 500 | INTERNAL | No |
| `RTM-027` | ERROR | Crypto | Invalid encryption key | 400 | INVALID_ARGUMENT | No |
| `RTM-028` | CRITICAL | System | Database connection lost | 503 | UNAVAILABLE | Yes |
| `RTM-029` | ERROR | Validation | Invalid function configuration | 400 | INVALID_ARGUMENT | No |
| `RTM-030` | ERROR | RateLimit | Rate limit exceeded | 429 | RESOURCE_EXHAUSTED | Yes |

---

### 1.4 Metering (MTR) Error Codes

| Code | Severity | Category | Message | HTTP | gRPC | Retryable |
|------|----------|----------|---------|------|------|-----------|
| `MTR-001` | CRITICAL | System | Metering service unavailable | 503 | UNAVAILABLE | Yes |
| `MTR-002` | ERROR | Validation | Invalid usage record format | 400 | INVALID_ARGUMENT | No |
| `MTR-003` | ERROR | Validation | Missing required metering fields | 400 | INVALID_ARGUMENT | No |
| `MTR-004` | ERROR | Authentication | Invalid metering API credentials | 401 | UNAUTHENTICATED | No |
| `MTR-005` | ERROR | Authorization | Insufficient permissions for metering data | 403 | PERMISSION_DENIED | No |
| `MTR-006` | ERROR | Resource | Usage record not found | 404 | NOT_FOUND | No |
| `MTR-007` | ERROR | Resource | Billing period not found | 404 | NOT_FOUND | No |
| `MTR-008` | ERROR | Validation | Invalid timestamp in usage record | 400 | INVALID_ARGUMENT | No |
| `MTR-009` | ERROR | Validation | Usage record timestamp in future | 400 | INVALID_ARGUMENT | No |
| `MTR-010` | ERROR | Validation | Usage record too old for processing | 400 | OUT_OF_RANGE | No |
| `MTR-011` | ERROR | Duplicate | Duplicate usage record detected | 409 | ALREADY_EXISTS | No |
| `MTR-012` | WARNING | Aggregation | Usage aggregation delayed | 200 | OK | N/A |
| `MTR-013` | ERROR | Aggregation | Usage aggregation failed | 500 | INTERNAL | Yes |
| `MTR-014` | ERROR | Dependency | Time-series database unavailable | 503 | UNAVAILABLE | Yes |
| `MTR-015` | ERROR | Dependency | Billing system integration error | 502 | UNAVAILABLE | Yes |
| `MTR-016` | ERROR | Quota | Storage quota exceeded for usage data | 507 | RESOURCE_EXHAUSTED | No |
| `MTR-017` | ERROR | Timeout | Usage query timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `MTR-018` | ERROR | Validation | Invalid metering dimension | 400 | INVALID_ARGUMENT | No |
| `MTR-019` | ERROR | Validation | Invalid usage quantity (negative value) | 400 | INVALID_ARGUMENT | No |
| `MTR-020` | ERROR | Crypto | Usage data encryption failed | 500 | INTERNAL | Yes |
| `MTR-021` | ERROR | Crypto | Usage data decryption failed | 500 | INTERNAL | No |
| `MTR-022` | CRITICAL | System | Metering database corruption detected | 500 | DATA_LOSS | No |
| `MTR-023` | ERROR | Export | Usage export failed | 500 | INTERNAL | Yes |
| `MTR-024` | ERROR | Export | Invalid export format requested | 400 | INVALID_ARGUMENT | No |
| `MTR-025` | ERROR | RateLimit | Metering API rate limit exceeded | 429 | RESOURCE_EXHAUSTED | Yes |
| `MTR-026` | WARNING | Reconciliation | Usage reconciliation mismatch detected | 200 | OK | N/A |
| `MTR-027` | ERROR | Reconciliation | Usage reconciliation failed | 500 | INTERNAL | Yes |
| `MTR-028` | ERROR | Validation | Invalid tenant identifier in usage record | 400 | INVALID_ARGUMENT | No |
| `MTR-029` | ERROR | Batch | Batch usage ingestion failed | 500 | INTERNAL | Yes |
| `MTR-030` | ERROR | Batch | Batch size exceeds maximum limit | 413 | OUT_OF_RANGE | No |

---

### 1.5 Oversight (OVS) Error Codes

| Code | Severity | Category | Message | HTTP | gRPC | Retryable |
|------|----------|----------|---------|------|------|-----------|
| `OVS-001` | CRITICAL | System | Oversight service unavailable | 503 | UNAVAILABLE | Yes |
| `OVS-002` | ERROR | Validation | Invalid policy definition | 400 | INVALID_ARGUMENT | No |
| `OVS-003` | ERROR | Validation | Missing required policy fields | 400 | INVALID_ARGUMENT | No |
| `OVS-004` | ERROR | Authentication | Invalid oversight API credentials | 401 | UNAUTHENTICATED | No |
| `OVS-005` | ERROR | Authorization | Insufficient permissions for policy management | 403 | PERMISSION_DENIED | No |
| `OVS-006` | ERROR | Resource | Policy not found | 404 | NOT_FOUND | No |
| `OVS-007` | ERROR | Resource | Audit log not found | 404 | NOT_FOUND | No |
| `OVS-008` | ERROR | Validation | Invalid policy rule syntax | 400 | INVALID_ARGUMENT | No |
| `OVS-009` | ERROR | Validation | Policy rule conflict detected | 409 | FAILED_PRECONDITION | No |
| `OVS-010` | ERROR | Validation | Circular policy dependency detected | 400 | FAILED_PRECONDITION | No |
| `OVS-011` | ERROR | Evaluation | Policy evaluation timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `OVS-012` | ERROR | Evaluation | Policy evaluation failed | 500 | INTERNAL | Yes |
| `OVS-013` | ERROR | Resource | Policy quota exceeded | 429 | RESOURCE_EXHAUSTED | No |
| `OVS-014` | ERROR | Audit | Audit log write failed | 500 | INTERNAL | Yes |
| `OVS-015` | CRITICAL | Audit | Audit log storage full | 507 | RESOURCE_EXHAUSTED | No |
| `OVS-016` | ERROR | Audit | Audit log encryption failed | 500 | INTERNAL | Yes |
| `OVS-017` | ERROR | Dependency | Policy engine unavailable | 503 | UNAVAILABLE | Yes |
| `OVS-018` | ERROR | Validation | Invalid audit query parameters | 400 | INVALID_ARGUMENT | No |
| `OVS-019` | ERROR | Timeout | Audit query timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `OVS-020` | ERROR | Export | Audit log export failed | 500 | INTERNAL | Yes |
| `OVS-021` | ERROR | Crypto | Audit log signature verification failed | 401 | UNAUTHENTICATED | No |
| `OVS-022` | CRITICAL | Integrity | Audit log tampering detected | 500 | DATA_LOSS | No |
| `OVS-023` | ERROR | Validation | Invalid compliance framework specified | 400 | INVALID_ARGUMENT | No |
| `OVS-024` | WARNING | Compliance | Compliance violation detected | 200 | OK | N/A |
| `OVS-025` | ERROR | RateLimit | Policy evaluation rate limit exceeded | 429 | RESOURCE_EXHAUSTED | Yes |
| `OVS-026` | ERROR | Validation | Invalid time range for audit query | 400 | INVALID_ARGUMENT | No |
| `OVS-027` | ERROR | Resource | Maximum policy version limit reached | 429 | RESOURCE_EXHAUSTED | No |
| `OVS-028` | ERROR | Validation | Policy syntax parser error | 400 | INVALID_ARGUMENT | No |
| `OVS-029` | ERROR | Dependency | Alert notification system unavailable | 503 | UNAVAILABLE | Yes |
| `OVS-030` | ERROR | Validation | Invalid policy scope definition | 400 | INVALID_ARGUMENT | No |

---

### 1.6 Messaging (MSG) Error Codes

| Code | Severity | Category | Message | HTTP | gRPC | Retryable |
|------|----------|----------|---------|------|------|-----------|
| `MSG-001` | CRITICAL | System | Messaging service unavailable | 503 | UNAVAILABLE | Yes |
| `MSG-002` | ERROR | Validation | Invalid message format | 400 | INVALID_ARGUMENT | No |
| `MSG-003` | ERROR | Validation | Missing required message fields | 400 | INVALID_ARGUMENT | No |
| `MSG-004` | ERROR | Authentication | Invalid messaging API credentials | 401 | UNAUTHENTICATED | No |
| `MSG-005` | ERROR | Authorization | Insufficient permissions for message operations | 403 | PERMISSION_DENIED | No |
| `MSG-006` | ERROR | Resource | Message queue not found | 404 | NOT_FOUND | No |
| `MSG-007` | ERROR | Resource | Message not found | 404 | NOT_FOUND | No |
| `MSG-008` | ERROR | Validation | Message payload exceeds size limit | 413 | OUT_OF_RANGE | No |
| `MSG-009` | ERROR | Validation | Invalid message TTL value | 400 | INVALID_ARGUMENT | No |
| `MSG-010` | ERROR | Resource | Message queue full | 507 | RESOURCE_EXHAUSTED | Yes |
| `MSG-011` | ERROR | Delivery | Message delivery timeout | 504 | DEADLINE_EXCEEDED | Yes |
| `MSG-012` | ERROR | Delivery | Message delivery failed after max retries | 500 | INTERNAL | No |
| `MSG-013` | ERROR | Validation | Invalid routing key | 400 | INVALID_ARGUMENT | No |
| `MSG-014` | ERROR | Validation | Invalid message priority | 400 | INVALID_ARGUMENT | No |
| `MSG-015` | ERROR | Subscription | Subscription not found | 404 | NOT_FOUND | No |
| `MSG-016` | ERROR | Subscription | Maximum subscriptions limit reached | 429 | RESOURCE_EXHAUSTED | No |
| `MSG-017` | ERROR | Crypto | Message encryption failed | 500 | INTERNAL | Yes |
| `MSG-018` | ERROR | Crypto | Message decryption failed | 500 | INTERNAL | No |
| `MSG-019` | ERROR | Validation | Invalid message filter expression | 400 | INVALID_ARGUMENT | No |
| `MSG-020` | ERROR | Dependency | Message broker unavailable | 503 | UNAVAILABLE | Yes |
| `MSG-021` | ERROR | Dependency | Dead letter queue unavailable | 503 | UNAVAILABLE | Yes |
| `MSG-022` | ERROR | Ordering | Message ordering violation detected | 409 | FAILED_PRECONDITION | No |
| `MSG-023` | ERROR | Duplicate | Duplicate message ID detected | 409 | ALREADY_EXISTS | No |
| `MSG-024` | WARNING | Delivery | Message delivery delayed | 200 | OK | N/A |
| `MSG-025` | ERROR | RateLimit | Message publish rate limit exceeded | 429 | RESOURCE_EXHAUSTED | Yes |
| `MSG-026` | ERROR | Batch | Batch publish size exceeds limit | 413 | OUT_OF_RANGE | No |
| `MSG-027` | ERROR | Batch | Batch publish partially failed | 207 | INTERNAL | Yes |
| `MSG-028` | ERROR | Validation | Invalid message schema version | 400 | INVALID_ARGUMENT | No |
| `MSG-029` | ERROR | Resource | Consumer group not found | 404 | NOT_FOUND | No |
| `MSG-030` | ERROR | Validation | Invalid acknowledgment token | 400 | INVALID_ARGUMENT | No |

---

## 2. Error Categories

### 2.1 Validation Errors

**Characteristics**:
- Client-side errors due to invalid input
- Non-retryable
- HTTP 400 (Bad Request) or 422 (Unprocessable Entity)
- gRPC INVALID_ARGUMENT or FAILED_PRECONDITION

**Examples**:
- `RTM-002`: Invalid function execution request
- `MTR-003`: Missing required metering fields
- `OVS-008`: Invalid policy rule syntax
- `MSG-003`: Missing required message fields

**Handling**:
- Log error details for debugging
- Return descriptive error message to client
- Include field-level validation errors in response
- No retry attempts

---

### 2.2 Authentication/Authorization Errors

**Characteristics**:
- Security-related errors
- Non-retryable (unless credentials refreshed)
- HTTP 401 (Unauthorized) or 403 (Forbidden)
- gRPC UNAUTHENTICATED or PERMISSION_DENIED

**Examples**:
- `RTM-004`: Invalid or expired API key
- `MTR-005`: Insufficient permissions for metering data
- `OVS-005`: Insufficient permissions for policy management
- `MSG-004`: Invalid messaging API credentials

**Handling**:
- Log security event for audit trail
- Return generic error message (avoid leaking security details)
- Trigger security monitoring alerts for repeated failures
- Client should refresh credentials before retry

---

### 2.3 Resource Errors

**Characteristics**:
- Resource not found or exhausted
- Mixed retryability (not found = no, exhausted = maybe)
- HTTP 404 (Not Found), 507 (Insufficient Storage), 429 (Too Many Requests)
- gRPC NOT_FOUND or RESOURCE_EXHAUSTED

**Examples**:
- `RTM-008`: Function not found
- `MTR-016`: Storage quota exceeded
- `OVS-015`: Audit log storage full
- `MSG-010`: Message queue full

**Handling**:
- **Not Found**: Log and return error, no retry
- **Exhausted**: Implement backoff and retry with reduced load
- Alert operations team for capacity issues
- Client may need to request quota increase

---

### 2.4 Timeout/Network Errors

**Characteristics**:
- Transient network or service delays
- Retryable with exponential backoff
- HTTP 504 (Gateway Timeout) or 503 (Service Unavailable)
- gRPC DEADLINE_EXCEEDED or UNAVAILABLE

**Examples**:
- `RTM-011`: Function execution timeout exceeded
- `MTR-017`: Usage query timeout
- `OVS-011`: Policy evaluation timeout
- `MSG-011`: Message delivery timeout

**Handling**:
- Log timeout details (duration, endpoint)
- Retry with exponential backoff
- Implement circuit breaker after threshold
- Alert if timeout rate exceeds threshold

---

### 2.5 Cryptographic Errors

**Characteristics**:
- Encryption/decryption failures
- Mixed retryability (key errors = no, transient = yes)
- HTTP 500 (Internal Server Error) or 400 (Bad Request)
- gRPC INTERNAL or INVALID_ARGUMENT

**Examples**:
- `RTM-025`: Encryption operation failed
- `MTR-020`: Usage data encryption failed
- `OVS-016`: Audit log encryption failed
- `MSG-017`: Message encryption failed

**Handling**:
- Log error with sanitized details (no key material)
- **Invalid Key**: Return error, no retry
- **Transient Failure**: Retry once with same key
- Alert security team for repeated failures

---

### 2.6 Quota/Rate Limit Errors

**Characteristics**:
- Resource usage limits exceeded
- Retryable after backoff period
- HTTP 429 (Too Many Requests)
- gRPC RESOURCE_EXHAUSTED

**Examples**:
- `RTM-030`: Rate limit exceeded
- `MTR-025`: Metering API rate limit exceeded
- `OVS-025`: Policy evaluation rate limit exceeded
- `MSG-025`: Message publish rate limit exceeded

**Handling**:
- Return `Retry-After` header with wait duration
- Implement token bucket or leaky bucket algorithm
- Client should respect backoff period before retry
- Log rate limit violations for capacity planning

---

### 2.7 Dependency Errors

**Characteristics**:
- Upstream service failures
- Retryable with circuit breaker
- HTTP 502 (Bad Gateway) or 503 (Service Unavailable)
- gRPC UNAVAILABLE or FAILED_PRECONDITION

**Examples**:
- `RTM-019`: Required dependency unavailable
- `MTR-014`: Time-series database unavailable
- `OVS-017`: Policy engine unavailable
- `MSG-020`: Message broker unavailable

**Handling**:
- Implement circuit breaker pattern
- Retry with exponential backoff
- Degrade gracefully if possible (cached data, default policy)
- Alert operations team for upstream failures

---

## 3. Retry Strategies

### 3.1 Retryable vs Non-Retryable Errors

| Error Type | Retryable | Max Retries | Backoff Strategy |
|------------|-----------|-------------|------------------|
| Validation | No | 0 | N/A |
| Authentication | No | 0 | N/A |
| Authorization | No | 0 | N/A |
| Not Found | No | 0 | N/A |
| Timeout | Yes | 3 | Exponential with jitter |
| Network | Yes | 3 | Exponential with jitter |
| Service Unavailable | Yes | 5 | Exponential with jitter |
| Resource Exhausted | Yes | 2 | Linear backoff |
| Rate Limit | Yes | 3 | Respect Retry-After header |
| Dependency Failure | Yes | 3 | Exponential with circuit breaker |

---

### 3.2 Exponential Backoff with Jitter

**Algorithm**:
```
wait_time = min(max_wait, base_delay * 2^(attempt - 1)) + random_jitter(0, jitter_range)
```

**Parameters**:
- `base_delay`: 100ms (initial wait time)
- `max_wait`: 30 seconds (maximum wait time)
- `jitter_range`: ±25% of calculated wait time
- `max_retries`: 3-5 attempts (error-dependent)

**Example Sequence**:
- Attempt 1: Fail → Wait 100ms (±25ms)
- Attempt 2: Fail → Wait 200ms (±50ms)
- Attempt 3: Fail → Wait 400ms (±100ms)
- Attempt 4: Fail → Wait 800ms (±200ms)
- Attempt 5: Fail → Return error to client

**Implementation**:
```typescript
async function retryWithBackoff<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= maxRetries + 1; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;

      if (!isRetryable(error) || attempt > maxRetries) {
        throw error;
      }

      const baseDelay = 100;
      const maxWait = 30000;
      const exponentialDelay = Math.min(
        maxWait,
        baseDelay * Math.pow(2, attempt - 1)
      );
      const jitter = exponentialDelay * 0.25 * (Math.random() * 2 - 1);
      const waitTime = exponentialDelay + jitter;

      await sleep(waitTime);
    }
  }

  throw lastError;
}
```

---

### 3.3 Circuit Breaker Pattern

**States**:
1. **CLOSED**: Normal operation, requests pass through
2. **OPEN**: Service failed, requests fail immediately
3. **HALF_OPEN**: Testing if service recovered

**Thresholds**:
- **Failure Threshold**: 5 consecutive failures or 50% error rate in 10-second window
- **Success Threshold**: 2 consecutive successes in HALF_OPEN state
- **Timeout**: 30 seconds before transitioning from OPEN to HALF_OPEN

**State Transitions**:
```
CLOSED --[failure threshold exceeded]--> OPEN
OPEN --[timeout elapsed]--> HALF_OPEN
HALF_OPEN --[success threshold met]--> CLOSED
HALF_OPEN --[failure]--> OPEN
```

**Implementation**:
```typescript
class CircuitBreaker {
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';
  private failureCount = 0;
  private successCount = 0;
  private lastFailureTime: number = 0;

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime > 30000) {
        this.state = 'HALF_OPEN';
        this.successCount = 0;
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess() {
    this.failureCount = 0;

    if (this.state === 'HALF_OPEN') {
      this.successCount++;
      if (this.successCount >= 2) {
        this.state = 'CLOSED';
      }
    }
  }

  private onFailure() {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= 5) {
      this.state = 'OPEN';
    }
  }
}
```

---

### 3.4 Rate Limit Retry Strategy

**Approach**: Respect `Retry-After` header from server

**Algorithm**:
1. Receive 429 response with `Retry-After` header
2. Parse header value (seconds or HTTP date)
3. Wait for specified duration + jitter
4. Retry request
5. If no `Retry-After` header, use exponential backoff

**Example**:
```typescript
async function retryWithRateLimit<T>(
  operation: () => Promise<Response>
): Promise<T> {
  let attempt = 0;
  const maxRetries = 3;

  while (attempt <= maxRetries) {
    const response = await operation();

    if (response.status !== 429) {
      return await response.json();
    }

    if (attempt >= maxRetries) {
      throw new Error('Rate limit retry exhausted');
    }

    const retryAfter = response.headers.get('Retry-After');
    const waitTime = retryAfter
      ? parseInt(retryAfter) * 1000
      : 100 * Math.pow(2, attempt);

    await sleep(waitTime + Math.random() * 1000);
    attempt++;
  }
}
```

---

## 4. Error Response Format

### 4.1 Standard JSON Error Envelope

**Structure**:
```json
{
  "error": {
    "code": "RTM-011",
    "message": "Function execution timeout exceeded",
    "severity": "ERROR",
    "category": "Timeout",
    "retryable": false,
    "timestamp": "2025-12-26T10:30:45.123Z",
    "trace_id": "550e8400-e29b-41d4-a716-446655440000",
    "details": {
      "function_id": "fn_abc123",
      "timeout_ms": 30000,
      "elapsed_ms": 30142
    },
    "help_url": "https://docs.enablement.io/errors/RTM-011"
  }
}
```

**Fields**:
- `code` (string, required): Error code (e.g., "RTM-011")
- `message` (string, required): Human-readable error description
- `severity` (enum, required): CRITICAL | ERROR | WARNING | INFO
- `category` (string, required): Error category
- `retryable` (boolean, required): Whether error is retryable
- `timestamp` (ISO8601, required): Error occurrence time
- `trace_id` (UUID, required): Distributed trace ID for correlation
- `details` (object, optional): Additional error-specific context
- `help_url` (string, optional): Link to documentation

---

### 4.2 gRPC Status Code Mapping

| Error Category | gRPC Status | HTTP Equivalent |
|----------------|-------------|-----------------|
| Validation | INVALID_ARGUMENT (3) | 400 Bad Request |
| Authentication | UNAUTHENTICATED (16) | 401 Unauthorized |
| Authorization | PERMISSION_DENIED (7) | 403 Forbidden |
| Not Found | NOT_FOUND (5) | 404 Not Found |
| Conflict | ALREADY_EXISTS (6) | 409 Conflict |
| Precondition Failed | FAILED_PRECONDITION (9) | 412 Precondition Failed |
| Out of Range | OUT_OF_RANGE (11) | 400 Bad Request |
| Internal Error | INTERNAL (13) | 500 Internal Server Error |
| Unavailable | UNAVAILABLE (14) | 503 Service Unavailable |
| Timeout | DEADLINE_EXCEEDED (4) | 504 Gateway Timeout |
| Resource Exhausted | RESOURCE_EXHAUSTED (8) | 429 Too Many Requests |
| Data Loss | DATA_LOSS (15) | 500 Internal Server Error |

**gRPC Error Details** (using `google.rpc.Status`):
```protobuf
message ErrorResponse {
  string code = 1;           // RTM-011
  string message = 2;        // Function execution timeout exceeded
  string severity = 3;       // ERROR
  string trace_id = 4;       // UUID
  google.protobuf.Struct details = 5;  // Additional context
}
```

---

### 4.3 HTTP Status Code Mapping

| HTTP Code | Description | Use Cases |
|-----------|-------------|-----------|
| 400 | Bad Request | Validation errors, invalid input |
| 401 | Unauthorized | Missing or invalid credentials |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Duplicate resource, constraint violation |
| 413 | Payload Too Large | Request/message size exceeds limit |
| 422 | Unprocessable Entity | Semantic validation failure |
| 429 | Too Many Requests | Rate limit or quota exceeded |
| 500 | Internal Server Error | Unhandled server error |
| 502 | Bad Gateway | Upstream dependency failure |
| 503 | Service Unavailable | Service down or overloaded |
| 504 | Gateway Timeout | Upstream timeout |
| 507 | Insufficient Storage | Storage quota exceeded |

**Response Headers**:
- `X-Error-Code`: Enablement error code (e.g., "RTM-011")
- `X-Trace-Id`: Distributed trace ID
- `Retry-After`: Seconds to wait before retry (for 429, 503)
- `Content-Type`: application/json

---

## 5. Cross-Product Error Propagation

### 5.1 Runtime → Metering Error Flow

**Scenario**: Runtime execution fails, metering record incomplete

**Error Propagation**:
```
1. Runtime: Function execution fails (RTM-016)
   ↓
2. Runtime: Attempts to record partial usage
   ↓
3. Metering: Receives incomplete usage record
   ↓
4. Metering: Logs warning (MTR-012: aggregation delayed)
   ↓
5. Metering: Stores partial data with "failed" status
   ↓
6. Metering: Returns success (202 Accepted) to Runtime
```

**Error Correlation**:
- Shared `trace_id` links Runtime and Metering records
- Metering records include `execution_status: "failed"`
- Billing system can filter out failed executions

---

### 5.2 Oversight → Runtime Error Flow

**Scenario**: Policy evaluation timeout delays function execution

**Error Propagation**:
```
1. Runtime: Requests policy evaluation from Oversight
   ↓
2. Oversight: Policy evaluation timeout (OVS-011)
   ↓
3. Oversight: Returns 504 with trace_id
   ↓
4. Runtime: Retries policy evaluation (max 2 attempts)
   ↓
5. Runtime: If still failing, applies default "deny" policy
   ↓
6. Runtime: Returns 403 to client (RTM-006) with original trace_id
   ↓
7. Client: Can trace full error chain via trace_id
```

**Fallback Strategy**:
- Runtime has configurable default policies (deny-by-default or allow-by-default)
- Oversight timeout does not block execution indefinitely
- Audit log records policy evaluation failure for compliance

---

### 5.3 Messaging → All Products Event Flow

**Scenario**: Messaging delivery failure affects event-driven workflows

**Error Propagation**:
```
1. Runtime: Publishes execution completion event to Messaging
   ↓
2. Messaging: Delivery timeout (MSG-011)
   ↓
3. Messaging: Retries delivery (3 attempts with backoff)
   ↓
4. Messaging: Final failure → moves to Dead Letter Queue (DLQ)
   ↓
5. Messaging: Publishes DLQ event with original trace_id
   ↓
6. Monitoring: Detects DLQ event, triggers alert
   ↓
7. Metering/Oversight: May miss event, reconciliation detects gap
```

**Reconciliation**:
- All products maintain idempotency keys for event deduplication
- Periodic reconciliation jobs detect missing events
- DLQ messages can be replayed after fixing root cause

---

### 5.4 Error Correlation via Trace ID

**Trace ID Format**: UUID v4 (e.g., `550e8400-e29b-41d4-a716-446655440000`)

**Propagation**:
1. Client generates trace_id (or API gateway generates)
2. All internal service calls include `X-Trace-Id` header
3. All logs and error responses include trace_id
4. Distributed tracing system (e.g., Jaeger) aggregates spans

**Example Multi-Product Error Trace**:
```
Trace ID: 550e8400-e29b-41d4-a716-446655440000

Span 1 (Runtime):
  - Operation: execute_function
  - Status: ERROR
  - Error: RTM-011 (timeout)
  - Duration: 30.2s

Span 2 (Oversight):
  - Operation: evaluate_policy
  - Status: ERROR
  - Error: OVS-011 (timeout)
  - Duration: 15.1s

Span 3 (Metering):
  - Operation: record_usage
  - Status: SUCCESS
  - Duration: 45ms
  - Note: Recorded as "failed execution"
```

---

## 6. Client Handling Guidelines

### 6.1 SDK Error Handling Patterns

**TypeScript/JavaScript SDK**:
```typescript
import { EnablementClient, EnablementError } from '@enablement/sdk';

const client = new EnablementClient({ apiKey: process.env.API_KEY });

try {
  const result = await client.runtime.execute({
    functionId: 'fn_abc123',
    payload: { input: 'data' }
  });
  console.log('Success:', result);

} catch (error) {
  if (error instanceof EnablementError) {
    console.error('Error Code:', error.code);        // RTM-011
    console.error('Message:', error.message);        // Function execution timeout
    console.error('Severity:', error.severity);      // ERROR
    console.error('Retryable:', error.retryable);    // false
    console.error('Trace ID:', error.traceId);       // UUID
    console.error('Details:', error.details);        // { timeout_ms: 30000 }

    // Handle specific error codes
    switch (error.code) {
      case 'RTM-011': // Timeout
        console.log('Consider increasing function timeout');
        break;
      case 'RTM-030': // Rate limit
        const retryAfter = error.retryAfter || 60;
        console.log(`Rate limited. Retry after ${retryAfter}s`);
        break;
      default:
        console.error('Unhandled error:', error);
    }

    // Retry if error is retryable
    if (error.retryable) {
      // SDK handles retry internally with backoff
    }
  } else {
    console.error('Unexpected error:', error);
  }
}
```

**Python SDK**:
```python
from enablement import EnablementClient, EnablementError

client = EnablementClient(api_key=os.getenv('API_KEY'))

try:
    result = client.runtime.execute(
        function_id='fn_abc123',
        payload={'input': 'data'}
    )
    print('Success:', result)

except EnablementError as error:
    print(f'Error Code: {error.code}')          # RTM-011
    print(f'Message: {error.message}')          # Function execution timeout
    print(f'Severity: {error.severity}')        # ERROR
    print(f'Retryable: {error.retryable}')      # False
    print(f'Trace ID: {error.trace_id}')        # UUID
    print(f'Details: {error.details}')          # {'timeout_ms': 30000}

    # Handle specific error codes
    if error.code == 'RTM-011':
        print('Consider increasing function timeout')
    elif error.code == 'RTM-030':
        retry_after = error.retry_after or 60
        print(f'Rate limited. Retry after {retry_after}s')
        time.sleep(retry_after)
        # Retry logic

    # SDK handles retries automatically for retryable errors
```

---

### 6.2 User-Facing Error Messages

**Guidelines**:
1. **Avoid Technical Jargon**: Use plain language
2. **Be Specific**: Explain what went wrong
3. **Provide Action**: Tell user what to do next
4. **Include Support Info**: Link to docs or support

**Error Message Mapping**:

| Internal Code | User-Facing Message |
|---------------|---------------------|
| `RTM-004` | "Your API key is invalid or has expired. Please check your credentials and try again." |
| `RTM-006` | "You don't have permission to perform this action. Contact your administrator to request access." |
| `RTM-008` | "The function you requested could not be found. Please verify the function ID and try again." |
| `RTM-011` | "The function took too long to execute and was stopped. Try again or contact support if the issue persists." |
| `RTM-030` | "You've exceeded the rate limit. Please wait a moment before trying again." |
| `MTR-025` | "Too many requests. Please slow down and try again in a few moments." |
| `OVS-024` | "This action violates your organization's compliance policy. Contact your compliance officer for details." |
| `MSG-010` | "The message queue is currently full. Please try again shortly." |

**Example UI Implementation**:
```tsx
function ErrorDisplay({ error }: { error: EnablementError }) {
  const userMessage = getUserFacingMessage(error.code);
  const severity = error.severity.toLowerCase();

  return (
    <div className={`alert alert-${severity}`}>
      <h4>{userMessage}</h4>
      {error.retryable && (
        <button onClick={handleRetry}>Try Again</button>
      )}
      <details>
        <summary>Technical Details</summary>
        <pre>
          Error Code: {error.code}
          Trace ID: {error.traceId}
          {JSON.stringify(error.details, null, 2)}
        </pre>
      </details>
      <a href={error.helpUrl}>Learn More</a>
    </div>
  );
}
```

---

### 6.3 Logging Requirements

**Client-Side Logging**:
```typescript
// Structured logging format
logger.error('Enablement API Error', {
  error_code: error.code,           // RTM-011
  error_message: error.message,     // Function execution timeout exceeded
  severity: error.severity,         // ERROR
  retryable: error.retryable,       // false
  trace_id: error.traceId,          // UUID
  function_id: context.functionId,  // fn_abc123
  user_id: context.userId,          // user_xyz789
  timestamp: new Date().toISOString(),
  details: error.details
});
```

**Minimum Log Fields**:
- `error_code`: Enablement error code
- `trace_id`: For cross-service correlation
- `timestamp`: ISO8601 format
- `severity`: ERROR, WARNING, etc.
- `user_id` or `tenant_id`: For user-specific analysis

**Log Levels**:
- **ERROR**: All non-retryable errors, final retry failures
- **WARN**: Retryable errors, rate limits, degraded performance
- **INFO**: Successful retries after failure
- **DEBUG**: Retry attempts, backoff calculations

---

## 7. Monitoring & Alerting

### 7.1 Error Rate Thresholds

| Metric | Threshold | Action | Alert Channel |
|--------|-----------|--------|---------------|
| **Overall Error Rate** | >5% of requests | Investigate | Slack #alerts |
| **Critical Error Rate** | >1% of requests | Page on-call | PagerDuty |
| **5xx Error Rate** | >2% of requests | Investigate | Slack #alerts |
| **4xx Error Rate** | >10% of requests | Review docs/SDK | Slack #api-quality |
| **Timeout Rate** | >3% of requests | Scale resources | Slack #infrastructure |
| **Rate Limit Hit Rate** | >5% of users | Review quotas | Slack #product |
| **Circuit Breaker Open** | Any instance | Investigate dependency | PagerDuty |

**Calculation**:
```
Error Rate = (Error Count / Total Request Count) * 100
Time Window = 5 minutes (sliding window)
```

---

### 7.2 PagerDuty Integration

**Alert Triggers**:
1. **CRITICAL Severity Errors**: Immediate page
2. **Service Unavailable** (>1 minute): Immediate page
3. **Error Rate > 10%**: Page after 2 minutes
4. **Circuit Breaker Open** (>5 minutes): Page
5. **Audit Log Tampering** (OVS-022): Immediate page to security team

**PagerDuty Event Format**:
```json
{
  "routing_key": "enablement_production_key",
  "event_action": "trigger",
  "dedup_key": "RTM-001-prod-usw2",
  "payload": {
    "summary": "Runtime service unavailable (RTM-001)",
    "severity": "critical",
    "source": "runtime-api-usw2-prod",
    "component": "runtime",
    "group": "enablement-platform",
    "class": "system",
    "custom_details": {
      "error_code": "RTM-001",
      "region": "us-west-2",
      "instance_count": 3,
      "failed_instances": ["i-abc123", "i-def456", "i-ghi789"],
      "duration_seconds": 120
    }
  },
  "links": [
    {
      "href": "https://grafana.enablement.io/d/runtime",
      "text": "Runtime Dashboard"
    }
  ]
}
```

**Escalation Policy**:
1. **Level 1**: On-call engineer (0-15 minutes)
2. **Level 2**: Engineering manager (15-30 minutes)
3. **Level 3**: VP Engineering (30+ minutes)

---

### 7.3 Error Aggregation Dashboards

**Grafana Dashboard Panels**:

1. **Error Rate Overview**:
   - Line graph: Error rate per product over time
   - Breakdown: By error category (validation, timeout, etc.)
   - Time range: Last 24 hours

2. **Error Distribution**:
   - Pie chart: Top 10 error codes by frequency
   - Table: Error code, count, percentage, trend

3. **Error by Severity**:
   - Stacked area chart: CRITICAL, ERROR, WARNING over time
   - Alert threshold lines

4. **Retry Analysis**:
   - Success rate after retries
   - Average retry count per error type
   - Circuit breaker state changes

5. **Cross-Product Correlation**:
   - Heatmap: Error correlation between products
   - Trace waterfall view: Multi-service error flows

6. **SLA Compliance**:
   - Availability: 99.9% uptime (excludes 5xx errors)
   - Success rate: 99.5% (includes retries)
   - Latency P95/P99: Excludes timeout errors

**Prometheus Queries**:
```promql
# Overall error rate
sum(rate(http_requests_total{status=~"5.."}[5m]))
/
sum(rate(http_requests_total[5m])) * 100

# Errors by product
sum by (product) (rate(enablement_errors_total[5m]))

# Critical errors
sum(rate(enablement_errors_total{severity="CRITICAL"}[5m]))

# Top error codes
topk(10, sum by (error_code) (rate(enablement_errors_total[1h])))

# Circuit breaker status
enablement_circuit_breaker_state{state="open"}
```

**Alert Rules** (Prometheus AlertManager):
```yaml
groups:
  - name: enablement_errors
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: |
          (sum(rate(http_requests_total{status=~"5.."}[5m]))
          / sum(rate(http_requests_total[5m]))) * 100 > 5
        for: 2m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }}% (threshold: 5%)"

      - alert: CriticalErrorSpike
        expr: |
          sum(rate(enablement_errors_total{severity="CRITICAL"}[5m])) > 0.01
        for: 1m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Critical errors detected"
          description: "{{ $value }} critical errors per second"

      - alert: CircuitBreakerOpen
        expr: enablement_circuit_breaker_state{state="open"} == 1
        for: 5m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Circuit breaker open for {{ $labels.service }}"
          description: "Dependency failure detected"
```

---

## Appendix A: Error Code Quick Reference

### Runtime (RTM)
- **001-005**: System & validation errors
- **006-007**: Authorization errors
- **008-010**: Resource errors
- **011-012**: Timeout errors
- **013-015**: Resource exhaustion
- **016-020**: Execution & dependency errors
- **021-030**: Quota, network, crypto errors

### Metering (MTR)
- **001-005**: System, validation, auth errors
- **006-010**: Resource & validation errors
- **011-015**: Duplicate, aggregation, dependency errors
- **016-021**: Quota, timeout, crypto errors
- **022-030**: System integrity, export, batch errors

### Oversight (OVS)
- **001-010**: System, validation, policy errors
- **011-015**: Evaluation, quota, audit errors
- **016-022**: Crypto, integrity, audit log errors
- **023-030**: Compliance, rate limit, dependency errors

### Messaging (MSG)
- **001-010**: System, validation, queue errors
- **011-015**: Delivery, subscription errors
- **016-021**: Crypto, dependency errors
- **022-030**: Ordering, duplicate, batch errors

---

## Appendix B: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-12-26 | System | Initial draft |

---

**End of Document**
