import { useQuery } from '@tanstack/react-query';
import type { Agent } from '../../shared/types';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

interface Connection {
  from: string;
  to: string;
  messageCount: number;
  encrypted: boolean;
}

export const useAgents = (tripId?: string) => {
  // Fetch agents
  const { data: agents = [] } = useQuery<Agent[]>({
    queryKey: ['agents', tripId],
    queryFn: async () => {
      if (!tripId) return [];
      const response = await fetch(`${API_BASE}/api/trips/${tripId}/agents`);
      if (!response.ok) throw new Error('Failed to fetch agents');
      return response.json();
    },
    enabled: !!tripId,
    refetchInterval: 3000, // Poll every 3 seconds
  });

  // Fetch connections
  const { data: connections = [] } = useQuery<Connection[]>({
    queryKey: ['connections', tripId],
    queryFn: async () => {
      if (!tripId) return [];
      const response = await fetch(`${API_BASE}/api/trips/${tripId}/connections`);
      if (!response.ok) throw new Error('Failed to fetch connections');
      return response.json();
    },
    enabled: !!tripId,
    refetchInterval: 3000, // Poll every 3 seconds
  });

  return {
    agents,
    connections,
  };
};
