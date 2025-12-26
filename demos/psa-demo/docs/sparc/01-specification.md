# SPARC Phase 1: Specification - PSA Demo

## Executive Summary

The Professional Services Automation (PSA) Demo showcases autonomous engagement management for consulting and professional services organizations. This system demonstrates AI-driven project management, intelligent resource allocation, automated time tracking, and usage-based billing through creto-metering integration.

## 1. Functional Requirements

### FR-1: Client Management
- **FR-1.1**: System shall support multi-tenant client organization management
- **FR-1.2**: System shall track client contacts, billing information, and contract terms
- **FR-1.3**: System shall maintain client engagement history and preferences
- **FR-1.4**: System shall support hierarchical client structures (parent/subsidiary)

### FR-2: Project & Engagement Management
- **FR-2.1**: System shall create and manage project engagements with defined scope, timeline, and budget
- **FR-2.2**: System shall support multiple engagement types (T&M, Fixed Price, Retainer, Outcome-based)
- **FR-2.3**: System shall track project phases, milestones, and deliverables
- **FR-2.4**: System shall provide project health indicators (budget burn, timeline status, resource utilization)
- **FR-2.5**: System shall support project templates for common engagement patterns

### FR-3: Resource Allocation & Planning
- **FR-3.1**: System shall maintain consultant profiles with skills, availability, and rates
- **FR-3.2**: System shall provide AI-driven resource recommendations based on skills matching
- **FR-3.3**: System shall track resource capacity and utilization across projects
- **FR-3.4**: System shall detect and flag resource conflicts and over-allocation
- **FR-3.5**: System shall support role-based staffing (e.g., Senior Consultant, Junior Analyst)

### FR-4: Time Tracking & Approval
- **FR-4.1**: System shall capture time entries with project, task, and activity codes
- **FR-4.2**: System shall support multiple time entry methods (manual, timer, bulk import)
- **FR-4.3**: System shall implement approval workflows for time entries
- **FR-4.4**: System shall validate time entries against project budgets and rules
- **FR-4.5**: System shall track billable vs. non-billable hours

### FR-5: Billing & Invoicing
- **FR-5.1**: System shall generate invoices from approved time entries and expenses
- **FR-5.2**: System shall support multiple billing models (hourly, fixed fee, milestone-based)
- **FR-5.3**: System shall apply rate cards and pricing rules automatically
- **FR-5.4**: System shall track invoice status (draft, sent, paid, overdue)
- **FR-5.5**: System shall support invoice customization and branding

### FR-6: Usage Metering & Analytics
- **FR-6.1**: System shall integrate with creto-metering for real-time usage tracking
- **FR-6.2**: System shall meter billable activities (consulting hours, API calls, report generations)
- **FR-6.3**: System shall support tiered pricing models (volume discounts, overage charges)
- **FR-6.4**: System shall provide usage dashboards and forecasting
- **FR-6.5**: System shall track metering events with full audit trail

### FR-7: Task Execution & Sandboxing
- **FR-7.1**: System shall integrate with creto-runtime for sandboxed task execution
- **FR-7.2**: System shall execute report generation in isolated environments
- **FR-7.3**: System shall track task execution metrics and costs
- **FR-7.4**: System shall support automated task scheduling and retry logic

### FR-8: Reporting & Analytics
- **FR-8.1**: System shall provide executive dashboards (revenue, utilization, profitability)
- **FR-8.2**: System shall generate project performance reports
- **FR-8.3**: System shall track KPIs (utilization rate, realization rate, collection rate)
- **FR-8.4**: System shall support custom report generation
- **FR-8.5**: System shall export data in multiple formats (PDF, Excel, CSV)

## 2. Non-Functional Requirements

### NFR-1: Performance
- **NFR-1.1**: System shall load dashboards within 2 seconds
- **NFR-1.2**: System shall process invoice generation within 5 seconds for 1000 time entries
- **NFR-1.3**: System shall support concurrent users (100+ simultaneous sessions)
- **NFR-1.4**: System shall handle 10,000+ time entries per day

### NFR-2: Accuracy & Reliability
- **NFR-2.1**: Billing calculations shall be accurate to 2 decimal places
- **NFR-2.2**: System shall maintain 99.9% uptime for core services
- **NFR-2.3**: Time tracking shall be accurate to the minute
- **NFR-2.4**: System shall prevent duplicate time entries and invoices

### NFR-3: Security & Compliance
- **NFR-3.1**: System shall implement role-based access control (RBAC)
- **NFR-3.2**: System shall encrypt sensitive data at rest and in transit
- **NFR-3.3**: System shall maintain audit logs for all financial transactions
- **NFR-3.4**: System shall comply with SOC 2 Type II standards

