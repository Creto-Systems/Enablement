# SPARC Phase 2: Pseudocode - PSA Demo

## Overview

This document contains the core algorithms and logic flows for the PSA Demo system. Each algorithm is presented in structured pseudocode with complexity analysis and implementation notes.

---

## 1. Project Scheduling Algorithm

### Purpose
Optimize project timelines considering resource availability, dependencies, and constraints.

### Algorithm: Critical Path Method (CPM) with Resource Leveling

```pseudocode
FUNCTION scheduleProject(project, resources, constraints)
  INPUT:
    project: Project with tasks, dependencies, durations
    resources: Available consultant pool with availability
    constraints: Business rules (max hours/week, holidays, etc.)
  OUTPUT:
    schedule: Optimized project schedule with resource assignments

  ALGORITHM:
    // Step 1: Build dependency graph
    graph = buildDependencyGraph(project.tasks)

    // Step 2: Calculate Critical Path
    criticalPath = calculateCriticalPath(graph)
    FOR EACH task IN criticalPath
      task.priority = "HIGH"
      task.slack = 0
    END FOR

    // Step 3: Calculate Early/Late Start Times
    FOR EACH task IN topologicalSort(graph)
      task.earlyStart = MAX(predecessors.earlyFinish)
      task.earlyFinish = task.earlyStart + task.duration
    END FOR

    FOR EACH task IN reverseTopologicalSort(graph)
      task.lateFinish = MIN(successors.lateStart)
      task.lateStart = task.lateFinish - task.duration
      task.slack = task.lateStart - task.earlyStart
    END FOR

    // Step 4: Resource Allocation with Leveling
    schedule = []
    FOR EACH task IN sortByPriority(project.tasks)
      // Find best resource match
      candidates = findEligibleResources(task, resources)
      bestMatch = selectOptimalResource(candidates, task)

      // Check for conflicts
      IF hasConflict(bestMatch, task.earlyStart, task.duration)
        // Try to shift task within slack time
        newStart = findNextAvailableSlot(bestMatch, task.duration, task.lateStart)
        IF newStart <= task.lateStart THEN
          task.scheduledStart = newStart
        ELSE
          // Need to delay or find alternative resource
          alternativeResource = findNextBestResource(candidates, task)
          task.assignedResource = alternativeResource
          task.scheduledStart = task.earlyStart
        END IF
      ELSE
        task.scheduledStart = task.earlyStart
        task.assignedResource = bestMatch
      END IF

      schedule.append(task)
    END FOR

    // Step 5: Validate and optimize
    schedule = optimizeResourceUtilization(schedule, resources)

    RETURN schedule
END FUNCTION

FUNCTION calculateCriticalPath(graph)
  // Use forward and backward pass to find longest path
  longestPaths = []
  FOR EACH path FROM start TO end IN graph
    pathLength = SUM(task.duration FOR task IN path)
    longestPaths.append({path: path, length: pathLength})
  END FOR

  criticalPath = MAX(longestPaths BY length)
  RETURN criticalPath.path
END FUNCTION

FUNCTION optimizeResourceUtilization(schedule, resources)
  // Minimize resource idle time and over-allocation
  FOR EACH resource IN resources
    assignments = schedule.filter(task => task.assignedResource == resource)

    // Sort by scheduled start
    assignments.sort(BY scheduledStart)

    // Try to compress gaps
    FOR i = 0 TO assignments.length - 2
      current = assignments[i]
      next = assignments[i + 1]

      gap = next.scheduledStart - (current.scheduledStart + current.duration)
      IF gap > 0 AND next.slack >= gap THEN
        // Shift next task earlier to reduce gap
        next.scheduledStart -= gap
      END IF
    END FOR
  END FOR

  RETURN schedule
END FUNCTION
```

**Complexity**: O(V + E + R*T) where V=vertices, E=edges, R=resources, T=tasks
**Space Complexity**: O(V + E)

---

## 2. Resource Allocation Optimization

### Purpose
Match consultants to projects based on skills, availability, cost, and utilization targets.

