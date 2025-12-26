# SPARC Pseudocode: Real-Time Data Synchronization

## Overview
This document contains the algorithmic design for real-time data synchronization in the Trading Demo application. All algorithms are designed for optimal performance, fault tolerance, and user experience.

---

## 1. WebSocket Connection Management

### 1.1 Connection Initialization

```
ALGORITHM: InitializeWebSocket
INPUT: userId (string), authToken (string)
OUTPUT: WebSocketConnection object

CONSTANTS:
    RECONNECT_DELAY_MS = 1000
    MAX_RECONNECT_DELAY_MS = 30000
    BACKOFF_MULTIPLIER = 1.5
    HEARTBEAT_INTERVAL_MS = 30000

BEGIN
    // Create connection configuration
    config ← {
        url: WS_ENDPOINT + "/realtime",
        protocols: ["trading-v1"],
        headers: {
            Authorization: "Bearer " + authToken,
            User-Id: userId
        }
    }

    // Initialize connection state
    connection ← CreateWebSocketConnection(config)
    reconnectAttempts ← 0
    subscriptions ← SET()
    messageQueue ← QUEUE()
    isConnected ← false

    // Set up event handlers
    ON connection.open DO
        Log("WebSocket connected for user: " + userId)
        isConnected ← true
        reconnectAttempts ← 0

        // Subscribe to user-specific channels
        channels ← [
            "user:" + userId + ":portfolio",
            "user:" + userId + ":trades",
            "user:" + userId + ":oversight",
            "user:" + userId + ":metering"
        ]

        FOR EACH channel IN channels DO
            SendSubscriptionMessage(connection, channel)
            subscriptions.add(channel)
        END FOR

        // Flush queued messages
        WHILE NOT messageQueue.isEmpty() DO
            msg ← messageQueue.dequeue()
            connection.send(msg)
        END WHILE

        // Start heartbeat
        StartHeartbeat(connection, HEARTBEAT_INTERVAL_MS)

        // Trigger connection event
        EmitEvent("ws:connected", {userId: userId})
    END ON

    ON connection.message DO (event)
        HandleIncomingMessage(event.data)
    END ON

    ON connection.error DO (error)
        Log("WebSocket error: " + error.message)
        EmitEvent("ws:error", {error: error})
    END ON

    ON connection.close DO (event)
        Log("WebSocket closed: " + event.code)
        isConnected ← false
        StopHeartbeat()

        // Attempt reconnection
        IF NOT event.wasClean THEN
            ScheduleReconnect(connection, reconnectAttempts)
            reconnectAttempts ← reconnectAttempts + 1
        END IF

        EmitEvent("ws:disconnected", {code: event.code})
    END ON

    RETURN connection
END

SUBROUTINE: ScheduleReconnect
INPUT: connection, attemptNumber
OUTPUT: void

BEGIN
    // Exponential backoff with jitter
    baseDelay ← RECONNECT_DELAY_MS * POWER(BACKOFF_MULTIPLIER, attemptNumber)
    jitter ← RANDOM(0, baseDelay * 0.3)
    delay ← MIN(baseDelay + jitter, MAX_RECONNECT_DELAY_MS)

    Log("Reconnecting in " + delay + "ms (attempt " + attemptNumber + ")")

    ScheduleTimeout(delay, FUNCTION()
        connection.reconnect()
    END FUNCTION)
END

SUBROUTINE: StartHeartbeat
INPUT: connection, interval
OUTPUT: void

BEGIN
    heartbeatTimer ← SetInterval(interval, FUNCTION()
        IF connection.isConnected THEN
            connection.send(JSON.stringify({
                type: "ping",
                timestamp: CurrentTimestamp()
            }))
        END IF
    END FUNCTION)

    connection.heartbeatTimer ← heartbeatTimer
END

SUBROUTINE: StopHeartbeat
INPUT: connection
OUTPUT: void

BEGIN
    IF connection.heartbeatTimer EXISTS THEN
        ClearInterval(connection.heartbeatTimer)
        connection.heartbeatTimer ← null
    END IF
END
```

**Time Complexity**: O(1) for connection setup, O(n) for subscribing to n channels
**Space Complexity**: O(n) where n = number of queued messages

---

## 2. Event Broadcasting

### 2.1 Server-Side Broadcasting

