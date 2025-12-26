# SPARC Phase 2: Pseudocode - Trading Demo Algorithms

## Overview
This document provides detailed algorithmic designs for the AI Trading Agent Demo, focusing on agent lifecycle, trade execution, portfolio management, and integration with creto-metering and creto-oversight systems.

---

## 1. Agent Lifecycle Management

### 1.1 Create Agent

```
ALGORITHM: CreateTradingAgent
INPUT:
    config (object):
        - name (string)
        - monthlyBudget (decimal, range: $1,000 - $1,000,000)
        - riskProfile (enum: conservative, moderate, aggressive)
        - allowedAssets (array of strings)
        - maxPositionSize (decimal, optional)

OUTPUT:
    agent (object) or error

CONSTANTS:
    MIN_BUDGET = 1000
    MAX_BUDGET = 1000000
    DEFAULT_RISK_TOLERANCE = 0.20  // 20% portfolio risk

BEGIN
    // Phase 1: Input Validation
    IF config.monthlyBudget < MIN_BUDGET OR config.monthlyBudget > MAX_BUDGET THEN
        RETURN error("Budget must be between $1K and $1M")
    END IF

    IF config.name is empty OR LENGTH(config.name) > 100 THEN
        RETURN error("Invalid agent name")
    END IF

    IF config.riskProfile NOT IN [conservative, moderate, aggressive] THEN
        RETURN error("Invalid risk profile")
    END IF

    // Phase 2: Initialize Metering Quota
    meteringConfig ← {
        entityId: GENERATE_UUID(),
        entityType: "trading_agent",
        quotaLimit: config.monthlyBudget,
        billingPeriod: "monthly",
        resetDay: DAY_OF_MONTH(),
        overage: "block"  // Prevent trades beyond budget
    }

    meteringResult ← CretoMetering.createQuota(meteringConfig)

    IF meteringResult.error THEN
        RETURN error("Failed to initialize budget quota: " + meteringResult.error)
    END IF

    // Phase 3: Set Risk Parameters
    riskParameters ← {
        maxDailyLoss: config.monthlyBudget * 0.05,  // 5% daily loss limit
        maxPositionSize: config.maxPositionSize OR (config.monthlyBudget * 0.30),
        maxConcentration: 0.40,  // 40% max in single sector
        riskTolerance: MAP_RISK_PROFILE(config.riskProfile)
    }

    // Phase 4: Create Agent Record
    agent ← {
        id: meteringConfig.entityId,
        name: config.name,
        monthlyBudget: config.monthlyBudget,
        riskProfile: config.riskProfile,
        riskParameters: riskParameters,
        allowedAssets: config.allowedAssets OR ["AAPL", "GOOGL", "MSFT", "TSLA"],
        status: "active",
        createdAt: CURRENT_TIMESTAMP(),
        meteringQuotaId: meteringResult.quotaId,
        portfolio: {
            cash: config.monthlyBudget,
            positions: [],
            totalValue: config.monthlyBudget,
            dailyPnL: 0,
            totalPnL: 0
        }
    }

    // Phase 5: Persist to Database
    TRY
        Database.agents.insert(agent)

        // Log agent creation event
        AuditLog.record({
            event: "agent_created",
            agentId: agent.id,
            timestamp: CURRENT_TIMESTAMP(),
            metadata: {
                budget: config.monthlyBudget,
                riskProfile: config.riskProfile
            }
        })

        RETURN {success: true, agent: agent}

    CATCH error
        // Rollback metering quota
        CretoMetering.deleteQuota(meteringResult.quotaId)
        RETURN error("Database error: " + error.message)
    END TRY
END

SUBROUTINE: MAP_RISK_PROFILE
INPUT: profile (string)
OUTPUT: riskTolerance (float)

BEGIN
    CASE profile OF
        "conservative": RETURN 0.10  // 10% risk tolerance
        "moderate": RETURN 0.20      // 20% risk tolerance
        "aggressive": RETURN 0.35    // 35% risk tolerance
        DEFAULT: RETURN 0.20
    END CASE
END

TIME COMPLEXITY: O(1)
SPACE COMPLEXITY: O(1)
```

### 1.2 Terminate Agent