### Algorithm: Weighted Skill Matching with Constraint Satisfaction

```pseudocode
FUNCTION allocateResources(project, resourcePool, constraints)
  INPUT:
    project: Project requiring resource allocation
    resourcePool: Available consultants with skills and rates
    constraints: Utilization targets, budget limits, skill requirements
  OUTPUT:
    allocation: Optimal resource assignments with scores

  ALGORITHM:
    requiredSkills = project.requiredSkills
    budget = project.budget
    duration = project.duration

    // Step 1: Filter eligible resources
    eligibleResources = []
    FOR EACH resource IN resourcePool
      skillMatch = calculateSkillMatch(resource.skills, requiredSkills)
      availability = checkAvailability(resource, project.startDate, duration)

      IF skillMatch >= constraints.minSkillMatch AND availability
        eligibleResources.append(resource)
      END IF
    END FOR

    IF eligibleResources.isEmpty()
      RETURN NO_SUITABLE_RESOURCES_FOUND
    END IF

    // Step 2: Score each resource
    resourceScores = []
    FOR EACH resource IN eligibleResources
      score = calculateAllocationScore(resource, project, constraints)
      resourceScores.append({resource: resource, score: score})
    END FOR

    // Step 3: Select optimal allocation
    resourceScores.sort(BY score DESCENDING)

    allocation = []
    remainingBudget = budget
    remainingHours = project.estimatedHours

    FOR EACH item IN resourceScores
      resource = item.resource

      // Calculate optimal allocation percentage
      hoursToAllocate = MIN(
        remainingHours,
        resource.availableHours,
        remainingBudget / resource.rate
      )

      IF hoursToAllocate >= constraints.minHoursPerResource THEN
        allocation.append({
          resource: resource,
          hours: hoursToAllocate,
          utilizationImpact: hoursToAllocate / resource.capacity,
          cost: hoursToAllocate * resource.rate
        })

        remainingBudget -= (hoursToAllocate * resource.rate)
        remainingHours -= hoursToAllocate
      END IF

      IF remainingHours <= 0 OR remainingBudget <= 0
        BREAK
      END IF
    END FOR

    // Step 4: Validate allocation
    IF remainingHours > project.estimatedHours * 0.1 THEN
      // More than 10% unallocated
      RETURN INSUFFICIENT_RESOURCES
    END IF

    RETURN allocation
END FUNCTION

FUNCTION calculateAllocationScore(resource, project, constraints)
  // Multi-factor scoring system
  weights = {
    skillMatch: 0.35,
    costEfficiency: 0.25,
    utilizationOptimization: 0.20,
    availability: 0.15,
    pastPerformance: 0.05
  }

  // Skill Match Score (0-100)
  skillScore = calculateSkillMatch(resource.skills, project.requiredSkills)

  // Cost Efficiency Score (0-100)
  // Lower cost = higher score, but penalize if too junior
  avgMarketRate = getAverageMarketRate(project.requiredSkills)
  costRatio = resource.rate / avgMarketRate
  costScore = 100 * (1 / (1 + ABS(costRatio - 1)))

  // Utilization Optimization Score (0-100)
  // Prefer resources below target utilization
  targetUtilization = constraints.targetUtilization // e.g., 0.75
  currentUtilization = resource.currentUtilization
  IF currentUtilization < targetUtilization THEN
    utilizationScore = 100
  ELSE
    utilizationScore = 100 - (currentUtilization - targetUtilization) * 200
  END IF
  utilizationScore = MAX(0, utilizationScore)

  // Availability Score (0-100)
  availabilityRatio = resource.availableHours / project.estimatedHours
  availabilityScore = MIN(100, availabilityRatio * 100)

  // Past Performance Score (0-100)
  performanceScore = resource.performanceRating * 20 // 5-star rating

  // Calculate weighted score
  totalScore =
    skillScore * weights.skillMatch +
    costScore * weights.costEfficiency +
    utilizationScore * weights.utilizationOptimization +
    availabilityScore * weights.availability +
    performanceScore * weights.pastPerformance

  RETURN totalScore
END FUNCTION

FUNCTION calculateSkillMatch(resourceSkills, requiredSkills)
  // Weighted skill matching algorithm
  totalWeight = SUM(skill.importance FOR skill IN requiredSkills)
  matchedWeight = 0

  FOR EACH requiredSkill IN requiredSkills
    resourceSkill = resourceSkills.find(s => s.name == requiredSkill.name)

    IF resourceSkill EXISTS THEN
      // Calculate proficiency match
      proficiencyGap = ABS(resourceSkill.level - requiredSkill.level)
      proficiencyScore = MAX(0, 1 - proficiencyGap / 10) // 0-10 scale

      matchedWeight += requiredSkill.importance * proficiencyScore
    END IF
  END FOR

  matchScore = (matchedWeight / totalWeight) * 100
  RETURN matchScore
END FUNCTION
```