```
ALGORITHM: BroadcastEvent
INPUT: channel (string), event (object), options (object)
OUTPUT: BroadcastResult

CONSTANTS:
    MAX_RETRY_ATTEMPTS = 3
    SEND_TIMEOUT_MS = 5000
    MAX_BACKPRESSURE_QUEUE = 100

BEGIN
    // Serialize event payload
    payload ← JSON.stringify({
        channel: channel,
        event: event.type,
        data: event.data,
        timestamp: CurrentTimestamp(),
        messageId: GenerateUUID()
    })

    // Get subscribers for channel
    subscribers ← ChannelRegistry.getSubscribers(channel)

    IF subscribers.isEmpty() THEN
        Log("No subscribers for channel: " + channel)
        RETURN {sent: 0, failed: 0}
    END IF

    // Track results
    successCount ← 0
    failureCount ← 0
    pendingOperations ← []

    // Broadcast to all subscribers
    FOR EACH subscriber IN subscribers DO
        operation ← SendToSubscriberAsync(subscriber, payload, options)
        pendingOperations.append(operation)
    END FOR

    // Wait for all sends to complete
    results ← AwaitAll(pendingOperations)

    FOR EACH result IN results DO
        IF result.success THEN
            successCount ← successCount + 1
        ELSE
            failureCount ← failureCount + 1
            Log("Failed to send to subscriber: " + result.subscriberId)
        END IF
    END FOR

    // Log broadcast statistics
    LogMetric("broadcast", {
        channel: channel,
        eventType: event.type,
        subscribers: subscribers.size(),
        successful: successCount,
        failed: failureCount,
        duration: CurrentTimestamp() - event.timestamp
    })

    RETURN {
        sent: successCount,
        failed: failureCount,
        channel: channel
    }
END

SUBROUTINE: SendToSubscriberAsync
INPUT: subscriber, payload, options
OUTPUT: Promise<SendResult>

BEGIN
    RETURN AsyncFunction() BEGIN
        // Check connection health
        IF NOT subscriber.connection.isAlive THEN
            RemoveSubscriber(subscriber)
            RETURN {success: false, subscriberId: subscriber.id, reason: "dead_connection"}
        END IF

        // Check backpressure
        queueSize ← subscriber.connection.getQueueSize()
        IF queueSize > MAX_BACKPRESSURE_QUEUE THEN
            Log("Backpressure detected for subscriber: " + subscriber.id)

            IF options.dropOnBackpressure THEN
                RETURN {success: false, subscriberId: subscriber.id, reason: "backpressure"}
            ELSE
                // Wait for queue to drain
                AwaitCondition(FUNCTION()
                    RETURN subscriber.connection.getQueueSize() < MAX_BACKPRESSURE_QUEUE / 2
                END FUNCTION, SEND_TIMEOUT_MS)
            END IF
        END IF

        // Attempt to send
        attempt ← 0
        WHILE attempt < MAX_RETRY_ATTEMPTS DO
            TRY
                AwaitWithTimeout(
                    subscriber.connection.send(payload),
                    SEND_TIMEOUT_MS
                )
                RETURN {success: true, subscriberId: subscriber.id}
            CATCH TimeoutError
                attempt ← attempt + 1
                IF attempt < MAX_RETRY_ATTEMPTS THEN
                    Delay(100 * attempt) // Backoff
                END IF
            CATCH error
                RETURN {success: false, subscriberId: subscriber.id, reason: error.message}
            END TRY
        END WHILE

        RETURN {success: false, subscriberId: subscriber.id, reason: "max_retries"}
    END AsyncFunction
END
```

**Time Complexity**: O(n) where n = number of subscribers
**Space Complexity**: O(n) for pending operations array

---

## 3. Portfolio Update Pipeline

### 3.1 Real-Time Portfolio Calculation

```
ALGORITHM: UpdatePortfolio
INPUT: agentId (string), priceUpdates (array of PriceUpdate)
OUTPUT: PortfolioUpdate

CONSTANTS:
    DEBOUNCE_WINDOW_MS = 100
    CACHE_TTL_MS = 5000

DATA STRUCTURES:
    PriceUpdate: {symbol: string, price: decimal, timestamp: long}
    Position: {symbol: string, quantity: decimal, avgCost: decimal}
    PortfolioSnapshot: {totalValue: decimal, dailyPnL: decimal, positions: array}

BEGIN
    // Check debounce timer
    lastUpdateTime ← DebounceTimers.get(agentId)
    currentTime ← CurrentTimestamp()

    IF lastUpdateTime EXISTS AND (currentTime - lastUpdateTime) < DEBOUNCE_WINDOW_MS THEN
        // Queue update for batching
        UpdateQueue.add(agentId, priceUpdates)
        RETURN null // Will process in batch
    END IF

    // Update debounce timer
    DebounceTimers.set(agentId, currentTime)

    // Fetch current positions (with caching)
    positions ← GetPositionsWithCache(agentId)

    IF positions.isEmpty() THEN
        RETURN {
            agentId: agentId,
            totalValue: 0,
            dailyPnL: 0,
            positions: []
        }
    END IF

    // Create price lookup map for O(1) access
    priceMap ← MAP()
    FOR EACH update IN priceUpdates DO
        priceMap.set(update.symbol, update.price)
    END FOR

    // Calculate portfolio metrics
    totalValue ← 0
    dailyPnL ← 0
    updatedPositions ← []

    FOR EACH position IN positions DO
        // Get current price (use cached if not in update)
        currentPrice ← priceMap.get(position.symbol)
        IF currentPrice IS null THEN
            currentPrice ← PriceCache.get(position.symbol)
        ELSE
            PriceCache.set(position.symbol, currentPrice, CACHE_TTL_MS)
        END IF

        // Calculate position value and P&L
        positionValue ← position.quantity * currentPrice
        costBasis ← position.quantity * position.avgCost
        unrealizedPnL ← positionValue - costBasis

        // Get previous day's close for daily P&L
        previousClose ← GetPreviousClose(position.symbol)
        dailyPositionPnL ← position.quantity * (currentPrice - previousClose)

        // Calculate percentage changes
        unrealizedPnLPercent ← (unrealizedPnL / costBasis) * 100
        dailyPnLPercent ← ((currentPrice - previousClose) / previousClose) * 100

        // Accumulate totals
        totalValue ← totalValue + positionValue
        dailyPnL ← dailyPnL + dailyPositionPnL

        // Add to updated positions
        updatedPositions.append({
            symbol: position.symbol,
            quantity: position.quantity,
            avgCost: position.avgCost,
            currentPrice: currentPrice,
            marketValue: positionValue,
            unrealizedPnL: unrealizedPnL,
            unrealizedPnLPercent: unrealizedPnLPercent,
            dailyPnL: dailyPositionPnL,
            dailyPnLPercent: dailyPnLPercent
        })
    END FOR

    // Create portfolio snapshot
    snapshot ← {
        agentId: agentId,
        timestamp: currentTime,
        totalValue: totalValue,
        dailyPnL: dailyPnL,
        dailyPnLPercent: (dailyPnL / (totalValue - dailyPnL)) * 100,
        positions: updatedPositions,
        positionCount: updatedPositions.length
    }

    // Broadcast update
    BroadcastEvent("user:" + GetUserIdForAgent(agentId) + ":portfolio", {
        type: "portfolio_update",
        data: snapshot
    }, {dropOnBackpressure: false})

    // Update cache
    PortfolioCache.set(agentId, snapshot, CACHE_TTL_MS)

    RETURN snapshot
END

SUBROUTINE: GetPositionsWithCache
INPUT: agentId
OUTPUT: array of Position

BEGIN
    // Check cache first
    cached ← PositionCache.get(agentId)
    IF cached EXISTS AND NOT cached.isExpired() THEN
        RETURN cached.data
    END IF

    // Fetch from database
    positions ← Database.query(
        "SELECT symbol, quantity, avg_cost FROM positions WHERE agent_id = ?",
        [agentId]
    )

    // Update cache
    PositionCache.set(agentId, positions, CACHE_TTL_MS)

    RETURN positions
END

SUBROUTINE: GetPreviousClose
INPUT: symbol
OUTPUT: decimal

BEGIN
    // Check cache
    cached ← PreviousCloseCache.get(symbol)
    IF cached EXISTS THEN
        RETURN cached
    END IF

    // Fetch from market data service
    previousClose ← MarketDataService.getPreviousClose(symbol)

    // Cache for the day
    cacheUntilEndOfDay ← CalculateTimeUntilEndOfDay()
    PreviousCloseCache.set(symbol, previousClose, cacheUntilEndOfDay)

    RETURN previousClose
END
```

