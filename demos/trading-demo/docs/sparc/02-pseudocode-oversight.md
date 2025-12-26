# SPARC Pseudocode: Trading Demo Oversight System

## Overview
This document contains the algorithmic design for human-in-the-loop oversight integration with creto-oversight. The system provides approval workflows for high-risk trading decisions with multi-channel notifications and state management.

---

## 1. Policy Evaluation Algorithm

### 1.1 Main Policy Evaluator

```
ALGORITHM: EvaluateOversightPolicy
INPUT:
    trade (Trade): Proposed trade with { symbol, quantity, side, estimatedValue }
    agent (Agent): Agent state with { id, budget, performance, trustScore }
    portfolio (Portfolio): Current holdings and metrics
    config (PolicyConfig): Oversight rules and thresholds
OUTPUT:
    decision (PolicyDecision): { required: bool, reason: string, approvers: array, priority: enum }

BEGIN
    decision ← {
        required: false,
        reason: "",
        approvers: [],
        priority: "NORMAL"
    }

    // Initialize trigger checks
    triggers ← []

    // TRIGGER 1: Amount Threshold
    IF trade.estimatedValue >= config.amountThreshold THEN
        triggers.append({
            type: "AMOUNT_THRESHOLD",
            severity: CalculateSeverity(trade.estimatedValue, config.amountThreshold),
            message: "Trade value ($" + FormatCurrency(trade.estimatedValue) +
                     ") exceeds threshold ($" + FormatCurrency(config.amountThreshold) + ")"
        })
    END IF

    // TRIGGER 2: Risk Score Threshold
    riskScore ← CalculateTradeRisk(trade, portfolio, agent)
    IF riskScore >= config.riskScoreThreshold THEN
        triggers.append({
            type: "RISK_SCORE",
            severity: MapRiskToSeverity(riskScore),
            message: "Risk score (" + riskScore + ") exceeds threshold (" +
                     config.riskScoreThreshold + ")"
        })
    END IF

    // TRIGGER 3: Concentration Limits
    concentrationCheck ← CheckConcentrationLimits(trade, portfolio, config)
    IF concentrationCheck.violated THEN
        triggers.append({
            type: "CONCENTRATION_LIMIT",
            severity: "HIGH",
            message: concentrationCheck.message
        })
    END IF

    // TRIGGER 4: Budget Utilization
    budgetUtilization ← (agent.budget - trade.estimatedValue) / agent.initialBudget
    IF budgetUtilization <= config.minBudgetThreshold THEN
        triggers.append({
            type: "LOW_BUDGET",
            severity: "MEDIUM",
            message: "Trade would reduce budget to " +
                     FormatPercentage(budgetUtilization * 100) + "% of initial"
        })
    END IF

    // TRIGGER 5: First Trade of Day
    IF IsFirstTradeOfDay(agent.id) AND config.requireApprovalForFirstTrade THEN
        triggers.append({
            type: "FIRST_TRADE",
            severity: "LOW",
            message: "First trade of trading session requires approval"
        })
    END IF

    // TRIGGER 6: Agent Trust Score
    IF agent.trustScore < config.minTrustScore THEN
        triggers.append({
            type: "LOW_TRUST",
            severity: "HIGH",
            message: "Agent trust score (" + agent.trustScore +
                     ") below minimum (" + config.minTrustScore + ")"
        })
    END IF

    // TRIGGER 7: Unusual Pattern Detection
    patternAnalysis ← DetectUnusualPatterns(trade, agent, portfolio)
    IF patternAnalysis.isUnusual THEN
        triggers.append({
            type: "UNUSUAL_PATTERN",
            severity: patternAnalysis.severity,
            message: patternAnalysis.description
        })
    END IF

    // Determine if oversight required
    IF triggers.length > 0 THEN
        decision.required ← true
        decision.reason ← BuildReasonMessage(triggers)
        decision.priority ← DeterminePriority(triggers)
        decision.approvers ← SelectApprovers(triggers, config)
    END IF

    // Log policy evaluation
    LogPolicyEvaluation({
        tradeId: trade.id,
        agentId: agent.id,
        triggers: triggers,
        decision: decision,
        timestamp: GetCurrentTime()
    })

    RETURN decision
END
```

### 1.2 Risk Calculation Subroutine

```
SUBROUTINE: CalculateTradeRisk
INPUT:
    trade (Trade)
    portfolio (Portfolio)
    agent (Agent)
OUTPUT:
    riskScore (float): 0.0 to 1.0

BEGIN
    // Component 1: Volatility Risk
    historicalVolatility ← GetHistoricalVolatility(trade.symbol, period: 30)
    volatilityScore ← MIN(historicalVolatility / 0.5, 1.0)

    // Component 2: Position Size Risk
    positionValue ← trade.quantity * trade.estimatedPrice
    portfolioValue ← portfolio.totalValue
    positionSizeRatio ← positionValue / portfolioValue
    positionSizeScore ← MIN(positionSizeRatio / 0.25, 1.0)

    // Component 3: Liquidity Risk
    avgVolume ← GetAverageDailyVolume(trade.symbol, period: 30)
    liquidityRatio ← trade.quantity / avgVolume
    liquidityScore ← MIN(liquidityRatio / 0.1, 1.0)

    // Component 4: Market Condition Risk
    marketVolatilityIndex ← GetMarketVolatilityIndex()  // VIX
    marketConditionScore ← MIN(marketVolatilityIndex / 40, 1.0)

    // Component 5: Agent Performance Risk
    agentWinRate ← agent.performance.winRate
    agentPerformanceScore ← 1.0 - agentWinRate

    // Weighted combination
    weights ← {
        volatility: 0.25,
        positionSize: 0.30,
        liquidity: 0.20,
        marketCondition: 0.15,
        agentPerformance: 0.10
    }

    riskScore ←
        (volatilityScore * weights.volatility) +
        (positionSizeScore * weights.positionSize) +
        (liquidityScore * weights.liquidity) +
        (marketConditionScore * weights.marketCondition) +
        (agentPerformanceScore * weights.agentPerformance)

    RETURN riskScore
END
```

