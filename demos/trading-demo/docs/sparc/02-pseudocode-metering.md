# SPARC Pseudocode: Trading Demo Metering Integration

## Overview
This document provides detailed algorithmic designs for integrating the Trading Demo with creto-metering for usage tracking, quota enforcement, and cost management.

---

## 1. Event Ingestion

### 1.1 Record Metering Event

```
ALGORITHM: RecordMeteringEvent
INPUT: agentId (string), eventType (enum), metadata (object), quantity (integer)
OUTPUT: success (boolean) or error

CONSTANTS:
    MAX_RETRIES = 3
    RETRY_DELAY_MS = 1000
    CIRCUIT_BREAKER_THRESHOLD = 5
    CIRCUIT_BREAKER_TIMEOUT_MS = 30000

DATA STRUCTURES:
    BillableEvent:
        - transaction_id: UUID
        - agent_id: string
        - event_type: EventType (AGENT_API_CALL, TRADE_EXECUTION, MARKET_DATA_REQUEST)
        - timestamp: ISO8601 timestamp
        - quantity: integer (default: 1)
        - properties: map<string, any>
        - idempotency_key: string

BEGIN
    // Generate unique transaction ID for idempotency
    transactionId ← GenerateUUID()
    idempotencyKey ← HASH(agentId + eventType + timestamp + metadata)

    // Build billable event payload
    event ← BillableEvent{
        transaction_id: transactionId,
        agent_id: agentId,
        event_type: eventType,
        timestamp: GetCurrentTimestamp(),
        quantity: quantity OR 1,
        properties: metadata,
        idempotency_key: idempotencyKey
    }

    // Check circuit breaker state
    IF CircuitBreaker.isOpen() THEN
        Log.warn("Circuit breaker open, queueing event for retry")
        EventQueue.enqueue(event)
        RETURN {success: false, error: "Service temporarily unavailable"}
    END IF

    // Attempt to send event with retry logic
    retryCount ← 0
    WHILE retryCount < MAX_RETRIES DO
        TRY
            // Send to creto-metering gRPC API
            response ← MeteringClient.ingestEvent(event)

            IF response.status == SUCCESS THEN
                // Update local metrics cache
                MetricsCache.increment(agentId, eventType, quantity)

                // Reset circuit breaker on success
                CircuitBreaker.recordSuccess()

                Log.info("Event recorded", {
                    transaction_id: transactionId,
                    agent_id: agentId,
                    event_type: eventType
                })

                RETURN {success: true, transaction_id: transactionId}
            END IF

        CATCH GrpcException AS e
            retryCount ← retryCount + 1

            // Record failure for circuit breaker
            CircuitBreaker.recordFailure()

            IF retryCount < MAX_RETRIES THEN
                // Exponential backoff
                delay ← RETRY_DELAY_MS * (2 ^ (retryCount - 1))
                Log.warn("Retry attempt", {attempt: retryCount, delay: delay})
                Sleep(delay)
            ELSE
                // Max retries exceeded
                Log.error("Failed to record event", {
                    error: e.message,
                    transaction_id: transactionId
                })

                // Queue for async retry
                EventQueue.enqueue(event)

                RETURN {success: false, error: e.message, queued: true}
            END IF
        END TRY
    END WHILE

END
```

### 1.2 Async Event Queue Processor

```
ALGORITHM: ProcessEventQueue
INPUT: none
OUTPUT: processed count (integer)

CONSTANTS:
    BATCH_SIZE = 100
    PROCESSING_INTERVAL_MS = 5000

BEGIN
    processedCount ← 0

    WHILE true DO
        // Wait for processing interval
        Sleep(PROCESSING_INTERVAL_MS)

        // Skip if circuit breaker is open
        IF CircuitBreaker.isOpen() THEN
            CONTINUE
        END IF

        // Fetch batch of queued events
        events ← EventQueue.dequeueBatch(BATCH_SIZE)

        IF events.isEmpty() THEN
            CONTINUE
        END IF

        // Process batch
        FOR EACH event IN events DO
            result ← RecordMeteringEvent(
                event.agent_id,
                event.event_type,
                event.properties,
                event.quantity
            )

            IF result.success THEN
                processedCount ← processedCount + 1
            ELSE
                // Re-queue with exponential backoff metadata
                event.retry_count ← event.retry_count + 1
                event.next_retry ← Now() + (1000 * 2 ^ event.retry_count)

                IF event.retry_count < 10 THEN
                    EventQueue.enqueue(event)
                ELSE
                    // Move to dead letter queue after 10 retries
                    DeadLetterQueue.enqueue(event)
                    Log.error("Event moved to DLQ", {transaction_id: event.transaction_id})
                END IF
            END IF
        END FOR
    END WHILE

    RETURN processedCount
END
```