**Time Complexity**: O(n + m) where n = positions, m = price updates
**Space Complexity**: O(n) for position calculations

---

## 4. Optimistic Updates

### 4.1 Trade Submission with Optimistic UI

```
ALGORITHM: HandleOptimisticUpdate
INPUT: action (function), rollback (function), optimisticState (object)
OUTPUT: Promise<Result>

BEGIN
    // Generate unique transaction ID
    transactionId ← GenerateUUID()

    // Store rollback function
    RollbackRegistry.set(transactionId, rollback)

    // Apply optimistic state immediately
    currentState ← GetCurrentState()
    newState ← ApplyOptimisticChange(currentState, optimisticState)
    UpdateUIState(newState)

    // Mark transaction as pending
    PendingTransactions.set(transactionId, {
        startTime: CurrentTimestamp(),
        action: action.name,
        state: optimisticState
    })

    TRY
        // Execute server action
        result ← Await action()

        // Verify result matches optimistic state
        IF NOT ValidateResult(result, optimisticState) THEN
            Log("Server result differs from optimistic state")
            // Reconcile differences
            reconciledState ← ReconcileStates(optimisticState, result)
            UpdateUIState(reconciledState)
        END IF

        // Confirm transaction
        PendingTransactions.delete(transactionId)
        RollbackRegistry.delete(transactionId)

        // Emit success event
        EmitEvent("optimistic:confirmed", {
            transactionId: transactionId,
            result: result
        })

        RETURN {success: true, data: result}

    CATCH error
        // Rollback optimistic changes
        rollbackState ← rollback()
        UpdateUIState(rollbackState)

        // Clean up transaction
        PendingTransactions.delete(transactionId)
        RollbackRegistry.delete(transactionId)

        // Show error to user
        ShowErrorNotification(error.message)

        // Emit failure event
        EmitEvent("optimistic:failed", {
            transactionId: transactionId,
            error: error
        })

        RETURN {success: false, error: error}
    END TRY
END

SUBROUTINE: ApplyOptimisticChange
INPUT: currentState, optimisticState
OUTPUT: newState

BEGIN
    // Clone current state (immutable update)
    newState ← DeepClone(currentState)

    // Apply optimistic changes based on action type
    IF optimisticState.type = "trade_submit" THEN
        // Add trade to pending list
        trade ← {
            id: optimisticState.tradeId,
            status: "pending",
            symbol: optimisticState.symbol,
            side: optimisticState.side,
            quantity: optimisticState.quantity,
            price: optimisticState.price,
            timestamp: CurrentTimestamp(),
            isOptimistic: true
        }
        newState.pendingTrades.append(trade)

        // Update portfolio optimistically
        IF optimisticState.side = "buy" THEN
            newState.cash ← newState.cash - (optimisticState.quantity * optimisticState.price)
        ELSE IF optimisticState.side = "sell" THEN
            position ← FindPosition(newState.positions, optimisticState.symbol)
            IF position EXISTS THEN
                position.quantity ← position.quantity - optimisticState.quantity
            END IF
        END IF

    ELSE IF optimisticState.type = "oversight_request" THEN
        // Add request to pending list
        request ← {
            id: optimisticState.requestId,
            status: "pending_approval",
            agentId: optimisticState.agentId,
            requestType: optimisticState.requestType,
            timestamp: CurrentTimestamp(),
            isOptimistic: true
        }
        newState.oversightRequests.append(request)
    END IF

    RETURN newState
END

SUBROUTINE: ReconcileStates
INPUT: optimisticState, serverState
OUTPUT: reconciledState

BEGIN
    // Server state is source of truth
    reconciledState ← serverState

    // Check for conflicts
    conflicts ← FindConflicts(optimisticState, serverState)

    IF NOT conflicts.isEmpty() THEN
        Log("State conflicts detected: " + conflicts)

        // Apply conflict resolution strategy
        FOR EACH conflict IN conflicts DO
            IF conflict.type = "price_mismatch" THEN
                // Use server price, recalculate totals
                reconciledState.price ← serverState.price
                reconciledState.total ← serverState.quantity * serverState.price
            ELSE IF conflict.type = "quantity_partial_fill" THEN
                // Server may have partially filled order
                reconciledState.quantity ← serverState.filledQuantity
                reconciledState.status ← "partially_filled"
            END IF
        END FOR
    END IF

    RETURN reconciledState
END
```

