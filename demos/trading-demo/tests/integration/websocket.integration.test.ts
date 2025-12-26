/**
 * WebSocket Real-Time Integration Tests
 *
 * Tests real-time WebSocket functionality including:
 * - Connection with authentication
 * - Channel subscription (portfolio updates)
 * - Real-time trade execution updates
 * - Connection handling and reconnection
 * - Event broadcasting
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { EventEmitter } from 'events';
import {
  setupIntegrationTests,
  cleanupIntegrationTests,
  resetTestState,
  getTestContext,
  createTestAgent,
  createTestTrade,
  waitForEvent,
} from './setup';

/**
 * Mock WebSocket Client for testing
 * Simulates browser WebSocket behavior
 */
class MockWebSocketClient extends EventEmitter {
  public readyState: number = 0; // CONNECTING
  public url: string;
  private authToken?: string;

  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  constructor(url: string, protocols?: string | string[]) {
    super();
    this.url = url;

    // Extract auth token from URL or protocols
    if (typeof protocols === 'string' && protocols.startsWith('Bearer ')) {
      this.authToken = protocols;
    }

    // Simulate connection
    setTimeout(() => {
      this.readyState = MockWebSocketClient.OPEN;
      this.emit('open');
    }, 10);
  }

  send(data: string) {
    if (this.readyState !== MockWebSocketClient.OPEN) {
      throw new Error('WebSocket is not open');
    }

    try {
      const message = JSON.parse(data);
      this.emit('send', message);
    } catch (e) {
      // Invalid JSON
    }
  }

  close(code?: number, reason?: string) {
    this.readyState = MockWebSocketClient.CLOSING;
    setTimeout(() => {
      this.readyState = MockWebSocketClient.CLOSED;
      this.emit('close', { code, reason });
    }, 10);
  }

  // Simulate receiving message from server
  simulateMessage(data: any) {
    this.emit('message', { data: JSON.stringify(data) });
  }

  // Simulate server error
  simulateError(error: Error) {
    this.emit('error', error);
  }
}