### NFR-4: Integration & Extensibility
- **NFR-4.1**: System shall provide REST APIs for third-party integrations
- **NFR-4.2**: System shall integrate with accounting systems (QuickBooks, NetSuite)
- **NFR-4.3**: System shall support webhook notifications for key events
- **NFR-4.4**: System shall provide SDK for custom extensions

### NFR-5: Usability
- **NFR-5.1**: System shall provide intuitive UI requiring minimal training
- **NFR-5.2**: System shall support mobile-responsive design
- **NFR-5.3**: System shall provide contextual help and tooltips
- **NFR-5.4**: System shall support keyboard shortcuts for power users

### NFR-6: Scalability
- **NFR-6.1**: System shall scale horizontally to support growing client base
- **NFR-6.2**: System shall partition data by tenant for multi-tenancy
- **NFR-6.3**: System shall support database sharding for large datasets

### NFR-7: Observability
- **NFR-7.1**: System shall provide real-time monitoring dashboards
- **NFR-7.2**: System shall log all critical operations with context
- **NFR-7.3**: System shall alert on anomalies and errors
- **NFR-7.4**: System shall track performance metrics (response time, throughput)

## 3. User Stories

### Consultant Perspective
1. **As a consultant**, I want to quickly log my time entries so that I can accurately bill clients without administrative burden
2. **As a consultant**, I want to see my weekly utilization so that I can balance billable and non-billable work
3. **As a consultant**, I want to receive notifications when my time entries are approved or rejected
4. **As a consultant**, I want to view my project assignments and upcoming deadlines in one dashboard
5. **As a consultant**, I want to track my time with a running timer so that I don't forget to log hours

### Project Manager Perspective
6. **As a project manager**, I want to monitor budget burn rate in real-time so that I can prevent cost overruns
7. **As a project manager**, I want to allocate resources based on skill matching so that I can deliver quality work
8. **As a project manager**, I want to approve time entries in bulk so that I can efficiently manage large teams
9. **As a project manager**, I want to generate client-facing status reports so that I can communicate progress effectively
10. **As a project manager**, I want to receive alerts when projects are at risk so that I can take corrective action

### Finance/Billing Perspective
11. **As a billing administrator**, I want to generate invoices automatically from approved time so that I can reduce manual errors
12. **As a billing administrator**, I want to apply rate cards and discounts automatically so that pricing is consistent
13. **As a billing administrator**, I want to track invoice payment status so that I can follow up on overdue payments
14. **As a billing administrator**, I want to reconcile metered usage with invoiced amounts so that billing is accurate

### Client Perspective
15. **As a client**, I want to view real-time project status so that I know where my investment is going
16. **As a client**, I want to see detailed time breakdowns so that I can understand what work was performed
17. **As a client**, I want to receive usage alerts so that I can manage my budget proactively
18. **As a client**, I want to access historical reports so that I can analyze trends and ROI

### Executive Perspective
19. **As an executive**, I want to view firm-wide utilization metrics so that I can optimize resource allocation
20. **As an executive**, I want to track revenue forecasts so that I can make informed business decisions
21. **As an executive**, I want to analyze profitability by client and project so that I can focus on high-value engagements

## 4. Data Models

### Client
```typescript
interface Client {
  id: string;
  name: string;
  industry: string;
  parentClientId?: string;
  contacts: Contact[];
  billingInfo: {
    paymentTerms: number; // days
    preferredMethod: 'ACH' | 'Wire' | 'Check' | 'CreditCard';
    taxId: string;
    currency: string;
  };
  contractTerms: {
    defaultRate: number;
    discountTier: number;
    msa: boolean;
    msaExpiryDate?: Date;
  };
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    status: 'Active' | 'Inactive' | 'Prospect';
  };
}
```

### Project / Engagement
```typescript
interface Project {
  id: string;
  clientId: string;
  name: string;
  description: string;
  type: 'TimeAndMaterial' | 'FixedPrice' | 'Retainer' | 'OutcomeBased';
  status: 'Planning' | 'Active' | 'OnHold' | 'Completed' | 'Cancelled';
  timeline: {
    startDate: Date;
    endDate: Date;
    milestones: Milestone[];
  };
  budget: {
    totalHours?: number;
    totalAmount?: number;
    currency: string;
    burnRate: number; // percentage
    remainingBudget: number;
  };
  team: {
    projectManagerId: string;
    resources: ResourceAssignment[];
  };
  billing: {
    billingCycle: 'Weekly' | 'BiWeekly' | 'Monthly' | 'Milestone';
    rateCard: RateCard;
    invoiceSchedule: Date[];
  };
  health: {
    budgetStatus: 'OnTrack' | 'AtRisk' | 'OverBudget';
    scheduleStatus: 'OnTrack' | 'Delayed' | 'Ahead';
    overallHealth: number; // 0-100 score
  };
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    tags: string[];
  };
}
```

