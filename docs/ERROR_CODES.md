# Creto Enablement Error Codes

This document lists all error codes used in the Creto Enablement platform.

## Format

Error codes follow the format: `ENABLE-XXX` where:
- `ENABLE` is the repository prefix
- `XXX` is a 3-digit sequential number

## Code Ranges

| Range | Category | Source File |
|-------|----------|-------------|
| ENABLE-001 to ENABLE-034 | Core Errors | `creto-common/src/error.rs` |
| ENABLE-100 to ENABLE-111 | Validation Errors | `creto-metering/src/validation.rs` |
| ENABLE-200 to ENABLE-201 | Deduplication Errors | `creto-metering/src/dedup.rs` |
| ENABLE-300 to ENABLE-303 | Quota Enforcer Errors | `creto-metering/src/quota/enforcer.rs` |
| ENABLE-400 to ENABLE-405 | Reservation Errors | `creto-metering/src/quota/reservation.rs` |
| ENABLE-500 to ENABLE-506 | Checkpoint Errors | `creto-runtime/src/checkpoint.rs` |

---

## Core Errors (CretoError)

### Metering Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-001 | `QuotaExceeded` | Usage quota exceeded for a resource | Agent used 1000/1000 API calls |
| ENABLE-002 | `InvalidUsageEvent` | Usage event failed validation | Missing required fields in event |
| ENABLE-003 | `DuplicateTransaction` | Transaction ID already processed | Duplicate event submission |
| ENABLE-004 | `BillingPeriodNotFound` | Billing period not found | Invalid billing period reference |

### Oversight Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-005 | `ApprovalNotFound` | Approval request not found | Invalid approval request ID |
| ENABLE-006 | `InvalidStateTransition` | Invalid state transition | Attempting to approve already-rejected request |
| ENABLE-007 | `ApprovalTimeout` | Approval request timed out | No response within timeout period |
| ENABLE-008 | `QuorumNotReached` | Quorum not reached for approval | Insufficient approver votes |
| ENABLE-009 | `UnauthorizedApprover` | Approver not authorized | User not in approvers list |

### Runtime Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-010 | `SandboxNotFound` | Sandbox not found | Invalid sandbox ID |
| ENABLE-011 | `SandboxCreationFailed` | Sandbox creation failed | Resource exhaustion, invalid config |
| ENABLE-012 | `ExecutionTimeout` | Execution timed out | Code running past time limit |
| ENABLE-013 | `ResourceLimitExceeded` | Resource limit exceeded | Memory or CPU limit hit |
| ENABLE-014 | `NetworkEgressDenied` | Network egress denied | Attempting to reach blocked destination |

### Messaging Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-015 | `ChannelNotFound` | Channel not found | Invalid channel ID |
| ENABLE-016 | `EncryptionFailed` | Encryption failed | Invalid key material |
| ENABLE-017 | `DecryptionFailed` | Decryption failed | Corrupted ciphertext, wrong key |
| ENABLE-018 | `InvalidKeyBundle` | Invalid key bundle | Malformed key bundle |
| ENABLE-019 | `MessageDeliveryFailed` | Message delivery failed | Recipient offline, network failure |

### Authorization Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-020 | `AuthorizationDenied` | Authorization denied | Policy denied access |
| ENABLE-021 | `PolicyEvaluationFailed` | Policy evaluation failed | Invalid policy syntax |

### Infrastructure Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-022 | `Database` | Database error | Connection failure, query error |
| ENABLE-023 | `Configuration` | Configuration error | Missing or invalid config |
| ENABLE-024 | `Internal` | Internal error | Unexpected system error |

### Crypto Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-025 | `CryptoError` | Cryptography error | Signature verification failed |
| ENABLE-026 | `SecretResolutionFailed` | Secret resolution failed | Secret not found in vault |

### Session Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-027 | `SessionError` | Session error | Invalid or expired session |
| ENABLE-028 | `ChannelError` | Channel error | Channel communication failure |
| ENABLE-029 | `NotAuthorized` | Not authorized for action | Permission denied for operation |

### Serialization Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-030 | `SerializationError` | Serialization error | Invalid JSON, encoding failure |

### Generic Errors

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-031 | `NotFound` | Resource not found | Generic resource lookup failure |
| ENABLE-032 | `Unauthorized` | Unauthorized access | Authentication failure |
| ENABLE-033 | `LimitExceeded` | Limit exceeded | Generic rate/quota limit |
| ENABLE-034 | `ValidationFailed` | Validation failed | Generic validation failure |