---

## 2. Quota Management

### 2.1 Initialize Agent Quota

```
ALGORITHM: InitializeQuota
INPUT: agentId (string), monthlyBudget (decimal), pricingModel (object)
OUTPUT: quota configuration (object) or error

CONSTANTS:
    DEFAULT_API_RATE = 0.001  // $0.001 per call
    DEFAULT_TRADE_COST = 0.01  // $0.01 per trade
    ALERT_THRESHOLDS = [0.5, 0.8, 0.95]  // 50%, 80%, 95%

DATA STRUCTURES:
    QuotaConfiguration:
        - agent_id: string
        - monthly_budget: decimal
        - limits: map<EventType, integer>
        - alert_thresholds: array<decimal>
        - reset_period: string (MONTHLY)
        - created_at: timestamp

BEGIN
    // Calculate quota limits by event type
    limits ← MAP()

    // API Calls quota (60% of budget)
    apiCallBudget ← monthlyBudget * 0.6
    apiCallLimit ← FLOOR(apiCallBudget / pricingModel.api_call_cost)
    limits.set(AGENT_API_CALL, apiCallLimit)

    // Trade Executions quota (30% of budget)
    tradeBudget ← monthlyBudget * 0.3
    tradeLimit ← FLOOR(tradeBudget / pricingModel.trade_execution_cost)
    limits.set(TRADE_EXECUTION, tradeLimit)

    // Market Data Requests (10% of budget, or unlimited if free tier)
    IF pricingModel.market_data_cost > 0 THEN
        marketDataBudget ← monthlyBudget * 0.1
        marketDataLimit ← FLOOR(marketDataBudget / pricingModel.market_data_cost)
        limits.set(MARKET_DATA_REQUEST, marketDataLimit)
    ELSE
        limits.set(MARKET_DATA_REQUEST, UNLIMITED)
    END IF

    // Build quota configuration
    quotaConfig ← QuotaConfiguration{
        agent_id: agentId,
        monthly_budget: monthlyBudget,
        limits: limits,
        alert_thresholds: ALERT_THRESHOLDS,
        reset_period: "MONTHLY",
        created_at: GetCurrentTimestamp()
    }

    TRY
        // Register quota with creto-metering
        response ← MeteringClient.registerQuota(quotaConfig)

        IF response.status == SUCCESS THEN
            // Cache quota configuration locally
            QuotaCache.set(agentId, quotaConfig)

            // Schedule alert threshold checks
            FOR EACH threshold IN ALERT_THRESHOLDS DO
                AlertScheduler.schedule(agentId, threshold)
            END FOR

            Log.info("Quota initialized", {
                agent_id: agentId,
                budget: monthlyBudget,
                limits: limits
            })

            RETURN quotaConfig
        END IF

    CATCH Exception AS e
        Log.error("Failed to initialize quota", {
            agent_id: agentId,
            error: e.message
        })

        RETURN {error: e.message}
    END TRY
END
```

### 2.2 Check Quota Before Event

```
ALGORITHM: CheckQuota
INPUT: agentId (string), eventType (enum), quantity (integer)
OUTPUT: quota check result (object)

DATA STRUCTURES:
    QuotaCheckResult:
        - allowed: boolean
        - remaining: integer
        - limit: integer
        - usage_percentage: decimal
        - reset_at: timestamp
        - next_threshold: decimal (nullable)

BEGIN
    // Try to get cached quota state
    cachedQuota ← QuotaCache.get(agentId)

    IF cachedQuota IS NULL OR cachedQuota.isExpired() THEN
        // Fetch fresh quota data from creto-metering
        TRY
            quotaData ← MeteringClient.getQuota(agentId)
            QuotaCache.set(agentId, quotaData, TTL=60)  // Cache for 60 seconds
        CATCH Exception AS e
            Log.error("Failed to fetch quota", {agent_id: agentId, error: e.message})
            // Fail open: allow request but log warning
            Log.warn("Allowing request due to quota fetch failure")
            RETURN {allowed: true, error: "quota_check_failed"}
        END TRY
    ELSE
        quotaData ← cachedQuota
    END IF

    // Get limit and current usage for event type
    limit ← quotaData.limits.get(eventType)
    currentUsage ← quotaData.usage.get(eventType) OR 0

    // Check for unlimited quota
    IF limit == UNLIMITED THEN
        RETURN QuotaCheckResult{
            allowed: true,
            remaining: UNLIMITED,
            limit: UNLIMITED,
            usage_percentage: 0,
            reset_at: quotaData.reset_at
        }
    END IF

    // Calculate remaining quota
    remaining ← limit - currentUsage
    usagePercentage ← (currentUsage / limit) * 100

    // Determine if request is allowed
    allowed ← (currentUsage + quantity) <= limit

    // Find next alert threshold
    nextThreshold ← NULL
    FOR EACH threshold IN quotaData.alert_thresholds DO
        IF usagePercentage < (threshold * 100) THEN
            nextThreshold ← threshold
            BREAK
        END IF
    END FOR

    // Build result
    result ← QuotaCheckResult{
        allowed: allowed,
        remaining: remaining,
        limit: limit,
        usage_percentage: usagePercentage,
        reset_at: quotaData.reset_at,
        next_threshold: nextThreshold
    }

    // Log warning if quota is low
    IF NOT allowed THEN
        Log.warn("Quota exceeded", {
            agent_id: agentId,
            event_type: eventType,
            usage: currentUsage,
            limit: limit
        })
    ELSE IF usagePercentage > 90 THEN
        Log.warn("Quota nearly exhausted", {
            agent_id: agentId,
            event_type: eventType,
            usage_percentage: usagePercentage
        })
    END IF

    RETURN result
END
```

