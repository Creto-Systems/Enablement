import React from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, LineChart, Line } from 'recharts';

// Mock data for demo
const utilizationData = [
  { name: 'Mon', billable: 7.5, nonBillable: 0.5 },
  { name: 'Tue', billable: 8, nonBillable: 0 },
  { name: 'Wed', billable: 6.5, nonBillable: 1.5 },
  { name: 'Thu', billable: 7, nonBillable: 1 },
  { name: 'Fri', billable: 8, nonBillable: 0 },
];

const revenueData = [
  { month: 'Jan', revenue: 125000, forecast: 120000 },
  { month: 'Feb', revenue: 132000, forecast: 130000 },
  { month: 'Mar', revenue: 148000, forecast: 140000 },
  { month: 'Apr', revenue: 155000, forecast: 150000 },
  { month: 'May', revenue: 162000, forecast: 160000 },
  { month: 'Jun', revenue: 0, forecast: 170000 },
];

const projectHealthData = [
  { name: 'Website Redesign', status: 'On Track', budget: 85, schedule: 90, health: 88 },
  { name: 'Mobile App', status: 'At Risk', budget: 92, schedule: 78, health: 70 },
  { name: 'API Integration', status: 'On Track', budget: 65, schedule: 70, health: 95 },
  { name: 'Data Migration', status: 'Over Budget', budget: 105, schedule: 95, health: 55 },
];

const Dashboard: React.FC = () => {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
        <div className="text-sm text-gray-500">
          Last updated: {new Date().toLocaleString()}
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <MetricCard
          title="Utilization Rate"
          value="78%"
          change="+5%"
          changeType="positive"
          description="This week"
        />
        <MetricCard
          title="Active Projects"
          value="12"
          change="+2"
          changeType="positive"
          description="vs last month"
        />
        <MetricCard
          title="Revenue (MTD)"
          value="$162K"
          change="+12%"
          changeType="positive"
          description="vs forecast"
        />
        <MetricCard
          title="Pending Approvals"
          value="24"
          change="-8"
          changeType="positive"
          description="time entries"
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Weekly Utilization */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold mb-4">Weekly Utilization</h2>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={utilizationData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="billable" fill="#3b82f6" name="Billable" />
              <Bar dataKey="nonBillable" fill="#94a3b8" name="Non-Billable" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Revenue Trend */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold mb-4">Revenue Trend</h2>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={revenueData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="month" />
              <YAxis />
              <Tooltip formatter={(value) => `$${(value as number).toLocaleString()}`} />
              <Legend />
              <Line type="monotone" dataKey="revenue" stroke="#10b981" name="Actual" strokeWidth={2} />
              <Line type="monotone" dataKey="forecast" stroke="#94a3b8" name="Forecast" strokeDasharray="5 5" />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Project Health */}
      <div className="bg-white rounded-lg shadow">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold">Project Health</h2>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Project
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Budget %
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Schedule %
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Health Score
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {projectHealthData.map((project) => (
                <tr key={project.name} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {project.name}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <StatusBadge status={project.status} />
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <ProgressBar value={project.budget} />
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <ProgressBar value={project.schedule} />
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <HealthScore score={project.health} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

// Helper Components
const MetricCard: React.FC<{
  title: string;
  value: string;
  change: string;
  changeType: 'positive' | 'negative';
  description: string;
}> = ({ title, value, change, changeType, description }) => (
  <div className="bg-white rounded-lg shadow p-6">
    <div className="text-sm font-medium text-gray-500 mb-1">{title}</div>
    <div className="text-3xl font-bold text-gray-900 mb-2">{value}</div>
    <div className="flex items-center text-sm">
      <span
        className={`font-medium ${
          changeType === 'positive' ? 'text-green-600' : 'text-red-600'
        }`}
      >
        {change}
      </span>
      <span className="ml-2 text-gray-500">{description}</span>
    </div>
  </div>
);

const StatusBadge: React.FC<{ status: string }> = ({ status }) => {
  const colors: { [key: string]: string } = {
    'On Track': 'bg-green-100 text-green-800',
    'At Risk': 'bg-yellow-100 text-yellow-800',
    'Over Budget': 'bg-red-100 text-red-800',
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

const ProgressBar: React.FC<{ value: number }> = ({ value }) => {
  const color = value <= 80 ? 'bg-green-600' : value <= 95 ? 'bg-yellow-600' : 'bg-red-600';

  return (
    <div className="flex items-center">
      <div className="w-24 bg-gray-200 rounded-full h-2 mr-2">
        <div className={`h-2 rounded-full ${color}`} style={{ width: `${Math.min(value, 100)}%` }}></div>
      </div>
      <span className="text-sm text-gray-600">{value}%</span>
    </div>
  );
};

const HealthScore: React.FC<{ score: number }> = ({ score }) => {
  const color = score >= 80 ? 'text-green-600' : score >= 60 ? 'text-yellow-600' : 'text-red-600';

  return <span className={`text-sm font-semibold ${color}`}>{score}</span>;
};

export default Dashboard;