---

## Validation Errors (ValidationError)

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-100 | `EmptyTransactionId` | Transaction ID is empty | Missing transaction_id field |
| ENABLE-101 | `TransactionIdTooLong` | Transaction ID too long | Transaction ID exceeds max length |
| ENABLE-102 | `NonPositiveQuantity` | Quantity must be positive | quantity <= 0 |
| ENABLE-103 | `QuantityTooLarge` | Quantity exceeds maximum | quantity > max_quantity |
| ENABLE-104 | `TimestampTooFuture` | Timestamp too far in future | Event timestamp > now + max_future |
| ENABLE-105 | `TimestampTooOld` | Timestamp too old | Event timestamp < now - max_past |
| ENABLE-106 | `EmptyMetricCode` | Metric code is empty | Missing code field |
| ENABLE-107 | `InvalidMetricCode` | Invalid metric code format | Non-alphanumeric characters in code |
| ENABLE-108 | `PropertiesTooLarge` | Properties JSON too large | Properties exceed max size |
| ENABLE-109 | `DelegationDepthTooDeep` | Delegation depth too deep | Exceeds max delegation chain |
| ENABLE-110 | `ExternalSubscriptionIdTooLong` | External subscription ID too long | ID exceeds max length |
| ENABLE-111 | `Multiple` | Multiple validation errors | Multiple fields failed validation |

---

## Deduplication Errors (DedupError)

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-200 | `Connection` | Redis connection error | Redis unavailable, network failure |
| ENABLE-201 | `Unavailable` | Dedup service unavailable | Service not initialized |

---

## Quota Enforcer Errors (EnforcerError)

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-300 | `QuotaExceeded` | Quota exceeded (enforcer level) | Usage exceeds limit |
| ENABLE-301 | `ReservationError` | Reservation operation failed | See ENABLE-4xx errors |
| ENABLE-302 | `CacheError` | Cache operation failed | Lock poisoned, cache full |
| ENABLE-303 | `RedisError` | Redis operation failed | Redis connection/command error |

---

## Reservation Errors (ReservationError)

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-400 | `InsufficientQuota` | Insufficient quota for reservation | Requested > available |
| ENABLE-401 | `NotFound` | Reservation not found | Invalid reservation ID |
| ENABLE-402 | `InvalidStatus` | Invalid reservation status | Committing already-committed reservation |
| ENABLE-403 | `ExceedsReserved` | Actual exceeds reserved | Actual usage > reserved amount |
| ENABLE-404 | `Expired` | Reservation expired | Reservation TTL exceeded |
| ENABLE-405 | `LockError` | Lock acquisition failed | Concurrent access conflict |

---

## Checkpoint Errors (CheckpointError)

| Code | Variant | Description | Example Cause |
|------|---------|-------------|---------------|
| ENABLE-500 | `NotFound` | Checkpoint not found | Invalid checkpoint ID |
| ENABLE-501 | `InvalidSandboxState` | Invalid sandbox state | Cannot checkpoint running sandbox |
| ENABLE-502 | `CreationFailed` | Checkpoint creation failed | Filesystem/memory error |
| ENABLE-503 | `RestoreFailed` | Checkpoint restore failed | Corrupted checkpoint data |
| ENABLE-504 | `IntegrityCheckFailed` | Integrity check failed | Hash mismatch on restore |
| ENABLE-505 | `CompressionError` | Compression/decompression error | Invalid compressed data |
| ENABLE-506 | `StorageError` | Storage backend error | S3/disk storage failure |

---

## Usage

### Rust Code

```rust
use creto_common::CretoError;

fn example() -> Result<(), CretoError> {
    let err = CretoError::QuotaExceeded {
        resource: "api_calls".to_string(),
        used: 1000,
        limit: 1000,
    };

    // Get error code
    println!("Error code: {}", err.code()); // ENABLE-001

    Err(err)
}
```

### API Responses

Error codes are included in API error responses:

```json
{
  "error": {
    "code": "ENABLE-001",
    "message": "Quota exceeded for api_calls: used 1000, limit 1000",
    "details": {
      "resource": "api_calls",
      "used": 1000,
      "limit": 1000
    }
  }
}
```

### Monitoring & Alerting

Use error codes for:
- Metrics: `creto_errors_total{code="ENABLE-001"}`
- Alerts: Alert on specific error codes
- Dashboards: Group errors by code range
- Support: Reference error codes in tickets