```
ALGORITHM: TerminateTradingAgent
INPUT:
    agentId (string)
    closePositions (boolean, default: false)

OUTPUT:
    terminationReport (object) or error

BEGIN
    // Phase 1: Retrieve Agent
    agent ← Database.agents.findById(agentId)

    IF agent is null THEN
        RETURN error("Agent not found")
    END IF

    IF agent.status = "terminated" THEN
        RETURN error("Agent already terminated")
    END IF

    // Phase 2: Cancel Pending Trades
    pendingTrades ← Database.trades.find({
        agentId: agentId,
        status: "pending_approval"
    })

    canceledCount ← 0
    FOR EACH trade IN pendingTrades DO
        trade.status ← "canceled"
        trade.canceledAt ← CURRENT_TIMESTAMP()
        trade.cancelReason ← "agent_termination"
        Database.trades.update(trade)
        canceledCount ← canceledCount + 1
    END FOR

    // Phase 3: Close Positions (if requested)
    closedPositions ← []
    IF closePositions = true AND LENGTH(agent.portfolio.positions) > 0 THEN
        FOR EACH position IN agent.portfolio.positions DO
            TRY
                closeOrder ← {
                    agentId: agentId,
                    symbol: position.symbol,
                    quantity: position.quantity,
                    orderType: "market",
                    side: "sell",
                    reason: "agent_termination"
                }

                result ← ExecuteTrade(agentId, closeOrder, bypassOversight: true)
                closedPositions.append(result)

            CATCH error
                // Log error but continue closing other positions
                ErrorLog.record({
                    agentId: agentId,
                    error: "Failed to close position: " + error.message,
                    position: position
                })
            END TRY
        END FOR
    END IF

    // Phase 4: Finalize Metering Period
    meteringReport ← CretoMetering.finalizeQuota({
        quotaId: agent.meteringQuotaId,
        finalizeReason: "agent_termination"
    })

    // Phase 5: Update Agent Status
    agent.status ← "terminated"
    agent.terminatedAt ← CURRENT_TIMESTAMP()
    agent.terminationReport ← {
        pendingTradesCanceled: canceledCount,
        positionsClosed: LENGTH(closedPositions),
        finalCash: agent.portfolio.cash,
        finalValue: CalculatePortfolioValue(agent.portfolio),
        totalPnL: agent.portfolio.totalPnL,
        meteringUsage: meteringReport.totalUsage
    }

    Database.agents.update(agent)

    // Phase 6: Archive Agent Data
    ArchiveAgent(agent)

    RETURN {
        success: true,
        report: agent.terminationReport
    }
END

TIME COMPLEXITY: O(n + m) where n = pending trades, m = open positions
SPACE COMPLEXITY: O(n + m)
```

---

## 2. Trade Execution Pipeline

### 2.1 Main Trade Execution Flow

```
ALGORITHM: ExecuteTrade
INPUT:
    agentId (string)
    tradeRequest (object):
        - symbol (string)
        - quantity (integer)
        - orderType (enum: market, limit)
        - side (enum: buy, sell)
        - limitPrice (decimal, optional)
    bypassOversight (boolean, default: false)

OUTPUT:
    tradeResult (object) or error

CONSTANTS:
    OVERSIGHT_THRESHOLD = 50000  // $50K
    MAX_RETRY_ATTEMPTS = 3
    TRADE_TIMEOUT_SECONDS = 30

BEGIN
    // Phase 1: Input Validation
    validationResult ← ValidateTradeRequest(agentId, tradeRequest)

    IF validationResult.error THEN
        RETURN error(validationResult.error)
    END IF

    agent ← validationResult.agent
    currentPrice ← validationResult.price

    // Phase 2: Calculate Trade Value
    IF tradeRequest.side = "buy" THEN
        estimatedValue ← tradeRequest.quantity * currentPrice
    ELSE  // sell
        estimatedValue ← tradeRequest.quantity * currentPrice
    END IF

    // Phase 3: Check Budget and Quota
    budgetCheck ← CheckBudgetAndQuota(agentId, estimatedValue, tradeRequest.side)

    IF NOT budgetCheck.allowed THEN
        RETURN error("Budget exceeded: " + budgetCheck.message)
    END IF

    // Phase 4: Assess Risk
    riskAssessment ← AssessTradeRisk(tradeRequest, agent.portfolio, estimatedValue)

    IF riskAssessment.riskScore > 80 AND NOT bypassOversight THEN
        RETURN error("Trade rejected: Risk score too high (" + riskAssessment.riskScore + ")")
    END IF

    // Phase 5: Check Oversight Requirement
    requiresOversight ← (estimatedValue > OVERSIGHT_THRESHOLD) AND NOT bypassOversight

    IF requiresOversight THEN
        oversightResult ← RequestOversightApproval(agentId, tradeRequest, riskAssessment)

        IF oversightResult.status = "pending" THEN
            RETURN {
                success: true,
                status: "pending_approval",
                tradeId: oversightResult.tradeId,
                message: "Trade submitted for human approval"
            }
        ELSE IF oversightResult.status = "rejected" THEN
            RETURN error("Trade rejected by oversight: " + oversightResult.reason)
        END IF

        // If approved, continue to execution
    END IF

    // Phase 6: Execute Trade
    executionResult ← ExecuteOrderWithRetry(tradeRequest, currentPrice)

    IF executionResult.error THEN
        RETURN error("Execution failed: " + executionResult.error)
    END IF

    // Phase 7: Record Metering Event
    meteringEvent ← {
        agentId: agentId,
        eventType: "trade_executed",
        amount: executionResult.fillValue,
        metadata: {
            symbol: tradeRequest.symbol,
            quantity: executionResult.fillQuantity,
            price: executionResult.fillPrice,
            side: tradeRequest.side
        }
    }

    CretoMetering.recordUsage(meteringEvent)

    // Phase 8: Update Portfolio State
    UpdatePortfolio(agentId, executionResult, tradeRequest.side)

    // Phase 9: Broadcast Real-time Update
    BroadcastPortfolioUpdate(agentId, {
        trade: executionResult,
        portfolio: GetPortfolio(agentId)
    })

    // Phase 10: Return Success Result
    RETURN {
        success: true,
        status: "executed",
        tradeId: executionResult.tradeId,
        fillPrice: executionResult.fillPrice,
        fillQuantity: executionResult.fillQuantity,
        fillValue: executionResult.fillValue,
        timestamp: executionResult.timestamp
    }
END

TIME COMPLEXITY: O(1) for single trade execution
SPACE COMPLEXITY: O(1)
```