### 2.3 Enforce Quota Inline

```
ALGORITHM: EnforceQuota
INPUT: agentId (string), eventType (enum), quantity (integer)
OUTPUT: enforcement result (object)

DATA STRUCTURES:
    EnforcementResult:
        - allowed: boolean
        - reason: string (nullable)
        - quota_info: QuotaCheckResult

BEGIN
    // Check quota before allowing operation
    quotaCheck ← CheckQuota(agentId, eventType, quantity)

    IF NOT quotaCheck.allowed THEN
        // Quota exceeded - block operation
        RETURN EnforcementResult{
            allowed: false,
            reason: "quota_exceeded",
            quota_info: quotaCheck
        }
    END IF

    // Check if approaching threshold for proactive warning
    IF quotaCheck.usage_percentage >= 80 THEN
        // Trigger proactive alert (non-blocking)
        AlertManager.triggerAlert(agentId, "quota_warning", {
            event_type: eventType,
            usage_percentage: quotaCheck.usage_percentage,
            remaining: quotaCheck.remaining
        })
    END IF

    RETURN EnforcementResult{
        allowed: true,
        reason: NULL,
        quota_info: quotaCheck
    }
END
```

---

## 3. Usage Aggregation

### 3.1 Aggregate Usage by Period

```
ALGORITHM: AggregateUsage
INPUT: agentId (string), period (enum: HOURLY, DAILY, WEEKLY, MONTHLY)
OUTPUT: usage summary (object)

DATA STRUCTURES:
    UsageSummary:
        - agent_id: string
        - period: string
        - start_time: timestamp
        - end_time: timestamp
        - total_events: integer
        - breakdown: map<EventType, EventBreakdown>
        - total_cost: decimal

    EventBreakdown:
        - event_type: EventType
        - count: integer
        - cost: decimal
        - percentage: decimal

BEGIN
    // Calculate time range for period
    timeRange ← CalculateTimeRange(period)

    TRY
        // Query creto-metering aggregation API
        aggregationRequest ← {
            agent_id: agentId,
            start_time: timeRange.start,
            end_time: timeRange.end,
            group_by: ["event_type"],
            metrics: ["count", "sum"]
        }

        rawData ← MeteringClient.aggregateEvents(aggregationRequest)

        // Initialize summary
        summary ← UsageSummary{
            agent_id: agentId,
            period: period,
            start_time: timeRange.start,
            end_time: timeRange.end,
            total_events: 0,
            breakdown: MAP(),
            total_cost: 0
        }

        // Process each event type
        FOR EACH eventData IN rawData.groups DO
            eventType ← eventData.event_type
            count ← eventData.metrics.count

            // Calculate cost for this event type
            cost ← CalculateEventCost(eventType, count)

            // Add to breakdown
            summary.breakdown.set(eventType, EventBreakdown{
                event_type: eventType,
                count: count,
                cost: cost,
                percentage: 0  // Calculate after total
            })

            summary.total_events ← summary.total_events + count
            summary.total_cost ← summary.total_cost + cost
        END FOR

        // Calculate percentages
        FOR EACH eventType, breakdown IN summary.breakdown DO
            breakdown.percentage ← (breakdown.count / summary.total_events) * 100
        END FOR

        // Cache aggregated data
        UsageCache.set(agentId + period, summary, TTL=300)  // 5 minute cache

        RETURN summary

    CATCH Exception AS e
        Log.error("Failed to aggregate usage", {
            agent_id: agentId,
            period: period,
            error: e.message
        })

        RETURN {error: e.message}
    END TRY
END

SUBROUTINE: CalculateTimeRange
INPUT: period (enum)
OUTPUT: time range (object)

BEGIN
    now ← GetCurrentTimestamp()

    SWITCH period DO
        CASE HOURLY:
            start ← now.startOfHour()
            end ← now.endOfHour()
        CASE DAILY:
            start ← now.startOfDay()
            end ← now.endOfDay()
        CASE WEEKLY:
            start ← now.startOfWeek()
            end ← now.endOfWeek()
        CASE MONTHLY:
            start ← now.startOfMonth()
            end ← now.endOfMonth()
    END SWITCH

    RETURN {start: start, end: end}
END
```