**Example Use Case: Trade Submission**
```
// User submits trade
HandleOptimisticUpdate(
    action: FUNCTION() BEGIN
        RETURN API.submitTrade(tradeParams)
    END,
    rollback: FUNCTION() BEGIN
        RemovePendingTrade(tradeId)
        RestoreCashBalance(previousCash)
        RETURN GetCurrentState()
    END,
    optimisticState: {
        type: "trade_submit",
        tradeId: "temp-123",
        symbol: "AAPL",
        side: "buy",
        quantity: 10,
        price: 150.00
    }
)
```

**Time Complexity**: O(1) for state application, O(n) for reconciliation
**Space Complexity**: O(1) for single transaction

---

## 5. State Synchronization

### 5.1 Client-Server State Reconciliation

```
ALGORITHM: SynchronizeState
INPUT: clientState (object), serverState (object)
OUTPUT: ReconciledState

DATA STRUCTURES:
    StateVersion: {version: integer, timestamp: long, checksum: string}
    Conflict: {path: string, clientValue: any, serverValue: any, resolution: string}

BEGIN
    // Compare state versions
    clientVersion ← clientState.version
    serverVersion ← serverState.version

    IF clientVersion = serverVersion THEN
        // States are in sync
        RETURN {
            state: clientState,
            conflicts: [],
            synchronized: true
        }
    END IF

    // Detect conflicts
    conflicts ← []
    reconciledState ← DeepClone(serverState) // Server is source of truth
    clientActions ← []

    // Check portfolio differences
    IF clientState.portfolio.version < serverState.portfolio.version THEN
        // Server has newer portfolio data - accept it
        reconciledState.portfolio ← serverState.portfolio
    ELSE IF clientState.portfolio.version > serverState.portfolio.version THEN
        // Client has changes not yet on server - queue for retry
        conflicts.append({
            path: "portfolio",
            clientValue: clientState.portfolio,
            serverValue: serverState.portfolio,
            resolution: "client_ahead"
        })
    END IF

    // Check pending trades
    clientPendingTrades ← FilterOptimisticItems(clientState.pendingTrades)
    serverPendingTrades ← serverState.pendingTrades

    FOR EACH clientTrade IN clientPendingTrades DO
        serverTrade ← FindById(serverPendingTrades, clientTrade.id)

        IF serverTrade IS null THEN
            // Trade completed or failed on server
            IF clientTrade.isOptimistic THEN
                // Remove optimistic trade
                conflicts.append({
                    path: "pendingTrades." + clientTrade.id,
                    clientValue: clientTrade,
                    serverValue: null,
                    resolution: "server_completed"
                })
            ELSE
                // Queue for resubmission
                clientActions.append({
                    action: "resubmit_trade",
                    trade: clientTrade
                })
            END IF
        ELSE IF clientTrade.status != serverTrade.status THEN
            // Status changed on server
            conflicts.append({
                path: "pendingTrades." + clientTrade.id + ".status",
                clientValue: clientTrade.status,
                serverValue: serverTrade.status,
                resolution: "use_server"
            })
        END IF
    END FOR

    // Merge pending trades (server first, then client-only optimistic)
    reconciledState.pendingTrades ← serverPendingTrades
    FOR EACH clientTrade IN clientPendingTrades DO
        IF clientTrade.isOptimistic AND NOT ExistsInServer(clientTrade, serverPendingTrades) THEN
            reconciledState.pendingTrades.append(clientTrade)
        END IF
    END FOR

    // Check oversight requests
    SyncArrayField(clientState.oversightRequests, serverState.oversightRequests,
                   reconciledState, conflicts, "oversightRequests")

    // Update version and timestamp
    reconciledState.version ← serverVersion
    reconciledState.lastSync ← CurrentTimestamp()
    reconciledState.clientVersion ← clientVersion

    // Log synchronization
    LogSyncEvent({
        clientVersion: clientVersion,
        serverVersion: serverVersion,
        conflictCount: conflicts.length,
        actionsQueued: clientActions.length
    })

    RETURN {
        state: reconciledState,
        conflicts: conflicts,
        pendingActions: clientActions,
        synchronized: conflicts.isEmpty() AND clientActions.isEmpty()
    }
END

SUBROUTINE: SyncArrayField
INPUT: clientArray, serverArray, reconciledState, conflicts, fieldName
OUTPUT: void

BEGIN
    // Create ID maps for O(1) lookup
    clientMap ← CreateIdMap(clientArray)
    serverMap ← CreateIdMap(serverArray)

    // Start with server items
    merged ← serverArray.clone()

    // Check for client-only items
    FOR EACH clientItem IN clientArray DO
        IF NOT serverMap.has(clientItem.id) THEN
            IF clientItem.isOptimistic THEN
                // Keep optimistic item
                merged.append(clientItem)
            ELSE
                // Item missing on server - potential conflict
                conflicts.append({
                    path: fieldName + "." + clientItem.id,
                    clientValue: clientItem,
                    serverValue: null,
                    resolution: "client_only"
                })
            END IF
        END IF
    END FOR

    // Sort by timestamp
    merged.sortBy(item => item.timestamp, descending: true)

    // Update reconciled state
    reconciledState[fieldName] ← merged
END
```

