import React, { useEffect, useRef } from 'react';
import type { AgentMessage } from '../../shared/types';

interface MessagePanelProps {
  messages: AgentMessage[];
  onSend?: (message: Partial<AgentMessage>) => void;
}

const MessagePanel: React.FC<MessagePanelProps> = ({ messages, onSend }) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const getAgentIcon = (agentId: string) => {
    if (agentId.includes('flight')) return '✈️';
    if (agentId.includes('hotel')) return '🏨';
    if (agentId.includes('activity')) return '🎭';
    if (agentId.includes('budget')) return '💰';
    return '🤖';
  };

  const getAgentName = (agentId: string) => {
    if (agentId.includes('flight')) return 'Flight Agent';
    if (agentId.includes('hotel')) return 'Hotel Agent';
    if (agentId.includes('activity')) return 'Activity Agent';
    if (agentId.includes('budget')) return 'Budget Agent';
    return agentId;
  };

  const getMessageTypeColor = (type: string) => {
    switch (type) {
      case 'request':
        return 'bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200';
      case 'response':
        return 'bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200';
      case 'notification':
        return 'bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200';
      case 'error':
        return 'bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-200';
      default:
        return 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200';
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  const formatPayload = (payload: any) => {
    if (typeof payload === 'string') return payload;
    return JSON.stringify(payload, null, 2);
  };

  return (
    <div className="flex flex-col h-[600px]">
      {/* Messages Container */}
      <div className="flex-1 overflow-y-auto space-y-4 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
        {messages.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
            <div className="text-center">
              <p className="text-lg font-medium">No messages yet</p>
              <p className="text-sm mt-2">
                Agent communication will appear here with E2E encryption
              </p>
            </div>
          </div>
        ) : (
          messages.map((message) => (
            <div
              key={message.id}
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4"
            >
              {/* Message Header */}
              <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-3">
                  {/* From Agent */}
                  <div className="flex items-center gap-2">
                    <span className="text-xl">{getAgentIcon(message.from)}</span>
                    <div>
                      <p className="text-sm font-medium text-gray-900 dark:text-white">
                        {getAgentName(message.from)}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {message.from.slice(0, 12)}...
                      </p>
                    </div>
                  </div>

                  {/* Arrow */}
                  <div className="text-gray-400">→</div>

                  {/* To Agent */}
                  <div className="flex items-center gap-2">
                    <span className="text-xl">{getAgentIcon(message.to)}</span>
                    <div>
                      <p className="text-sm font-medium text-gray-900 dark:text-white">
                        {getAgentName(message.to)}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {message.to.slice(0, 12)}...
                      </p>
                    </div>
                  </div>
                </div>

                {/* Timestamp & Type */}
                <div className="text-right">
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    {formatTime(message.timestamp)}
                  </p>
                  <span
                    className={`inline-block mt-1 px-2 py-0.5 rounded-full text-xs font-medium ${getMessageTypeColor(
                      message.type
                    )}`}
                  >
                    {message.type}
                  </span>
                </div>
              </div>

              {/* Encryption Badge */}
              {message.encrypted && (
                <div className="mb-3 flex items-center gap-2">
                  <div className="encrypted-badge">
                    🔒 End-to-End Encrypted
                  </div>
                  {message.signature && (
                    <div className="encrypted-badge bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200">
                      ✍️ Signed
                    </div>
                  )}
                </div>
              )}

              {/* Message Payload */}
              <div className="bg-gray-50 dark:bg-gray-900 rounded-md p-3">
                <p className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">
                  Action: {message.payload.action}
                </p>
                {message.payload.data && (
                  <pre className="text-xs text-gray-700 dark:text-gray-300 overflow-x-auto">
                    {formatPayload(message.payload.data)}
                  </pre>
                )}
              </div>

              {/* Correlation ID */}
              <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  Correlation ID: {message.correlationId}
                </p>
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Info Footer */}
      <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
        <div className="flex items-start gap-3">
          <div className="text-2xl">🔐</div>
          <div>
            <p className="text-sm font-medium text-blue-900 dark:text-blue-200">
              creto-messaging Security
            </p>
            <p className="text-xs text-blue-700 dark:text-blue-300 mt-1">
              All agent communication is encrypted end-to-end using the Signal Protocol.
              Messages are signed, encrypted, and include forward secrecy for maximum security.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default MessagePanel;
