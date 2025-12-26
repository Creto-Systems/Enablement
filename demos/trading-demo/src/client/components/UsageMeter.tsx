import React from 'react';
import clsx from 'clsx';

export interface UsageMeterProps {
  used: number;
  total: number;
  label: string;
  showWarning?: boolean;
}

export function UsageMeter({ used, total, label, showWarning = true }: UsageMeterProps) {
  const percentage = Math.round((used / total) * 100);
  const remaining = total - used;

  const getColorClass = () => {
    if (percentage >= 80) return 'bg-red-500';
    if (percentage >= 60) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  const getTextColorClass = () => {
    if (percentage >= 80) return 'text-red-700';
    if (percentage >= 60) return 'text-yellow-700';
    return 'text-green-700';
  };

  const showWarningIcon = showWarning && percentage >= 80;
  const isCritical = percentage >= 90;

  return (
    <div className="space-y-2">
      {/* Header */}
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700">{label}</span>
        <div className="flex items-center gap-2">
          <span className={clsx('text-sm font-bold', getTextColorClass())}>
            {percentage}%
          </span>
          {showWarningIcon && (
            <svg
              className={clsx('w-4 h-4', {
                'text-red-600': isCritical,
                'text-yellow-600': !isCritical,
              })}
              fill="currentColor"
              viewBox="0 0 20 20"
              title="High usage warning"
            >
              <path
                fillRule="evenodd"
                d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
          )}
        </div>
      </div>

      {/* Progress Bar */}
      <div
        role="progressbar"
        aria-valuenow={percentage}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={`${label} usage: ${percentage}%`}
        className="w-full bg-gray-200 rounded-full h-3 overflow-hidden"
      >
        <div
          className={clsx('h-full transition-all duration-300', getColorClass())}
          style={{ width: `${percentage}%` }}
        />
      </div>

      {/* Usage Details */}
      <div className="flex items-center justify-between text-xs text-gray-600">
        <span>
          ${used.toLocaleString()} / ${total.toLocaleString()}
        </span>
        <span className="font-medium">${remaining.toLocaleString()} remaining</span>
      </div>

      {/* Critical Warning */}
      {isCritical && (
        <div className="bg-red-50 border border-red-200 rounded-md p-2">
          <p className="text-xs font-medium text-red-800">
            Critical: Usage exceeds 90%
          </p>
        </div>
      )}
    </div>
  );
}
