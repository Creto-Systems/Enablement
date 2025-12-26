import React from 'react';
import clsx from 'clsx';

export interface Agent {
  id: string;
  name: string;
  status: 'active' | 'inactive' | 'paused';
  budget: {
    total: number;
    used: number;
    remaining: number;
  };
  performance: {
    dailyPnL: number;
    totalPnL: number;
    winRate: number;
  };
}

export interface AgentCardProps {
  agent: Agent;
  onSelect: (agentId: string) => void;
  selected?: boolean;
}

export function AgentCard({ agent, onSelect, selected = false }: AgentCardProps) {
  const utilizationPercent = (agent.budget.used / agent.budget.total) * 100;
  const isPnLPositive = agent.performance.dailyPnL >= 0;

  const handleClick = () => {
    onSelect(agent.id);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onSelect(agent.id);
    }
  };

  return (
    <button
      role="button"
      aria-label={`${agent.name} - ${agent.status}`}
      aria-pressed={selected}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      className={clsx(
        'w-full p-4 rounded-lg border-2 transition-all text-left',
        'hover:shadow-lg focus:outline-none focus:ring-2 focus:ring-offset-2',
        {
          'opacity-60': agent.status === 'inactive',
          'ring-2 ring-blue-500': selected,
          'border-gray-200 hover:border-gray-300': !selected,
        }
      )}
    >
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-lg font-semibold text-gray-900">{agent.name}</h3>
        <span
          className={clsx('px-2 py-1 rounded-full text-xs font-medium', {
            'bg-green-100 text-green-800': agent.status === 'active',
            'bg-gray-100 text-gray-800': agent.status === 'inactive',
            'bg-yellow-100 text-yellow-800': agent.status === 'paused',
          })}
        >
          {agent.status}
        </span>
      </div>

      {/* Budget Utilization */}
      <div className="mb-3">
        <div className="flex justify-between text-sm mb-1">
          <span className="text-gray-600">Budget Used</span>
          <span className="font-medium">
            ${agent.budget.used.toLocaleString()} / ${agent.budget.total.toLocaleString()}
          </span>
        </div>
        <div
          role="progressbar"
          aria-valuenow={Math.round(utilizationPercent)}
          aria-valuemin={0}
          aria-valuemax={100}
          className="w-full bg-gray-200 rounded-full h-2 overflow-hidden"
        >
          <div
            className={clsx('h-full transition-all', {
              'bg-green-500': utilizationPercent < 60,
              'bg-yellow-500': utilizationPercent >= 60 && utilizationPercent < 80,
              'bg-red-500': utilizationPercent >= 80,
            })}
            style={{ width: `${utilizationPercent}%` }}
          />
        </div>
      </div>

      {/* Performance Metrics */}
      <div className="grid grid-cols-2 gap-3">
        <div>
          <span className="text-xs text-gray-600 block">Daily P&L</span>
          <span
            className={clsx('text-lg font-bold', {
              'text-green-600': isPnLPositive,
              'text-red-600': !isPnLPositive,
            })}
          >
            {isPnLPositive ? '+' : '-'}${Math.abs(agent.performance.dailyPnL).toFixed(2)}
          </span>
        </div>
        <div>
          <span className="text-xs text-gray-600 block">Win Rate</span>
          <span className="text-lg font-bold text-gray-900">
            {Math.round(agent.performance.winRate * 100)}%
          </span>
        </div>
      </div>
    </button>
  );
}