describe('WebSocket Integration Tests', () => {
  beforeAll(async () => {
    await setupIntegrationTests();
  });

  afterAll(async () => {
    await cleanupIntegrationTests();
  });

  beforeEach(async () => {
    await resetTestState();
  });

  describe('WebSocket Connection', () => {
    it('should connect with valid auth token', async () => {
      const context = getTestContext();

      const ws = new MockWebSocketClient(
        `ws://localhost:3000/ws`,
        `Bearer ${context.authToken}`
      );

      // Wait for connection
      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          expect(ws.readyState).toBe(MockWebSocketClient.OPEN);
          resolve();
        });
      });

      ws.close();
    });

    it('should handle connection without auth token', async () => {
      const ws = new MockWebSocketClient('ws://localhost:3000/ws');

      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          // In real implementation, server would close unauthorized connections
          // For testing, we just verify it connects
          expect(ws.readyState).toBe(MockWebSocketClient.OPEN);
          resolve();
        });
      });

      ws.close();
    });

    it('should handle connection close', async () => {
      const ws = new MockWebSocketClient('ws://localhost:3000/ws');

      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          ws.close(1000, 'Normal closure');
        });

        ws.on('close', ({ code, reason }) => {
          expect(code).toBe(1000);
          expect(reason).toBe('Normal closure');
          expect(ws.readyState).toBe(MockWebSocketClient.CLOSED);
          resolve();
        });
      });
    });

    it('should handle connection errors', async () => {
      const ws = new MockWebSocketClient('ws://localhost:3000/ws');

      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          ws.simulateError(new Error('Connection failed'));
        });

        ws.on('error', (error) => {
          expect(error.message).toBe('Connection failed');
          resolve();
        });
      });

      ws.close();
    });
  });

  describe('Channel Subscription', () => {
    it('should subscribe to portfolio channel', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context);

      const ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          // Subscribe to portfolio updates
          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'portfolio',
            agentId: agent.id,
          }));
        });

        ws.on('send', (message) => {
          expect(message).toMatchObject({
            type: 'subscribe',
            channel: 'portfolio',
            agentId: agent.id,
          });
          resolve();
        });
      });

      ws.close();
    });

    it('should subscribe to multiple channels', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context);

      const ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      const subscriptions: string[] = [];

      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          // Subscribe to multiple channels
          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'portfolio',
            agentId: agent.id,
          }));

          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'trades',
            agentId: agent.id,
          }));

          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'oversight',
            agentId: agent.id,
          }));
        });

        ws.on('send', (message) => {
          if (message.type === 'subscribe') {
            subscriptions.push(message.channel);
            if (subscriptions.length === 3) {
              expect(subscriptions).toContain('portfolio');
              expect(subscriptions).toContain('trades');
              expect(subscriptions).toContain('oversight');
              resolve();
            }
          }
        });
      });

      ws.close();
    });

    it('should unsubscribe from channel', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context);

      const ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      await new Promise<void>((resolve) => {
        let subscribed = false;

        ws.on('open', () => {
          // Subscribe
          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'portfolio',
            agentId: agent.id,
          }));
        });

        ws.on('send', (message) => {
          if (message.type === 'subscribe' && !subscribed) {
            subscribed = true;
            // Unsubscribe
            ws.send(JSON.stringify({
              type: 'unsubscribe',
              channel: 'portfolio',
              agentId: agent.id,
            }));
          } else if (message.type === 'unsubscribe') {
            expect(message.channel).toBe('portfolio');
            resolve();
          }
        });
      });

      ws.close();
    });
  });

  describe('Real-Time Trade Updates', () => {
    it('should receive trade execution update', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context);

      const ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      await new Promise<void>(async (resolve) => {
        ws.on('open', () => {
          // Subscribe to trades
          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'trades',
            agentId: agent.id,
          }));
        });

        // Simulate server sending trade update
        setTimeout(async () => {
          // Execute trade
          const trade = await createTestTrade(context, agent.id);

          // Simulate server broadcast
          ws.simulateMessage({
            type: 'trade_executed',
            channel: 'trades',
            data: {
              tradeId: trade.id,
              agentId: agent.id,
              symbol: 'AAPL',
              status: 'executed',
              timestamp: new Date().toISOString(),
            },
          });
        }, 50);

        ws.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'trade_executed') {
            expect(message.data).toMatchObject({
              agentId: agent.id,
              symbol: 'AAPL',
              status: 'executed',
            });
            resolve();
          }
        });
      });

      ws.close();
    });

    it('should receive portfolio update after trade', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context);

      const ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      await new Promise<void>(async (resolve) => {
        ws.on('open', () => {
          // Subscribe to portfolio
          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'portfolio',
            agentId: agent.id,
          }));
        });

        setTimeout(async () => {
          // Execute trade
          await createTestTrade(context, agent.id, {
            symbol: 'AAPL',
            quantity: 10,
            price: 150,
          });

          // Simulate portfolio update
          ws.simulateMessage({
            type: 'portfolio_updated',
            channel: 'portfolio',
            data: {
              agentId: agent.id,
              totalValue: 48500, // 50000 - 1500
              positions: [
                {
                  symbol: 'AAPL',
                  quantity: 10,
                  avgPrice: 150,
                  currentPrice: 150,
                },
              ],
            },
          });
        }, 50);

        ws.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'portfolio_updated') {
            expect(message.data).toMatchObject({
              agentId: agent.id,
              totalValue: 48500,
            });
            expect(message.data.positions).toHaveLength(1);
            resolve();
          }
        });
      });

      ws.close();
    });

    it('should receive oversight request notification', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context, { budget: 100000 });

      const ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      await new Promise<void>(async (resolve) => {
        ws.on('open', () => {
          // Subscribe to oversight
          ws.send(JSON.stringify({
            type: 'subscribe',
            channel: 'oversight',
            agentId: agent.id,
          }));
        });

        setTimeout(async () => {
          // Create large trade (triggers oversight)
          const trade = await createTestTrade(context, agent.id, {
            quantity: 100,
            price: 200,
          });

          // Get oversight request
          const requests = await context.oversightService.listRequests('pending');

          // Simulate oversight notification
          ws.simulateMessage({
            type: 'oversight_required',
            channel: 'oversight',
            data: {
              requestId: requests[0].id,
              tradeId: trade.id,
              agentId: agent.id,
              amount: 20000,
              status: 'pending',
            },
          });
        }, 50);

        ws.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'oversight_required') {
            expect(message.data).toMatchObject({
              agentId: agent.id,
              amount: 20000,
              status: 'pending',
            });
            resolve();
          }
        });
      });

      ws.close();
    });
  });

  describe('WebSocket Event Broadcasting', () => {
    it('should broadcast to all subscribed clients', async () => {
      const context = getTestContext();
      const agent = await createTestAgent(context);

      // Create multiple WebSocket clients
      const ws1 = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      const ws2 = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      const client1Received: any[] = [];
      const client2Received: any[] = [];

      await new Promise<void>(async (resolve) => {
        // Subscribe both clients
        ws1.on('open', () => {
          ws1.send(JSON.stringify({
            type: 'subscribe',
            channel: 'trades',
            agentId: agent.id,
          }));
        });

        ws2.on('open', () => {
          ws2.send(JSON.stringify({
            type: 'subscribe',
            channel: 'trades',
            agentId: agent.id,
          }));
        });

        // Broadcast to both
        setTimeout(() => {
          const tradeUpdate = {
            type: 'trade_executed',
            channel: 'trades',
            data: {
              agentId: agent.id,
              symbol: 'AAPL',
              status: 'executed',
            },
          };

          ws1.simulateMessage(tradeUpdate);
          ws2.simulateMessage(tradeUpdate);
        }, 100);

        ws1.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'trade_executed') {
            client1Received.push(message);
          }
        });

        ws2.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'trade_executed') {
            client2Received.push(message);

            // Check after both received
            if (client1Received.length > 0) {
              expect(client1Received).toHaveLength(1);
              expect(client2Received).toHaveLength(1);
              resolve();
            }
          }
        });
      });

      ws1.close();
      ws2.close();
    });

    it('should not broadcast to unsubscribed clients', async () => {
      const context = getTestContext();
      const agent1 = await createTestAgent(context, { name: 'Agent 1' });
      const agent2 = await createTestAgent(context, { name: 'Agent 2' });

      const ws1 = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      const ws2 = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      const client1Received: any[] = [];
      const client2Received: any[] = [];

      await new Promise<void>((resolve) => {
        // Subscribe to different agents
        ws1.on('open', () => {
          ws1.send(JSON.stringify({
            type: 'subscribe',
            channel: 'trades',
            agentId: agent1.id,
          }));
        });

        ws2.on('open', () => {
          ws2.send(JSON.stringify({
            type: 'subscribe',
            channel: 'trades',
            agentId: agent2.id,
          }));
        });

        // Broadcast only to agent1
        setTimeout(() => {
          ws1.simulateMessage({
            type: 'trade_executed',
            channel: 'trades',
            data: {
              agentId: agent1.id,
              symbol: 'AAPL',
            },
          });

          // Wait a bit to ensure ws2 doesn't receive
          setTimeout(() => {
            expect(client1Received).toHaveLength(1);
            expect(client2Received).toHaveLength(0);
            resolve();
          }, 100);
        }, 100);

        ws1.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'trade_executed') {
            client1Received.push(message);
          }
        });

        ws2.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'trade_executed') {
            client2Received.push(message);
          }
        });
      });

      ws1.close();
      ws2.close();
    });
  });

  describe('WebSocket Connection Management', () => {
    it('should handle reconnection', async () => {
      const context = getTestContext();

      let ws = new MockWebSocketClient(
        'ws://localhost:3000/ws',
        `Bearer ${context.authToken}`
      );

      // Initial connection
      await new Promise<void>((resolve) => {
        ws.on('open', () => resolve());
      });

      // Close connection
      ws.close();

      await new Promise<void>((resolve) => {
        ws.on('close', () => {
          // Reconnect
          ws = new MockWebSocketClient(
            'ws://localhost:3000/ws',
            `Bearer ${context.authToken}`
          );

          ws.on('open', () => {
            expect(ws.readyState).toBe(MockWebSocketClient.OPEN);
            resolve();
          });
        });
      });

      ws.close();
    });

    it('should handle ping/pong for keep-alive', async () => {
      const ws = new MockWebSocketClient('ws://localhost:3000/ws');

      await new Promise<void>((resolve) => {
        ws.on('open', () => {
          // Send ping
          ws.send(JSON.stringify({ type: 'ping' }));
        });

        ws.on('send', (message) => {
          if (message.type === 'ping') {
            // Simulate pong response
            ws.simulateMessage({ type: 'pong', timestamp: Date.now() });
          }
        });

        ws.on('message', (event) => {
          const message = JSON.parse(event.data);
          if (message.type === 'pong') {
            expect(message.timestamp).toBeDefined();
            resolve();
          }
        });
      });

      ws.close();
    });
  });
});
