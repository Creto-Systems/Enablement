import { useState, useEffect } from 'react';
import type { AgentMessage } from '../../shared/types';

const WS_BASE = import.meta.env.VITE_WS_URL || 'ws://localhost:3000';

export const useMessages = (tripId?: string) => {
  const [messages, setMessages] = useState<AgentMessage[]>([]);
  const [ws, setWs] = useState<WebSocket | null>(null);

  useEffect(() => {
    if (!tripId) return;

    // Connect to WebSocket
    const websocket = new WebSocket(`${WS_BASE}/ws/trips/${tripId}/messages`);

    websocket.onopen = () => {
      console.log('WebSocket connected');
    };

    websocket.onmessage = (event) => {
      try {
        const message: AgentMessage = JSON.parse(event.data);
        setMessages((prev) => [...prev, message]);
      } catch (error) {
        console.error('Failed to parse message:', error);
      }
    };

    websocket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    websocket.onclose = () => {
      console.log('WebSocket disconnected');
    };

    setWs(websocket);

    return () => {
      websocket.close();
    };
  }, [tripId]);

  const sendMessage = (message: Partial<AgentMessage>) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message));
    }
  };

  return {
    messages,
    sendMessage,
  };
};
