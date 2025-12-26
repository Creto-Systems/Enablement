import { TimeEntryModel } from '../../src/server/models/TimeEntry';
import { TimeEntry } from '../../src/shared/types';

describe('TimeEntryModel', () => {
  describe('create', () => {
    it('should create a new time entry with default values', () => {
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Worked on feature development',
        activityCode: 'DEV',
      });

      expect(entry.id).toBeDefined();
      expect(entry.userId).toBe('user-1');
      expect(entry.projectId).toBe('project-1');
      expect(entry.hours).toBe(8);
      expect(entry.status).toBe('Draft');
      expect(entry.billable).toBe(true);
      expect(entry.metadata.source).toBe('Manual');
    });

    it('should allow custom billable flag', () => {
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 2,
        description: 'Internal meeting',
        activityCode: 'MEETING',
        billable: false,
      });

      expect(entry.billable).toBe(false);
    });
  });

  describe('submit', () => {
    it('should change status from Draft to Submitted', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Worked on feature',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);

      expect(entry.status).toBe('Submitted');
    });

    it('should throw error if entry is not in Draft status', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Worked on feature',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);

      expect(() => TimeEntryModel.submit(entry)).toThrow(
        'Only draft entries can be submitted'
      );
    });
  });

  describe('approve', () => {
    it('should approve a submitted entry', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Worked on feature',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);
      entry = TimeEntryModel.approve(entry, 'manager-1', 'Looks good');

      expect(entry.status).toBe('Approved');
      expect(entry.approver).toBeDefined();
      expect(entry.approver?.userId).toBe('manager-1');
      expect(entry.approver?.comments).toBe('Looks good');
    });

    it('should throw error if entry is not submitted', () => {
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Worked on feature',
        activityCode: 'DEV',
      });

      expect(() => TimeEntryModel.approve(entry, 'manager-1')).toThrow(
        'Only submitted entries can be approved'
      );
    });
  });

  describe('reject', () => {
    it('should reject a submitted entry with comments', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 12,
        description: 'Worked late',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);
      entry = TimeEntryModel.reject(entry, 'manager-1', 'Please provide more detail');

      expect(entry.status).toBe('Rejected');
      expect(entry.approver?.userId).toBe('manager-1');
      expect(entry.approver?.comments).toBe('Please provide more detail');
    });
  });

  describe('calculateBillingAmount', () => {
    it('should calculate billing amount correctly', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Consulting work',
        activityCode: 'CONSULT',
      });

      entry = TimeEntryModel.calculateBillingAmount(entry, 200);

      expect(entry.billing.rate).toBe(200);
      expect(entry.billing.amount).toBe(1600);
    });

    it('should handle fractional hours', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 6.5,
        description: 'Consulting work',
        activityCode: 'CONSULT',
      });

      entry = TimeEntryModel.calculateBillingAmount(entry, 150);

      expect(entry.billing.amount).toBe(975);
    });
  });

  describe('validate', () => {
    it('should validate a correct time entry', () => {
      const validation = TimeEntryModel.validate({
        userId: 'user-1',
        projectId: 'project-1',
        date: new Date(),
        hours: 8,
        description: 'Worked on feature development for the client project',
        activityCode: 'DEV',
      });

      expect(validation.isValid).toBe(true);
      expect(validation.errors).toHaveLength(0);
    });

    it('should reject entry with no hours', () => {
      const validation = TimeEntryModel.validate({
        userId: 'user-1',
        projectId: 'project-1',
        date: new Date(),
        hours: 0,
        description: 'Some work',
        activityCode: 'DEV',
      });

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Hours must be greater than 0');
    });

    it('should reject entry with hours > 24', () => {
      const validation = TimeEntryModel.validate({
        userId: 'user-1',
        projectId: 'project-1',
        date: new Date(),
        hours: 25,
        description: 'Too many hours',
        activityCode: 'DEV',
      });

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Hours cannot exceed 24 in a day');
    });

    it('should reject entry with future date', () => {
      const futureDate = new Date();
      futureDate.setDate(futureDate.getDate() + 1);

      const validation = TimeEntryModel.validate({
        userId: 'user-1',
        projectId: 'project-1',
        date: futureDate,
        hours: 8,
        description: 'Future work',
        activityCode: 'DEV',
      });

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Cannot log time in the future');
    });

    it('should reject entry with short description', () => {
      const validation = TimeEntryModel.validate({
        userId: 'user-1',
        projectId: 'project-1',
        date: new Date(),
        hours: 8,
        description: 'Short',
        activityCode: 'DEV',
      });

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Description must be at least 10 characters');
    });

    it('should reject entry without activity code', () => {
      const validation = TimeEntryModel.validate({
        userId: 'user-1',
        projectId: 'project-1',
        date: new Date(),
        hours: 8,
        description: 'Worked on development',
        activityCode: '',
      });

      expect(validation.isValid).toBe(false);
      expect(validation.errors).toContain('Activity code is required');
    });
  });

  describe('canEdit', () => {
    it('should allow editing draft entries', () => {
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Development work',
        activityCode: 'DEV',
      });

      expect(TimeEntryModel.canEdit(entry)).toBe(true);
    });

    it('should allow editing rejected entries', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Development work',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);
      entry = TimeEntryModel.reject(entry, 'manager-1', 'Needs more detail');

      expect(TimeEntryModel.canEdit(entry)).toBe(true);
    });

    it('should not allow editing submitted entries', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Development work',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);

      expect(TimeEntryModel.canEdit(entry)).toBe(false);
    });

    it('should not allow editing approved entries', () => {
      let entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        hours: 8,
        description: 'Development work',
        activityCode: 'DEV',
      });

      entry = TimeEntryModel.submit(entry);
      entry = TimeEntryModel.approve(entry, 'manager-1');

      expect(TimeEntryModel.canEdit(entry)).toBe(false);
    });
  });

  describe('isWeekend', () => {
    it('should detect Saturday as weekend', () => {
      const saturday = new Date('2025-12-27'); // Saturday
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        date: saturday,
        hours: 4,
        description: 'Weekend work',
        activityCode: 'DEV',
      });

      expect(TimeEntryModel.isWeekend(entry)).toBe(true);
    });

    it('should detect Sunday as weekend', () => {
      const sunday = new Date('2025-12-28'); // Sunday
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        date: sunday,
        hours: 4,
        description: 'Weekend work',
        activityCode: 'DEV',
      });

      expect(TimeEntryModel.isWeekend(entry)).toBe(true);
    });

    it('should detect weekday as not weekend', () => {
      const monday = new Date('2025-12-29'); // Monday
      const entry = TimeEntryModel.create({
        userId: 'user-1',
        projectId: 'project-1',
        date: monday,
        hours: 8,
        description: 'Regular work',
        activityCode: 'DEV',
      });

      expect(TimeEntryModel.isWeekend(entry)).toBe(false);
    });
  });
});