### 1.3 Concentration Limit Check

```
SUBROUTINE: CheckConcentrationLimits
INPUT:
    trade (Trade)
    portfolio (Portfolio)
    config (PolicyConfig)
OUTPUT:
    result (ConcentrationCheckResult): { violated: bool, message: string }

BEGIN
    result ← { violated: false, message: "" }

    // Check 1: Single Position Concentration
    currentHolding ← portfolio.positions.get(trade.symbol) OR 0
    newHolding ← currentHolding + (trade.quantity * (trade.side == "BUY" ? 1 : -1))
    positionValue ← newHolding * trade.estimatedPrice
    concentrationRatio ← positionValue / portfolio.totalValue

    IF concentrationRatio > config.maxSinglePositionConcentration THEN
        result.violated ← true
        result.message ← "Position in " + trade.symbol +
                         " would be " + FormatPercentage(concentrationRatio * 100) +
                         "% of portfolio (max: " +
                         FormatPercentage(config.maxSinglePositionConcentration * 100) + "%)"
        RETURN result
    END IF

    // Check 2: Sector Concentration
    sector ← GetSectorForSymbol(trade.symbol)
    sectorExposure ← CalculateSectorExposure(portfolio, sector, trade)

    IF sectorExposure > config.maxSectorConcentration THEN
        result.violated ← true
        result.message ← "Sector " + sector +
                         " exposure would be " + FormatPercentage(sectorExposure * 100) +
                         "% (max: " + FormatPercentage(config.maxSectorConcentration * 100) + "%)"
        RETURN result
    END IF

    // Check 3: Correlated Assets
    correlationRisk ← CheckCorrelatedAssets(trade, portfolio, config)
    IF correlationRisk.violated THEN
        result ← correlationRisk
        RETURN result
    END IF

    RETURN result
END
```

---

## 2. Oversight Request Creation

### 2.1 Request Builder

```
ALGORITHM: CreateOversightRequest
INPUT:
    trade (Trade): Proposed trade details
    agent (Agent): Agent state and context
    reason (string): Why oversight is required
    approvers (array): List of approver IDs
    priority (enum): NORMAL | HIGH | CRITICAL
OUTPUT:
    request (OversightRequest): Complete request object with PENDING status

BEGIN
    // Generate unique request ID
    requestId ← GenerateUUID()
    timestamp ← GetCurrentTime()

    // Capture portfolio snapshot
    portfolioSnapshot ← {
        totalValue: CalculatePortfolioValue(),
        positions: CloneCurrentPositions(),
        cash: GetAvailableCash(),
        unrealizedPnL: GetUnrealizedPnL(),
        realizedPnL: GetRealizedPnL(),
        timestamp: timestamp
    }

    // Capture agent state
    agentSnapshot ← {
        id: agent.id,
        budget: agent.budget,
        budgetUsed: agent.initialBudget - agent.budget,
        performance: CloneObject(agent.performance),
        trustScore: agent.trustScore,
        totalTrades: agent.totalTrades,
        recentTrades: GetRecentTrades(agent.id, limit: 10)
    }

    // Perform risk assessment
    riskAssessment ← {
        tradeRisk: CalculateTradeRisk(trade, portfolioSnapshot, agent),
        portfolioRisk: CalculatePortfolioRisk(portfolioSnapshot),
        concentrationRisk: AssessConcentrationRisk(trade, portfolioSnapshot),
        liquidityRisk: AssessLiquidityRisk(trade),
        marketRisk: AssessMarketRisk(trade.symbol),
        overallRisk: "TBD"  // Calculated below
    }

    // Calculate overall risk level
    riskScores ← [
        riskAssessment.tradeRisk,
        riskAssessment.portfolioRisk,
        riskAssessment.concentrationRisk,
        riskAssessment.liquidityRisk,
        riskAssessment.marketRisk
    ]
    riskAssessment.overallRisk ← CalculateOverallRisk(riskScores)

    // Determine timeout based on priority
    timeout ← SWITCH priority:
        CASE "CRITICAL": 4 * HOURS
        CASE "HIGH": 12 * HOURS
        CASE "NORMAL": 24 * HOURS
        DEFAULT: 24 * HOURS

    // Build request context
    context ← {
        trade: {
            id: trade.id,
            symbol: trade.symbol,
            side: trade.side,
            quantity: trade.quantity,
            estimatedPrice: trade.estimatedPrice,
            estimatedValue: trade.estimatedValue,
            orderType: trade.orderType,
            metadata: trade.metadata
        },
        portfolio: portfolioSnapshot,
        agent: agentSnapshot,
        risk: riskAssessment,
        marketData: FetchMarketData(trade.symbol),
        timestamp: timestamp
    }

    // Construct request object
    request ← {
        id: requestId,
        status: "PENDING",
        priority: priority,
        reason: reason,
        context: context,
        approvers: BuildApproverList(approvers),
        requiredApprovals: CalculateRequiredApprovals(approvers, priority),
        decisions: [],  // Will hold approval/rejection records
        createdAt: timestamp,
        expiresAt: timestamp + timeout,
        escalationPath: DefineEscalationPath(priority, approvers),
        metadata: {
            version: "1.0",
            source: "trading-demo",
            agentId: agent.id,
            tradeId: trade.id
        }
    }

    // Persist to creto-oversight
    TRY
        persistResult ← CretoOversightClient.createRequest(request)

        IF NOT persistResult.success THEN
            LogError("Failed to persist oversight request", {
                requestId: requestId,
                error: persistResult.error
            })
            THROW OversightPersistenceException(persistResult.error)
        END IF

    CATCH exception AS e
        LogError("Exception during oversight request creation", {
            requestId: requestId,
            exception: e
        })
        THROW e
    END TRY

    // Emit event for monitoring
    EmitEvent("OVERSIGHT_REQUEST_CREATED", {
        requestId: requestId,
        agentId: agent.id,
        tradeId: trade.id,
        priority: priority,
        approverCount: approvers.length
    })

    RETURN request
END
```

