import React from 'react';
import type { Agent } from '../../shared/types';

interface AgentCardProps {
  agent: Agent;
}

const AgentCard: React.FC<AgentCardProps> = ({ agent }) => {
  const getAgentIcon = (type: string) => {
    switch (type) {
      case 'flight':
        return '✈️';
      case 'hotel':
        return '🏨';
      case 'activity':
        return '🎭';
      case 'budget':
        return '💰';
      default:
        return '🤖';
    }
  };

  const getAgentColor = (type: string) => {
    switch (type) {
      case 'flight':
        return 'from-blue-500 to-blue-600';
      case 'hotel':
        return 'from-purple-500 to-purple-600';
      case 'activity':
        return 'from-green-500 to-green-600';
      case 'budget':
        return 'from-orange-500 to-orange-600';
      default:
        return 'from-gray-500 to-gray-600';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'idle':
        return 'bg-gray-400';
      case 'working':
        return 'bg-blue-500 animate-pulse';
      case 'completed':
        return 'bg-green-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-400';
    }
  };

  const formatResponseTime = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-shadow">
      {/* Header with gradient */}
      <div className={`bg-gradient-to-r ${getAgentColor(agent.type)} p-4 text-white`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-2xl">{getAgentIcon(agent.type)}</span>
            <div>
              <h3 className="font-semibold capitalize">
                {agent.type} Agent
              </h3>
              <p className="text-xs opacity-90">{agent.id.slice(0, 8)}...</p>
            </div>
          </div>
          <div className={`w-3 h-3 rounded-full ${getStatusColor(agent.status)}`} />
        </div>
      </div>

      {/* Body */}
      <div className="p-4 space-y-3">
        {/* Status */}
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Status</p>
          <p className="text-sm font-medium capitalize text-gray-900 dark:text-white">
            {agent.status}
          </p>
        </div>

        {/* Metrics */}
        <div className="grid grid-cols-2 gap-3">
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400">Response Time</p>
            <p className="text-sm font-medium text-gray-900 dark:text-white">
              {formatResponseTime(agent.metrics.responseTime)}
            </p>
          </div>
          <div>
            <p className="text-xs text-gray-500 dark:text-gray-400">Success Rate</p>
            <p className="text-sm font-medium text-gray-900 dark:text-white">
              {(agent.metrics.successRate * 100).toFixed(0)}%
            </p>
          </div>
        </div>

        {/* Messages Processed */}
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Messages Processed</p>
          <p className="text-sm font-medium text-gray-900 dark:text-white">
            {agent.metrics.messagesProcessed}
          </p>
        </div>

        {/* Last Active */}
        <div>
          <p className="text-xs text-gray-500 dark:text-gray-400">Last Active</p>
          <p className="text-sm font-medium text-gray-900 dark:text-white">
            {new Date(agent.lastActive).toLocaleTimeString()}
          </p>
        </div>

        {/* Encryption Badge */}
        <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
          <div className="encrypted-badge">
            🔒 E2E Encrypted
          </div>
        </div>
      </div>
    </div>
  );
};

export default AgentCard;