### 3.2 Get Usage Trend Analysis

```
ALGORITHM: GetUsageTrend
INPUT: agentId (string), intervals (integer), intervalType (enum)
OUTPUT: trend data (object)

DATA STRUCTURES:
    TrendData:
        - agent_id: string
        - interval_type: string
        - intervals: array<IntervalData>
        - prediction: PredictionData
        - statistics: TrendStatistics

    IntervalData:
        - start_time: timestamp
        - end_time: timestamp
        - total_events: integer
        - total_cost: decimal
        - breakdown: map<EventType, integer>

    PredictionData:
        - budget_exhaustion_date: timestamp (nullable)
        - projected_monthly_cost: decimal
        - confidence: decimal

    TrendStatistics:
        - avg_daily_events: decimal
        - avg_daily_cost: decimal
        - peak_usage_time: timestamp
        - growth_rate: decimal

BEGIN
    // Fetch historical data for intervals
    historicalData ← []

    FOR i ← 0 TO intervals - 1 DO
        // Calculate interval time range
        intervalStart ← CalculateIntervalStart(intervalType, i)
        intervalEnd ← CalculateIntervalEnd(intervalType, i)

        // Get aggregated data for interval
        intervalUsage ← AggregateUsageForRange(agentId, intervalStart, intervalEnd)

        historicalData.append(IntervalData{
            start_time: intervalStart,
            end_time: intervalEnd,
            total_events: intervalUsage.total_events,
            total_cost: intervalUsage.total_cost,
            breakdown: intervalUsage.breakdown
        })
    END FOR

    // Calculate statistics
    totalEvents ← SUM(data.total_events FOR data IN historicalData)
    totalCost ← SUM(data.total_cost FOR data IN historicalData)
    avgDailyEvents ← totalEvents / intervals
    avgDailyCost ← totalCost / intervals

    // Find peak usage
    peakInterval ← MAX(historicalData, BY total_events)
    peakUsageTime ← peakInterval.start_time

    // Calculate growth rate (linear regression)
    growthRate ← CalculateGrowthRate(historicalData)

    statistics ← TrendStatistics{
        avg_daily_events: avgDailyEvents,
        avg_daily_cost: avgDailyCost,
        peak_usage_time: peakUsageTime,
        growth_rate: growthRate
    }

    // Predict budget exhaustion
    prediction ← PredictBudgetExhaustion(
        agentId,
        avgDailyCost,
        growthRate,
        historicalData
    )

    RETURN TrendData{
        agent_id: agentId,
        interval_type: intervalType,
        intervals: historicalData,
        prediction: prediction,
        statistics: statistics
    }
END

SUBROUTINE: PredictBudgetExhaustion
INPUT: agentId (string), avgDailyCost (decimal), growthRate (decimal), historicalData (array)
OUTPUT: prediction data (object)

BEGIN
    // Get quota configuration
    quota ← QuotaCache.get(agentId)
    monthlyBudget ← quota.monthly_budget

    // Get current month usage
    currentUsage ← AggregateUsage(agentId, MONTHLY)
    spentSoFar ← currentUsage.total_cost
    remaining ← monthlyBudget - spentSoFar

    // Calculate days in current month
    daysInMonth ← GetDaysInCurrentMonth()
    daysPassed ← GetDayOfMonth()
    daysRemaining ← daysInMonth - daysPassed

    // Project future daily cost with growth rate
    projectedDailyCost ← avgDailyCost * (1 + growthRate)

    // Estimate days until budget exhaustion
    IF projectedDailyCost > 0 THEN
        daysUntilExhaustion ← remaining / projectedDailyCost
    ELSE
        daysUntilExhaustion ← NULL  // No exhaustion expected
    END IF

    // Calculate budget exhaustion date
    IF daysUntilExhaustion != NULL AND daysUntilExhaustion > 0 THEN
        exhaustionDate ← GetCurrentDate() + daysUntilExhaustion
    ELSE
        exhaustionDate ← NULL
    END IF

    // Project end-of-month cost
    projectedMonthlyCost ← spentSoFar + (projectedDailyCost * daysRemaining)

    // Calculate prediction confidence based on data variance
    variance ← CalculateVariance(historicalData)
    confidence ← 1 / (1 + variance)  // Higher variance = lower confidence

    RETURN PredictionData{
        budget_exhaustion_date: exhaustionDate,
        projected_monthly_cost: projectedMonthlyCost,
        confidence: confidence
    }
END
```