**Time Complexity**: O(n + m) where n = client items, m = server items
**Space Complexity**: O(n + m) for maps and merged arrays

---

## 6. Rate Limiting (Client-Side)

### 6.1 Adaptive Rate Limiter with Caching

```
ALGORITHM: RateLimitedFetch
INPUT: key (string), fetcher (function), config (RateLimitConfig)
OUTPUT: Promise<FetchResult>

DATA STRUCTURES:
    RateLimitConfig: {
        maxRequests: integer,
        windowMs: integer,
        cacheTtlMs: integer,
        strategy: "sliding_window" | "token_bucket" | "fixed_window"
    }

    RateLimitState: {
        requestTimestamps: array,
        tokens: integer,
        lastRefill: long,
        nextAvailable: long
    }

CONSTANTS:
    DEFAULT_CONFIG = {
        maxRequests: 10,
        windowMs: 1000,
        cacheTtlMs: 5000,
        strategy: "sliding_window"
    }

BEGIN
    // Merge with default config
    config ← MergeConfigs(DEFAULT_CONFIG, config)

    // Check cache first
    cached ← FetchCache.get(key)
    IF cached EXISTS AND NOT cached.isExpired() THEN
        Log("Cache hit for key: " + key)
        RETURN {
            data: cached.data,
            source: "cache",
            timestamp: cached.timestamp
        }
    END IF

    // Get rate limit state for this key
    state ← RateLimitStates.get(key)
    IF state IS null THEN
        state ← InitializeRateLimitState(config)
        RateLimitStates.set(key, state)
    END IF

    // Check rate limit based on strategy
    IF config.strategy = "sliding_window" THEN
        allowed ← CheckSlidingWindow(state, config)
    ELSE IF config.strategy = "token_bucket" THEN
        allowed ← CheckTokenBucket(state, config)
    ELSE IF config.strategy = "fixed_window" THEN
        allowed ← CheckFixedWindow(state, config)
    END IF

    IF NOT allowed THEN
        // Rate limit exceeded
        waitTime ← CalculateWaitTime(state, config)

        Log("Rate limit exceeded for key: " + key + ", wait: " + waitTime + "ms")

        // Return cached data if available (even if expired)
        IF cached EXISTS THEN
            RETURN {
                data: cached.data,
                source: "stale_cache",
                timestamp: cached.timestamp,
                rateLimited: true,
                retryAfter: waitTime
            }
        END IF

        // Queue request for later
        QueueDelayedRequest(key, fetcher, waitTime)

        RETURN {
            data: null,
            source: "rate_limited",
            rateLimited: true,
            retryAfter: waitTime
        }
    END IF

    // Execute fetch
    startTime ← CurrentTimestamp()

    TRY
        data ← Await fetcher()
        duration ← CurrentTimestamp() - startTime

        // Update rate limit state
        RecordRequest(state, config)

        // Cache result
        FetchCache.set(key, {
            data: data,
            timestamp: CurrentTimestamp()
        }, config.cacheTtlMs)

        // Log metrics
        LogMetric("fetch", {
            key: key,
            duration: duration,
            source: "server",
            cached: false
        })

        RETURN {
            data: data,
            source: "server",
            timestamp: CurrentTimestamp(),
            duration: duration
        }

    CATCH error
        Log("Fetch error for key: " + key + ", error: " + error.message)

        // Return stale cache on error
        IF cached EXISTS THEN
            RETURN {
                data: cached.data,
                source: "stale_cache_error",
                timestamp: cached.timestamp,
                error: error
            }
        END IF

        THROW error
    END TRY
END

SUBROUTINE: CheckSlidingWindow
INPUT: state, config
OUTPUT: boolean

BEGIN
    currentTime ← CurrentTimestamp()
    windowStart ← currentTime - config.windowMs

    // Remove timestamps outside window
    state.requestTimestamps ← state.requestTimestamps.filter(
        timestamp => timestamp > windowStart
    )

    // Check if under limit
    IF state.requestTimestamps.length < config.maxRequests THEN
        RETURN true
    END IF

    // Calculate when next request will be available
    oldestInWindow ← state.requestTimestamps[0]
    state.nextAvailable ← oldestInWindow + config.windowMs

    RETURN false
END

SUBROUTINE: CheckTokenBucket
INPUT: state, config
OUTPUT: boolean

BEGIN
    currentTime ← CurrentTimestamp()

    // Calculate tokens to add based on time elapsed
    elapsed ← currentTime - state.lastRefill
    refillRate ← config.maxRequests / (config.windowMs / 1000) // requests per second
    tokensToAdd ← (elapsed / 1000) * refillRate

    // Refill bucket
    state.tokens ← MIN(state.tokens + tokensToAdd, config.maxRequests)
    state.lastRefill ← currentTime

    // Check if token available
    IF state.tokens >= 1 THEN
        RETURN true
    END IF

    // Calculate when next token will be available
    timeToNextToken ← (1 - state.tokens) / refillRate * 1000
    state.nextAvailable ← currentTime + timeToNextToken

    RETURN false
END

SUBROUTINE: CheckFixedWindow
INPUT: state, config
OUTPUT: boolean

BEGIN
    currentTime ← CurrentTimestamp()
    windowStart ← Floor(currentTime / config.windowMs) * config.windowMs

    // Reset if new window
    IF state.windowStart != windowStart THEN
        state.windowStart ← windowStart
        state.requestCount ← 0
    END IF

    // Check if under limit
    IF state.requestCount < config.maxRequests THEN
        RETURN true
    END IF

    // Next window starts at
    state.nextAvailable ← windowStart + config.windowMs

    RETURN false
END

SUBROUTINE: RecordRequest
INPUT: state, config
OUTPUT: void

BEGIN
    currentTime ← CurrentTimestamp()

    IF config.strategy = "sliding_window" THEN
        state.requestTimestamps.append(currentTime)
    ELSE IF config.strategy = "token_bucket" THEN
        state.tokens ← state.tokens - 1
    ELSE IF config.strategy = "fixed_window" THEN
        state.requestCount ← state.requestCount + 1
    END IF
END

SUBROUTINE: CalculateWaitTime
INPUT: state, config
OUTPUT: integer (milliseconds)

BEGIN
    IF state.nextAvailable EXISTS THEN
        currentTime ← CurrentTimestamp()
        waitTime ← MAX(0, state.nextAvailable - currentTime)
        RETURN waitTime
    END IF

    // Fallback to window duration
    RETURN config.windowMs
END
```