### 2.2 Trade Request Validation

```
ALGORITHM: ValidateTradeRequest
INPUT: agentId (string), tradeRequest (object)
OUTPUT: validationResult (object) or error

BEGIN
    // Validate agent exists and is active
    agent ← Database.agents.findById(agentId)

    IF agent is null THEN
        RETURN error("Agent not found")
    END IF

    IF agent.status != "active" THEN
        RETURN error("Agent is not active (status: " + agent.status + ")")
    END IF

    // Validate symbol
    IF tradeRequest.symbol NOT IN agent.allowedAssets THEN
        RETURN error("Symbol not in allowed assets list")
    END IF

    IF NOT IsValidSymbol(tradeRequest.symbol) THEN
        RETURN error("Invalid symbol: " + tradeRequest.symbol)
    END IF

    // Validate quantity
    IF tradeRequest.quantity <= 0 THEN
        RETURN error("Quantity must be positive")
    END IF

    IF tradeRequest.quantity > 1000000 THEN
        RETURN error("Quantity exceeds maximum limit")
    END IF

    // Validate order type
    IF tradeRequest.orderType NOT IN ["market", "limit"] THEN
        RETURN error("Invalid order type")
    END IF

    IF tradeRequest.orderType = "limit" AND tradeRequest.limitPrice is null THEN
        RETURN error("Limit price required for limit orders")
    END IF

    // Validate side
    IF tradeRequest.side NOT IN ["buy", "sell"] THEN
        RETURN error("Invalid side (must be 'buy' or 'sell')")
    END IF

    // For sell orders, check position exists
    IF tradeRequest.side = "sell" THEN
        position ← FindPosition(agent.portfolio, tradeRequest.symbol)

        IF position is null THEN
            RETURN error("No position to sell for symbol: " + tradeRequest.symbol)
        END IF

        IF position.quantity < tradeRequest.quantity THEN
            RETURN error("Insufficient quantity (have: " + position.quantity + ", requested: " + tradeRequest.quantity + ")")
        END IF
    END IF

    // Get current price
    priceResult ← MarketData.getCurrentPrice(tradeRequest.symbol)

    IF priceResult.error THEN
        RETURN error("Failed to get current price: " + priceResult.error)
    END IF

    RETURN {
        success: true,
        agent: agent,
        price: priceResult.price
    }
END

TIME COMPLEXITY: O(n) where n = number of positions (for sell validation)
SPACE COMPLEXITY: O(1)
```

### 2.3 Order Execution with Retry

```
ALGORITHM: ExecuteOrderWithRetry
INPUT: tradeRequest (object), currentPrice (decimal)
OUTPUT: executionResult (object) or error

CONSTANTS:
    MAX_ATTEMPTS = 3
    RETRY_DELAY_MS = 1000

BEGIN
    attempt ← 1
    lastError ← null

    WHILE attempt <= MAX_ATTEMPTS DO
        TRY
            // Simulate market order execution
            IF tradeRequest.orderType = "market" THEN
                fillPrice ← currentPrice * RANDOM(0.998, 1.002)  // ±0.2% slippage

            ELSE  // limit order
                IF tradeRequest.side = "buy" AND currentPrice <= tradeRequest.limitPrice THEN
                    fillPrice ← tradeRequest.limitPrice
                ELSE IF tradeRequest.side = "sell" AND currentPrice >= tradeRequest.limitPrice THEN
                    fillPrice ← tradeRequest.limitPrice
                ELSE
                    RETURN error("Limit order not filled (current price: " + currentPrice + ", limit: " + tradeRequest.limitPrice + ")")
                END IF
            END IF

            fillQuantity ← tradeRequest.quantity
            fillValue ← fillQuantity * fillPrice

            // Create trade record
            tradeRecord ← {
                id: GENERATE_UUID(),
                agentId: tradeRequest.agentId,
                symbol: tradeRequest.symbol,
                side: tradeRequest.side,
                quantity: fillQuantity,
                price: fillPrice,
                value: fillValue,
                orderType: tradeRequest.orderType,
                status: "filled",
                timestamp: CURRENT_TIMESTAMP(),
                attempt: attempt
            }

            Database.trades.insert(tradeRecord)

            RETURN {
                success: true,
                tradeId: tradeRecord.id,
                fillPrice: fillPrice,
                fillQuantity: fillQuantity,
                fillValue: fillValue,
                timestamp: tradeRecord.timestamp
            }

        CATCH error
            lastError ← error

            IF attempt < MAX_ATTEMPTS THEN
                ErrorLog.record({
                    message: "Trade execution failed, retrying",
                    attempt: attempt,
                    error: error.message
                })

                SLEEP(RETRY_DELAY_MS)
                attempt ← attempt + 1
            ELSE
                RETURN error("Execution failed after " + MAX_ATTEMPTS + " attempts: " + lastError.message)
            END IF
        END TRY
    END WHILE
END

TIME COMPLEXITY: O(1) per attempt, O(k) total where k = MAX_ATTEMPTS
SPACE COMPLEXITY: O(1)
```

---

## 3. Portfolio Management

### 3.1 Calculate Portfolio Value