**Complexity**: O(R * S) where R=resources, S=skills
**Optimization**: Use caching for market rates and performance ratings

---

## 3. Time Entry Validation & Approval

### Purpose
Validate time entries against business rules and automate approval workflows.

### Algorithm: Rule-Based Validation with Smart Approval

```pseudocode
FUNCTION validateTimeEntry(entry, project, user)
  INPUT:
    entry: TimeEntry to validate
    project: Associated project
    user: User submitting entry
  OUTPUT:
    validationResult: {isValid, errors[], warnings[]}

  ALGORITHM:
    errors = []
    warnings = []

    // Rule 1: Basic field validation
    IF entry.hours <= 0 OR entry.hours > 24
      errors.append("Invalid hours: must be between 0 and 24")
    END IF

    IF entry.date > TODAY()
      errors.append("Cannot log time in the future")
    END IF

    IF entry.date < project.startDate OR entry.date > project.endDate
      warnings.append("Entry date outside project timeline")
    END IF

    // Rule 2: User authorization check
    IF NOT isUserAssignedToProject(user, project)
      errors.append("User not assigned to this project")
    END IF

    // Rule 3: Duplicate entry check
    existingEntries = getTimeEntries(user, project, entry.date)
    FOR EACH existing IN existingEntries
      IF existing.taskId == entry.taskId AND existing.id != entry.id
        errors.append("Duplicate entry for this task on this date")
      END IF
    END FOR

    // Rule 4: Daily hour limit check
    dailyTotal = SUM(e.hours FOR e IN existingEntries) + entry.hours
    IF dailyTotal > 16
      warnings.append("Daily hours exceed 16 - verify accuracy")
    END IF

    // Rule 5: Budget validation
    IF project.budget.totalHours IS_SET THEN
      usedHours = getTotalProjectHours(project)
      IF (usedHours + entry.hours) > project.budget.totalHours
        warnings.append("Entry will exceed project budget")
      END IF
    END IF

    // Rule 6: Billable validation
    IF entry.billable AND NOT project.allowsBillableTime
      errors.append("Project does not allow billable time")
    END IF

    // Rule 7: Activity code validation
    validActivityCodes = project.activityCodes
    IF NOT validActivityCodes.contains(entry.activityCode)
      errors.append("Invalid activity code for this project")
    END IF

    // Rule 8: Cutoff date validation
    cutoffDate = getPayrollCutoffDate(entry.date)
    IF entry.date < cutoffDate AND entry.status == 'Draft'
      warnings.append("Entry is for a closed payroll period")
    END IF

    isValid = errors.isEmpty()

    RETURN {
      isValid: isValid,
      errors: errors,
      warnings: warnings
    }
END FUNCTION

FUNCTION autoApproveTimeEntries(entries, approvalRules)
  INPUT:
    entries: TimeEntry[] pending approval
    approvalRules: Configured auto-approval rules
  OUTPUT:
    approvalResults: {autoApproved[], requiresReview[]}

  ALGORITHM:
    autoApproved = []
    requiresReview = []

    FOR EACH entry IN entries
      shouldAutoApprove = TRUE
      reasons = []

      // Rule 1: Amount threshold
      IF entry.billing.amount > approvalRules.maxAutoApproveAmount
        shouldAutoApprove = FALSE
        reasons.append("Exceeds auto-approval amount threshold")
      END IF

      // Rule 2: User trust score
      userTrustScore = getUserTrustScore(entry.userId)
      IF userTrustScore < approvalRules.minTrustScore
        shouldAutoApprove = FALSE
        reasons.append("User trust score below threshold")
      END IF

      // Rule 3: Historical accuracy
      historicalAccuracy = getUserHistoricalAccuracy(entry.userId)
      IF historicalAccuracy < approvalRules.minHistoricalAccuracy
        shouldAutoApprove = FALSE
        reasons.append("Historical accuracy below threshold")
      END IF

      // Rule 4: Pattern detection (anomaly detection)
      isAnomalous = detectAnomalousEntry(entry, getUserHistoricalPattern(entry.userId))
      IF isAnomalous
        shouldAutoApprove = FALSE
        reasons.append("Entry deviates from normal pattern")
      END IF

      // Rule 5: Weekend/holiday check
      IF isWeekendOrHoliday(entry.date) AND entry.hours > 4
        shouldAutoApprove = FALSE
        reasons.append("Weekend/holiday hours exceed threshold")
      END IF

      // Rule 6: Late submission
      daysSinceWork = TODAY() - entry.date
      IF daysSinceWork > approvalRules.maxDaysSinceWork
        shouldAutoApprove = FALSE
        reasons.append("Submitted too long after work date")
      END IF

      IF shouldAutoApprove THEN
        entry.status = 'Approved'
        entry.approver = {
          userId: 'SYSTEM_AUTO_APPROVE',
          approvedAt: NOW(),
          comments: 'Auto-approved based on rules'
        }
        autoApproved.append(entry)
      ELSE
        requiresReview.append({
          entry: entry,
          reasons: reasons
        })
      END IF
    END FOR

    RETURN {
      autoApproved: autoApproved,
      requiresReview: requiresReview
    }
END FUNCTION

FUNCTION detectAnomalousEntry(entry, historicalPattern)
  // Simple statistical anomaly detection
  mean = historicalPattern.averageHoursPerDay
  stdDev = historicalPattern.standardDeviation

  // Z-score calculation
  zScore = ABS(entry.hours - mean) / stdDev

  // Consider anomalous if more than 2 standard deviations
  isAnomalous = zScore > 2

  RETURN isAnomalous
END FUNCTION
```