### 2.2 Approver Selection

```
SUBROUTINE: SelectApprovers
INPUT:
    triggers (array): Policy triggers that fired
    config (PolicyConfig): Approver rules
OUTPUT:
    approvers (array): List of approver user IDs

BEGIN
    approvers ← SET()

    // Map triggers to approver roles
    FOR EACH trigger IN triggers DO
        SWITCH trigger.type:
            CASE "AMOUNT_THRESHOLD":
                IF trigger.severity == "CRITICAL" THEN
                    approvers.add(config.approvers.cfo)
                    approvers.add(config.approvers.riskManager)
                ELSE IF trigger.severity == "HIGH" THEN
                    approvers.add(config.approvers.riskManager)
                ELSE
                    approvers.add(config.approvers.tradingDesk)
                END IF

            CASE "RISK_SCORE":
                approvers.add(config.approvers.riskManager)
                IF trigger.severity == "CRITICAL" THEN
                    approvers.add(config.approvers.chiefRiskOfficer)
                END IF

            CASE "CONCENTRATION_LIMIT":
                approvers.add(config.approvers.portfolioManager)
                approvers.add(config.approvers.riskManager)

            CASE "LOW_TRUST":
                approvers.add(config.approvers.agentSupervisor)
                approvers.add(config.approvers.complianceOfficer)

            CASE "UNUSUAL_PATTERN":
                approvers.add(config.approvers.fraudDetection)
                approvers.add(config.approvers.complianceOfficer)

            DEFAULT:
                approvers.add(config.approvers.tradingDesk)
        END SWITCH
    END FOR

    // Ensure minimum approver count
    IF approvers.size < config.minApprovers THEN
        // Add default approvers
        approvers.add(config.approvers.tradingDesk)
        approvers.add(config.approvers.riskManager)
    END IF

    // Check availability (are approvers online?)
    availableApprovers ← FilterAvailableApprovers(approvers)

    IF availableApprovers.size == 0 THEN
        // Escalate to backup approvers
        availableApprovers ← config.approvers.emergencyBackup
    END IF

    RETURN Array.from(availableApprovers)
END
```

---

## 3. Notification Dispatch System

### 3.1 Multi-Channel Dispatcher

```
ALGORITHM: NotifyApprovers
INPUT:
    request (OversightRequest): The approval request
    channels (array): Notification channels to use
OUTPUT:
    result (NotificationResult): { sent: array, failed: array }

BEGIN
    result ← {
        sent: [],
        failed: []
    }

    // Build notification message
    message ← BuildNotificationMessage(request)

    // Add action buttons/links
    actions ← {
        approve: BuildApprovalAction(request.id),
        reject: BuildRejectionAction(request.id),
        viewDetails: BuildDetailsLink(request.id)
    }

    // Dispatch to each channel concurrently
    FOR EACH channel IN channels DO
        TRY
            SWITCH channel.type:
                CASE "SLACK":
                    SendSlackNotification(channel, request, message, actions)

                CASE "EMAIL":
                    SendEmailNotification(channel, request, message, actions)

                CASE "WEBHOOK":
                    SendWebhookNotification(channel, request, message, actions)

                CASE "SMS":
                    SendSMSNotification(channel, request, message)

                CASE "WEBSOCKET":
                    BroadcastWebSocketNotification(channel, request, message, actions)
            END SWITCH

            result.sent.append({
                channel: channel.type,
                timestamp: GetCurrentTime(),
                status: "DELIVERED"
            })

        CATCH exception AS e
            LogError("Notification dispatch failed", {
                channel: channel.type,
                requestId: request.id,
                error: e
            })

            result.failed.append({
                channel: channel.type,
                error: e.message,
                timestamp: GetCurrentTime()
            })

            // Attempt retry with exponential backoff
            ScheduleRetry(channel, request, message, actions, attempt: 1)
        END TRY
    END FOR

    // Record notification status
    UpdateRequestNotificationStatus(request.id, result)

    // If all primary channels failed, trigger escalation
    IF result.sent.length == 0 THEN
        TriggerNotificationEscalation(request)
    END IF

    RETURN result
END
```

### 3.2 Slack Notification

```
SUBROUTINE: SendSlackNotification
INPUT:
    channel (SlackChannel): Webhook URL and settings
    request (OversightRequest): Request details
    message (NotificationMessage): Formatted message
    actions (Actions): Approve/reject buttons
OUTPUT:
    void

BEGIN
    // Build rich Slack message with blocks
    slackPayload ← {
        channel: channel.channelId,
        username: "Trading Oversight Bot",
        icon_emoji: ":chart_with_upwards_trend:",
        attachments: [
            {
                color: GetColorByPriority(request.priority),
                title: "Trade Approval Request #" + request.id,
                title_link: BuildDetailsLink(request.id),
                fields: [
                    {
                        title: "Priority",
                        value: request.priority,
                        short: true
                    },
                    {
                        title: "Agent",
                        value: request.context.agent.id,
                        short: true
                    },
                    {
                        title: "Trade",
                        value: request.context.trade.side + " " +
                               request.context.trade.quantity + " " +
                               request.context.trade.symbol,
                        short: true
                    },
                    {
                        title: "Estimated Value",
                        value: "$" + FormatCurrency(request.context.trade.estimatedValue),
                        short: true
                    },
                    {
                        title: "Risk Score",
                        value: FormatRiskScore(request.context.risk.overallRisk),
                        short: true
                    },
                    {
                        title: "Expires",
                        value: FormatRelativeTime(request.expiresAt),
                        short: true
                    },
                    {
                        title: "Reason",
                        value: request.reason,
                        short: false
                    }
                ],
                actions: [
                    {
                        type: "button",
                        text: "✅ Approve",
                        style: "primary",
                        url: actions.approve
                    },
                    {
                        type: "button",
                        text: "❌ Reject",
                        style: "danger",
                        url: actions.reject
                    },
                    {
                        type: "button",
                        text: "📊 View Details",
                        url: actions.viewDetails
                    }
                ],
                footer: "Trading Oversight System",
                footer_icon: "https://platform.slack-edge.com/img/default_application_icon.png",
                ts: GetUnixTimestamp()
            }
        ]
    }

    // Send to Slack webhook
    response ← HTTPPost(channel.webhookUrl, slackPayload)

    IF response.statusCode != 200 THEN
        THROW SlackNotificationException(response.error)
    END IF

    // Store message ID for future updates
    StoreNotificationMessageId(request.id, "SLACK", response.messageId)
END
```

