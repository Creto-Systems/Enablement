/**
 * Database Seed Script for PSA Demo
 * Populates the database with demo data for testing and demonstrations
 */

import { ClientModel } from '../src/server/models/Client';
import { ProjectModel } from '../src/server/models/Project';
import { TimeEntryModel } from '../src/server/models/TimeEntry';
import { InvoiceModel } from '../src/server/models/Invoice';

async function seed() {
  console.log('🌱 Seeding database with demo data...\n');

  // Create demo clients
  console.log('Creating clients...');
  const clients = [
    ClientModel.create({
      name: 'Acme Corporation',
      industry: 'Technology',
      billingInfo: {
        paymentTerms: 30,
        preferredMethod: 'ACH',
        taxId: '12-3456789',
        currency: 'USD',
        billingAddress: {
          street: '123 Tech Drive',
          city: 'San Francisco',
          state: 'CA',
          postalCode: '94105',
          country: 'US',
        },
      },
      contractTerms: {
        defaultRate: 200,
        discountTier: 10,
        msa: true,
        msaExpiryDate: new Date('2026-12-31'),
      },
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        status: 'Active',
      },
    }),
    ClientModel.create({
      name: 'TechStart Inc',
      industry: 'Software',
      billingInfo: {
        paymentTerms: 15,
        preferredMethod: 'Wire',
        taxId: '98-7654321',
        currency: 'USD',
        billingAddress: {
          street: '456 Innovation Blvd',
          city: 'Austin',
          state: 'TX',
          postalCode: '78701',
          country: 'US',
        },
      },
      contractTerms: {
        defaultRate: 180,
        discountTier: 5,
        msa: true,
      },
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        status: 'Active',
      },
    }),
    ClientModel.create({
      name: 'Global Systems LLC',
      industry: 'Enterprise',
      billingInfo: {
        paymentTerms: 45,
        preferredMethod: 'Check',
        taxId: '55-9876543',
        currency: 'USD',
        billingAddress: {
          street: '789 Corporate Way',
          city: 'New York',
          state: 'NY',
          postalCode: '10001',
          country: 'US',
        },
      },
      contractTerms: {
        defaultRate: 250,
        discountTier: 15,
        msa: true,
        msaExpiryDate: new Date('2027-06-30'),
      },
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        status: 'Active',
      },
    }),
  ];

  console.log(`✓ Created ${clients.length} clients\n`);

  // Create demo projects
  console.log('Creating projects...');
  const projects = [
    ProjectModel.create({
      clientId: clients[0].id,
      name: 'Website Redesign',
      description: 'Complete overhaul of corporate website with modern design and improved UX',
      type: 'TimeAndMaterial',
      status: 'Active',
      timeline: {
        startDate: new Date('2025-01-01'),
        endDate: new Date('2025-04-30'),
        milestones: [
          {
            id: 'milestone-1',
            name: 'Design Approval',
            description: 'Client approves new design mockups',
            dueDate: new Date('2025-02-01'),
            status: 'Completed',
            deliverables: ['Design mockups', 'Style guide'],
          },
          {
            id: 'milestone-2',
            name: 'Frontend Development',
            description: 'Complete responsive frontend implementation',
            dueDate: new Date('2025-03-15'),
            status: 'InProgress',
            deliverables: ['Responsive components', 'Navigation system'],
          },
        ],
      },
      budget: {
        totalAmount: 50000,
        currency: 'USD',
        burnRate: 45,
        remainingBudget: 27500,
      },
      team: {
        projectManagerId: 'pm-001',
        resources: [
          {
            userId: 'dev-001',
            role: 'Senior Frontend Developer',
            allocatedHours: 320,
            startDate: new Date('2025-01-01'),
            rate: 200,
          },
          {
            userId: 'dev-002',
            role: 'UI/UX Designer',
            allocatedHours: 160,
            startDate: new Date('2025-01-01'),
            rate: 180,
          },
        ],
      },
      billing: {
        billingCycle: 'Monthly',
        rateCard: {
          id: 'rate-standard',
          name: 'Standard Rate Card',
          baseRates: {
            DEV: 200,
            DESIGN: 180,
            REVIEW: 150,
          },
        },
        invoiceSchedule: [
          new Date('2025-02-01'),
          new Date('2025-03-01'),
          new Date('2025-04-01'),
        ],
      },
      health: {
        budgetStatus: 'OnTrack',
        scheduleStatus: 'OnTrack',
        overallHealth: 88,
      },
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        tags: ['web', 'design', 'frontend'],
      },
    }),
    ProjectModel.create({
      clientId: clients[1].id,
      name: 'Mobile App Development',
      description: 'Native iOS and Android mobile application for customer engagement',
      type: 'FixedPrice',
      status: 'Active',
      timeline: {
        startDate: new Date('2025-02-01'),
        endDate: new Date('2025-07-31'),
        milestones: [
          {
            id: 'milestone-3',
            name: 'Prototype',
            description: 'Working prototype with core features',
            dueDate: new Date('2025-04-01'),
            status: 'Pending',
            deliverables: ['iOS prototype', 'Android prototype'],
          },
        ],
      },
      budget: {
        totalAmount: 100000,
        currency: 'USD',
        burnRate: 32,
        remainingBudget: 68000,
      },
      team: {
        projectManagerId: 'pm-002',
        resources: [
          {
            userId: 'dev-003',
            role: 'Mobile Developer (iOS)',
            allocatedHours: 400,
            startDate: new Date('2025-02-01'),
            rate: 220,
          },
          {
            userId: 'dev-004',
            role: 'Mobile Developer (Android)',
            allocatedHours: 400,
            startDate: new Date('2025-02-01'),
            rate: 220,
          },
        ],
      },
      billing: {
        billingCycle: 'Milestone',
        rateCard: {
          id: 'rate-mobile',
          name: 'Mobile Development Rate Card',
          baseRates: {
            MOBILE_DEV: 220,
            MOBILE_TEST: 180,
          },
        },
        invoiceSchedule: [new Date('2025-04-01'), new Date('2025-07-31')],
      },
      health: {
        budgetStatus: 'OnTrack',
        scheduleStatus: 'Delayed',
        overallHealth: 72,
      },
      metadata: {
        createdAt: new Date(),
        updatedAt: new Date(),
        tags: ['mobile', 'ios', 'android'],
      },
    }),
  ];

  console.log(`✓ Created ${projects.length} projects\n`);

  // Create demo time entries
  console.log('Creating time entries...');
  const timeEntries = [];

  // Generate 30 time entries for the past 2 weeks
  for (let i = 0; i < 30; i++) {
    const daysAgo = Math.floor(i / 2); // 2 entries per day
    const date = new Date();
    date.setDate(date.getDate() - daysAgo);

    const entry = TimeEntryModel.create({
      userId: i % 2 === 0 ? 'dev-001' : 'dev-002',
      projectId: projects[0].id,
      date,
      hours: Math.random() > 0.5 ? 8 : 6.5,
      billable: Math.random() > 0.2, // 80% billable
      description: [
        'Implemented responsive navigation component with mobile menu',
        'Developed hero section with parallax scrolling effect',
        'Created reusable form components with validation',
        'Integrated API endpoints for user authentication',
        'Fixed cross-browser compatibility issues in Safari',
        'Optimized images and assets for faster loading',
        'Conducted code review for pull request #42',
        'Refactored state management using Redux Toolkit',
      ][i % 8],
      activityCode: ['DEV', 'DESIGN', 'REVIEW'][i % 3],
      status: i < 20 ? 'Approved' : i < 25 ? 'Submitted' : 'Draft',
    });

    // Calculate billing for approved entries
    if (entry.status === 'Approved') {
      const rate = entry.activityCode === 'DEV' ? 200 : entry.activityCode === 'DESIGN' ? 180 : 150;
      TimeEntryModel.calculateBillingAmount(entry, rate);
    }

    timeEntries.push(entry);
  }

  console.log(`✓ Created ${timeEntries.length} time entries\n`);

  // Create demo invoices
  console.log('Creating invoices...');
  const invoices = [
    InvoiceModel.create({
      invoiceNumber: 'INV-202512-001',
      clientId: clients[0].id,
      projectId: projects[0].id,
      status: 'Paid',
      dateIssued: new Date('2025-12-01'),
      dateDue: new Date('2025-12-31'),
      datePaid: new Date('2025-12-28'),
      lineItems: [
        {
          id: 'line-1',
          description: 'Frontend Development - 80 hours',
          quantity: 80,
          unitPrice: 200,
          discount: 0,
          amount: 16000,
          timeEntryIds: ['entry-1', 'entry-2', 'entry-3'],
        },
        {
          id: 'line-2',
          description: 'UI/UX Design - 40 hours',
          quantity: 40,
          unitPrice: 180,
          discount: 0,
          amount: 7200,
        },
        {
          id: 'line-3',
          description: 'Code Review - 16 hours',
          quantity: 16,
          unitPrice: 150,
          discount: 0,
          amount: 2400,
        },
      ],
      subtotal: 25600,
      tax: 2048,
      total: 27648,
      currency: 'USD',
      paymentInfo: {
        method: 'ACH',
        transactionId: 'ACH-20251228-0042',
        paidAmount: 27648,
        paidAt: new Date('2025-12-28'),
      },
      metering: {
        meteringPeriodStart: new Date('2025-12-01'),
        meteringPeriodEnd: new Date('2025-12-31'),
        usageMetrics: [
          {
            eventType: 'ReportGeneration',
            quantity: 3,
            unitPrice: 50,
            totalAmount: 150,
          },
        ],
      },
    }),
  ];

  console.log(`✓ Created ${invoices.length} invoices\n`);

  // Summary
  console.log('📊 Seed Summary:');
  console.log(`   Clients: ${clients.length}`);
  console.log(`   Projects: ${projects.length}`);
  console.log(`   Time Entries: ${timeEntries.length}`);
  console.log(`   Invoices: ${invoices.length}`);
  console.log('\n✅ Database seeding completed successfully!');
}

// Run seed if called directly
if (require.main === module) {
  seed()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error('❌ Seeding failed:', error);
      process.exit(1);
    });
}

export { seed };
