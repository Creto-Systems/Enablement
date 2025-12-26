import { Invoice, InvoiceLineItem, TimeEntry, MeteringEvent, RateCard } from '@shared/types';
import { InvoiceModel } from '../models/Invoice';
import { v4 as uuidv4 } from 'uuid';

export class BillingService {
  /**
   * Generate invoice from approved time entries
   */
  static generateInvoiceFromTimeEntries(
    projectId: string,
    clientId: string,
    timeEntries: TimeEntry[],
    rateCard: RateCard,
    taxRate: number = 0,
    meteringEvents: MeteringEvent[] = []
  ): Invoice {
    // Group entries by resource and activity code
    const grouped = this.groupTimeEntries(timeEntries);

    // Create line items from grouped entries
    const timeLineItems: InvoiceLineItem[] = [];

    for (const [key, entries] of Object.entries(grouped)) {
      const [userId, activityCode] = key.split('|');
      const totalHours = entries.reduce((sum, e) => sum + e.hours, 0);

      // Get rate from rate card
      const rate = this.getRate(rateCard, userId, activityCode);

      // Calculate tiered pricing
      const baseAmount = this.calculateTieredPricing(totalHours, rate, rateCard);

      // No discount for demo (could be enhanced)
      const discount = 0;

      const lineItem: InvoiceLineItem = {
        id: uuidv4(),
        description: `${activityCode} - ${totalHours.toFixed(2)} hours`,
        quantity: totalHours,
        unitPrice: baseAmount / totalHours,
        discount,
        amount: baseAmount - discount,
        timeEntryIds: entries.map((e) => e.id),
      };

      timeLineItems.push(lineItem);
    }

    // Add metering events as line items
    const meteringLineItems: InvoiceLineItem[] = meteringEvents.map((event) => ({
      id: uuidv4(),
      description: this.formatMeteringDescription(event),
      quantity: event.quantity,
      unitPrice: event.unitPrice,
      discount: 0,
      amount: event.totalAmount,
      meteringEventId: event.id,
    }));

    const allLineItems = [...timeLineItems, ...meteringLineItems];

    // Calculate totals
    const subtotal = allLineItems.reduce((sum, item) => sum + item.amount, 0);
    const tax = subtotal * taxRate;
    const total = subtotal + tax;

    // Get billing period
    const sortedEntries = [...timeEntries].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()
    );
    const meteringPeriodStart =
      sortedEntries.length > 0 ? new Date(sortedEntries[0].date) : new Date();
    const meteringPeriodEnd =
      sortedEntries.length > 0
        ? new Date(sortedEntries[sortedEntries.length - 1].date)
        : new Date();

    // Create invoice
    const invoice = InvoiceModel.create({
      clientId,
      projectId,
      lineItems: allLineItems,
      subtotal,
      tax,
      total,
      currency: 'USD',
      metering: {
        meteringPeriodStart,
        meteringPeriodEnd,
        usageMetrics: meteringEvents.map((e) => ({
          eventType: e.eventType,
          quantity: e.quantity,
          unitPrice: e.unitPrice,
          totalAmount: e.totalAmount,
        })),
      },
    });

    return invoice;
  }

  /**
   * Group time entries by resource and activity code
   */
  private static groupTimeEntries(entries: TimeEntry[]): { [key: string]: TimeEntry[] } {
    return entries.reduce((acc, entry) => {
      const key = `${entry.userId}|${entry.activityCode}`;
      if (!acc[key]) {
        acc[key] = [];
      }
      acc[key].push(entry);
      return acc;
    }, {} as { [key: string]: TimeEntry[] });
  }

  /**
   * Get billing rate from rate card
   */
  private static getRate(rateCard: RateCard, userId: string, activityCode: string): number {
    // In real implementation, look up user's role and get rate
    // For demo, use base rate from activity code or default
    return rateCard.baseRates[activityCode] || rateCard.baseRates['default'] || 150;
  }

  /**
   * Calculate tiered pricing
   */
  private static calculateTieredPricing(
    totalHours: number,
    baseRate: number,
    rateCard: RateCard
  ): number {
    if (!rateCard.pricingTiers || rateCard.pricingTiers.length === 0) {
      return totalHours * baseRate;
    }

    let amount = 0;
    let remainingHours = totalHours;

    const sortedTiers = [...rateCard.pricingTiers].sort((a, b) => a.minHours - b.minHours);

    for (const tier of sortedTiers) {
      const hoursInTier = Math.min(
        remainingHours,
        tier.maxHours - tier.minHours
      );

      if (hoursInTier > 0) {
        amount += hoursInTier * tier.rate;
        remainingHours -= hoursInTier;
      }

      if (remainingHours <= 0) break;
    }

    // Any remaining hours at base rate
    if (remainingHours > 0) {
      amount += remainingHours * baseRate;
    }

    return amount;
  }

  /**
   * Format metering event description for invoice
   */
  private static formatMeteringDescription(event: MeteringEvent): string {
    const typeLabels: { [key: string]: string } = {
      ConsultingHour: 'Consulting Hours',
      APICall: 'API Calls',
      ReportGeneration: 'Report Generation',
      StorageGB: 'Storage (GB)',
      TaskExecution: 'Task Execution',
    };

    const label = typeLabels[event.eventType] || event.eventType;
    return `${label} - ${event.quantity} units`;
  }

  /**
   * Apply discounts to invoice
   */
  static applyDiscount(
    invoice: Invoice,
    discountPercent: number,
    reason: string
  ): Invoice {
    const discountAmount = (invoice.subtotal * discountPercent) / 100;

    // Add discount as negative line item
    const discountLineItem: InvoiceLineItem = {
      id: uuidv4(),
      description: `Discount - ${reason}`,
      quantity: 1,
      unitPrice: -discountAmount,
      discount: 0,
      amount: -discountAmount,
    };

    return InvoiceModel.addLineItem(invoice, discountLineItem);
  }

  /**
   * Calculate invoice aging (days past due)
   */
  static getInvoiceAging(invoice: Invoice): number {
    if (invoice.status === 'Paid' || invoice.status === 'Cancelled') {
      return 0;
    }

    const daysOverdue = InvoiceModel.getDaysOverdue(invoice);
    return Math.max(0, daysOverdue);
  }

  /**
   * Get aging category for reporting
   */
  static getAgingCategory(invoice: Invoice): 'Current' | '1-30' | '31-60' | '61-90' | '90+' {
    const aging = this.getInvoiceAging(invoice);

    if (aging === 0) return 'Current';
    if (aging <= 30) return '1-30';
    if (aging <= 60) return '31-60';
    if (aging <= 90) return '61-90';
    return '90+';
  }

  /**
   * Calculate late fees
   */
  static calculateLateFee(invoice: Invoice, lateFeePercent: number): number {
    const daysOverdue = InvoiceModel.getDaysOverdue(invoice);
    if (daysOverdue <= 0) return 0;

    const amountDue = InvoiceModel.getAmountDue(invoice);
    return (amountDue * lateFeePercent) / 100;
  }

  /**
   * Validate invoice before sending
   */
  static validateForSending(invoice: Invoice): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (invoice.lineItems.length === 0) {
      errors.push('Invoice has no line items');
    }

    if (invoice.total <= 0) {
      errors.push('Invoice total must be greater than zero');
    }

    if (!invoice.clientId) {
      errors.push('Client ID is missing');
    }

    if (!invoice.dateDue) {
      errors.push('Due date is missing');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}