```
ALGORITHM: CalculatePortfolioValue
INPUT:
    portfolio (object):
        - cash (decimal)
        - positions (array of {symbol, quantity, avgPrice})
    prices (map: symbol → currentPrice)

OUTPUT:
    portfolioValue (object)

BEGIN
    // Handle empty portfolio
    IF LENGTH(portfolio.positions) = 0 THEN
        RETURN {
            totalValue: portfolio.cash,
            cashValue: portfolio.cash,
            positionsValue: 0,
            dailyPnL: 0,
            totalPnL: 0,
            percentReturn: 0
        }
    END IF

    positionsValue ← 0
    totalCostBasis ← 0

    // Calculate positions value
    FOR EACH position IN portfolio.positions DO
        IF NOT prices.has(position.symbol) THEN
            // Handle missing price data
            ErrorLog.record({
                message: "Missing price for symbol",
                symbol: position.symbol
            })

            // Use last known price or average price as fallback
            currentPrice ← position.avgPrice
        ELSE
            currentPrice ← prices.get(position.symbol)
        END IF

        positionValue ← position.quantity * currentPrice
        positionCost ← position.quantity * position.avgPrice

        positionsValue ← positionsValue + positionValue
        totalCostBasis ← totalCostBasis + positionCost
    END FOR

    // Total portfolio value
    totalValue ← portfolio.cash + positionsValue

    // Calculate P&L
    totalPnL ← positionsValue - totalCostBasis

    // Calculate daily P&L (requires previous day's value)
    previousValue ← GetPreviousPortfolioValue(portfolio.agentId)
    dailyPnL ← totalValue - previousValue

    // Calculate percentage return
    initialInvestment ← GetInitialInvestment(portfolio.agentId)
    percentReturn ← ((totalValue - initialInvestment) / initialInvestment) * 100

    RETURN {
        totalValue: ROUND(totalValue, 2),
        cashValue: ROUND(portfolio.cash, 2),
        positionsValue: ROUND(positionsValue, 2),
        dailyPnL: ROUND(dailyPnL, 2),
        totalPnL: ROUND(totalPnL, 2),
        percentReturn: ROUND(percentReturn, 4),
        positions: EnrichPositions(portfolio.positions, prices)
    }
END

SUBROUTINE: EnrichPositions
INPUT: positions (array), prices (map)
OUTPUT: enrichedPositions (array)

BEGIN
    enriched ← []

    FOR EACH position IN positions DO
        currentPrice ← prices.get(position.symbol) OR position.avgPrice
        marketValue ← position.quantity * currentPrice
        costBasis ← position.quantity * position.avgPrice
        unrealizedPnL ← marketValue - costBasis
        percentChange ← ((currentPrice - position.avgPrice) / position.avgPrice) * 100

        enriched.append({
            symbol: position.symbol,
            quantity: position.quantity,
            avgPrice: ROUND(position.avgPrice, 2),
            currentPrice: ROUND(currentPrice, 2),
            marketValue: ROUND(marketValue, 2),
            costBasis: ROUND(costBasis, 2),
            unrealizedPnL: ROUND(unrealizedPnL, 2),
            percentChange: ROUND(percentChange, 2)
        })
    END FOR

    RETURN enriched
END

TIME COMPLEXITY: O(n) where n = number of positions
SPACE COMPLEXITY: O(n)
```

### 3.2 Update Portfolio After Trade

```
ALGORITHM: UpdatePortfolio
INPUT:
    agentId (string)
    executionResult (object)
    side (enum: buy, sell)

OUTPUT:
    updatedPortfolio (object) or error

BEGIN
    agent ← Database.agents.findById(agentId)
    portfolio ← agent.portfolio

    // Calculate trade impact
    tradeValue ← executionResult.fillQuantity * executionResult.fillPrice

    IF side = "buy" THEN
        // Deduct cash for purchase
        portfolio.cash ← portfolio.cash - tradeValue

        // Update or create position
        existingPosition ← FindPosition(portfolio, executionResult.symbol)

        IF existingPosition is null THEN
            // Create new position
            newPosition ← {
                symbol: executionResult.symbol,
                quantity: executionResult.fillQuantity,
                avgPrice: executionResult.fillPrice,
                openedAt: CURRENT_TIMESTAMP()
            }
            portfolio.positions.append(newPosition)
        ELSE
            // Update existing position (calculate new average price)
            totalQuantity ← existingPosition.quantity + executionResult.fillQuantity
            totalCost ← (existingPosition.quantity * existingPosition.avgPrice) + tradeValue

            existingPosition.quantity ← totalQuantity
            existingPosition.avgPrice ← totalCost / totalQuantity
            existingPosition.updatedAt ← CURRENT_TIMESTAMP()
        END IF

    ELSE  // sell
        // Add cash from sale
        portfolio.cash ← portfolio.cash + tradeValue

        // Update position
        existingPosition ← FindPosition(portfolio, executionResult.symbol)

        IF existingPosition is null THEN
            RETURN error("Position not found for symbol: " + executionResult.symbol)
        END IF

        // Calculate realized P&L
        costBasis ← existingPosition.avgPrice * executionResult.fillQuantity
        realizedPnL ← tradeValue - costBasis
        portfolio.totalPnL ← portfolio.totalPnL + realizedPnL

        // Reduce or close position
        existingPosition.quantity ← existingPosition.quantity - executionResult.fillQuantity

        IF existingPosition.quantity = 0 THEN
            // Remove closed position
            portfolio.positions ← REMOVE(portfolio.positions, existingPosition)
        ELSE
            existingPosition.updatedAt ← CURRENT_TIMESTAMP()
        END IF
    END IF

    // Recalculate portfolio value
    prices ← MarketData.getCurrentPrices(GetSymbols(portfolio.positions))
    portfolioValue ← CalculatePortfolioValue(portfolio, prices)

    portfolio.totalValue ← portfolioValue.totalValue
    portfolio.dailyPnL ← portfolioValue.dailyPnL

    // Persist changes
    Database.agents.updatePortfolio(agentId, portfolio)

    // Store historical snapshot
    PortfolioHistory.snapshot({
        agentId: agentId,
        timestamp: CURRENT_TIMESTAMP(),
        value: portfolio.totalValue,
        cash: portfolio.cash,
        positions: portfolio.positions
    })

    RETURN portfolio
END

TIME COMPLEXITY: O(n) where n = number of positions
SPACE COMPLEXITY: O(n)
```

