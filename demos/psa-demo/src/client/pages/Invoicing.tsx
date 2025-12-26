import React, { useState } from 'react';

interface Invoice {
  id: string;
  invoiceNumber: string;
  client: string;
  project: string;
  dateIssued: string;
  dateDue: string;
  amount: number;
  status: 'Draft' | 'Sent' | 'Paid' | 'Overdue';
}

const mockInvoices: Invoice[] = [
  {
    id: '1',
    invoiceNumber: 'INV-202512-001',
    client: 'Acme Corp',
    project: 'Website Redesign',
    dateIssued: '2025-12-01',
    dateDue: '2025-12-31',
    amount: 24000,
    status: 'Paid',
  },
  {
    id: '2',
    invoiceNumber: 'INV-202512-002',
    client: 'TechStart Inc',
    project: 'Mobile App',
    dateIssued: '2025-12-15',
    dateDue: '2026-01-14',
    amount: 18500,
    status: 'Sent',
  },
  {
    id: '3',
    invoiceNumber: 'INV-202512-003',
    client: 'Global Systems',
    project: 'API Integration',
    dateIssued: '2025-12-20',
    dateDue: '2026-01-19',
    amount: 12750,
    status: 'Draft',
  },
];

const Invoicing: React.FC = () => {
  const [invoices] = useState<Invoice[]>(mockInvoices);
  const [selectedInvoice, setSelectedInvoice] = useState<Invoice | null>(null);

  const totalBilled = invoices.reduce((sum, inv) => sum + inv.amount, 0);
  const totalPaid = invoices.filter((inv) => inv.status === 'Paid').reduce((sum, inv) => sum + inv.amount, 0);
  const totalOutstanding = totalBilled - totalPaid;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-gray-900">Invoicing</h1>
        <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition">
          + Generate Invoice
        </button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <SummaryCard title="Total Billed" value={`$${(totalBilled / 1000).toFixed(1)}K`} />
        <SummaryCard title="Total Paid" value={`$${(totalPaid / 1000).toFixed(1)}K`} />
        <SummaryCard title="Outstanding" value={`$${(totalOutstanding / 1000).toFixed(1)}K`} />
        <SummaryCard title="Avg. Collection Time" value="18 days" />
      </div>

      {/* Invoices Table */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Invoice #
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Client
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Project
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Date Issued
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Due Date
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Amount
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {invoices.map((invoice) => (
                <tr key={invoice.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-blue-600">
                    {invoice.invoiceNumber}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {invoice.client}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {invoice.project}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {new Date(invoice.dateIssued).toLocaleDateString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {new Date(invoice.dateDue).toLocaleDateString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-semibold text-gray-900">
                    ${invoice.amount.toLocaleString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <StatusBadge status={invoice.status} />
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                    <button
                      onClick={() => setSelectedInvoice(invoice)}
                      className="text-blue-600 hover:text-blue-900 mr-3"
                    >
                      View
                    </button>
                    {invoice.status === 'Draft' && (
                      <button className="text-green-600 hover:text-green-900">Send</button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Invoice Detail Modal */}
      {selectedInvoice && (
        <InvoiceDetailModal invoice={selectedInvoice} onClose={() => setSelectedInvoice(null)} />
      )}
    </div>
  );
};

const SummaryCard: React.FC<{ title: string; value: string }> = ({ title, value }) => (
  <div className="bg-white rounded-lg shadow p-6">
    <div className="text-sm font-medium text-gray-500 mb-1">{title}</div>
    <div className="text-3xl font-bold text-gray-900">{value}</div>
  </div>
);

const StatusBadge: React.FC<{ status: string }> = ({ status }) => {
  const colors: { [key: string]: string } = {
    Draft: 'bg-gray-100 text-gray-800',
    Sent: 'bg-blue-100 text-blue-800',
    Paid: 'bg-green-100 text-green-800',
    Overdue: 'bg-red-100 text-red-800',
  };

  return (
    <span
      className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
        colors[status] || 'bg-gray-100 text-gray-800'
      }`}
    >
      {status}
    </span>
  );
};

const InvoiceDetailModal: React.FC<{ invoice: Invoice; onClose: () => void }> = ({
  invoice,
  onClose,
}) => (
  <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
      <div className="p-6 border-b border-gray-200">
        <div className="flex justify-between items-center">
          <h2 className="text-2xl font-bold text-gray-900">{invoice.invoiceNumber}</h2>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <div className="p-6 space-y-6">
        <div className="grid grid-cols-2 gap-6">
          <div>
            <div className="text-sm font-medium text-gray-500">Client</div>
            <div className="text-lg text-gray-900">{invoice.client}</div>
          </div>
          <div>
            <div className="text-sm font-medium text-gray-500">Project</div>
            <div className="text-lg text-gray-900">{invoice.project}</div>
          </div>
          <div>
            <div className="text-sm font-medium text-gray-500">Date Issued</div>
            <div className="text-lg text-gray-900">{new Date(invoice.dateIssued).toLocaleDateString()}</div>
          </div>
          <div>
            <div className="text-sm font-medium text-gray-500">Due Date</div>
            <div className="text-lg text-gray-900">{new Date(invoice.dateDue).toLocaleDateString()}</div>
          </div>
        </div>

        <div className="border-t border-gray-200 pt-6">
          <h3 className="text-lg font-semibold mb-4">Line Items</h3>
          <table className="min-w-full">
            <thead>
              <tr className="border-b border-gray-200">
                <th className="text-left py-2 text-sm font-medium text-gray-500">Description</th>
                <th className="text-right py-2 text-sm font-medium text-gray-500">Qty</th>
                <th className="text-right py-2 text-sm font-medium text-gray-500">Rate</th>
                <th className="text-right py-2 text-sm font-medium text-gray-500">Amount</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-gray-100">
                <td className="py-3 text-sm">Consulting Hours - Frontend Development</td>
                <td className="text-right text-sm">80</td>
                <td className="text-right text-sm">$200</td>
                <td className="text-right text-sm font-medium">$16,000</td>
              </tr>
              <tr className="border-b border-gray-100">
                <td className="py-3 text-sm">Consulting Hours - Code Review</td>
                <td className="text-right text-sm">40</td>
                <td className="text-right text-sm">$200</td>
                <td className="text-right text-sm font-medium">$8,000</td>
              </tr>
            </tbody>
            <tfoot>
              <tr className="border-t-2 border-gray-300">
                <td colSpan={3} className="py-3 text-right font-semibold">Total:</td>
                <td className="text-right font-bold text-lg">${invoice.amount.toLocaleString()}</td>
              </tr>
            </tfoot>
          </table>
        </div>

        <div className="flex justify-end space-x-3 pt-6 border-t border-gray-200">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition"
          >
            Close
          </button>
          <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition">
            Download PDF
          </button>
        </div>
      </div>
    </div>
  </div>
);

export default Invoicing;