**Time Complexity**: O(n) for sliding window cleanup, O(1) for other strategies
**Space Complexity**: O(n) for sliding window, O(1) for other strategies

---

## 7. Subscription Management

### 7.1 Subscription Registry and Broadcasting

```
ALGORITHM: SubscriptionManager
OUTPUT: SubscriptionManager instance

CLASS SubscriptionManager:
    PRIVATE subscriptions: Map<channel, Set<Subscription>>
    PRIVATE activeChannels: Set<channel>
    PRIVATE errorHandlers: Map<channel, ErrorHandler>
    PRIVATE metrics: Map<channel, ChannelMetrics>

    CONSTRUCTOR:
        subscriptions ← MAP()
        activeChannels ← SET()
        errorHandlers ← MAP()
        metrics ← MAP()
    END CONSTRUCTOR

    METHOD: subscribe(channel, callback, options)
    INPUT: channel (string), callback (function), options (object)
    OUTPUT: UnsubscribeFunction

    BEGIN
        // Validate inputs
        IF channel IS empty OR callback IS null THEN
            THROW Error("Invalid subscription parameters")
        END IF

        // Create subscription object
        subscription ← {
            id: GenerateUUID(),
            channel: channel,
            callback: callback,
            createdAt: CurrentTimestamp(),
            options: options OR {},
            errorCount: 0,
            lastError: null
        }

        // Add to subscriptions map
        IF NOT this.subscriptions.has(channel) THEN
            this.subscriptions.set(channel, SET())
        END IF

        subscribers ← this.subscriptions.get(channel)
        subscribers.add(subscription)

        // Activate channel if first subscriber
        IF subscribers.size() = 1 THEN
            this.activateChannel(channel)
        END IF

        // Initialize metrics
        IF NOT this.metrics.has(channel) THEN
            this.metrics.set(channel, {
                subscriberCount: 0,
                messageCount: 0,
                errorCount: 0,
                lastMessageAt: null
            })
        END IF

        channelMetrics ← this.metrics.get(channel)
        channelMetrics.subscriberCount ← subscribers.size()

        Log("Subscribed to channel: " + channel + ", subscribers: " + subscribers.size())

        // Return unsubscribe function
        RETURN FUNCTION() BEGIN
            this.unsubscribe(channel, subscription.id)
        END
    END METHOD

    METHOD: unsubscribe(channel, subscriptionId)
    INPUT: channel (string), subscriptionId (string)
    OUTPUT: boolean

    BEGIN
        subscribers ← this.subscriptions.get(channel)

        IF subscribers IS null THEN
            RETURN false
        END IF

        // Find and remove subscription
        subscription ← subscribers.findById(subscriptionId)
        IF subscription IS null THEN
            RETURN false
        END IF

        subscribers.delete(subscription)

        // Deactivate channel if no subscribers
        IF subscribers.isEmpty() THEN
            this.deactivateChannel(channel)
            this.subscriptions.delete(channel)
            this.metrics.delete(channel)
        ELSE
            // Update metrics
            channelMetrics ← this.metrics.get(channel)
            channelMetrics.subscriberCount ← subscribers.size()
        END IF

        Log("Unsubscribed from channel: " + channel + ", remaining: " + subscribers.size())

        RETURN true
    END METHOD

    METHOD: broadcast(channel, data)
    INPUT: channel (string), data (object)
    OUTPUT: BroadcastResult

    BEGIN
        subscribers ← this.subscriptions.get(channel)

        IF subscribers IS null OR subscribers.isEmpty() THEN
            Log("No subscribers for channel: " + channel)
            RETURN {delivered: 0, failed: 0}
        END IF

        deliveredCount ← 0
        failedCount ← 0
        failedSubscriptions ← []

        // Broadcast to all subscribers
        FOR EACH subscription IN subscribers DO
            TRY
                // Call subscriber callback
                subscription.callback(data)
                deliveredCount ← deliveredCount + 1

                // Reset error count on success
                subscription.errorCount ← 0

            CATCH error
                failedCount ← failedCount + 1
                subscription.errorCount ← subscription.errorCount + 1
                subscription.lastError ← error

                Log("Error in subscription callback: " + error.message)

                // Remove subscription if too many errors
                IF subscription.errorCount >= (subscription.options.maxErrors OR 5) THEN
                    Log("Removing subscription after " + subscription.errorCount + " errors")
                    failedSubscriptions.append(subscription.id)
                END IF

                // Call error handler if registered
                errorHandler ← this.errorHandlers.get(channel)
                IF errorHandler EXISTS THEN
                    errorHandler(error, subscription, data)
                END IF
            END TRY
        END FOR

        // Remove failed subscriptions
        FOR EACH subscriptionId IN failedSubscriptions DO
            this.unsubscribe(channel, subscriptionId)
        END FOR

        // Update metrics
        channelMetrics ← this.metrics.get(channel)
        IF channelMetrics EXISTS THEN
            channelMetrics.messageCount ← channelMetrics.messageCount + 1
            channelMetrics.lastMessageAt ← CurrentTimestamp()
            channelMetrics.errorCount ← channelMetrics.errorCount + failedCount
        END IF

        RETURN {
            delivered: deliveredCount,
            failed: failedCount,
            channel: channel,
            timestamp: CurrentTimestamp()
        }
    END METHOD

    METHOD: activateChannel(channel)
    INPUT: channel (string)
    OUTPUT: void

    BEGIN
        IF this.activeChannels.has(channel) THEN
            RETURN // Already active
        END IF

        Log("Activating channel: " + channel)

        // Send subscription request to server
        WebSocketConnection.send({
            type: "subscribe",
            channel: channel,
            timestamp: CurrentTimestamp()
        })

        this.activeChannels.add(channel)

        // Emit activation event
        EmitEvent("channel:activated", {channel: channel})
    END METHOD

    METHOD: deactivateChannel(channel)
    INPUT: channel (string)
    OUTPUT: void

    BEGIN
        IF NOT this.activeChannels.has(channel) THEN
            RETURN // Not active
        END IF

        Log("Deactivating channel: " + channel)

        // Send unsubscription request to server
        WebSocketConnection.send({
            type: "unsubscribe",
            channel: channel,
            timestamp: CurrentTimestamp()
        })

        this.activeChannels.delete(channel)

        // Emit deactivation event
        EmitEvent("channel:deactivated", {channel: channel})
    END METHOD

    METHOD: getMetrics(channel)
    INPUT: channel (string, optional)
    OUTPUT: ChannelMetrics or Map<channel, ChannelMetrics>

    BEGIN
        IF channel EXISTS THEN
            RETURN this.metrics.get(channel)
        ELSE
            RETURN this.metrics
        END IF
    END METHOD

    METHOD: registerErrorHandler(channel, handler)
    INPUT: channel (string), handler (function)
    OUTPUT: void

    BEGIN
        this.errorHandlers.set(channel, handler)
    END METHOD

    METHOD: clearChannel(channel)
    INPUT: channel (string)
    OUTPUT: integer (removed count)

    BEGIN
        subscribers ← this.subscriptions.get(channel)

        IF subscribers IS null THEN
            RETURN 0
        END IF

        count ← subscribers.size()

        this.deactivateChannel(channel)
        this.subscriptions.delete(channel)
        this.metrics.delete(channel)

        Log("Cleared channel: " + channel + ", removed " + count + " subscribers")

        RETURN count
    END METHOD

END CLASS
```

