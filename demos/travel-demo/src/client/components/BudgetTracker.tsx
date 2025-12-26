import React from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Legend, Tooltip } from 'recharts';
import type { BudgetAnalysis } from '../../shared/types';

interface BudgetTrackerProps {
  budget: BudgetAnalysis;
}

const BudgetTracker: React.FC<BudgetTrackerProps> = ({ budget }) => {
  const COLORS = {
    flights: '#3B82F6',
    hotels: '#8B5CF6',
    activities: '#10B981',
    buffer: '#F59E0B',
    remaining: '#E5E7EB',
  };

  const pieData = [
    { name: 'Flights', value: budget.breakdown.flights, color: COLORS.flights },
    { name: 'Hotels', value: budget.breakdown.hotels, color: COLORS.hotels },
    { name: 'Activities', value: budget.breakdown.activities, color: COLORS.activities },
    { name: 'Buffer', value: budget.breakdown.buffer, color: COLORS.buffer },
  ];

  const remaining = budget.budget.max - budget.totalWithBuffer;
  if (remaining > 0) {
    pieData.push({ name: 'Remaining', value: remaining, color: COLORS.remaining });
  }

  const utilizationPercentage = (budget.utilizationRate * 100).toFixed(1);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'low':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'high':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
    }
  };

  return (
    <div className="space-y-6">
      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Cost</p>
          <p className="text-3xl font-bold text-gray-900 dark:text-white mt-2">
            {budget.budget.currency} {budget.totalCost.toLocaleString()}
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            With buffer: {budget.budget.currency} {budget.totalWithBuffer.toLocaleString()}
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Budget Range</p>
          <p className="text-3xl font-bold text-gray-900 dark:text-white mt-2">
            {budget.budget.currency} {budget.budget.min.toLocaleString()} -{' '}
            {budget.budget.max.toLocaleString()}
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">Target spending range</p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Utilization</p>
          <p className="text-3xl font-bold text-gray-900 dark:text-white mt-2">
            {utilizationPercentage}%
          </p>
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mt-3">
            <div
              className={`h-2 rounded-full ${
                budget.overBudget
                  ? 'bg-red-500'
                  : budget.underBudget
                  ? 'bg-yellow-500'
                  : 'bg-green-500'
              }`}
              style={{ width: `${Math.min(parseFloat(utilizationPercentage), 100)}%` }}
            />
          </div>
        </div>
      </div>

      {/* Budget Status */}
      {(budget.overBudget || budget.underBudget) && (
        <div
          className={`p-4 rounded-lg ${
            budget.overBudget
              ? 'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800'
              : 'bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800'
          }`}
        >
          <p
            className={`text-sm font-medium ${
              budget.overBudget
                ? 'text-red-800 dark:text-red-200'
                : 'text-yellow-800 dark:text-yellow-200'
            }`}
          >
            {budget.overBudget
              ? '⚠️ Over Budget'
              : '💡 Under Budget - Room for upgrades or additional activities'}
          </p>
        </div>
      )}

      {/* Charts and Breakdown */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Pie Chart */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Budget Distribution
          </h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={pieData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                {pieData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip formatter={(value: number) => `${budget.budget.currency} ${value.toLocaleString()}`} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Breakdown Bars */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Cost Breakdown
          </h3>
          <div className="space-y-4">
            {[
              { label: 'Flights', amount: budget.breakdown.flights, color: COLORS.flights },
              { label: 'Hotels', amount: budget.breakdown.hotels, color: COLORS.hotels },
              { label: 'Activities', amount: budget.breakdown.activities, color: COLORS.activities },
              { label: 'Buffer (10%)', amount: budget.breakdown.buffer, color: COLORS.buffer },
            ].map((item) => (
              <div key={item.label}>
                <div className="flex justify-between mb-1">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    {item.label}
                  </span>
                  <span className="text-sm font-semibold text-gray-900 dark:text-white">
                    {budget.budget.currency} {item.amount.toLocaleString()}
                  </span>
                </div>
                <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                  <div
                    className="h-3 rounded-full"
                    style={{
                      width: `${(item.amount / budget.budget.max) * 100}%`,
                      backgroundColor: item.color,
                    }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Optimization Suggestions */}
      {budget.suggestions.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            💡 Optimization Suggestions
          </h3>
          <div className="space-y-3">
            {budget.suggestions.map((suggestion, index) => (
              <div
                key={index}
                className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800"
              >
                <div className="flex items-start gap-3">
                  <span className="text-xl">
                    {suggestion.category === 'flights' && '✈️'}
                    {suggestion.category === 'hotels' && '🏨'}
                    {suggestion.category === 'activities' && '🎭'}
                  </span>
                  <div className="flex-1">
                    <p className="text-sm font-medium text-blue-900 dark:text-blue-200 capitalize">
                      {suggestion.category}
                    </p>
                    <p className="text-sm text-blue-700 dark:text-blue-300 mt-1">
                      {suggestion.action}
                    </p>
                    {suggestion.potentialSavings && (
                      <p className="text-xs text-green-600 dark:text-green-400 mt-2">
                        💰 Potential savings: {budget.budget.currency}{' '}
                        {suggestion.potentialSavings.toLocaleString()}
                      </p>
                    )}
                    {suggestion.additionalCost && (
                      <p className="text-xs text-orange-600 dark:text-orange-400 mt-2">
                        ⚠️ Additional cost: {budget.budget.currency}{' '}
                        {suggestion.additionalCost.toLocaleString()}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default BudgetTracker;