---

## 4. Alert System

### 4.1 Check and Trigger Alert Thresholds

```
ALGORITHM: CheckAlertThresholds
INPUT: agentId (string), currentUsage (object)
OUTPUT: triggered alerts (array)

CONSTANTS:
    ALERT_DEBOUNCE_MS = 3600000  // 1 hour debounce

DATA STRUCTURES:
    Alert:
        - alert_id: UUID
        - agent_id: string
        - alert_type: AlertType (QUOTA_THRESHOLD, QUOTA_EXCEEDED, COST_SPIKE)
        - severity: Severity (INFO, WARNING, CRITICAL)
        - threshold: decimal
        - current_value: decimal
        - message: string
        - triggered_at: timestamp
        - acknowledged: boolean

BEGIN
    triggeredAlerts ← []

    // Get quota configuration
    quota ← QuotaCache.get(agentId)
    thresholds ← quota.alert_thresholds

    // Check each event type against thresholds
    FOR EACH eventType, usage IN currentUsage.breakdown DO
        limit ← quota.limits.get(eventType)

        IF limit == UNLIMITED THEN
            CONTINUE
        END IF

        usagePercentage ← (usage.count / limit) * 100

        // Check threshold crossings
        FOR EACH threshold IN thresholds DO
            thresholdPercentage ← threshold * 100

            IF usagePercentage >= thresholdPercentage THEN
                // Check if alert already fired recently (debounce)
                alertKey ← agentId + eventType + threshold
                lastAlertTime ← AlertCache.get(alertKey)

                IF lastAlertTime IS NULL OR
                   (Now() - lastAlertTime) > ALERT_DEBOUNCE_MS THEN

                    // Determine severity
                    severity ← DetermineSeverity(usagePercentage)

                    // Create alert
                    alert ← Alert{
                        alert_id: GenerateUUID(),
                        agent_id: agentId,
                        alert_type: QUOTA_THRESHOLD,
                        severity: severity,
                        threshold: threshold,
                        current_value: usagePercentage / 100,
                        message: "Usage threshold exceeded",
                        triggered_at: GetCurrentTimestamp(),
                        acknowledged: false
                    }

                    triggeredAlerts.append(alert)

                    // Update debounce cache
                    AlertCache.set(alertKey, Now())

                    // Notify via configured channels
                    NotifyAlert(alert)
                END IF
            END IF
        END FOR
    END FOR

    // Check for cost spikes
    costSpike ← DetectCostSpike(agentId, currentUsage)
    IF costSpike != NULL THEN
        triggeredAlerts.append(costSpike)
        NotifyAlert(costSpike)
    END IF

    RETURN triggeredAlerts
END

SUBROUTINE: DetermineSeverity
INPUT: usagePercentage (decimal)
OUTPUT: severity (enum)

BEGIN
    IF usagePercentage >= 95 THEN
        RETURN CRITICAL
    ELSE IF usagePercentage >= 80 THEN
        RETURN WARNING
    ELSE
        RETURN INFO
    END IF
END

SUBROUTINE: DetectCostSpike
INPUT: agentId (string), currentUsage (object)
OUTPUT: alert (object) or NULL

CONSTANTS:
    SPIKE_THRESHOLD = 2.0  // 2x average

BEGIN
    // Get historical average
    trendData ← GetUsageTrend(agentId, 7, DAILY)
    avgDailyCost ← trendData.statistics.avg_daily_cost

    // Get today's cost
    todayUsage ← AggregateUsage(agentId, DAILY)
    todayCost ← todayUsage.total_cost

    // Check for spike
    IF todayCost > (avgDailyCost * SPIKE_THRESHOLD) THEN
        RETURN Alert{
            alert_id: GenerateUUID(),
            agent_id: agentId,
            alert_type: COST_SPIKE,
            severity: WARNING,
            threshold: SPIKE_THRESHOLD,
            current_value: todayCost / avgDailyCost,
            message: "Unusual cost spike detected",
            triggered_at: GetCurrentTimestamp(),
            acknowledged: false
        }
    END IF

    RETURN NULL
END
```