**Complexity**: O(E) where E=entries to validate
**Database Queries**: Optimized with indexed lookups

---

## 4. Invoice Generation Logic

### Purpose
Generate accurate invoices from approved time entries with rate cards and discounts.

### Algorithm: Invoice Generation with Tiered Pricing

```pseudocode
FUNCTION generateInvoice(projectId, billingPeriod)
  INPUT:
    projectId: Project to invoice
    billingPeriod: {startDate, endDate}
  OUTPUT:
    invoice: Generated invoice object

  ALGORITHM:
    project = getProject(projectId)
    client = getClient(project.clientId)

    // Step 1: Gather approved time entries
    timeEntries = getApprovedTimeEntries(projectId, billingPeriod)
    IF timeEntries.isEmpty()
      RETURN NO_BILLABLE_TIME_FOUND
    END IF

    // Step 2: Group entries by rate category
    entriesByRate = groupBy(timeEntries, entry => {
      resource: entry.userId,
      activityCode: entry.activityCode
    })

    lineItems = []
    subtotal = 0

    // Step 3: Calculate line items with rate cards
    FOR EACH group IN entriesByRate
      resource = getResource(group.resource)
      rateCard = getRateCard(project, resource, group.activityCode)

      totalHours = SUM(entry.hours FOR entry IN group.entries)

      // Apply tiered pricing if applicable
      baseAmount = calculateTieredPricing(totalHours, rateCard)

      // Apply discounts
      discountAmount = applyDiscounts(baseAmount, client, project, totalHours)

      netAmount = baseAmount - discountAmount

      lineItem = {
        description: formatLineItemDescription(resource, group.activityCode, totalHours),
        quantity: totalHours,
        unitPrice: baseAmount / totalHours,
        discount: discountAmount,
        amount: netAmount,
        timeEntryIds: group.entries.map(e => e.id)
      }

      lineItems.append(lineItem)
      subtotal += netAmount

      // Mark time entries as invoiced
      FOR EACH entry IN group.entries
        entry.status = 'Invoiced'
        entry.billing.invoiceId = PENDING_INVOICE_ID
      END FOR
    END FOR

    // Step 4: Calculate tax
    taxRate = getTaxRate(client.billingInfo, project)
    tax = subtotal * taxRate
    total = subtotal + tax

    // Step 5: Add metering events
    meteringEvents = getMeteringEvents(projectId, billingPeriod)
    FOR EACH event IN meteringEvents
      lineItem = {
        description: formatMeteringDescription(event),
        quantity: event.quantity,
        unitPrice: event.unitPrice,
        discount: 0,
        amount: event.totalAmount
      }
      lineItems.append(lineItem)
      total += event.totalAmount
    END FOR

    // Step 6: Create invoice
    invoice = {
      id: generateUUID(),
      invoiceNumber: generateInvoiceNumber(client, project),
      clientId: client.id,
      projectId: project.id,
      status: 'Draft',
      dateIssued: TODAY(),
      dateDue: TODAY() + client.billingInfo.paymentTerms,
      lineItems: lineItems,
      subtotal: subtotal,
      tax: tax,
      total: total,
      currency: project.budget.currency,
      metering: {
        meteringPeriodStart: billingPeriod.startDate,
        meteringPeriodEnd: billingPeriod.endDate,
        usageMetrics: meteringEvents
      }
    }

    // Step 7: Save invoice
    savedInvoice = saveInvoice(invoice)

    // Step 8: Update time entries with invoice ID
    FOR EACH lineItem IN lineItems
      IF lineItem.timeEntryIds IS_SET THEN
        updateTimeEntries(lineItem.timeEntryIds, {
          billing: {invoiceId: savedInvoice.id}
        })
      END IF
    END FOR

    RETURN savedInvoice
END FUNCTION

FUNCTION calculateTieredPricing(totalHours, rateCard)
  // Apply volume-based tiered pricing
  tiers = rateCard.pricingTiers
  amount = 0
  remainingHours = totalHours

  FOR EACH tier IN tiers.sortBy(minHours)
    hoursInTier = MIN(
      remainingHours,
      tier.maxHours - tier.minHours
    )

    IF hoursInTier > 0 THEN
      amount += hoursInTier * tier.rate
      remainingHours -= hoursInTier
    END IF

    IF remainingHours <= 0
      BREAK
    END IF
  END FOR

  RETURN amount
END FUNCTION

FUNCTION applyDiscounts(baseAmount, client, project, hours)
  totalDiscount = 0

  // Client-level discount
  IF client.contractTerms.discountTier > 0 THEN
    clientDiscount = baseAmount * (client.contractTerms.discountTier / 100)
    totalDiscount += clientDiscount
  END IF

  // Volume discount
  volumeDiscounts = project.billing.rateCard.discounts.volumeTier
  FOR EACH discount IN volumeDiscounts
    IF hours >= discount.threshold THEN
      volumeDiscount = baseAmount * (discount.discount / 100)
      totalDiscount = MAX(totalDiscount, volumeDiscount) // Take best discount
    END IF
  END FOR

  // Promotional discount
  IF project.billing.promotionalDiscount IS_SET THEN
    promoDiscount = baseAmount * (project.billing.promotionalDiscount / 100)
    totalDiscount += promoDiscount
  END IF

  // Cap total discount
  maxDiscountPercent = 30 // Never discount more than 30%
  maxDiscount = baseAmount * (maxDiscountPercent / 100)
  totalDiscount = MIN(totalDiscount, maxDiscount)

  RETURN totalDiscount
END FUNCTION
```