**Time Complexity**:
- Subscribe: O(1)
- Unsubscribe: O(1)
- Broadcast: O(n) where n = subscriber count
- Activate/Deactivate: O(1)

**Space Complexity**: O(n * m) where n = channels, m = avg subscribers per channel

---

## 8. Performance Optimizations

### 8.1 Batching and Debouncing

```
ALGORITHM: BatchUpdates
INPUT: updates (array), batchWindow (integer)
OUTPUT: ProcessedBatch

CONSTANTS:
    MAX_BATCH_SIZE = 100
    DEFAULT_BATCH_WINDOW_MS = 100

BEGIN
    batchQueue ← QUEUE()
    batchTimer ← null
    processing ← false

    FUNCTION addToBatch(update):
        batchQueue.enqueue(update)

        // Cancel existing timer
        IF batchTimer EXISTS THEN
            ClearTimeout(batchTimer)
        END IF

        // Process immediately if batch is full
        IF batchQueue.size() >= MAX_BATCH_SIZE THEN
            processBatch()
        ELSE
            // Schedule batch processing
            batchTimer ← SetTimeout(batchWindow OR DEFAULT_BATCH_WINDOW_MS, FUNCTION()
                processBatch()
            END FUNCTION)
        END IF
    END FUNCTION

    FUNCTION processBatch():
        IF processing OR batchQueue.isEmpty() THEN
            RETURN
        END IF

        processing ← true
        batchTimer ← null

        // Collect all queued updates
        batch ← []
        WHILE NOT batchQueue.isEmpty() AND batch.length < MAX_BATCH_SIZE DO
            batch.append(batchQueue.dequeue())
        END WHILE

        // Group by type for efficient processing
        grouped ← GroupByType(batch)

        // Process each group
        FOR EACH (type, items) IN grouped DO
            ProcessUpdateGroup(type, items)
        END FOR

        processing ← false

        // Process remaining items if queue is not empty
        IF NOT batchQueue.isEmpty() THEN
            ScheduleImmediate(processBatch)
        END IF
    END FUNCTION

    RETURN {
        addToBatch: addToBatch,
        processBatch: processBatch
    }
END

SUBROUTINE: GroupByType
INPUT: updates (array)
OUTPUT: Map<type, array>

BEGIN
    grouped ← MAP()

    FOR EACH update IN updates DO
        type ← update.type

        IF NOT grouped.has(type) THEN
            grouped.set(type, [])
        END IF

        grouped.get(type).append(update)
    END FOR

    RETURN grouped
END
```