### TimeEntry
```typescript
interface TimeEntry {
  id: string;
  userId: string;
  projectId: string;
  taskId?: string;
  date: Date;
  hours: number;
  billable: boolean;
  status: 'Draft' | 'Submitted' | 'Approved' | 'Rejected' | 'Invoiced';
  description: string;
  activityCode: string; // e.g., 'CONSULT', 'ANALYSIS', 'MEETING'
  approver?: {
    userId: string;
    approvedAt: Date;
    comments?: string;
  };
  billing: {
    rate: number;
    amount: number;
    invoiceId?: string;
  };
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    source: 'Manual' | 'Timer' | 'Import';
  };
}
```

### Invoice
```typescript
interface Invoice {
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
  paymentInfo: {
    method?: string;
    transactionId?: string;
    paidAmount?: number;
  };
  metering: {
    meteringPeriodStart: Date;
    meteringPeriodEnd: Date;
    usageMetrics: UsageMetric[];
  };
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    sentAt?: Date;
  };
}
```

### Task (for creto-runtime execution)
```typescript
interface Task {
  id: string;
  type: 'ReportGeneration' | 'DataAnalysis' | 'BulkOperation';
  projectId?: string;
  status: 'Queued' | 'Running' | 'Completed' | 'Failed';
  priority: 'Low' | 'Medium' | 'High';
  runtime: {
    sandboxId: string;
    executionTime: number; // milliseconds
    memoryUsed: number; // MB
    costIncurred: number;
  };
  input: Record<string, any>;
  output?: Record<string, any>;
  error?: {
    message: string;
    stack: string;
  };
  metadata: {
    createdAt: Date;
    startedAt?: Date;
    completedAt?: Date;
    retryCount: number;
  };
}
```

### MeteringEvent
```typescript
interface MeteringEvent {
  id: string;
  clientId: string;
  projectId?: string;
  eventType: 'ConsultingHour' | 'APICall' | 'ReportGeneration' | 'StorageGB' | 'TaskExecution';
  quantity: number;
  unitPrice: number;
  totalAmount: number;
  timestamp: Date;
  metadata: {
    userId?: string;
    resourceId?: string;
    dimensions: Record<string, any>;
  };
  billing: {
    invoiceId?: string;
    billingPeriod: {
      start: Date;
      end: Date;
    };
  };
}
```

## 5. API Endpoints

### Client Management
- `GET /api/clients` - List clients
- `POST /api/clients` - Create client
- `GET /api/clients/:id` - Get client details
- `PUT /api/clients/:id` - Update client
- `DELETE /api/clients/:id` - Deactivate client

### Project Management
- `GET /api/projects` - List projects
- `POST /api/projects` - Create project
- `GET /api/projects/:id` - Get project details
- `PUT /api/projects/:id` - Update project
- `GET /api/projects/:id/health` - Get project health metrics
- `POST /api/projects/:id/milestones` - Add milestone
- `GET /api/projects/:id/budget` - Get budget status

### Resource Management
- `GET /api/resources` - List resources
- `GET /api/resources/:id/availability` - Get resource availability
- `POST /api/resources/allocate` - Allocate resource to project
- `GET /api/resources/recommendations` - Get AI-driven resource recommendations

### Time Tracking
- `GET /api/time-entries` - List time entries
- `POST /api/time-entries` - Create time entry
- `PUT /api/time-entries/:id` - Update time entry
- `POST /api/time-entries/:id/submit` - Submit for approval
- `POST /api/time-entries/:id/approve` - Approve time entry
- `POST /api/time-entries/bulk-approve` - Bulk approve entries

### Billing & Invoicing
- `GET /api/invoices` - List invoices
- `POST /api/invoices/generate` - Generate invoice from time entries
- `GET /api/invoices/:id` - Get invoice details
- `POST /api/invoices/:id/send` - Send invoice to client
- `POST /api/invoices/:id/payment` - Record payment
- `GET /api/billing/rate-cards` - Get rate cards

### Metering
- `POST /api/metering/events` - Record metering event
- `GET /api/metering/usage` - Get usage metrics
- `GET /api/metering/forecast` - Get usage forecast
- `GET /api/metering/pricing` - Get pricing tiers