**Complexity**: O(E + M) where E=time entries, M=metering events
**Transaction Safety**: Use database transactions for invoice creation

---

## 5. Usage Metering Calculations

### Purpose
Track and calculate usage-based billing with tiered pricing and overage handling.

### Algorithm: Real-Time Metering with Aggregation

```pseudocode
FUNCTION recordMeteringEvent(event)
  INPUT:
    event: {clientId, projectId, eventType, quantity, timestamp, metadata}
  OUTPUT:
    meteringEvent: Persisted event with calculated pricing

  ALGORITHM:
    client = getClient(event.clientId)
    project = getProject(event.projectId)

    // Get pricing tier for this event type
    pricingTier = getPricingTier(client, event.eventType)

    // Calculate current usage in billing period
    billingPeriod = getCurrentBillingPeriod(client, event.timestamp)
    currentUsage = aggregateUsage(
      event.clientId,
      event.eventType,
      billingPeriod
    )

    // Determine unit price based on tiered pricing
    unitPrice = calculateUnitPrice(
      currentUsage,
      event.quantity,
      pricingTier
    )

    totalAmount = event.quantity * unitPrice

    // Create metering event
    meteringEvent = {
      id: generateUUID(),
      clientId: event.clientId,
      projectId: event.projectId,
      eventType: event.eventType,
      quantity: event.quantity,
      unitPrice: unitPrice,
      totalAmount: totalAmount,
      timestamp: event.timestamp,
      metadata: event.metadata,
      billing: {
        billingPeriod: billingPeriod,
        invoiceId: NULL // Will be set when invoiced
      }
    }

    // Persist event
    savedEvent = saveMeteringEvent(meteringEvent)

    // Check for usage alerts
    checkUsageAlerts(client, event.eventType, currentUsage + event.quantity, pricingTier)

    RETURN savedEvent
END FUNCTION

FUNCTION calculateUnitPrice(currentUsage, newQuantity, pricingTier)
  // Calculate blended price across tiers
  totalQuantity = currentUsage + newQuantity
  tiers = pricingTier.tiers.sortBy(minUsage)

  prices = []
  remainingQuantity = newQuantity
  usagePointer = currentUsage

  FOR EACH tier IN tiers
    // How much of new quantity falls in this tier?
    tierCapacity = tier.maxUsage - tier.minUsage
    tierUsed = MAX(0, usagePointer - tier.minUsage)
    tierAvailable = tierCapacity - tierUsed

    quantityInTier = MIN(remainingQuantity, tierAvailable)

    IF quantityInTier > 0 THEN
      prices.append({
        quantity: quantityInTier,
        price: tier.unitPrice
      })

      remainingQuantity -= quantityInTier
      usagePointer += quantityInTier
    END IF

    IF remainingQuantity <= 0
      BREAK
    END IF
  END FOR

  // Calculate blended unit price
  totalCost = SUM(p.quantity * p.price FOR p IN prices)
  blendedUnitPrice = totalCost / newQuantity

  RETURN blendedUnitPrice
END FUNCTION

FUNCTION aggregateUsage(clientId, eventType, billingPeriod)
  // Efficient aggregation query
  query = """
    SELECT SUM(quantity) as total
    FROM metering_events
    WHERE clientId = :clientId
      AND eventType = :eventType
      AND timestamp >= :periodStart
      AND timestamp < :periodEnd
  """

  result = executeQuery(query, {
    clientId: clientId,
    eventType: eventType,
    periodStart: billingPeriod.start,
    periodEnd: billingPeriod.end
  })

  RETURN result.total OR 0
END FUNCTION

FUNCTION checkUsageAlerts(client, eventType, currentUsage, pricingTier)
  // Check if usage crosses alert thresholds
  alertThresholds = client.usageAlerts.get(eventType)

  IF alertThresholds IS_NOT_SET
    RETURN
  END IF

  FOR EACH threshold IN alertThresholds
    IF currentUsage >= threshold.limit THEN
      // Check if alert already sent
      lastAlert = getLastAlert(client.id, eventType, threshold.limit)

      IF lastAlert IS_NULL OR lastAlert.timestamp < billingPeriod.start THEN
        sendUsageAlert(client, eventType, currentUsage, threshold)
        logAlert(client.id, eventType, threshold.limit, NOW())
      END IF
    END IF
  END FOR
END FUNCTION

FUNCTION forecastUsage(clientId, eventType, forecastPeriod)
  INPUT:
    clientId: Client to forecast
    eventType: Type of usage to forecast
    forecastPeriod: Number of days to forecast
  OUTPUT:
    forecast: {estimatedUsage, estimatedCost, confidence}

  ALGORITHM:
    // Get historical usage (last 90 days)
    historicalData = getMeteringEvents(
      clientId,
      eventType,
      TODAY() - 90,
      TODAY()
    )

    // Group by day
    dailyUsage = groupBy(historicalData, e => e.timestamp.date())

    // Calculate statistics
    usageValues = dailyUsage.map(d => d.totalQuantity)
    avgDailyUsage = MEAN(usageValues)
    stdDev = STANDARD_DEVIATION(usageValues)

    // Detect trend (linear regression)
    trend = calculateTrend(dailyUsage)

    // Forecast
    estimatedDailyUsage = avgDailyUsage + (trend * forecastPeriod / 2)
    estimatedTotalUsage = estimatedDailyUsage * forecastPeriod

    // Confidence interval (95%)
    marginOfError = 1.96 * stdDev * SQRT(forecastPeriod)
    confidenceInterval = {
      lower: MAX(0, estimatedTotalUsage - marginOfError),
      upper: estimatedTotalUsage + marginOfError
    }

    // Calculate estimated cost
    pricingTier = getPricingTier(client, eventType)
    estimatedCost = calculateTieredCost(estimatedTotalUsage, pricingTier)

    RETURN {
      estimatedUsage: estimatedTotalUsage,
      estimatedCost: estimatedCost,
      confidence: confidenceInterval,
      trend: trend > 0 ? 'Increasing' : trend < 0 ? 'Decreasing' : 'Stable'
    }
END FUNCTION
```