### 4.2 Notify Alert via Channels

```
ALGORITHM: NotifyAlert
INPUT: alert (Alert object)
OUTPUT: notification results (array)

DATA STRUCTURES:
    NotificationChannel:
        - type: ChannelType (EMAIL, SLACK, WEBHOOK, UI)
        - enabled: boolean
        - config: map<string, any>

BEGIN
    // Get configured notification channels for agent
    channels ← NotificationConfig.getChannels(alert.agent_id)
    results ← []

    FOR EACH channel IN channels DO
        IF NOT channel.enabled THEN
            CONTINUE
        END IF

        TRY
            SWITCH channel.type DO
                CASE EMAIL:
                    result ← SendEmailAlert(alert, channel.config)

                CASE SLACK:
                    result ← SendSlackAlert(alert, channel.config)

                CASE WEBHOOK:
                    result ← SendWebhookAlert(alert, channel.config)

                CASE UI:
                    result ← BroadcastUIAlert(alert)
            END SWITCH

            results.append({
                channel: channel.type,
                success: true,
                timestamp: GetCurrentTimestamp()
            })

        CATCH Exception AS e
            Log.error("Failed to send alert", {
                alert_id: alert.alert_id,
                channel: channel.type,
                error: e.message
            })

            results.append({
                channel: channel.type,
                success: false,
                error: e.message
            })
        END TRY
    END FOR

    // Store alert in database
    AlertStore.save(alert)

    // Update real-time dashboard
    UpdateDashboard(alert)

    RETURN results
END

SUBROUTINE: BroadcastUIAlert
INPUT: alert (Alert object)
OUTPUT: success (boolean)

BEGIN
    // Format alert for UI
    uiPayload ← {
        type: "METERING_ALERT",
        severity: alert.severity,
        message: alert.message,
        details: {
            agent_id: alert.agent_id,
            alert_type: alert.alert_type,
            threshold: alert.threshold,
            current_value: alert.current_value,
            triggered_at: alert.triggered_at
        }
    }

    // Broadcast via WebSocket
    WebSocketServer.broadcast(alert.agent_id, uiPayload)

    // Update alert badge count
    AlertBadge.increment(alert.agent_id)

    RETURN true
END
```

---

## 5. Cost Calculation

### 5.1 Calculate Event Costs with Tiered Pricing

```
ALGORITHM: CalculateEventCost
INPUT: eventType (enum), quantity (integer)
OUTPUT: total cost (decimal)

DATA STRUCTURES:
    PricingTier:
        - min_quantity: integer
        - max_quantity: integer (nullable)
        - price_per_unit: decimal

    PricingModel:
        - event_type: EventType
        - tiers: array<PricingTier>
        - currency: string

CONSTANTS:
    PRICING_MODELS = {
        AGENT_API_CALL: [
            {min: 0, max: 10000, price: 0.001},
            {min: 10001, max: 100000, price: 0.0005},
            {min: 100001, max: NULL, price: 0.0001}
        ],
        TRADE_EXECUTION: [
            {min: 0, max: 1000, price: 0.01},
            {min: 1001, max: 10000, price: 0.007},
            {min: 10001, max: NULL, price: 0.005}
        ],
        MARKET_DATA_REQUEST: [
            {min: 0, max: NULL, price: 0}  // Free tier
        ]
    }

BEGIN
    pricingModel ← PRICING_MODELS.get(eventType)

    IF pricingModel IS NULL THEN
        Log.warn("Unknown event type for pricing", {event_type: eventType})
        RETURN 0
    END IF

    totalCost ← 0
    remainingQuantity ← quantity

    // Apply tiered pricing
    FOR EACH tier IN pricingModel DO
        IF remainingQuantity <= 0 THEN
            BREAK
        END IF

        // Calculate quantity in this tier
        tierQuantity ← CalculateTierQuantity(
            remainingQuantity,
            tier.min_quantity,
            tier.max_quantity
        )

        // Calculate cost for this tier
        tierCost ← tierQuantity * tier.price_per_unit
        totalCost ← totalCost + tierCost

        // Reduce remaining quantity
        remainingQuantity ← remainingQuantity - tierQuantity

        Log.debug("Tier calculation", {
            tier_min: tier.min_quantity,
            tier_max: tier.max_quantity,
            tier_quantity: tierQuantity,
            tier_price: tier.price_per_unit,
            tier_cost: tierCost
        })
    END FOR

    // Round to 4 decimal places
    totalCost ← ROUND(totalCost, 4)

    RETURN totalCost
END

SUBROUTINE: CalculateTierQuantity
INPUT: remainingQuantity (integer), tierMin (integer), tierMax (integer or NULL)
OUTPUT: quantity in tier (integer)

BEGIN
    // Determine the range for this tier
    IF tierMax IS NULL THEN
        // Unlimited tier - use all remaining
        RETURN remainingQuantity
    END IF

    tierRange ← tierMax - tierMin + 1

    IF remainingQuantity <= tierRange THEN
        RETURN remainingQuantity
    ELSE
        RETURN tierRange
    END IF
END
```