### 3.3 Retry Logic

```
SUBROUTINE: ScheduleRetry
INPUT:
    channel (Channel): Notification channel
    request (OversightRequest): Request to notify
    message (NotificationMessage): Message content
    actions (Actions): Action buttons
    attempt (integer): Current retry attempt
OUTPUT:
    void

CONSTANTS:
    MAX_RETRIES = 3
    BASE_DELAY = 5  // seconds

BEGIN
    IF attempt > MAX_RETRIES THEN
        LogError("Max retries exceeded for notification", {
            requestId: request.id,
            channel: channel.type,
            attempts: attempt
        })

        // Try fallback channel
        IF channel.fallbackChannel EXISTS THEN
            TRY
                NotifyViaFallback(channel.fallbackChannel, request, message, actions)
            CATCH e
                LogError("Fallback notification also failed", e)
            END TRY
        END IF

        RETURN
    END IF

    // Exponential backoff: 5s, 10s, 20s
    delay ← BASE_DELAY * (2 ^ (attempt - 1))

    // Schedule retry
    ScheduleTask(
        delay: delay,
        task: λ() => {
            TRY
                SendNotification(channel, request, message, actions)
                LogInfo("Retry successful", {
                    requestId: request.id,
                    channel: channel.type,
                    attempt: attempt
                })
            CATCH e
                ScheduleRetry(channel, request, message, actions, attempt + 1)
            END TRY
        }
    )
END
```

---

## 4. Approval Processing Engine

### 4.1 Decision Processor

```
ALGORITHM: ProcessApproval
INPUT:
    requestId (string): Unique request identifier
    approverId (string): User ID of approver
    decision (enum): APPROVE | REJECT
    reason (string): Optional reason for decision
    metadata (object): Additional context
OUTPUT:
    result (ProcessingResult): { status: enum, message: string, request: OversightRequest }

BEGIN
    // Retrieve request from storage
    request ← CretoOversightClient.getRequest(requestId)

    IF request IS NULL THEN
        RETURN {
            status: "ERROR",
            message: "Request not found: " + requestId,
            request: null
        }
    END IF

    // Validate request is still pending
    IF request.status != "PENDING" AND request.status != "ESCALATED" THEN
        RETURN {
            status: "ERROR",
            message: "Request is no longer pending (status: " + request.status + ")",
            request: request
        }
    END IF

    // Check if request has expired
    IF GetCurrentTime() > request.expiresAt THEN
        request.status ← "EXPIRED"
        UpdateRequest(request)

        RETURN {
            status: "ERROR",
            message: "Request has expired",
            request: request
        }
    END IF

    // Validate approver is authorized
    IF NOT IsAuthorizedApprover(approverId, request.approvers) THEN
        LogWarning("Unauthorized approval attempt", {
            requestId: requestId,
            approverId: approverId,
            authorizedApprovers: request.approvers
        })

        RETURN {
            status: "ERROR",
            message: "User is not authorized to approve this request",
            request: request
        }
    END IF

    // Check for duplicate decision from same approver
    existingDecision ← FindDecisionByApprover(request.decisions, approverId)
    IF existingDecision EXISTS THEN
        RETURN {
            status: "ERROR",
            message: "Approver has already made a decision: " + existingDecision.decision,
            request: request
        }
    END IF

    // Record decision
    decisionRecord ← {
        approverId: approverId,
        decision: decision,
        reason: reason,
        timestamp: GetCurrentTime(),
        metadata: metadata
    }

    request.decisions.append(decisionRecord)

    // Check for immediate rejection (veto)
    IF decision == "REJECT" AND IsVetoApprover(approverId, request) THEN
        request.status ← "REJECTED"
        request.finalDecision ← {
            decision: "REJECTED",
            reason: "Vetoed by " + approverId + ": " + reason,
            timestamp: GetCurrentTime()
        }

        UpdateRequest(request)
        NotifyRejection(request)
        TriggerTradeRejectionCallback(request)

        RETURN {
            status: "REJECTED",
            message: "Request vetoed and rejected",
            request: request
        }
    END IF

    // Check quorum for approval
    quorumResult ← CheckApprovalQuorum(request)

    IF quorumResult.approved THEN
        // Sufficient approvals received
        request.status ← "APPROVED"
        request.finalDecision ← {
            decision: "APPROVED",
            approvalCount: quorumResult.approvalCount,
            requiredApprovals: request.requiredApprovals,
            timestamp: GetCurrentTime()
        }

        UpdateRequest(request)
        NotifyApproval(request)
        TriggerTradeExecutionCallback(request)

        RETURN {
            status: "APPROVED",
            message: "Request approved with " + quorumResult.approvalCount + " approvals",
            request: request
        }

    ELSE IF quorumResult.rejected THEN
        // Sufficient rejections to block
        request.status ← "REJECTED"
        request.finalDecision ← {
            decision: "REJECTED",
            rejectionCount: quorumResult.rejectionCount,
            timestamp: GetCurrentTime()
        }

        UpdateRequest(request)
        NotifyRejection(request)
        TriggerTradeRejectionCallback(request)

        RETURN {
            status: "REJECTED",
            message: "Request rejected by quorum",
            request: request
        }

    ELSE
        // Still waiting for more decisions
        UpdateRequest(request)
        NotifyPartialDecision(request, decisionRecord)

        RETURN {
            status: "PENDING",
            message: "Decision recorded, waiting for quorum (" +
                     quorumResult.approvalCount + "/" + request.requiredApprovals + ")",
            request: request
        }
    END IF
END
```