**Complexity**: O(1) for event recording, O(log N) for aggregation with indexes
**Real-time Performance**: Uses materialized views for fast aggregation

---

## 6. Budget Burn Rate Tracking

### Purpose
Monitor project budget consumption and predict when budget will be exhausted.

### Algorithm: Burn Rate Analysis with Forecasting

```pseudocode
FUNCTION calculateBudgetBurnRate(projectId)
  INPUT:
    projectId: Project to analyze
  OUTPUT:
    burnAnalysis: {burnRate, projectedDepletion, status, recommendations}

  ALGORITHM:
    project = getProject(projectId)

    IF project.budget.totalAmount IS_NULL
      RETURN UNLIMITED_BUDGET
    END IF

    // Calculate time-based metrics
    totalDays = project.timeline.endDate - project.timeline.startDate
    elapsedDays = TODAY() - project.timeline.startDate
    remainingDays = project.timeline.endDate - TODAY()

    // Calculate spend metrics
    totalBudget = project.budget.totalAmount
    spentAmount = calculateTotalSpend(projectId)
    remainingBudget = totalBudget - spentAmount

    // Calculate burn rate (spend per day)
    actualBurnRate = spentAmount / elapsedDays
    plannedBurnRate = totalBudget / totalDays

    // Forecast budget depletion
    IF actualBurnRate > 0 THEN
      daysUntilDepletion = remainingBudget / actualBurnRate
      projectedDepletionDate = TODAY() + daysUntilDepletion
    ELSE
      daysUntilDepletion = INFINITY
      projectedDepletionDate = NULL
    END IF

    // Calculate variance
    plannedSpendToDate = plannedBurnRate * elapsedDays
    spendVariance = spentAmount - plannedSpendToDate
    spendVariancePercent = (spendVariance / plannedSpendToDate) * 100

    // Determine status
    status = 'OnTrack'
    IF spendVariancePercent > 10 THEN
      status = 'OverBudget'
    ELSE IF spendVariancePercent > 5 THEN
      status = 'AtRisk'
    END IF

    // Generate recommendations
    recommendations = []

    IF projectedDepletionDate < project.timeline.endDate THEN
      daysShort = project.timeline.endDate - projectedDepletionDate
      recommendations.append({
        priority: 'HIGH',
        action: 'Budget will be depleted ' + daysShort + ' days before project end',
        suggestion: 'Reduce burn rate or request budget increase'
      })
    END IF

    IF actualBurnRate > plannedBurnRate * 1.2 THEN
      recommendations.append({
        priority: 'MEDIUM',
        action: 'Burn rate 20% above plan',
        suggestion: 'Review resource allocation and scope'
      })
    END IF

    // Calculate efficiency metrics
    hoursSpent = getTotalProjectHours(projectId)
    IF hoursSpent > 0 THEN
      averageRate = spentAmount / hoursSpent

      recommendations.append({
        priority: 'INFO',
        action: 'Average blended rate: $' + averageRate + '/hour',
        suggestion: 'Consider resource mix optimization'
      })
    END IF

    RETURN {
      budget: {
        total: totalBudget,
        spent: spentAmount,
        remaining: remainingBudget,
        percentUsed: (spentAmount / totalBudget) * 100
      },
      timeline: {
        totalDays: totalDays,
        elapsedDays: elapsedDays,
        remainingDays: remainingDays,
        percentComplete: (elapsedDays / totalDays) * 100
      },
      burnRate: {
        actual: actualBurnRate,
        planned: plannedBurnRate,
        variance: actualBurnRate - plannedBurnRate,
        variancePercent: ((actualBurnRate - plannedBurnRate) / plannedBurnRate) * 100
      },
      forecast: {
        projectedDepletionDate: projectedDepletionDate,
        daysUntilDepletion: daysUntilDepletion,
        projectedTotalSpend: actualBurnRate * totalDays
      },
      status: status,
      recommendations: recommendations
    }
END FUNCTION

FUNCTION predictProjectCompletion(projectId)
  // Use historical velocity to predict completion
  project = getProject(projectId)
  completedTasks = getTasks(projectId, status='Completed')
  totalTasks = getTasks(projectId)

  // Calculate velocity (tasks per day)
  elapsedDays = TODAY() - project.timeline.startDate
  velocity = completedTasks.length / elapsedDays

  // Predict completion
  remainingTasks = totalTasks.length - completedTasks.length
  IF velocity > 0 THEN
    estimatedDaysToComplete = remainingTasks / velocity
    estimatedCompletionDate = TODAY() + estimatedDaysToComplete
  ELSE
    estimatedCompletionDate = NULL
  END IF

  // Compare with budget forecast
  burnAnalysis = calculateBudgetBurnRate(projectId)

  RETURN {
    taskCompletion: {
      percentComplete: (completedTasks.length / totalTasks.length) * 100,
      velocity: velocity,
      estimatedCompletionDate: estimatedCompletionDate
    },
    budgetAlignment: {
      budgetDepletionDate: burnAnalysis.forecast.projectedDepletionDate,
      taskCompletionDate: estimatedCompletionDate,
      aligned: estimatedCompletionDate <= burnAnalysis.forecast.projectedDepletionDate
    }
  }
END FUNCTION
```

**Complexity**: O(1) with aggregated metrics
**Update Frequency**: Calculate daily or on-demand

---

## Summary

These algorithms form the core intelligence of the PSA Demo system:

1. **Project Scheduling**: Critical path analysis with resource leveling
2. **Resource Allocation**: Multi-factor scoring with constraint satisfaction
3. **Time Validation**: Rule-based validation with smart auto-approval
4. **Invoice Generation**: Tiered pricing with discount application
5. **Usage Metering**: Real-time tracking with predictive analytics
6. **Budget Tracking**: Burn rate analysis with forecasting

**Next Phase**: [03-architecture.md](./03-architecture.md) - System architecture design

---

**Document Version**: 1.0
**Last Updated**: 2025-12-26
**Status**: Approved