### 5.2 Calculate Detailed Cost Breakdown

```
ALGORITHM: CalculateCostBreakdown
INPUT: events (array of BillableEvent), pricingModels (object)
OUTPUT: cost breakdown (object)

DATA STRUCTURES:
    CostBreakdown:
        - total_cost: decimal
        - currency: string
        - breakdown_by_type: map<EventType, TypeBreakdown>
        - breakdown_by_tier: array<TierBreakdown>

    TypeBreakdown:
        - event_type: EventType
        - quantity: integer
        - cost: decimal
        - percentage: decimal

    TierBreakdown:
        - event_type: EventType
        - tier_name: string
        - quantity: integer
        - unit_price: decimal
        - total_cost: decimal

BEGIN
    // Group events by type
    eventsByType ← GroupBy(events, "event_type")

    // Initialize breakdown
    breakdown ← CostBreakdown{
        total_cost: 0,
        currency: "USD",
        breakdown_by_type: MAP(),
        breakdown_by_tier: []
    }

    // Calculate cost for each event type
    FOR EACH eventType, eventList IN eventsByType DO
        totalQuantity ← SUM(event.quantity FOR event IN eventList)

        // Calculate cost with tiered pricing
        cost ← CalculateEventCost(eventType, totalQuantity)

        breakdown.breakdown_by_type.set(eventType, TypeBreakdown{
            event_type: eventType,
            quantity: totalQuantity,
            cost: cost,
            percentage: 0  // Calculate after total
        })

        breakdown.total_cost ← breakdown.total_cost + cost

        // Get tier breakdown for this event type
        tierBreakdown ← GetTierBreakdown(eventType, totalQuantity)
        breakdown.breakdown_by_tier.append(tierBreakdown)
    END FOR

    // Calculate percentages
    FOR EACH eventType, typeBreakdown IN breakdown.breakdown_by_type DO
        IF breakdown.total_cost > 0 THEN
            typeBreakdown.percentage ← (typeBreakdown.cost / breakdown.total_cost) * 100
        ELSE
            typeBreakdown.percentage ← 0
        END IF
    END FOR

    RETURN breakdown
END

SUBROUTINE: GetTierBreakdown
INPUT: eventType (enum), totalQuantity (integer)
OUTPUT: tier breakdowns (array)

BEGIN
    pricingModel ← PRICING_MODELS.get(eventType)
    tierBreakdowns ← []
    remainingQuantity ← totalQuantity

    FOR EACH tier IN pricingModel DO
        IF remainingQuantity <= 0 THEN
            BREAK
        END IF

        tierQuantity ← CalculateTierQuantity(
            remainingQuantity,
            tier.min_quantity,
            tier.max_quantity
        )

        tierCost ← tierQuantity * tier.price_per_unit

        tierName ← FormatTierName(tier.min_quantity, tier.max_quantity)

        tierBreakdowns.append(TierBreakdown{
            event_type: eventType,
            tier_name: tierName,
            quantity: tierQuantity,
            unit_price: tier.price_per_unit,
            total_cost: tierCost
        })

        remainingQuantity ← remainingQuantity - tierQuantity
    END FOR

    RETURN tierBreakdowns
END

SUBROUTINE: FormatTierName
INPUT: min (integer), max (integer or NULL)
OUTPUT: tier name (string)

BEGIN
    IF max IS NULL THEN
        RETURN min + "+"
    ELSE
        RETURN min + "-" + max
    END IF
END
```

---

## 6. Integration Points

### 6.1 gRPC Client Configuration

