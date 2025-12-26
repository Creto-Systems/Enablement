import React, { useEffect, useRef } from 'react';
import type { Agent } from '../../shared/types';

interface Connection {
  from: string;
  to: string;
  messageCount: number;
  encrypted: boolean;
}

interface NetworkGraphProps {
  agents: Agent[];
  connections: Connection[];
}

const NetworkGraph: React.FC<NetworkGraphProps> = ({ agents, connections }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width;
    canvas.height = rect.height;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    if (agents.length === 0) {
      ctx.fillStyle = '#9CA3AF';
      ctx.font = '16px sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText('No agents active', canvas.width / 2, canvas.height / 2);
      return;
    }

    // Calculate positions (circular layout)
    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    const radius = Math.min(canvas.width, canvas.height) * 0.35;

    const positions = agents.map((agent, index) => {
      const angle = (index / agents.length) * 2 * Math.PI - Math.PI / 2;
      return {
        agent,
        x: centerX + radius * Math.cos(angle),
        y: centerY + radius * Math.sin(angle),
      };
    });

    // Draw connections
    connections.forEach((conn) => {
      const from = positions.find((p) => p.agent.id === conn.from);
      const to = positions.find((p) => p.agent.id === conn.to);

      if (from && to) {
        ctx.beginPath();
        ctx.moveTo(from.x, from.y);
        ctx.lineTo(to.x, to.y);
        ctx.strokeStyle = conn.encrypted ? '#10B981' : '#9CA3AF';
        ctx.lineWidth = Math.min(2 + conn.messageCount / 5, 6);
        ctx.stroke();

        // Draw message count
        const midX = (from.x + to.x) / 2;
        const midY = (from.y + to.y) / 2;
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(midX - 15, midY - 10, 30, 20);
        ctx.fillStyle = conn.encrypted ? '#10B981' : '#9CA3AF';
        ctx.font = '12px sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(conn.messageCount.toString(), midX, midY);
      }
    });

    // Draw agents
    positions.forEach(({ agent, x, y }) => {
      // Draw circle
      ctx.beginPath();
      ctx.arc(x, y, 40, 0, 2 * Math.PI);

      // Color based on type
      let color = '#9CA3AF';
      if (agent.type === 'flight') color = '#3B82F6';
      else if (agent.type === 'hotel') color = '#8B5CF6';
      else if (agent.type === 'activity') color = '#10B981';
      else if (agent.type === 'budget') color = '#F59E0B';

      ctx.fillStyle = color;
      ctx.fill();

      // Draw status ring
      if (agent.status === 'working') {
        ctx.strokeStyle = '#FFFFFF';
        ctx.lineWidth = 3;
        ctx.stroke();
      }

      // Draw icon
      ctx.fillStyle = '#FFFFFF';
      ctx.font = 'bold 24px sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      const icon =
        agent.type === 'flight'
          ? '✈️'
          : agent.type === 'hotel'
          ? '🏨'
          : agent.type === 'activity'
          ? '🎭'
          : '💰';
      ctx.fillText(icon, x, y);

      // Draw label
      ctx.fillStyle = '#1F2937';
      ctx.font = '12px sans-serif';
      const label = agent.type.charAt(0).toUpperCase() + agent.type.slice(1);
      ctx.fillText(label, x, y + 55);
    });
  }, [agents, connections]);

  return (
    <div className="space-y-4">
      {/* Canvas */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <canvas ref={canvasRef} className="w-full h-[500px]" />
      </div>

      {/* Legend */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
        <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">Legend</h4>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded-full bg-blue-500" />
            <span className="text-sm text-gray-700 dark:text-gray-300">Flight Agent</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded-full bg-purple-500" />
            <span className="text-sm text-gray-700 dark:text-gray-300">Hotel Agent</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded-full bg-green-500" />
            <span className="text-sm text-gray-700 dark:text-gray-300">Activity Agent</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded-full bg-orange-500" />
            <span className="text-sm text-gray-700 dark:text-gray-300">Budget Agent</span>
          </div>
        </div>
        <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div className="w-12 h-0.5 bg-green-500" />
              <span className="text-sm text-gray-700 dark:text-gray-300">Encrypted</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-12 h-0.5 bg-gray-400" />
              <span className="text-sm text-gray-700 dark:text-gray-300">Unencrypted</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-full border-2 border-white bg-blue-500" />
              <span className="text-sm text-gray-700 dark:text-gray-300">Active</span>
            </div>
          </div>
        </div>
      </div>

      {/* Connection Stats */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
        <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
          Network Statistics
        </h4>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Total Connections</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {connections.length}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Total Messages</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {connections.reduce((sum, conn) => sum + conn.messageCount, 0)}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Encryption Rate</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {connections.length > 0
                ? (
                    (connections.filter((c) => c.encrypted).length / connections.length) *
                    100
                  ).toFixed(0)
                : 0}
              %
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default NetworkGraph;