---

## 4. Budget & Quota Management

### 4.1 Check Budget and Quota

```
ALGORITHM: CheckBudgetAndQuota
INPUT:
    agentId (string)
    requestedAmount (decimal)
    side (enum: buy, sell)

OUTPUT:
    budgetCheckResult (object)

BEGIN
    agent ← Database.agents.findById(agentId)

    // Sell orders don't consume budget
    IF side = "sell" THEN
        RETURN {
            allowed: true,
            remainingBudget: null,
            message: "Sell order does not consume budget"
        }
    END IF

    // Check current usage from creto-metering
    usageResult ← CretoMetering.getUsage({
        quotaId: agent.meteringQuotaId,
        billingPeriod: "current"
    })

    IF usageResult.error THEN
        RETURN {
            allowed: false,
            error: "Failed to retrieve usage: " + usageResult.error
        }
    END IF

    currentUsage ← usageResult.totalUsage
    quotaLimit ← agent.monthlyBudget

    // Check if new trade would exceed quota
    projectedUsage ← currentUsage + requestedAmount

    IF projectedUsage > quotaLimit THEN
        overage ← projectedUsage - quotaLimit

        RETURN {
            allowed: false,
            message: "Budget exceeded (overage: $" + overage + ")",
            currentUsage: currentUsage,
            quotaLimit: quotaLimit,
            requestedAmount: requestedAmount,
            remainingBudget: quotaLimit - currentUsage
        }
    END IF

    // Check warning thresholds
    utilizationPercent ← (projectedUsage / quotaLimit) * 100
    warningLevel ← null

    IF utilizationPercent >= 90 THEN
        warningLevel ← "critical"
        EmitBudgetWarning(agentId, "critical", utilizationPercent)
    ELSE IF utilizationPercent >= 80 THEN
        warningLevel ← "warning"
        EmitBudgetWarning(agentId, "warning", utilizationPercent)
    END IF

    RETURN {
        allowed: true,
        currentUsage: currentUsage,
        quotaLimit: quotaLimit,
        requestedAmount: requestedAmount,
        remainingBudget: quotaLimit - projectedUsage,
        utilizationPercent: ROUND(utilizationPercent, 2),
        warningLevel: warningLevel
    }
END

SUBROUTINE: EmitBudgetWarning
INPUT: agentId (string), level (string), utilization (decimal)
OUTPUT: void

BEGIN
    notification ← {
        agentId: agentId,
        type: "budget_warning",
        level: level,
        message: "Budget utilization at " + ROUND(utilization, 1) + "%",
        timestamp: CURRENT_TIMESTAMP()
    }

    // Send to notification service
    NotificationService.send(notification)

    // Log event
    AuditLog.record({
        event: "budget_warning",
        agentId: agentId,
        level: level,
        utilization: utilization
    })
END

TIME COMPLEXITY: O(1)
SPACE COMPLEXITY: O(1)
```

---

## 5. Risk Assessment

### 5.1 Trade Risk Analysis