### 8.2 Virtual Scrolling for Large Lists

```
ALGORITHM: VirtualScrollManager
INPUT: items (array), viewportHeight (integer), itemHeight (integer)
OUTPUT: VirtualScrollState

BEGIN
    totalItems ← items.length
    itemsPerPage ← Ceiling(viewportHeight / itemHeight)
    bufferSize ← itemsPerPage // Pre-render buffer

    currentScrollTop ← 0
    visibleRange ← {start: 0, end: itemsPerPage}

    FUNCTION updateVisibleRange(scrollTop):
        currentScrollTop ← scrollTop

        // Calculate which items should be visible
        startIndex ← Floor(scrollTop / itemHeight)
        endIndex ← MIN(startIndex + itemsPerPage + bufferSize, totalItems)

        // Add buffer above
        startIndex ← MAX(0, startIndex - bufferSize)

        visibleRange ← {start: startIndex, end: endIndex}

        RETURN {
            visibleItems: items.slice(startIndex, endIndex),
            offsetY: startIndex * itemHeight,
            totalHeight: totalItems * itemHeight,
            visibleRange: visibleRange
        }
    END FUNCTION

    RETURN {
        updateVisibleRange: updateVisibleRange,
        visibleRange: visibleRange
    }
END
```

**Time Complexity**: O(1) for scroll updates
**Space Complexity**: O(visible items) instead of O(total items)

---

## 9. Performance Metrics

### Algorithm Complexity Summary

| Algorithm | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| WebSocket Init | O(n) | O(n) | n = channels |
| Broadcast Event | O(m) | O(m) | m = subscribers |
| Portfolio Update | O(p + u) | O(p) | p = positions, u = price updates |
| Optimistic Update | O(1) | O(1) | Single transaction |
| State Sync | O(n + m) | O(n + m) | n = client items, m = server items |
| Rate Limiter (Sliding) | O(r) | O(r) | r = requests in window |
| Rate Limiter (Token) | O(1) | O(1) | Constant time |
| Subscribe/Unsubscribe | O(1) | O(1) | Hash map operations |
| Broadcast to Subscribers | O(s) | O(1) | s = subscriber count |
| Batch Processing | O(b) | O(b) | b = batch size |
| Virtual Scroll | O(1) | O(v) | v = visible items |

### Performance Targets

- **WebSocket Latency**: < 50ms for message delivery
- **Portfolio Updates**: < 100ms debounced batch processing
- **Optimistic UI**: < 16ms for state application (60fps)
- **State Sync**: < 200ms for full reconciliation
- **Rate Limit Check**: < 1ms per request
- **Broadcast**: < 10ms per subscriber
- **Virtual Scroll**: < 16ms per scroll event

---

## 10. Error Handling and Edge Cases

### Common Edge Cases Handled

1. **WebSocket Disconnection**: Exponential backoff reconnection
2. **Partial Fills**: Trade quantity reconciliation
3. **Stale Price Data**: Cache fallback with TTL
4. **Rate Limit Bursts**: Token bucket smoothing
5. **Network Partitions**: Offline queue with retry
6. **Concurrent Updates**: Server state as source of truth
7. **Memory Leaks**: Subscription cleanup on unmount
8. **Backpressure**: Queue size limits with drop policy

---

## Conclusion

These algorithms provide a robust foundation for real-time data synchronization in the Trading Demo. Key design principles include:

- **Fault Tolerance**: Graceful degradation and retry mechanisms
- **Performance**: Batching, caching, and debouncing strategies
- **Consistency**: Server state as source of truth with optimistic updates
- **Scalability**: Efficient data structures and complexity analysis
- **User Experience**: Optimistic UI and real-time feedback

All algorithms are designed to handle production-scale loads while maintaining sub-second response times for critical operations.
