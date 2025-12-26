import { MeteringEvent, MeteringEventType, UsageForecast } from '@shared/types';
import { v4 as uuidv4 } from 'uuid';

interface PricingTier {
  minUsage: number;
  maxUsage: number;
  unitPrice: number;
}

interface PricingConfig {
  eventType: MeteringEventType;
  tiers: PricingTier[];
}

export class MeteringService {
  /**
   * Record a new metering event
   */
  static async recordEvent(
    clientId: string,
    projectId: string,
    eventType: MeteringEventType,
    quantity: number,
    metadata: Record<string, any> = {}
  ): Promise<MeteringEvent> {
    // Get pricing for this event type
    const pricingConfig = this.getPricingConfig(eventType);

    // Get current usage in billing period
    const billingPeriod = this.getCurrentBillingPeriod();
    const currentUsage = await this.getUsageForPeriod(
      clientId,
      eventType,
      billingPeriod.start,
      billingPeriod.end
    );

    // Calculate unit price based on tiered pricing
    const unitPrice = this.calculateUnitPrice(currentUsage, quantity, pricingConfig.tiers);
    const totalAmount = quantity * unitPrice;

    const event: MeteringEvent = {
      id: uuidv4(),
      clientId,
      projectId,
      eventType,
      quantity,
      unitPrice,
      totalAmount,
      timestamp: new Date(),
      metadata: {
        userId: metadata.userId,
        resourceId: metadata.resourceId,
        dimensions: metadata,
      },
      billing: {
        billingPeriod,
      },
    };

    // In real implementation, persist to database
    // await db.meteringEvents.create(event);

    // Check for usage alerts
    await this.checkUsageAlerts(clientId, eventType, currentUsage + quantity, pricingConfig.tiers);

    return event;
  }

  /**
   * Calculate unit price based on tiered pricing
   */
  private static calculateUnitPrice(
    currentUsage: number,
    newQuantity: number,
    tiers: PricingTier[]
  ): number {
    const totalQuantity = currentUsage + newQuantity;
    const sortedTiers = [...tiers].sort((a, b) => a.minUsage - b.minUsage);

    const prices: Array<{ quantity: number; price: number }> = [];
    let remainingQuantity = newQuantity;
    let usagePointer = currentUsage;

    for (const tier of sortedTiers) {
      const tierCapacity = tier.maxUsage - tier.minUsage;
      const tierUsed = Math.max(0, usagePointer - tier.minUsage);
      const tierAvailable = tierCapacity - tierUsed;

      const quantityInTier = Math.min(remainingQuantity, tierAvailable);

      if (quantityInTier > 0) {
        prices.push({
          quantity: quantityInTier,
          price: tier.unitPrice,
        });

        remainingQuantity -= quantityInTier;
        usagePointer += quantityInTier;
      }

      if (remainingQuantity <= 0) break;
    }

    // Calculate blended unit price
    const totalCost = prices.reduce((sum, p) => sum + p.quantity * p.price, 0);
    const blendedUnitPrice = newQuantity > 0 ? totalCost / newQuantity : 0;

    return blendedUnitPrice;
  }

  /**
   * Get usage for a specific period
   */
  private static async getUsageForPeriod(
    clientId: string,
    eventType: MeteringEventType,
    startDate: Date,
    endDate: Date
  ): Promise<number> {
    // In real implementation, query database
    // const result = await db.meteringEvents.aggregate({
    //   where: { clientId, eventType, timestamp: { gte: startDate, lt: endDate } },
    //   _sum: { quantity: true }
    // });
    // return result._sum.quantity || 0;

    // For demo, return 0
    return 0;
  }

