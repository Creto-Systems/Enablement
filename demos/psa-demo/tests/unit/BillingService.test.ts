import { BillingService } from '../../src/server/services/BillingService';
import { TimeEntry, RateCard, MeteringEvent } from '../../src/shared/types';

describe('BillingService', () => {
  const mockRateCard: RateCard = {
    id: 'rate-1',
    name: 'Standard Rate Card',
    baseRates: {
      DEV: 200,
      CONSULT: 250,
      REVIEW: 180,
      default: 150,
    },
    pricingTiers: [
      { minHours: 0, maxHours: 50, rate: 200 },
      { minHours: 50, maxHours: 200, rate: 180 },
      { minHours: 200, maxHours: Infinity, rate: 160 },
    ],
  };

  const createMockTimeEntry = (overrides: Partial<TimeEntry> = {}): TimeEntry => ({
    id: `entry-${Date.now()}`,
    userId: 'user-1',
    projectId: 'project-1',
    date: new Date(),
    hours: 8,
    billable: true,
    status: 'Approved',
    description: 'Development work',
    activityCode: 'DEV',
    billing: {
      rate: 0,
      amount: 0,
    },
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      source: 'Manual',
    },
    ...overrides,
  });

  describe('generateInvoiceFromTimeEntries', () => {
    it('should generate invoice from approved time entries', () => {
      const timeEntries: TimeEntry[] = [
        createMockTimeEntry({ hours: 40, activityCode: 'DEV' }),
        createMockTimeEntry({ hours: 20, activityCode: 'REVIEW' }),
      ];

      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      expect(invoice.clientId).toBe('client-1');
      expect(invoice.projectId).toBe('project-1');
      expect(invoice.lineItems).toHaveLength(2);
      expect(invoice.status).toBe('Draft');
    });

    it('should calculate totals correctly', () => {
      const timeEntries: TimeEntry[] = [
        createMockTimeEntry({ hours: 10, activityCode: 'DEV' }), // 10 * 200 = 2000
        createMockTimeEntry({ hours: 5, activityCode: 'CONSULT' }), // 5 * 250 = 1250
      ];

      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard,
        0.08 // 8% tax
      );

      expect(invoice.subtotal).toBe(3250);
      expect(invoice.tax).toBe(260); // 3250 * 0.08
      expect(invoice.total).toBe(3510);
    });

    it('should include metering events as line items', () => {
      const timeEntries: TimeEntry[] = [
        createMockTimeEntry({ hours: 10, activityCode: 'DEV' }),
      ];

      const meteringEvents: MeteringEvent[] = [
        {
          id: 'meter-1',
          clientId: 'client-1',
          projectId: 'project-1',
          eventType: 'ReportGeneration',
          quantity: 5,
          unitPrice: 50,
          totalAmount: 250,
          timestamp: new Date(),
          metadata: {
            dimensions: {},
          },
          billing: {
            billingPeriod: {
              start: new Date(),
              end: new Date(),
            },
          },
        },
      ];

      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard,
        0,
        meteringEvents
      );

      expect(invoice.lineItems).toHaveLength(2);
      expect(invoice.lineItems[1].meteringEventId).toBe('meter-1');
      expect(invoice.lineItems[1].amount).toBe(250);
    });

    it('should group time entries by user and activity code', () => {
      const timeEntries: TimeEntry[] = [
        createMockTimeEntry({ id: '1', userId: 'user-1', hours: 5, activityCode: 'DEV' }),
        createMockTimeEntry({ id: '2', userId: 'user-1', hours: 3, activityCode: 'DEV' }),
        createMockTimeEntry({ id: '3', userId: 'user-1', hours: 4, activityCode: 'REVIEW' }),
      ];

      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      expect(invoice.lineItems).toHaveLength(2); // DEV and REVIEW grouped
      const devLine = invoice.lineItems.find((item) => item.description.includes('DEV'));
      expect(devLine?.quantity).toBe(8); // 5 + 3
    });
  });

  describe('applyDiscount', () => {
    it('should apply discount to invoice', () => {
      const timeEntries: TimeEntry[] = [
        createMockTimeEntry({ hours: 10, activityCode: 'DEV' }),
      ];

      let invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice = BillingService.applyDiscount(invoice, 10, 'Volume discount');

      const discountLine = invoice.lineItems.find((item) =>
        item.description.includes('Discount')
      );

      expect(discountLine).toBeDefined();
      expect(discountLine?.amount).toBeLessThan(0);
    });
  });

  describe('getInvoiceAging', () => {
    it('should return 0 for paid invoices', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      let invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.status = 'Paid';

      const aging = BillingService.getInvoiceAging(invoice);
      expect(aging).toBe(0);
    });

    it('should return 0 for cancelled invoices', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      let invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.status = 'Cancelled';

      const aging = BillingService.getInvoiceAging(invoice);
      expect(aging).toBe(0);
    });
  });

  describe('getAgingCategory', () => {
    it('should categorize current invoices', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.status = 'Sent';
      invoice.dateDue = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000); // 10 days in future

      const category = BillingService.getAgingCategory(invoice);
      expect(category).toBe('Current');
    });
  });

  describe('calculateLateFee', () => {
    it('should calculate late fee for overdue invoice', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry({ hours: 10 })];
      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.status = 'Sent';
      invoice.dateDue = new Date(Date.now() - 10 * 24 * 60 * 60 * 1000); // 10 days ago

      const lateFee = BillingService.calculateLateFee(invoice, 5); // 5% late fee

      expect(lateFee).toBeGreaterThan(0);
      expect(lateFee).toBe(invoice.total * 0.05);
    });

    it('should return 0 late fee for invoice not overdue', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.dateDue = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000); // Future date

      const lateFee = BillingService.calculateLateFee(invoice, 5);
      expect(lateFee).toBe(0);
    });
  });

  describe('validateForSending', () => {
    it('should validate correct invoice for sending', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      const validation = BillingService.validateForSending(invoice);

      expect(validation.isValid).toBe(true);
      expect(validation.errors).toHaveLength(0);
    });

    it('should reject invoice with no line items', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.lineItems = [];

      const validation = BillingService.validateForSending(invoice);

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Invoice has no line items');
    });

    it('should reject invoice with zero total', () => {
      const timeEntries: TimeEntry[] = [createMockTimeEntry()];
      const invoice = BillingService.generateInvoiceFromTimeEntries(
        'project-1',
        'client-1',
        timeEntries,
        mockRateCard
      );

      invoice.total = 0;

      const validation = BillingService.validateForSending(invoice);

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Invoice total must be greater than zero');
    });
  });
});