```
ALGORITHM: AssessTradeRisk
INPUT:
    trade (object):
        - symbol (string)
        - quantity (integer)
        - side (enum: buy, sell)
    portfolio (object)
    tradeValue (decimal)

OUTPUT:
    riskAssessment (object)

BEGIN
    riskScore ← 0
    riskFactors ← []

    // Factor 1: Position Size Risk (0-30 points)
    positionSizeRisk ← CalculatePositionSizeRisk(tradeValue, portfolio.totalValue)
    riskScore ← riskScore + positionSizeRisk.score

    IF positionSizeRisk.score > 20 THEN
        riskFactors.append({
            factor: "position_size",
            score: positionSizeRisk.score,
            message: "Trade represents " + ROUND(positionSizeRisk.percent, 1) + "% of portfolio"
        })
    END IF

    // Factor 2: Concentration Risk (0-25 points)
    concentrationRisk ← CalculateConcentrationRisk(trade.symbol, portfolio, tradeValue)
    riskScore ← riskScore + concentrationRisk.score

    IF concentrationRisk.score > 15 THEN
        riskFactors.append({
            factor: "concentration",
            score: concentrationRisk.score,
            message: "Symbol concentration would be " + ROUND(concentrationRisk.percent, 1) + "%"
        })
    END IF

    // Factor 3: Sector Exposure Risk (0-20 points)
    sectorRisk ← CalculateSectorRisk(trade.symbol, portfolio, tradeValue)
    riskScore ← riskScore + sectorRisk.score

    IF sectorRisk.score > 10 THEN
        riskFactors.append({
            factor: "sector_exposure",
            score: sectorRisk.score,
            message: "Sector exposure would be " + ROUND(sectorRisk.percent, 1) + "%"
        })
    END IF

    // Factor 4: Volatility Risk (0-15 points)
    volatilityRisk ← CalculateVolatilityRisk(trade.symbol)
    riskScore ← riskScore + volatilityRisk.score

    IF volatilityRisk.score > 10 THEN
        riskFactors.append({
            factor: "volatility",
            score: volatilityRisk.score,
            message: "High volatility symbol (30-day: " + ROUND(volatilityRisk.volatility, 2) + "%)"
        })
    END IF

    // Factor 5: Liquidity Risk (0-10 points)
    liquidityRisk ← CalculateLiquidityRisk(trade.symbol, trade.quantity)
    riskScore ← riskScore + liquidityRisk.score

    IF liquidityRisk.score > 5 THEN
        riskFactors.append({
            factor: "liquidity",
            score: liquidityRisk.score,
            message: "Low liquidity for trade size"
        })
    END IF

    // Determine oversight requirement
    requiresOversight ← (riskScore > 60) OR (tradeValue > 50000)

    // Determine risk level
    riskLevel ← DetermineRiskLevel(riskScore)

    RETURN {
        riskScore: ROUND(riskScore, 1),
        riskLevel: riskLevel,
        requiresOversight: requiresOversight,
        factors: riskFactors,
        recommendation: GenerateRiskRecommendation(riskScore, riskFactors)
    }
END

SUBROUTINE: CalculatePositionSizeRisk
INPUT: tradeValue (decimal), portfolioValue (decimal)
OUTPUT: positionSizeRisk (object)

BEGIN
    IF portfolioValue = 0 THEN
        RETURN {score: 30, percent: 100}
    END IF

    percentOfPortfolio ← (tradeValue / portfolioValue) * 100

    // Score based on position size
    IF percentOfPortfolio > 30 THEN
        score ← 30
    ELSE IF percentOfPortfolio > 20 THEN
        score ← 25
    ELSE IF percentOfPortfolio > 10 THEN
        score ← 15
    ELSE IF percentOfPortfolio > 5 THEN
        score ← 5
    ELSE
        score ← 0
    END IF

    RETURN {
        score: score,
        percent: percentOfPortfolio
    }
END

SUBROUTINE: CalculateConcentrationRisk
INPUT: symbol (string), portfolio (object), tradeValue (decimal)
OUTPUT: concentrationRisk (object)

BEGIN
    // Find existing position
    existingPosition ← FindPosition(portfolio, symbol)

    currentPrice ← MarketData.getCurrentPrice(symbol)
    currentValue ← 0

    IF existingPosition is not null THEN
        currentValue ← existingPosition.quantity * currentPrice
    END IF

    projectedValue ← currentValue + tradeValue
    projectedPercent ← (projectedValue / portfolio.totalValue) * 100

    // Score based on concentration
    IF projectedPercent > 40 THEN
        score ← 25
    ELSE IF projectedPercent > 30 THEN
        score ← 20
    ELSE IF projectedPercent > 20 THEN
        score ← 10
    ELSE
        score ← 0
    END IF

    RETURN {
        score: score,
        percent: projectedPercent
    }
END

SUBROUTINE: CalculateSectorRisk
INPUT: symbol (string), portfolio (object), tradeValue (decimal)
OUTPUT: sectorRisk (object)

BEGIN
    // Get sector for symbol
    symbolSector ← MarketData.getSector(symbol)

    // Calculate current sector exposure
    sectorValue ← 0

    FOR EACH position IN portfolio.positions DO
        positionSector ← MarketData.getSector(position.symbol)

        IF positionSector = symbolSector THEN
            currentPrice ← MarketData.getCurrentPrice(position.symbol)
            sectorValue ← sectorValue + (position.quantity * currentPrice)
        END IF
    END FOR

    projectedSectorValue ← sectorValue + tradeValue
    sectorPercent ← (projectedSectorValue / portfolio.totalValue) * 100

    // Score based on sector concentration
    IF sectorPercent > 50 THEN
        score ← 20
    ELSE IF sectorPercent > 40 THEN
        score ← 15
    ELSE IF sectorPercent > 30 THEN
        score ← 10
    ELSE
        score ← 0
    END IF

    RETURN {
        score: score,
        percent: sectorPercent,
        sector: symbolSector
    }
END

SUBROUTINE: CalculateVolatilityRisk
INPUT: symbol (string)
OUTPUT: volatilityRisk (object)

BEGIN
    // Get 30-day historical volatility
    volatility ← MarketData.getHistoricalVolatility(symbol, days: 30)

    // Score based on volatility
    IF volatility > 50 THEN
        score ← 15
    ELSE IF volatility > 30 THEN
        score ← 10
    ELSE IF volatility > 20 THEN
        score ← 5
    ELSE
        score ← 0
    END IF

    RETURN {
        score: score,
        volatility: volatility
    }
END

SUBROUTINE: DetermineRiskLevel
INPUT: riskScore (decimal)
OUTPUT: riskLevel (string)

BEGIN
    IF riskScore >= 70 THEN
        RETURN "critical"
    ELSE IF riskScore >= 50 THEN
        RETURN "high"
    ELSE IF riskScore >= 30 THEN
        RETURN "medium"
    ELSE
        RETURN "low"
    END IF
END

SUBROUTINE: GenerateRiskRecommendation
INPUT: riskScore (decimal), riskFactors (array)
OUTPUT: recommendation (string)

BEGIN
    IF riskScore >= 70 THEN
        RETURN "Trade not recommended. Multiple high-risk factors detected. Consider reducing position size or diversifying."
    ELSE IF riskScore >= 50 THEN
        RETURN "Proceed with caution. High risk trade. Human oversight recommended."
    ELSE IF riskScore >= 30 THEN
        RETURN "Moderate risk. Review risk factors before proceeding."
    ELSE
        RETURN "Low risk trade. Proceed as planned."
    END IF
END

TIME COMPLEXITY: O(n) where n = number of positions
SPACE COMPLEXITY: O(k) where k = number of risk factors
```

