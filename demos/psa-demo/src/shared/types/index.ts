// Shared TypeScript type definitions for PSA Demo

export interface Client {
  id: string;
  name: string;
  industry: string;
  parentClientId?: string;
  contacts: Contact[];
  billingInfo: BillingInfo;
  contractTerms: ContractTerms;
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    status: 'Active' | 'Inactive' | 'Prospect';
  };
}

export interface Contact {
  id: string;
  name: string;
  email: string;
  phone?: string;
  role: string;
  isPrimary: boolean;
}

export interface BillingInfo {
  paymentTerms: number; // days
  preferredMethod: 'ACH' | 'Wire' | 'Check' | 'CreditCard';
  taxId: string;
  currency: string;
  billingAddress: Address;
}

export interface Address {
  street: string;
  city: string;
  state: string;
  postalCode: string;
  country: string;
}

export interface ContractTerms {
  defaultRate: number;
  discountTier: number; // percentage
  msa: boolean;
  msaExpiryDate?: Date;
}

export interface Project {
  id: string;
  clientId: string;
  name: string;
  description: string;
  type: 'TimeAndMaterial' | 'FixedPrice' | 'Retainer' | 'OutcomeBased';
  status: 'Planning' | 'Active' | 'OnHold' | 'Completed' | 'Cancelled';
  timeline: ProjectTimeline;
  budget: ProjectBudget;
  team: ProjectTeam;
  billing: ProjectBilling;
  health: ProjectHealth;
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    tags: string[];
  };
}

export interface ProjectTimeline {
  startDate: Date;
  endDate: Date;
  milestones: Milestone[];
}

export interface Milestone {
  id: string;
  name: string;
  description: string;
  dueDate: Date;
  status: 'Pending' | 'InProgress' | 'Completed' | 'Missed';
  deliverables: string[];
}

export interface ProjectBudget {
  totalHours?: number;
  totalAmount?: number;
  currency: string;
  burnRate: number; // percentage
  remainingBudget: number;
}

export interface ProjectTeam {
  projectManagerId: string;
  resources: ResourceAssignment[];
}

export interface ResourceAssignment {
  userId: string;
  role: string;
  allocatedHours: number;
  startDate: Date;
  endDate?: Date;
  rate: number;
}

export interface ProjectBilling {
  billingCycle: 'Weekly' | 'BiWeekly' | 'Monthly' | 'Milestone';
  rateCard: RateCard;
  invoiceSchedule: Date[];
}

export interface RateCard {
  id: string;
  name: string;
  baseRates: { [role: string]: number };
  pricingTiers?: PricingTier[];
  discounts?: Discount[];
}

export interface PricingTier {
  minHours: number;
  maxHours: number;
  rate: number;
}

export interface Discount {
  type: 'volume' | 'client' | 'promotional';
  threshold?: number;
  discount: number; // percentage
}

export interface ProjectHealth {
  budgetStatus: 'OnTrack' | 'AtRisk' | 'OverBudget';
  scheduleStatus: 'OnTrack' | 'Delayed' | 'Ahead';
  overallHealth: number; // 0-100 score
}

export interface Resource {
  id: string;
  userId: string;
  firstName: string;
  lastName: string;
  email: string;
  role: string;
  skills: Skill[];
  availability: ResourceAvailability;
  performance: ResourcePerformance;
  rate: number;
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    status: 'Active' | 'Inactive' | 'OnLeave';
  };
}

export interface Skill {
  name: string;
  level: number; // 1-10
  yearsExperience: number;
  certified: boolean;
}

export interface ResourceAvailability {
  capacity: number; // hours per week
  availableHours: number;
  currentUtilization: number; // percentage
  allocations: ResourceAssignment[];
}

export interface ResourcePerformance {
  performanceRating: number; // 1-5 stars
  projectsCompleted: number;
  averageClientRating: number;
  onTimeDeliveryRate: number; // percentage
}

export interface TimeEntry {
  id: string;
  userId: string;
  projectId: string;
  taskId?: string;
  date: Date;
  hours: number;
  billable: boolean;
  status: 'Draft' | 'Submitted' | 'Approved' | 'Rejected' | 'Invoiced';
  description: string;
  activityCode: string;
  approver?: TimeEntryApprover;
  billing: TimeEntryBilling;
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    source: 'Manual' | 'Timer' | 'Import';
  };
}

export interface TimeEntryApprover {
  userId: string;
  approvedAt: Date;
  comments?: string;
}

export interface TimeEntryBilling {
  rate: number;
  amount: number;
  invoiceId?: string;
}

export interface Invoice {
  id: string;
  invoiceNumber: string;
  clientId: string;
  projectId: string;
  status: 'Draft' | 'Sent' | 'Paid' | 'Overdue' | 'Cancelled';
  dateIssued: Date;
  dateDue: Date;
  datePaid?: Date;
  lineItems: InvoiceLineItem[];
  subtotal: number;
  tax: number;
  total: number;
  currency: string;
  paymentInfo?: PaymentInfo;
  metering: InvoiceMetering;
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    sentAt?: Date;
  };
}