### 4.2 Quorum Checker

```
SUBROUTINE: CheckApprovalQuorum
INPUT:
    request (OversightRequest): Request with decisions
OUTPUT:
    result (QuorumResult): { approved: bool, rejected: bool, approvalCount: int, rejectionCount: int }

BEGIN
    approvalCount ← 0
    rejectionCount ← 0

    // Count approvals and rejections
    FOR EACH decision IN request.decisions DO
        IF decision.decision == "APPROVE" THEN
            approvalCount ← approvalCount + 1
        ELSE IF decision.decision == "REJECT" THEN
            rejectionCount ← rejectionCount + 1
        END IF
    END FOR

    // Determine quorum rules based on priority
    SWITCH request.priority:
        CASE "CRITICAL":
            requiredApprovals ← request.requiredApprovals  // Usually 2-3
            maxRejections ← 1  // Single rejection blocks

        CASE "HIGH":
            requiredApprovals ← request.requiredApprovals  // Usually 2
            maxRejections ← 1

        CASE "NORMAL":
            requiredApprovals ← request.requiredApprovals  // Usually 1
            maxRejections ← 2  // Needs 2 rejections to block
    END SWITCH

    // Check for approval quorum
    approved ← approvalCount >= requiredApprovals

    // Check for rejection quorum
    rejected ← rejectionCount > maxRejections

    RETURN {
        approved: approved,
        rejected: rejected,
        approvalCount: approvalCount,
        rejectionCount: rejectionCount
    }
END
```

### 4.3 Trade Execution Callback

```
SUBROUTINE: TriggerTradeExecutionCallback
INPUT:
    request (OversightRequest): Approved request
OUTPUT:
    void

BEGIN
    trade ← request.context.trade
    agent ← request.context.agent

    // Prepare execution context
    executionContext ← {
        requestId: request.id,
        approvedAt: request.finalDecision.timestamp,
        approvers: ExtractApproverIds(request.decisions),
        originalRisk: request.context.risk,
        validUntil: GetCurrentTime() + (15 * MINUTES)  // 15 min execution window
    }

    // Call agent's trade execution handler
    TRY
        executionResult ← AgentExecutor.executeTrade({
            agentId: agent.id,
            trade: trade,
            context: executionContext
        })

        // Update request with execution result
        request.executionResult ← {
            status: executionResult.status,
            orderId: executionResult.orderId,
            executedAt: executionResult.timestamp,
            executedPrice: executionResult.price,
            executedQuantity: executionResult.quantity
        }

        IF executionResult.status == "FILLED" THEN
            request.status ← "EXECUTED"
        ELSE IF executionResult.status == "FAILED" THEN
            request.status ← "EXECUTION_FAILED"
        ELSE
            request.status ← "EXECUTING"
        END IF

        UpdateRequest(request)

        // Notify approvers of execution
        NotifyExecution(request, executionResult)

    CATCH exception AS e
        LogError("Trade execution callback failed", {
            requestId: request.id,
            tradeId: trade.id,
            error: e
        })

        request.status ← "EXECUTION_FAILED"
        request.executionError ← {
            message: e.message,
            timestamp: GetCurrentTime()
        }

        UpdateRequest(request)
        NotifyExecutionFailure(request, e)
    END TRY
END
```

---

## 5. Timeout and Escalation Handling

### 5.1 Timeout Monitor

```
ALGORITHM: MonitorRequestTimeouts
INPUT: void
OUTPUT: void

BEGIN
    WHILE true DO
        // Fetch all pending and escalated requests
        pendingRequests ← CretoOversightClient.getRequestsByStatus(["PENDING", "ESCALATED"])

        currentTime ← GetCurrentTime()

        FOR EACH request IN pendingRequests DO
            // Check if request has expired
            IF currentTime > request.expiresAt THEN
                HandleTimeout(request)
            ELSE IF ShouldEscalate(request, currentTime) THEN
                EscalateRequest(request)
            END IF
        END FOR

        // Sleep before next check (every 60 seconds)
        Sleep(60 * SECONDS)
    END WHILE
END
```

### 5.2 Timeout Handler