```
ALGORITHM: InitializeMeteringClient
INPUT: config (object)
OUTPUT: client instance

DATA STRUCTURES:
    MeteringClientConfig:
        - host: string
        - port: integer
        - tls_enabled: boolean
        - api_key: string
        - timeout_ms: integer
        - retry_policy: RetryPolicy

BEGIN
    // Build gRPC channel
    endpoint ← config.host + ":" + config.port

    channelCredentials ← NULL
    IF config.tls_enabled THEN
        channelCredentials ← CreateTLSCredentials()
    ELSE
        channelCredentials ← CreateInsecureCredentials()
    END IF

    // Create channel with retry policy
    channel ← CreateChannel(
        endpoint,
        channelCredentials,
        config.retry_policy
    )

    // Initialize service stubs
    client ← MeteringClient{
        ingestion_service: IngestionServiceStub(channel),
        quota_service: QuotaServiceStub(channel),
        aggregation_service: AggregationServiceStub(channel),
        api_key: config.api_key,
        timeout: config.timeout_ms
    }

    // Verify connection
    TRY
        healthCheck ← client.ingestion_service.HealthCheck()
        IF healthCheck.status != HEALTHY THEN
            THROW Exception("Service unhealthy")
        END IF
    CATCH Exception AS e
        Log.error("Failed to connect to metering service", {error: e.message})
        THROW e
    END TRY

    Log.info("Metering client initialized", {endpoint: endpoint})

    RETURN client
END
```

### 6.2 Inline Trade Quota Enforcement

```
ALGORITHM: ExecuteTradeWithQuota
INPUT: agentId (string), tradeRequest (object)
OUTPUT: trade result (object) or quota error

BEGIN
    // Check quota before executing trade
    quotaCheck ← CheckQuota(agentId, TRADE_EXECUTION, 1)

    IF NOT quotaCheck.allowed THEN
        Log.warn("Trade blocked by quota", {
            agent_id: agentId,
            remaining: quotaCheck.remaining,
            limit: quotaCheck.limit
        })

        RETURN {
            success: false,
            error: "quota_exceeded",
            quota_info: quotaCheck
        }
    END IF

    // Execute trade
    TRY
        tradeResult ← TradingEngine.executeTrade(tradeRequest)

        // Record metering event asynchronously
        RecordMeteringEvent(
            agentId,
            TRADE_EXECUTION,
            {
                symbol: tradeRequest.symbol,
                quantity: tradeRequest.quantity,
                price: tradeResult.execution_price,
                order_id: tradeResult.order_id
            },
            1
        )

        RETURN tradeResult

    CATCH Exception AS e
        Log.error("Trade execution failed", {
            agent_id: agentId,
            error: e.message
        })

        RETURN {
            success: false,
            error: e.message
        }
    END TRY
END
```

---

## Complexity Analysis

### Event Ingestion (RecordMeteringEvent)
**Time Complexity**: O(1) average, O(log n) worst case with retry
- gRPC call: O(1)
- Circuit breaker check: O(1)
- Cache update: O(1)
- Retry with backoff: O(log n) for n = MAX_RETRIES

**Space Complexity**: O(1)
- Event payload: O(1) fixed size
- Retry state: O(1)

### Quota Check (CheckQuota)
**Time Complexity**: O(1) with caching, O(log n) without cache
- Cache lookup: O(1)
- gRPC fetch on miss: O(log n)
- Threshold comparison: O(k) where k = number of thresholds

**Space Complexity**: O(1)
- Cached quota data: O(1) per agent
- Result object: O(1)

### Usage Aggregation (AggregateUsage)
**Time Complexity**: O(m) where m = number of event types
- Database query: O(m) for m event types
- Breakdown calculation: O(m)
- Percentage calculation: O(m)

**Space Complexity**: O(m)
- Breakdown map: O(m)
- Summary object: O(1)

### Cost Calculation (CalculateEventCost)
**Time Complexity**: O(t) where t = number of pricing tiers
- Tier iteration: O(t) typically ≤ 3
- Cost accumulation: O(t)

**Space Complexity**: O(1)
- No dynamic allocation

### Alert System (CheckAlertThresholds)
**Time Complexity**: O(e * t) where e = event types, t = thresholds
- Event iteration: O(e)
- Threshold checks: O(t) per event
- Alert creation: O(1)

**Space Complexity**: O(a) where a = number of triggered alerts
- Alert array: O(a)
- Debounce cache: O(e * t)

---

## Summary

This pseudocode provides detailed algorithmic designs for:
1. ✅ Resilient event ingestion with retry and circuit breaker
2. ✅ Flexible quota management with tiered limits
3. ✅ Efficient usage aggregation and trend analysis
4. ✅ Intelligent alert system with debouncing
5. ✅ Precise cost calculation with tiered pricing
6. ✅ Inline quota enforcement for trades
7. ✅ gRPC client integration patterns

All algorithms are designed for production use with error handling, caching, and performance optimization.
