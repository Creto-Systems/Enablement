import React, { useState } from 'react';

interface TimeEntry {
  id: string;
  date: string;
  project: string;
  task: string;
  hours: number;
  description: string;
  status: 'Draft' | 'Submitted' | 'Approved' | 'Rejected';
}

const mockEntries: TimeEntry[] = [
  {
    id: '1',
    date: '2025-12-23',
    project: 'Website Redesign',
    task: 'Frontend Development',
    hours: 8,
    description: 'Implemented responsive navigation component',
    status: 'Approved',
  },
  {
    id: '2',
    date: '2025-12-24',
    project: 'Mobile App',
    task: 'API Integration',
    hours: 6.5,
    description: 'Connected authentication endpoints',
    status: 'Submitted',
  },
  {
    id: '3',
    date: '2025-12-25',
    project: 'Website Redesign',
    task: 'Code Review',
    hours: 2,
    description: 'Reviewed pull requests',
    status: 'Draft',
  },
];

const TimeTracker: React.FC = () => {
  const [entries, setEntries] = useState<TimeEntry[]>(mockEntries);
  const [showForm, setShowForm] = useState(false);
  const [newEntry, setNewEntry] = useState({
    date: new Date().toISOString().split('T')[0],
    project: '',
    task: '',
    hours: 0,
    description: '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const entry: TimeEntry = {
      id: Date.now().toString(),
      ...newEntry,
      status: 'Draft',
    };

    setEntries([entry, ...entries]);
    setShowForm(false);
    setNewEntry({
      date: new Date().toISOString().split('T')[0],
      project: '',
      task: '',
      hours: 0,
      description: '',
    });
  };

  const handleSubmitForApproval = (id: string) => {
    setEntries(
      entries.map((e) => (e.id === id && e.status === 'Draft' ? { ...e, status: 'Submitted' } : e))
    );
  };

  const totalHours = entries.reduce((sum, e) => sum + e.hours, 0);
  const billableHours = entries.filter((e) => e.status === 'Approved').reduce((sum, e) => sum + e.hours, 0);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-gray-900">Time Tracker</h1>
        <button
          onClick={() => setShowForm(!showForm)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          {showForm ? 'Cancel' : '+ New Entry'}
        </button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <SummaryCard title="Total Hours (This Week)" value={totalHours.toFixed(1)} />
        <SummaryCard title="Billable Hours" value={billableHours.toFixed(1)} />
        <SummaryCard
          title="Utilization"
          value={`${((billableHours / 40) * 100).toFixed(0)}%`}
        />
      </div>

      {/* Entry Form */}
      {showForm && (
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold mb-4">New Time Entry</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Date</label>
                <input
                  type="date"
                  value={newEntry.date}
                  onChange={(e) => setNewEntry({ ...newEntry, date: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Hours</label>
                <input
                  type="number"
                  step="0.5"
                  min="0"
                  max="24"
                  value={newEntry.hours || ''}
                  onChange={(e) => setNewEntry({ ...newEntry, hours: parseFloat(e.target.value) })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Project</label>
              <select
                value={newEntry.project}
                onChange={(e) => setNewEntry({ ...newEntry, project: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                required
              >
                <option value="">Select a project</option>
                <option value="Website Redesign">Website Redesign</option>
                <option value="Mobile App">Mobile App</option>
                <option value="API Integration">API Integration</option>
                <option value="Data Migration">Data Migration</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Task</label>
              <input
                type="text"
                value={newEntry.task}
                onChange={(e) => setNewEntry({ ...newEntry, task: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="e.g., Frontend Development"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
              <textarea
                value={newEntry.description}
                onChange={(e) => setNewEntry({ ...newEntry, description: e.target.value })}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Describe what you worked on..."
                required
              />
            </div>

            <div className="flex justify-end">
              <button
                type="submit"
                className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
              >
                Save Entry
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Entries Table */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Date
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Project
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Task
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Hours
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
            {entries.map((entry) => (
              <tr key={entry.id} className="hover:bg-gray-50">
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {new Date(entry.date).toLocaleDateString()}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {entry.project}
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">{entry.task}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {entry.hours}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <StatusBadge status={entry.status} />
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm">
                  {entry.status === 'Draft' && (
                    <button
                      onClick={() => handleSubmitForApproval(entry.id)}
                      className="text-blue-600 hover:text-blue-900"
                    >
                      Submit
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
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
    Submitted: 'bg-blue-100 text-blue-800',
    Approved: 'bg-green-100 text-green-800',
    Rejected: 'bg-red-100 text-red-800',
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

export default TimeTracker;