### Task Execution
- `POST /api/tasks` - Create task for execution
- `GET /api/tasks/:id` - Get task status
- `GET /api/tasks/:id/logs` - Get task logs
- `POST /api/tasks/:id/retry` - Retry failed task

### Reporting
- `GET /api/reports/utilization` - Get utilization report
- `GET /api/reports/revenue` - Get revenue report
- `GET /api/reports/profitability` - Get profitability analysis
- `POST /api/reports/custom` - Generate custom report

## 6. Billing Rules & Metering Events

### Rate Card Rules
```typescript
interface RateCardRule {
  role: string; // 'Senior Consultant', 'Junior Analyst', etc.
  baseRate: number;
  overtime: {
    enabled: boolean;
    threshold: number; // hours per week
    multiplier: number; // e.g., 1.5x
  };
  discounts: {
    volumeTier: { threshold: number; discount: number }[];
    clientTier: { tier: string; discount: number }[];
  };
}
```

### Metering Event Types
1. **Consulting Hours**: Per-hour billing with role-based rates
2. **API Calls**: Usage-based pricing for API integrations
3. **Report Generations**: Per-report pricing with complexity tiers
4. **Storage**: Per-GB monthly pricing
5. **Task Executions**: Runtime-based pricing for sandboxed tasks

### Tiered Pricing Example
```typescript
const pricingTiers = {
  consultingHours: [
    { min: 0, max: 50, price: 200 },
    { min: 51, max: 200, price: 180 }, // 10% discount
    { min: 201, max: Infinity, price: 160 } // 20% discount
  ],
  apiCalls: [
    { min: 0, max: 10000, price: 0.001 },
    { min: 10001, max: 100000, price: 0.0008 },
    { min: 100001, max: Infinity, price: 0.0005 }
  ]
};
```

## 7. Integration Points

### Creto-Metering Integration
- Real-time usage tracking for all billable activities
- Automated event ingestion from time entries
- Tiered pricing calculations
- Usage forecasting and alerting

### Creto-Runtime Integration
- Sandboxed report generation
- Data analysis task execution
- Bulk operation processing
- Cost tracking per task execution

### Accounting System Integration
- Invoice export to QuickBooks/NetSuite
- Payment reconciliation
- GL code mapping
- Revenue recognition

## 8. Success Metrics

### Business Metrics
- **Utilization Rate**: Target 75%+ billable hours
- **Realization Rate**: Target 90%+ (actual revenue vs. potential)
- **Collection Rate**: Target 95%+ within payment terms
- **Revenue Growth**: Track month-over-month growth
- **Client Satisfaction**: NPS score from invoicing transparency

### System Metrics
- **Invoice Accuracy**: 99.5%+ accuracy
- **Time Entry Compliance**: 95%+ daily submissions
- **Budget Forecast Accuracy**: Within 10% variance
- **System Uptime**: 99.9%
- **Average Invoice Generation Time**: <5 seconds

## 9. Demo Scenarios

### Scenario 1: New Client Engagement
1. Create new client profile
2. Set up engagement with budget and timeline
3. Allocate resources using AI recommendations
4. Track time entries throughout engagement
5. Generate and send invoice

### Scenario 2: Resource Optimization
1. View resource utilization dashboard
2. Identify over/under-allocated consultants
3. Receive AI-driven reallocation suggestions
4. Optimize staffing for maximum profitability

### Scenario 3: Usage-Based Billing
1. Configure tiered pricing for client
2. Track real-time usage metrics
3. Generate usage alerts for budget thresholds
4. Produce invoice with detailed usage breakdown

### Scenario 4: Sandboxed Report Generation
1. Client requests custom analytics report
2. System queues report generation task
3. Creto-runtime executes in isolated sandbox
4. Track execution costs and meter appropriately
5. Deliver report and bill for compute time

## 10. Assumptions & Constraints

### Assumptions
- Users have reliable internet connectivity
- Clients accept electronic invoicing
- Time entries are submitted at least weekly
- Rate cards are defined before project start

### Constraints
- Demo uses mock data (not production-ready)
- Limited to English language support
- Single currency per client
- No offline mode for time tracking

## 11. Out of Scope (for Demo)
- Mobile native applications
- Multi-currency conversion
- Payroll processing
- Tax calculation and filing
- Expense management
- Document management system
- Advanced AI forecasting models
- Real-time collaboration features

---

**Document Version**: 1.0
**Last Updated**: 2025-12-26
**Status**: Approved
**Next Phase**: [02-pseudocode.md](./02-pseudocode.md)