---

## 6. Oversight Integration

### 6.1 Request Oversight Approval

```
ALGORITHM: RequestOversightApproval
INPUT:
    agentId (string)
    trade (object)
    riskAssessment (object)

OUTPUT:
    oversightResult (object) or error

BEGIN
    // Create approval request
    approvalRequest ← {
        id: GENERATE_UUID(),
        agentId: agentId,
        requestType: "trade_approval",
        status: "pending",
        createdAt: CURRENT_TIMESTAMP(),
        expiresAt: CURRENT_TIMESTAMP() + HOURS(24),
        trade: {
            symbol: trade.symbol,
            quantity: trade.quantity,
            side: trade.side,
            orderType: trade.orderType,
            estimatedValue: trade.quantity * MarketData.getCurrentPrice(trade.symbol)
        },
        riskAssessment: {
            riskScore: riskAssessment.riskScore,
            riskLevel: riskAssessment.riskLevel,
            factors: riskAssessment.factors,
            recommendation: riskAssessment.recommendation
        },
        context: {
            currentPortfolio: GetPortfolio(agentId),
            agentBudget: GetAgent(agentId).monthlyBudget,
            budgetUtilization: GetBudgetUtilization(agentId)
        }
    }

    // Submit to creto-oversight
    oversightResult ← CretoOversight.submitRequest({
        requestId: approvalRequest.id,
        severity: MapRiskLevelToSeverity(riskAssessment.riskLevel),
        title: "Trade Approval: " + trade.symbol + " (" + trade.side + ")",
        description: GenerateApprovalDescription(approvalRequest),
        data: approvalRequest,
        approvers: GetApprovers(agentId),
        autoApprove: false,
        timeout: 24 * 60 * 60  // 24 hours in seconds
    })

    IF oversightResult.error THEN
        RETURN error("Failed to submit oversight request: " + oversightResult.error)
    END IF

    // Store pending trade
    Database.pendingTrades.insert({
        tradeId: approvalRequest.id,
        agentId: agentId,
        oversightRequestId: oversightResult.requestId,
        trade: trade,
        status: "pending_approval",
        createdAt: CURRENT_TIMESTAMP()
    })

    // Notify agent owner
    NotificationService.send({
        userId: GetAgentOwner(agentId),
        type: "approval_required",
        message: "Trade approval required for " + trade.symbol,
        data: approvalRequest
    })

    RETURN {
        status: "pending",
        tradeId: approvalRequest.id,
        oversightRequestId: oversightResult.requestId,
        expiresAt: approvalRequest.expiresAt
    }
END

SUBROUTINE: GenerateApprovalDescription
INPUT: approvalRequest (object)
OUTPUT: description (string)

BEGIN
    trade ← approvalRequest.trade
    risk ← approvalRequest.riskAssessment

    description ← "Trade Details:\n"
    description ← description + "- Action: " + UPPERCASE(trade.side) + " " + trade.quantity + " shares of " + trade.symbol + "\n"
    description ← description + "- Estimated Value: $" + FORMAT_CURRENCY(trade.estimatedValue) + "\n"
    description ← description + "- Order Type: " + trade.orderType + "\n\n"

    description ← description + "Risk Assessment:\n"
    description ← description + "- Risk Score: " + risk.riskScore + "/100 (" + UPPERCASE(risk.riskLevel) + ")\n"
    description ← description + "- Recommendation: " + risk.recommendation + "\n\n"

    IF LENGTH(risk.factors) > 0 THEN
        description ← description + "Risk Factors:\n"
        FOR EACH factor IN risk.factors DO
            description ← description + "- " + factor.message + " (score: " + factor.score + ")\n"
        END FOR
    END IF

    RETURN description
END

SUBROUTINE: MapRiskLevelToSeverity
INPUT: riskLevel (string)
OUTPUT: severity (string)

BEGIN
    CASE riskLevel OF
        "critical": RETURN "critical"
        "high": RETURN "high"
        "medium": RETURN "medium"
        DEFAULT: RETURN "low"
    END CASE
END

TIME COMPLEXITY: O(k) where k = number of risk factors
SPACE COMPLEXITY: O(1)
```

---

## 7. Real-time Updates

### 7.1 Broadcast Portfolio Update

