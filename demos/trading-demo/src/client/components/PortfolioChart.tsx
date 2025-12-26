import React, { useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  TooltipProps,
} from 'recharts';
import clsx from 'clsx';

export interface PortfolioDataPoint {
  timestamp: string;
  value: number;
}

export interface PortfolioChartProps {
  data: PortfolioDataPoint[];
  height?: number;
}

interface ChartDataPoint {
  date: string;
  value: number;
  timestamp: string;
}

function CustomTooltip({ active, payload }: TooltipProps<number, string>) {
  if (!active || !payload || !payload.length) {
    return null;
  }

  const data = payload[0].payload as ChartDataPoint;

  return (
    <div
      role="tooltip"
      className="bg-white p-3 rounded-lg shadow-lg border border-gray-200"
    >
      <p className="text-sm text-gray-600 mb-1">{data.date}</p>
      <p className="text-lg font-bold text-gray-900">
        ${data.value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
      </p>
    </div>
  );
}

export function PortfolioChart({ data, height = 400 }: PortfolioChartProps) {
  const chartData = useMemo(() => {
    return data.map((point) => ({
      date: new Date(point.timestamp).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
      }),
      value: point.value,
      timestamp: point.timestamp,
    }));
  }, [data]);

  const performance = useMemo(() => {
    if (data.length < 2) {
      return { change: 0, changePercent: 0, isPositive: true };
    }

    const firstValue = data[0].value;
    const lastValue = data[data.length - 1].value;
    const change = lastValue - firstValue;
    const changePercent = (change / firstValue) * 100;

    return {
      change,
      changePercent,
      isPositive: change >= 0,
    };
  }, [data]);

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 bg-gray-50 rounded-lg">
        <p className="text-gray-500">No data available</p>
      </div>
    );
  }

  return (
    <div className="w-full">
      {/* Performance Summary */}
      <div className="mb-4 flex items-center gap-4">
        <div>
          <span className="text-sm text-gray-600">Total Change</span>
          <p
            className={clsx('text-2xl font-bold', {
              'text-green-600': performance.isPositive,
              'text-red-600': !performance.isPositive,
            })}
          >
            {performance.isPositive ? '+' : '-'}$
            {Math.abs(performance.change).toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </p>
        </div>
        <div>
          <span className="text-sm text-gray-600">Percentage</span>
          <p
            className={clsx('text-2xl font-bold', {
              'text-green-600': performance.isPositive,
              'text-red-600': !performance.isPositive,
            })}
          >
            {performance.isPositive ? '+' : ''}
            {performance.changePercent.toFixed(1)}%
          </p>
        </div>
      </div>

      {/* Chart */}
      <ResponsiveContainer width="100%" height={height}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
          <XAxis
            dataKey="date"
            stroke="#6b7280"
            style={{ fontSize: '12px' }}
          />
          <YAxis
            stroke="#6b7280"
            style={{ fontSize: '12px' }}
            tickFormatter={(value) =>
              `$${(value / 1000).toFixed(0)}k`
            }
          />
          <Tooltip content={<CustomTooltip />} />
          <Line
            type="monotone"
            dataKey="value"
            stroke={performance.isPositive ? '#10b981' : '#ef4444'}
            strokeWidth={2}
            dot={{ fill: performance.isPositive ? '#10b981' : '#ef4444', r: 4 }}
            activeDot={{ r: 6 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