```
ALGORITHM: HandleTimeout
INPUT:
    request (OversightRequest): Expired request
OUTPUT:
    void

BEGIN
    // Retrieve escalation policy
    policy ← GetEscalationPolicy(request.priority, request.context.agent)

    LogWarning("Request timeout", {
        requestId: request.id,
        priority: request.priority,
        approvalCount: CountApprovals(request.decisions),
        requiredApprovals: request.requiredApprovals
    })

    SWITCH policy.timeoutAction:
        CASE "AUTO_ESCALATE":
            // Escalate to next level of approvers
            IF request.escalationPath.length > 0 THEN
                nextLevel ← request.escalationPath.shift()

                request.status ← "ESCALATED"
                request.approvers ← nextLevel.approvers
                request.expiresAt ← GetCurrentTime() + nextLevel.timeout
                request.escalationHistory.append({
                    level: nextLevel.level,
                    reason: "Timeout - no quorum reached",
                    timestamp: GetCurrentTime()
                })

                UpdateRequest(request)
                NotifyEscalation(request, nextLevel)
            ELSE
                // No more escalation levels, apply final timeout policy
                ApplyFinalTimeoutPolicy(request, policy)
            END IF

        CASE "AUTO_REJECT":
            // Automatically reject the request
            request.status ← "REJECTED"
            request.finalDecision ← {
                decision: "REJECTED",
                reason: "Automatic rejection due to timeout",
                timestamp: GetCurrentTime()
            }

            UpdateRequest(request)
            NotifyTimeout(request, "AUTO_REJECT")
            TriggerTradeRejectionCallback(request)

        CASE "AUTO_APPROVE":
            // Automatically approve (for trusted agents only)
            IF IsTrustedAgent(request.context.agent) THEN
                request.status ← "APPROVED"
                request.finalDecision ← {
                    decision: "APPROVED",
                    reason: "Auto-approved for trusted agent after timeout",
                    timestamp: GetCurrentTime()
                }

                UpdateRequest(request)
                NotifyTimeout(request, "AUTO_APPROVE")
                TriggerTradeExecutionCallback(request)
            ELSE
                // Fall back to rejection for untrusted agents
                HandleTimeout(request)  // Recursive with AUTO_REJECT policy
            END IF

        CASE "ESCALATE_TO_EMERGENCY":
            // Escalate to emergency contact
            request.status ← "ESCALATED"
            request.approvers ← policy.emergencyApprovers
            request.expiresAt ← GetCurrentTime() + (1 * HOUR)
            request.priority ← "CRITICAL"

            UpdateRequest(request)
            NotifyEmergencyEscalation(request)

        DEFAULT:
            // Default to expiration
            request.status ← "EXPIRED"
            UpdateRequest(request)
            NotifyTimeout(request, "EXPIRED")
    END SWITCH
END
```

### 5.3 Smart Escalation

```
SUBROUTINE: ShouldEscalate
INPUT:
    request (OversightRequest): Current request
    currentTime (timestamp): Current time
OUTPUT:
    shouldEscalate (bool): Whether to escalate now

BEGIN
    // Don't escalate if already escalated recently
    IF request.status == "ESCALATED" THEN
        RETURN false
    END IF

    // Calculate time elapsed
    elapsed ← currentTime - request.createdAt
    timeRemaining ← request.expiresAt - currentTime

    // Escalate at 75% of timeout period if no approvals yet
    escalationThreshold ← (request.expiresAt - request.createdAt) * 0.75

    approvalCount ← CountApprovals(request.decisions)

    IF elapsed > escalationThreshold AND approvalCount == 0 THEN
        RETURN true
    END IF

    // Escalate if high priority and less than 1 hour remaining
    IF request.priority == "CRITICAL" AND timeRemaining < (1 * HOUR) THEN
        IF approvalCount < request.requiredApprovals THEN
            RETURN true
        END IF
    END IF

    RETURN false
END
```

---

## 6. State Machine Implementation

### 6.1 State Transition Engine

```
ALGORITHM: TransitionRequestState
INPUT:
    request (OversightRequest): Current request
    event (StateEvent): Event triggering transition
    metadata (object): Additional context
OUTPUT:
    request (OversightRequest): Updated request

BEGIN
    oldState ← request.status
    newState ← DetermineNewState(oldState, event)

    // Validate transition is allowed
    IF NOT IsValidTransition(oldState, newState) THEN
        LogError("Invalid state transition", {
            requestId: request.id,
            from: oldState,
            to: newState,
            event: event
        })
        THROW InvalidStateTransitionException(oldState, newState, event)
    END IF

    // Execute pre-transition hooks
    ExecutePreTransitionHooks(request, oldState, newState, event)

    // Update state
    request.status ← newState
    request.statusHistory.append({
        from: oldState,
        to: newState,
        event: event,
        timestamp: GetCurrentTime(),
        metadata: metadata
    })

    // Execute state-specific actions
    ExecuteStateActions(request, newState, event)

    // Execute post-transition hooks
    ExecutePostTransitionHooks(request, oldState, newState, event)

    // Persist changes
    UpdateRequest(request)

    // Emit state change event
    EmitEvent("REQUEST_STATE_CHANGED", {
        requestId: request.id,
        from: oldState,
        to: newState,
        event: event
    })

    RETURN request
END
```

### 6.2 State Transition Rules

```
FUNCTION: DetermineNewState
INPUT:
    currentState (enum): Current state
    event (StateEvent): Triggering event
OUTPUT:
    newState (enum): Next state

BEGIN
    SWITCH currentState:
        CASE "PENDING":
            SWITCH event.type:
                CASE "APPROVAL_QUORUM_MET":
                    RETURN "APPROVED"

                CASE "REJECTION_RECEIVED":
                    IF IsVetoRejection(event) THEN
                        RETURN "REJECTED"
                    ELSE
                        RETURN "PENDING"  // Stay pending
                    END IF

                CASE "TIMEOUT":
                    RETURN "ESCALATED"

                CASE "EXPIRED":
                    RETURN "EXPIRED"

                DEFAULT:
                    RETURN "PENDING"
            END SWITCH

        CASE "APPROVED":
            SWITCH event.type:
                CASE "TRADE_EXECUTED":
                    RETURN "EXECUTED"

                CASE "EXECUTION_FAILED":
                    RETURN "FAILED"

                DEFAULT:
                    RETURN "APPROVED"
            END SWITCH

        CASE "ESCALATED":
            SWITCH event.type:
                CASE "ESCALATED_APPROVAL":
                    RETURN "APPROVED"

                CASE "ESCALATED_REJECTION":
                    RETURN "REJECTED"

                CASE "ESCALATION_TIMEOUT":
                    RETURN "EXPIRED"

                DEFAULT:
                    RETURN "ESCALATED"
            END SWITCH

        CASE "REJECTED", "EXPIRED", "EXECUTED", "FAILED":
            // Terminal states - no transitions
            RETURN currentState

        DEFAULT:
            THROW UnknownStateException(currentState)
    END SWITCH
END
```

