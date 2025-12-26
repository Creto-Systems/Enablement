import { TimeEntry, TimeEntryApprover, TimeEntryBilling } from '@shared/types';
import { v4 as uuidv4 } from 'uuid';

export class TimeEntryModel {
  static create(data: Partial<TimeEntry>): TimeEntry {
    const now = new Date();

    return {
      id: uuidv4(),
      userId: data.userId || '',
      projectId: data.projectId || '',
      taskId: data.taskId,
      date: data.date || now,
      hours: data.hours || 0,
      billable: data.billable !== undefined ? data.billable : true,
      status: data.status || 'Draft',
      description: data.description || '',
      activityCode: data.activityCode || '',
      approver: data.approver,
      billing: data.billing || this.getDefaultBilling(),
      metadata: {
        createdAt: now,
        updatedAt: now,
        source: data.metadata?.source || 'Manual',
      },
    };
  }

  static update(entry: TimeEntry, updates: Partial<TimeEntry>): TimeEntry {
    return {
      ...entry,
      ...updates,
      metadata: {
        ...entry.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static submit(entry: TimeEntry): TimeEntry {
    if (entry.status !== 'Draft') {
      throw new Error('Only draft entries can be submitted');
    }

    return {
      ...entry,
      status: 'Submitted',
      metadata: {
        ...entry.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static approve(entry: TimeEntry, approverId: string, comments?: string): TimeEntry {
    if (entry.status !== 'Submitted') {
      throw new Error('Only submitted entries can be approved');
    }

    const approver: TimeEntryApprover = {
      userId: approverId,
      approvedAt: new Date(),
      comments,
    };

    return {
      ...entry,
      status: 'Approved',
      approver,
      metadata: {
        ...entry.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static reject(entry: TimeEntry, approverId: string, comments: string): TimeEntry {
    if (entry.status !== 'Submitted') {
      throw new Error('Only submitted entries can be rejected');
    }

    const approver: TimeEntryApprover = {
      userId: approverId,
      approvedAt: new Date(),
      comments,
    };

    return {
      ...entry,
      status: 'Rejected',
      approver,
      metadata: {
        ...entry.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static markAsInvoiced(entry: TimeEntry, invoiceId: string): TimeEntry {
    if (entry.status !== 'Approved') {
      throw new Error('Only approved entries can be invoiced');
    }

    return {
      ...entry,
      status: 'Invoiced',
      billing: {
        ...entry.billing,
        invoiceId,
      },
      metadata: {
        ...entry.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static calculateBillingAmount(entry: TimeEntry, rate: number): TimeEntry {
    return {
      ...entry,
      billing: {
        ...entry.billing,
        rate,
        amount: entry.hours * rate,
      },
      metadata: {
        ...entry.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static isDraft(entry: TimeEntry): boolean {
    return entry.status === 'Draft';
  }

  static isSubmitted(entry: TimeEntry): boolean {
    return entry.status === 'Submitted';
  }

  static isApproved(entry: TimeEntry): boolean {
    return entry.status === 'Approved';
  }

  static isRejected(entry: TimeEntry): boolean {
    return entry.status === 'Rejected';
  }

  static isInvoiced(entry: TimeEntry): boolean {
    return entry.status === 'Invoiced';
  }

  static isBillable(entry: TimeEntry): boolean {
    return entry.billable && entry.status === 'Approved';
  }

  static canEdit(entry: TimeEntry): boolean {
    return entry.status === 'Draft' || entry.status === 'Rejected';
  }

  static canSubmit(entry: TimeEntry): boolean {
    return entry.status === 'Draft' && entry.hours > 0 && entry.description.trim() !== '';
  }

  static canApprove(entry: TimeEntry): boolean {
    return entry.status === 'Submitted';
  }

  static canReject(entry: TimeEntry): boolean {
    return entry.status === 'Submitted';
  }

  static isOvertime(entry: TimeEntry): boolean {
    return entry.hours > 8; // Simple rule: more than 8 hours in a day
  }

  static isWeekend(entry: TimeEntry): boolean {
    const date = new Date(entry.date);
    const day = date.getDay();
    return day === 0 || day === 6; // Sunday or Saturday
  }

  static getDayOfWeek(entry: TimeEntry): string {
    const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
    const date = new Date(entry.date);
    return days[date.getDay()];
  }

  static getWeekNumber(entry: TimeEntry): number {
    const date = new Date(entry.date);
    const onejan = new Date(date.getFullYear(), 0, 1);
    const millisecsInDay = 86400000;
    return Math.ceil(((date.getTime() - onejan.getTime()) / millisecsInDay + onejan.getDay() + 1) / 7);
  }

  private static getDefaultBilling(): TimeEntryBilling {
    return {
      rate: 0,
      amount: 0,
    };
  }

  static validate(entry: Partial<TimeEntry>): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!entry.userId) {
      errors.push('User ID is required');
    }

    if (!entry.projectId) {
      errors.push('Project ID is required');
    }

    if (!entry.date) {
      errors.push('Date is required');
    } else {
      const entryDate = new Date(entry.date);
      const now = new Date();
      now.setHours(23, 59, 59, 999); // End of today

      if (entryDate > now) {
        errors.push('Cannot log time in the future');
      }

      // Check if entry is too old (e.g., more than 90 days ago)
      const ninetyDaysAgo = new Date();
      ninetyDaysAgo.setDate(ninetyDaysAgo.getDate() - 90);
      if (entryDate < ninetyDaysAgo) {
        errors.push('Cannot log time more than 90 days in the past');
      }
    }

    if (entry.hours === undefined || entry.hours <= 0) {
      errors.push('Hours must be greater than 0');
    } else if (entry.hours > 24) {
      errors.push('Hours cannot exceed 24 in a day');
    }

    if (!entry.description || entry.description.trim() === '') {
      errors.push('Description is required');
    } else if (entry.description.length < 10) {
      errors.push('Description must be at least 10 characters');
    }

    if (!entry.activityCode || entry.activityCode.trim() === '') {
      errors.push('Activity code is required');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }

  static validateBulk(entries: Partial<TimeEntry>[]): { isValid: boolean; errors: { index: number; errors: string[] }[] } {
    const allErrors: { index: number; errors: string[] }[] = [];

    entries.forEach((entry, index) => {
      const validation = this.validate(entry);
      if (!validation.isValid) {
        allErrors.push({ index, errors: validation.errors });
      }
    });

    return {
      isValid: allErrors.length === 0,
      errors: allErrors,
    };
  }
}