export interface InvoiceLineItem {
  id: string;
  description: string;
  quantity: number;
  unitPrice: number;
  discount: number;
  amount: number;
  timeEntryIds?: string[];
  meteringEventId?: string;
}

export interface PaymentInfo {
  method?: string;
  transactionId?: string;
  paidAmount?: number;
  paidAt?: Date;
}

export interface InvoiceMetering {
  meteringPeriodStart: Date;
  meteringPeriodEnd: Date;
  usageMetrics: UsageMetric[];
}

export interface UsageMetric {
  eventType: string;
  quantity: number;
  unitPrice: number;
  totalAmount: number;
}

export interface Task {
  id: string;
  type: 'ReportGeneration' | 'DataAnalysis' | 'BulkOperation';
  projectId?: string;
  status: 'Queued' | 'Running' | 'Completed' | 'Failed';
  priority: 'Low' | 'Medium' | 'High';
  runtime: TaskRuntime;
  input: Record<string, any>;
  output?: Record<string, any>;
  error?: TaskError;
  metadata: {
    createdAt: Date;
    startedAt?: Date;
    completedAt?: Date;
    retryCount: number;
  };
}

export interface TaskRuntime {
  sandboxId: string;
  executionTime: number; // milliseconds
  memoryUsed: number; // MB
  costIncurred: number;
}

export interface TaskError {
  message: string;
  stack: string;
}

export interface MeteringEvent {
  id: string;
  clientId: string;
  projectId?: string;
  eventType: MeteringEventType;
  quantity: number;
  unitPrice: number;
  totalAmount: number;
  timestamp: Date;
  metadata: MeteringEventMetadata;
  billing: MeteringEventBilling;
}

export type MeteringEventType =
  | 'ConsultingHour'
  | 'APICall'
  | 'ReportGeneration'
  | 'StorageGB'
  | 'TaskExecution';

export interface MeteringEventMetadata {
  userId?: string;
  resourceId?: string;
  dimensions: Record<string, any>;
}

export interface MeteringEventBilling {
  invoiceId?: string;
  billingPeriod: {
    start: Date;
    end: Date;
  };
}

export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: 'admin' | 'project_manager' | 'consultant' | 'client';
  tenantId: string;
  metadata: {
    createdAt: Date;
    lastLogin?: Date;
    status: 'Active' | 'Inactive';
  };
}

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export interface ApprovalResult {
  autoApproved: TimeEntry[];
  requiresReview: { entry: TimeEntry; reasons: string[] }[];
}

export interface BudgetAnalysis {
  budget: {
    total: number;
    spent: number;
    remaining: number;
    percentUsed: number;
  };
  timeline: {
    totalDays: number;
    elapsedDays: number;
    remainingDays: number;
    percentComplete: number;
  };
  burnRate: {
    actual: number;
    planned: number;
    variance: number;
    variancePercent: number;
  };
  forecast: {
    projectedDepletionDate?: Date;
    daysUntilDepletion: number;
    projectedTotalSpend: number;
  };
  status: 'OnTrack' | 'AtRisk' | 'OverBudget';
  recommendations: BudgetRecommendation[];
}

export interface BudgetRecommendation {
  priority: 'HIGH' | 'MEDIUM' | 'LOW' | 'INFO';
  action: string;
  suggestion: string;
}

export interface UsageForecast {
  estimatedUsage: number;
  estimatedCost: number;
  confidence: {
    lower: number;
    upper: number;
  };
  trend: 'Increasing' | 'Decreasing' | 'Stable';
}

export interface ResourceScore {
  resource: Resource;
  score: number;
  breakdown: {
    skillMatch: number;
    costEfficiency: number;
    utilizationOptimization: number;
    availability: number;
    pastPerformance: number;
  };
}

export interface ProjectSchedule {
  tasks: ScheduledTask[];
  criticalPath: ScheduledTask[];
  estimatedCompletionDate: Date;
  resourceUtilization: { [resourceId: string]: number };
}

export interface ScheduledTask {
  id: string;
  name: string;
  duration: number;
  earlyStart: Date;
  earlyFinish: Date;
  lateStart: Date;
  lateFinish: Date;
  slack: number;
  assignedResource?: string;
  scheduledStart: Date;
  priority: 'HIGH' | 'MEDIUM' | 'LOW';
}

// API Response types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    message: string;
    code?: string;
    details?: any;
  };
  meta?: {
    page?: number;
    perPage?: number;
    total?: number;
  };
}

export interface PaginationParams {
  page: number;
  perPage: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface FilterParams {
  status?: string;
  clientId?: string;
  projectId?: string;
  dateFrom?: Date;
  dateTo?: Date;
}
