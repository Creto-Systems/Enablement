import { Invoice, InvoiceLineItem, PaymentInfo, InvoiceMetering } from '@shared/types';
import { v4 as uuidv4 } from 'uuid';

export class InvoiceModel {
  static create(data: Partial<Invoice>): Invoice {
    const now = new Date();
    const dueDate = new Date();
    dueDate.setDate(dueDate.getDate() + 30); // Default 30 days payment terms

    return {
      id: uuidv4(),
      invoiceNumber: data.invoiceNumber || this.generateInvoiceNumber(),
      clientId: data.clientId || '',
      projectId: data.projectId || '',
      status: data.status || 'Draft',
      dateIssued: data.dateIssued || now,
      dateDue: data.dateDue || dueDate,
      datePaid: data.datePaid,
      lineItems: data.lineItems || [],
      subtotal: data.subtotal || 0,
      tax: data.tax || 0,
      total: data.total || 0,
      currency: data.currency || 'USD',
      paymentInfo: data.paymentInfo,
      metering: data.metering || this.getDefaultMetering(),
      metadata: {
        createdAt: now,
        updatedAt: now,
        sentAt: data.metadata?.sentAt,
      },
    };
  }

  static update(invoice: Invoice, updates: Partial<Invoice>): Invoice {
    return {
      ...invoice,
      ...updates,
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static addLineItem(invoice: Invoice, lineItem: Omit<InvoiceLineItem, 'id'>): Invoice {
    const newLineItem: InvoiceLineItem = {
      id: uuidv4(),
      ...lineItem,
    };

    const newLineItems = [...invoice.lineItems, newLineItem];
    const totals = this.calculateTotals(newLineItems, invoice.tax);

    return {
      ...invoice,
      lineItems: newLineItems,
      subtotal: totals.subtotal,
      total: totals.total,
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static removeLineItem(invoice: Invoice, lineItemId: string): Invoice {
    const newLineItems = invoice.lineItems.filter((item) => item.id !== lineItemId);
    const totals = this.calculateTotals(newLineItems, invoice.tax);

    return {
      ...invoice,
      lineItems: newLineItems,
      subtotal: totals.subtotal,
      total: totals.total,
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static updateLineItem(invoice: Invoice, lineItemId: string, updates: Partial<InvoiceLineItem>): Invoice {
    const newLineItems = invoice.lineItems.map((item) =>
      item.id === lineItemId ? { ...item, ...updates } : item
    );

    const totals = this.calculateTotals(newLineItems, invoice.tax);

    return {
      ...invoice,
      lineItems: newLineItems,
      subtotal: totals.subtotal,
      total: totals.total,
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static setTaxRate(invoice: Invoice, taxRate: number): Invoice {
    const tax = invoice.subtotal * taxRate;
    const total = invoice.subtotal + tax;

    return {
      ...invoice,
      tax,
      total,
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static send(invoice: Invoice): Invoice {
    if (invoice.status !== 'Draft') {
      throw new Error('Only draft invoices can be sent');
    }

    if (invoice.lineItems.length === 0) {
      throw new Error('Cannot send invoice with no line items');
    }

    return {
      ...invoice,
      status: 'Sent',
      metadata: {
        ...invoice.metadata,
        sentAt: new Date(),
        updatedAt: new Date(),
      },
    };
  }

  static recordPayment(invoice: Invoice, payment: PaymentInfo): Invoice {
    if (invoice.status !== 'Sent' && invoice.status !== 'Overdue') {
      throw new Error('Can only record payment for sent or overdue invoices');
    }

    return {
      ...invoice,
      status: 'Paid',
      datePaid: payment.paidAt || new Date(),
      paymentInfo: payment,
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static markOverdue(invoice: Invoice): Invoice {
    if (invoice.status !== 'Sent') {
      throw new Error('Only sent invoices can be marked as overdue');
    }

    return {
      ...invoice,
      status: 'Overdue',
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static cancel(invoice: Invoice): Invoice {
    if (invoice.status === 'Paid') {
      throw new Error('Cannot cancel paid invoices');
    }

    return {
      ...invoice,
      status: 'Cancelled',
      metadata: {
        ...invoice.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static isDraft(invoice: Invoice): boolean {
    return invoice.status === 'Draft';
  }

  static isSent(invoice: Invoice): boolean {
    return invoice.status === 'Sent';
  }

  static isPaid(invoice: Invoice): boolean {
    return invoice.status === 'Paid';
  }

  static isOverdue(invoice: Invoice): boolean {
    if (invoice.status === 'Paid' || invoice.status === 'Cancelled') {
      return false;
    }

    const now = new Date();
    const dueDate = new Date(invoice.dateDue);
    return now > dueDate;
  }

  static getDaysUntilDue(invoice: Invoice): number {
    const now = new Date();
    const dueDate = new Date(invoice.dateDue);
    const diffTime = dueDate.getTime() - now.getTime();
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }

  static getDaysOverdue(invoice: Invoice): number {
    if (!this.isOverdue(invoice)) return 0;

    const now = new Date();
    const dueDate = new Date(invoice.dateDue);
    const diffTime = now.getTime() - dueDate.getTime();
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }

  static getAmountDue(invoice: Invoice): number {
    if (invoice.status === 'Paid') return 0;
    return invoice.total - (invoice.paymentInfo?.paidAmount || 0);
  }

  static isPartiallyPaid(invoice: Invoice): boolean {
    const paidAmount = invoice.paymentInfo?.paidAmount || 0;
    return paidAmount > 0 && paidAmount < invoice.total;
  }

  private static calculateTotals(
    lineItems: InvoiceLineItem[],
    tax: number
  ): { subtotal: number; total: number } {
    const subtotal = lineItems.reduce((sum, item) => sum + item.amount, 0);
    const total = subtotal + tax;

    return { subtotal, total };
  }

  private static generateInvoiceNumber(): string {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const random = Math.floor(Math.random() * 10000)
      .toString()
      .padStart(4, '0');

    return `INV-${year}${month}-${random}`;
  }

  private static getDefaultMetering(): InvoiceMetering {
    const now = new Date();
    const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
    const endOfMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0);

    return {
      meteringPeriodStart: startOfMonth,
      meteringPeriodEnd: endOfMonth,
      usageMetrics: [],
    };
  }

  static validate(invoice: Partial<Invoice>): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!invoice.clientId) {
      errors.push('Client ID is required');
    }

    if (!invoice.projectId) {
      errors.push('Project ID is required');
    }

    if (!invoice.dateIssued) {
      errors.push('Issue date is required');
    }

    if (!invoice.dateDue) {
      errors.push('Due date is required');
    } else if (invoice.dateIssued && new Date(invoice.dateDue) < new Date(invoice.dateIssued)) {
      errors.push('Due date must be after issue date');
    }

    if (invoice.lineItems && invoice.lineItems.length === 0) {
      errors.push('Invoice must have at least one line item');
    }

    if (invoice.total !== undefined && invoice.total < 0) {
      errors.push('Total amount cannot be negative');
    }

    if (invoice.tax !== undefined && invoice.tax < 0) {
      errors.push('Tax amount cannot be negative');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}