### 6.3 Transition Validation

```
FUNCTION: IsValidTransition
INPUT:
    from (enum): Source state
    to (enum): Target state
OUTPUT:
    valid (bool): Whether transition is allowed

BEGIN
    // Define valid transition map
    validTransitions ← {
        "PENDING": ["APPROVED", "REJECTED", "ESCALATED", "EXPIRED"],
        "APPROVED": ["EXECUTED", "FAILED"],
        "REJECTED": [],  // Terminal
        "ESCALATED": ["APPROVED", "REJECTED", "EXPIRED"],
        "EXPIRED": [],  // Terminal
        "EXECUTED": [],  // Terminal
        "FAILED": []  // Terminal
    }

    IF from == to THEN
        // Self-transition always allowed for idempotency
        RETURN true
    END IF

    allowedTransitions ← validTransitions[from]

    IF allowedTransitions IS NULL THEN
        RETURN false
    END IF

    RETURN allowedTransitions.contains(to)
END
```

---

## 7. Integration Points

### 7.1 gRPC Client for creto-oversight

```
MODULE: CretoOversightClient

CONSTANT:
    GRPC_ADDRESS = "localhost:50052"
    TIMEOUT = 5000  // milliseconds

FUNCTION: createRequest
INPUT: request (OversightRequest)
OUTPUT: result (CreateRequestResult)

BEGIN
    client ← CreateGrpcClient(GRPC_ADDRESS)

    TRY
        response ← client.CreateRequest(
            request: SerializeRequest(request),
            timeout: TIMEOUT
        )

        RETURN {
            success: true,
            requestId: response.requestId,
            version: response.version
        }

    CATCH grpcException AS e
        LogError("gRPC CreateRequest failed", {
            error: e.message,
            code: e.code,
            requestId: request.id
        })

        RETURN {
            success: false,
            error: e.message,
            code: e.code
        }
    END TRY
END

FUNCTION: getRequest
INPUT: requestId (string)
OUTPUT: request (OversightRequest) or null

BEGIN
    client ← CreateGrpcClient(GRPC_ADDRESS)

    TRY
        response ← client.GetRequest(
            requestId: requestId,
            timeout: TIMEOUT
        )

        RETURN DeserializeRequest(response.request)

    CATCH grpcException AS e
        IF e.code == "NOT_FOUND" THEN
            RETURN null
        ELSE
            THROW e
        END IF
    END TRY
END

FUNCTION: updateRequest
INPUT: request (OversightRequest)
OUTPUT: success (bool)

BEGIN
    client ← CreateGrpcClient(GRPC_ADDRESS)

    TRY
        response ← client.UpdateRequest(
            request: SerializeRequest(request),
            timeout: TIMEOUT
        )

        RETURN response.success

    CATCH grpcException AS e
        LogError("gRPC UpdateRequest failed", e)
        RETURN false
    END TRY
END
```

### 7.2 WebSocket Real-Time Updates

```
MODULE: OversightWebSocketManager

FUNCTION: BroadcastStatusUpdate
INPUT:
    request (OversightRequest): Updated request
    event (string): Event type
OUTPUT: void

BEGIN
    // Build update message
    message ← {
        type: "REQUEST_UPDATE",
        event: event,
        data: {
            requestId: request.id,
            status: request.status,
            priority: request.priority,
            decisions: request.decisions,
            timestamp: GetCurrentTime()
        }
    }

    // Broadcast to all connected clients subscribed to this request
    subscribers ← GetSubscribers("request:" + request.id)

    FOR EACH subscriber IN subscribers DO
        TRY
            subscriber.send(JSON.stringify(message))
        CATCH e
            LogWarning("WebSocket send failed", {
                subscriberId: subscriber.id,
                error: e
            })

            // Remove dead connection
            RemoveSubscriber(subscriber)
        END TRY
    END FOR

    // Also broadcast to role-based channels
    FOR EACH approverId IN request.approvers DO
        roleChannel ← "approver:" + approverId
        BroadcastToChannel(roleChannel, message)
    END FOR
END

FUNCTION: SubscribeToRequest
INPUT:
    websocket (WebSocket): Client connection
    requestId (string): Request to subscribe to
OUTPUT: void

BEGIN
    subscriptionId ← GenerateUUID()

    subscription ← {
        id: subscriptionId,
        websocket: websocket,
        requestId: requestId,
        createdAt: GetCurrentTime()
    }

    AddSubscription("request:" + requestId, subscription)

    // Send initial state
    request ← GetRequest(requestId)
    IF request EXISTS THEN
        websocket.send(JSON.stringify({
            type: "INITIAL_STATE",
            data: request
        }))
    END IF
END
```

### 7.3 Event Bus Integration

```
MODULE: OversightEventBus

FUNCTION: EmitEvent
INPUT:
    eventType (string): Type of event
    payload (object): Event data
OUTPUT: void

BEGIN
    event ← {
        id: GenerateUUID(),
        type: eventType,
        payload: payload,
        timestamp: GetCurrentTime(),
        source: "oversight-system"
    }

    // Publish to event bus
    EventBus.publish("oversight." + eventType, event)

    // Trigger registered handlers
    handlers ← GetEventHandlers(eventType)

    FOR EACH handler IN handlers DO
        TRY
            handler.handle(event)
        CATCH e
            LogError("Event handler failed", {
                eventType: eventType,
                handler: handler.name,
                error: e
            })
        END TRY
    END FOR

    // Persist to event log
    EventLog.append(event)
END

FUNCTION: RegisterEventHandler
INPUT:
    eventType (string): Event to listen for
    handler (Function): Handler function
OUTPUT: handlerId (string)

BEGIN
    handlerId ← GenerateUUID()

    handlerRegistration ← {
        id: handlerId,
        eventType: eventType,
        handler: handler,
        registeredAt: GetCurrentTime()
    }

    AddEventHandler(eventType, handlerRegistration)

    RETURN handlerId
END
```