```
ALGORITHM: BroadcastPortfolioUpdate
INPUT:
    agentId (string)
    updateData (object):
        - trade (object, optional)
        - portfolio (object)

OUTPUT: void

BEGIN
    // Prepare update message
    updateMessage ← {
        type: "portfolio_update",
        agentId: agentId,
        timestamp: CURRENT_TIMESTAMP(),
        portfolio: {
            totalValue: updateData.portfolio.totalValue,
            cash: updateData.portfolio.cash,
            positionsValue: updateData.portfolio.positionsValue,
            dailyPnL: updateData.portfolio.dailyPnL,
            totalPnL: updateData.portfolio.totalPnL,
            positions: updateData.portfolio.positions
        }
    }

    IF updateData.trade is not null THEN
        updateMessage.trade ← {
            symbol: updateData.trade.symbol,
            side: updateData.trade.side,
            quantity: updateData.trade.quantity,
            price: updateData.trade.price,
            value: updateData.trade.value,
            timestamp: updateData.trade.timestamp
        }
    END IF

    // Broadcast via WebSocket
    WebSocketService.broadcast({
        channel: "agent:" + agentId,
        event: "portfolio_update",
        data: updateMessage
    })

    // Also broadcast to global trading channel
    WebSocketService.broadcast({
        channel: "trading:all",
        event: "agent_activity",
        data: {
            agentId: agentId,
            agentName: GetAgent(agentId).name,
            activity: "trade_executed",
            symbol: updateData.trade?.symbol,
            timestamp: CURRENT_TIMESTAMP()
        }
    })

    // Store update in time-series database for historical tracking
    TimeSeriesDB.insert({
        metric: "portfolio_value",
        agentId: agentId,
        timestamp: CURRENT_TIMESTAMP(),
        value: updateData.portfolio.totalValue,
        tags: {
            dailyPnL: updateData.portfolio.dailyPnL,
            positionCount: LENGTH(updateData.portfolio.positions)
        }
    })
END

TIME COMPLEXITY: O(1)
SPACE COMPLEXITY: O(n) where n = number of positions
```

---

## 8. Complexity Summary

| Algorithm | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| CreateTradingAgent | O(1) | O(1) | Single database insert |
| TerminateTradingAgent | O(n + m) | O(n + m) | n = pending trades, m = positions |
| ExecuteTrade | O(1) | O(1) | Single trade execution |
| ValidateTradeRequest | O(n) | O(1) | n = positions (for sell validation) |
| ExecuteOrderWithRetry | O(k) | O(1) | k = MAX_ATTEMPTS |
| CalculatePortfolioValue | O(n) | O(n) | n = positions |
| UpdatePortfolio | O(n) | O(n) | n = positions |
| CheckBudgetAndQuota | O(1) | O(1) | Single metering query |
| AssessTradeRisk | O(n) | O(k) | n = positions, k = risk factors |
| RequestOversightApproval | O(k) | O(1) | k = risk factors |
| BroadcastPortfolioUpdate | O(1) | O(n) | n = positions |

---

## 9. Integration Points

### 9.1 Creto-Metering Integration

**Key Functions:**
- `CretoMetering.createQuota()` - Initialize agent budget quota
- `CretoMetering.recordUsage()` - Track trade execution costs
- `CretoMetering.getUsage()` - Check current budget utilization
- `CretoMetering.finalizeQuota()` - Close quota on agent termination

**Data Flow:**
```
Agent Creation → createQuota() → quotaId stored in agent record
Trade Execution → recordUsage() → usage tracked against quota
Budget Check → getUsage() → current usage vs. limit
Agent Termination → finalizeQuota() → final usage report
```

### 9.2 Creto-Oversight Integration

**Key Functions:**
- `CretoOversight.submitRequest()` - Submit trade for human approval
- `CretoOversight.getStatus()` - Check approval status
- `CretoOversight.handleCallback()` - Process approval/rejection

**Approval Triggers:**
- Trade value > $50K
- Risk score > 60
- Manual override by agent owner

**Approval Flow:**
```
High-value Trade → submitRequest() → Pending Status
Human Reviews → Approve/Reject → Callback
Approved → Execute Trade → Update Portfolio
Rejected → Cancel Trade → Notify Agent
```

---

## 10. Error Handling Patterns

### General Error Strategy
```
TRY
    // Primary operation
    result ← Execute()

    // Validation
    IF result has errors THEN
        THROW ValidationError(result.errors)
    END IF

    RETURN result

CATCH ValidationError as e
    LogError("Validation failed", e)
    RETURN {error: e.message, type: "validation"}

CATCH DatabaseError as e
    LogError("Database operation failed", e)
    RollbackTransaction()
    RETURN {error: "Database error", type: "database"}

CATCH ExternalServiceError as e
    LogError("External service failed", e)
    IF IsRetryable(e) THEN
        RetryWithBackoff()
    ELSE
        RETURN {error: e.message, type: "external_service"}
    END IF

CATCH UnknownError as e
    LogCriticalError("Unexpected error", e)
    AlertEngineering(e)
    RETURN {error: "Internal error", type: "unknown"}
END TRY
```

---

## Conclusion

This pseudocode specification provides a complete algorithmic foundation for the Trading Demo, with clear integration points for creto-metering (budget tracking) and creto-oversight (approval workflows). All algorithms are designed for efficiency, reliability, and maintainability with proper error handling and complexity analysis.

**Next Phase:** Architecture design will define system components, database schemas, API contracts, and deployment topology.