  /**
   * Get current billing period
   */
  private static getCurrentBillingPeriod(): { start: Date; end: Date } {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), 1);
    const end = new Date(now.getFullYear(), now.getMonth() + 1, 0, 23, 59, 59, 999);

    return { start, end };
  }

  /**
   * Get pricing configuration for event type
   */
  private static getPricingConfig(eventType: MeteringEventType): PricingConfig {
    const configs: { [key in MeteringEventType]: PricingConfig } = {
      ConsultingHour: {
        eventType: 'ConsultingHour',
        tiers: [
          { minUsage: 0, maxUsage: 50, unitPrice: 200 },
          { minUsage: 50, maxUsage: 200, unitPrice: 180 },
          { minUsage: 200, maxUsage: Infinity, unitPrice: 160 },
        ],
      },
      APICall: {
        eventType: 'APICall',
        tiers: [
          { minUsage: 0, maxUsage: 10000, unitPrice: 0.001 },
          { minUsage: 10000, maxUsage: 100000, unitPrice: 0.0008 },
          { minUsage: 100000, maxUsage: Infinity, unitPrice: 0.0005 },
        ],
      },
      ReportGeneration: {
        eventType: 'ReportGeneration',
        tiers: [
          { minUsage: 0, maxUsage: 10, unitPrice: 50 },
          { minUsage: 10, maxUsage: 50, unitPrice: 45 },
          { minUsage: 50, maxUsage: Infinity, unitPrice: 40 },
        ],
      },
      StorageGB: {
        eventType: 'StorageGB',
        tiers: [
          { minUsage: 0, maxUsage: 100, unitPrice: 0.1 },
          { minUsage: 100, maxUsage: 1000, unitPrice: 0.08 },
          { minUsage: 1000, maxUsage: Infinity, unitPrice: 0.06 },
        ],
      },
      TaskExecution: {
        eventType: 'TaskExecution',
        tiers: [
          { minUsage: 0, maxUsage: 100, unitPrice: 0.5 },
          { minUsage: 100, maxUsage: 500, unitPrice: 0.4 },
          { minUsage: 500, maxUsage: Infinity, unitPrice: 0.3 },
        ],
      },
    };

    return configs[eventType];
  }

  /**
   * Check usage alerts
   */
  private static async checkUsageAlerts(
    clientId: string,
    eventType: MeteringEventType,
    currentUsage: number,
    tiers: PricingTier[]
  ): Promise<void> {
    // Alert thresholds (percentage of tier limits)
    const alertThresholds = [0.8, 0.9, 1.0]; // 80%, 90%, 100%

    for (const tier of tiers) {
      for (const threshold of alertThresholds) {
        const alertLimit = tier.maxUsage * threshold;

        if (currentUsage >= alertLimit) {
          // In real implementation, send alert
          console.log(
            `ALERT: Client ${clientId} has reached ${threshold * 100}% of ${eventType} tier limit`
          );

          // Could trigger webhook, email, or in-app notification
        }
      }
    }
  }

  /**
   * Forecast usage based on historical data
   */
  static forecastUsage(
    historicalEvents: MeteringEvent[],
    forecastDays: number
  ): UsageForecast {
    if (historicalEvents.length === 0) {
      return {
        estimatedUsage: 0,
        estimatedCost: 0,
        confidence: { lower: 0, upper: 0 },
        trend: 'Stable',
      };
    }

    // Group by day
    const dailyUsage: { [date: string]: number } = {};

    for (const event of historicalEvents) {
      const dateKey = event.timestamp.toISOString().split('T')[0];
      dailyUsage[dateKey] = (dailyUsage[dateKey] || 0) + event.quantity;
    }

    const usageValues = Object.values(dailyUsage);

    // Calculate statistics
    const avgDailyUsage = usageValues.reduce((sum, val) => sum + val, 0) / usageValues.length;
    const stdDev = this.calculateStandardDeviation(usageValues);

    // Simple linear trend
    const trend = this.calculateTrend(usageValues);
    const trendLabel = trend > 0.05 ? 'Increasing' : trend < -0.05 ? 'Decreasing' : 'Stable';

    // Forecast
    const estimatedDailyUsage = avgDailyUsage + (trend * forecastDays) / 2;
    const estimatedTotalUsage = estimatedDailyUsage * forecastDays;

    // Confidence interval (95%)
    const marginOfError = 1.96 * stdDev * Math.sqrt(forecastDays);
    const confidenceInterval = {
      lower: Math.max(0, estimatedTotalUsage - marginOfError),
      upper: estimatedTotalUsage + marginOfError,
    };

    // Calculate estimated cost (simplified)
    const eventType = historicalEvents[0].eventType;
    const pricingConfig = this.getPricingConfig(eventType);
    const estimatedCost = this.calculateUnitPrice(0, estimatedTotalUsage, pricingConfig.tiers);

    return {
      estimatedUsage: estimatedTotalUsage,
      estimatedCost: estimatedCost * estimatedTotalUsage,
      confidence: confidenceInterval,
      trend: trendLabel,
    };
  }

  /**
   * Calculate standard deviation
   */
  private static calculateStandardDeviation(values: number[]): number {
    const avg = values.reduce((sum, val) => sum + val, 0) / values.length;
    const squaredDiffs = values.map((val) => Math.pow(val - avg, 2));
    const variance = squaredDiffs.reduce((sum, val) => sum + val, 0) / values.length;
    return Math.sqrt(variance);
  }

  /**
   * Calculate trend (simple linear regression slope)
   */
  private static calculateTrend(values: number[]): number {
    const n = values.length;
    if (n < 2) return 0;

    const xSum = (n * (n - 1)) / 2; // Sum of 0, 1, 2, ..., n-1
    const ySum = values.reduce((sum, val) => sum + val, 0);
    const xySum = values.reduce((sum, val, idx) => sum + idx * val, 0);
    const xxSum = (n * (n - 1) * (2 * n - 1)) / 6; // Sum of squares

    const slope = (n * xySum - xSum * ySum) / (n * xxSum - xSum * xSum);
    return slope;
  }

  /**
   * Get usage summary for a client
   */
  static async getUsageSummary(clientId: string, startDate: Date, endDate: Date) {
    const eventTypes: MeteringEventType[] = [
      'ConsultingHour',
      'APICall',
      'ReportGeneration',
      'StorageGB',
      'TaskExecution',
    ];

    const summary: Array<{
      eventType: MeteringEventType;
      quantity: number;
      cost: number;
      tier: string;
    }> = [];

    for (const eventType of eventTypes) {
      const usage = await this.getUsageForPeriod(clientId, eventType, startDate, endDate);
      const pricingConfig = this.getPricingConfig(eventType);
      const unitPrice = this.calculateUnitPrice(0, usage, pricingConfig.tiers);
      const cost = usage * unitPrice;

      // Determine current tier
      const currentTier = pricingConfig.tiers.find(
        (t) => usage >= t.minUsage && usage < t.maxUsage
      );

      summary.push({
        eventType,
        quantity: usage,
        cost,
        tier: currentTier
          ? `${currentTier.minUsage}-${currentTier.maxUsage === Infinity ? '∞' : currentTier.maxUsage}`
          : 'N/A',
      });
    }

    return summary;
  }
}