---

## 8. Complexity Analysis

### Policy Evaluation
- **Time Complexity**: O(T + C) where T = number of triggers, C = concentration calculations
- **Space Complexity**: O(T) for trigger storage
- **Optimization**: Cache risk scores and portfolio metrics (5-minute TTL)

### Request Creation
- **Time Complexity**: O(1) for request construction + O(A) for approver selection
- **Space Complexity**: O(S) where S = snapshot size (portfolio + agent state)
- **Optimization**: Use shallow copies for large portfolios

### Notification Dispatch
- **Time Complexity**: O(C × A) where C = channels, A = approvers (parallelizable)
- **Space Complexity**: O(C) for concurrent dispatch
- **Optimization**: Concurrent dispatch with Promise.all or async workers

### Approval Processing
- **Time Complexity**: O(D) where D = number of decisions
- **Space Complexity**: O(1) for decision recording
- **Optimization**: Index decisions by approverId for duplicate check

### Timeout Monitor
- **Time Complexity**: O(R) where R = number of pending requests
- **Space Complexity**: O(1) per check
- **Optimization**: Use priority queue sorted by expiresAt

### State Transitions
- **Time Complexity**: O(1) for transition + O(H) for hooks
- **Space Complexity**: O(H) where H = history size
- **Optimization**: Limit history retention (last 100 transitions)

---

## 9. Error Handling and Recovery

### 9.1 Transactional Safety

```
PATTERN: TransactionalUpdate
INPUT: request (OversightRequest), updateFn (Function)
OUTPUT: success (bool)

BEGIN
    // Create backup of current state
    backup ← CloneRequest(request)
    version ← request.version

    TRY
        // Apply updates
        updateFn(request)

        // Increment version for optimistic locking
        request.version ← version + 1

        // Attempt to persist with version check
        success ← CretoOversightClient.updateRequestWithVersion(request, version)

        IF NOT success THEN
            // Concurrent modification detected
            LogWarning("Concurrent modification detected", {
                requestId: request.id,
                expectedVersion: version,
                currentVersion: request.version
            })

            // Restore from backup
            request ← backup

            // Retry with fresh data
            freshRequest ← CretoOversightClient.getRequest(request.id)
            RETURN TransactionalUpdate(freshRequest, updateFn)
        END IF

        RETURN true

    CATCH exception AS e
        LogError("Transactional update failed", e)

        // Rollback to backup
        request ← backup

        RETURN false
    END TRY
END
```

### 9.2 Circuit Breaker for External Services

```
CLASS: CircuitBreaker
    STATE:
        failureCount: integer ← 0
        successCount: integer ← 0
        state: enum ← CLOSED  // CLOSED, OPEN, HALF_OPEN
        lastFailureTime: timestamp ← null

    CONSTANTS:
        FAILURE_THRESHOLD = 5
        SUCCESS_THRESHOLD = 2
        TIMEOUT = 60 * SECONDS

    FUNCTION: execute(operation: Function)
    BEGIN
        SWITCH this.state:
            CASE CLOSED:
                TRY
                    result ← operation()
                    this.onSuccess()
                    RETURN result
                CATCH e
                    this.onFailure()
                    THROW e
                END TRY

            CASE OPEN:
                IF GetCurrentTime() - this.lastFailureTime > TIMEOUT THEN
                    this.state ← HALF_OPEN
                    RETURN this.execute(operation)  // Retry
                ELSE
                    THROW CircuitBreakerOpenException()
                END IF

            CASE HALF_OPEN:
                TRY
                    result ← operation()
                    this.onSuccess()
                    RETURN result
                CATCH e
                    this.onFailure()
                    THROW e
                END TRY
        END SWITCH
    END

    FUNCTION: onSuccess()
    BEGIN
        this.successCount ← this.successCount + 1

        IF this.state == HALF_OPEN AND this.successCount >= SUCCESS_THRESHOLD THEN
            this.state ← CLOSED
            this.failureCount ← 0
            this.successCount ← 0
        END IF
    END

    FUNCTION: onFailure()
    BEGIN
        this.failureCount ← this.failureCount + 1
        this.lastFailureTime ← GetCurrentTime()

        IF this.failureCount >= FAILURE_THRESHOLD THEN
            this.state ← OPEN
            this.successCount ← 0
        END IF
    END
END
```

---

## Summary

This pseudocode specification provides:

1. ✅ **Complete policy evaluation** with 7 trigger types and risk scoring
2. ✅ **Robust request creation** with comprehensive context capture
3. ✅ **Multi-channel notifications** (Slack, Email, Webhook, SMS, WebSocket)
4. ✅ **Quorum-based approval** with veto support and priority-based rules
5. ✅ **Smart timeout handling** with escalation paths and auto-policies
6. ✅ **Formal state machine** with transition validation
7. ✅ **Integration points** for gRPC, WebSocket, and event bus
8. ✅ **Complexity analysis** for all major algorithms
9. ✅ **Error handling** with transactional safety and circuit breakers

The design is optimized for:
- **Low latency**: O(1) operations where possible
- **High availability**: Circuit breakers and fallbacks
- **Data consistency**: Transactional updates with optimistic locking
- **Scalability**: Concurrent processing and efficient indexing
- **Observability**: Comprehensive logging and event emission

Ready for Architecture phase implementation!
