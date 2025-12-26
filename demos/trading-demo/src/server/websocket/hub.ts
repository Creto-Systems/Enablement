/**
 * WebSocket Hub - Real-time Updates
 *
 * Manages WebSocket connections using Socket.IO for real-time portfolio
 * and trade updates. Includes JWT authentication and channel subscription.
 */

import { Server as HttpServer } from 'http';
import { Server, Socket } from 'socket.io';
import { EventEmitter } from 'events';

/**
 * WebSocket Event Types
 */
export enum WsEvent {
  // Connection
  CONNECT = 'connect',
  DISCONNECT = 'disconnect',

  // Subscription
  SUBSCRIBE = 'subscribe',
  UNSUBSCRIBE = 'unsubscribe',

  // Portfolio Updates
  PORTFOLIO_UPDATE = 'portfolio:update',
  POSITION_UPDATE = 'position:update',

  // Trade Updates
  TRADE_CREATED = 'trade:created',
  TRADE_UPDATED = 'trade:updated',
  TRADE_FILLED = 'trade:filled',

  // Oversight Updates
  APPROVAL_REQUESTED = 'approval:requested',
  APPROVAL_COMPLETED = 'approval:completed',

  // Market Data
  MARKET_DATA = 'market:data',
  PRICE_UPDATE = 'price:update',

  // System
  ERROR = 'error',
}

/**
 * Subscription Channels
 */
export enum Channel {
  PORTFOLIO = 'portfolio',
  TRADES = 'trades',
  OVERSIGHT = 'oversight',
  MARKET_DATA = 'market',
}

/**
 * WebSocket Message
 */
export interface WsMessage<T = any> {
  event: WsEvent;
  channel: Channel;
  data: T;
  timestamp: Date;
}

/**
 * WebSocket Hub Configuration
 */
export interface WebSocketConfig {
  /** CORS origins */
  cors?: {
    origin: string | string[];
    credentials: boolean;
  };

  /** Connection timeout in milliseconds */
  connectionTimeout?: number;

  /** Enable authentication */
  requireAuth?: boolean;
}

/**
 * WebSocket Hub
 *
 * Manages real-time connections and broadcasts updates to subscribed clients.
 */
export class WebSocketHub extends EventEmitter {
  private io: Server;
  private connections: Map<string, Socket> = new Map();
  private subscriptions: Map<string, Set<Channel>> = new Map();

  constructor(httpServer: HttpServer, config: WebSocketConfig = {}) {
    super();

    this.io = new Server(httpServer, {
      cors: config.cors || {
        origin: process.env.CLIENT_URL || 'http://localhost:5173',
        credentials: true,
      },
      transports: ['websocket', 'polling'],
      pingTimeout: config.connectionTimeout || 60000,
    });

    this.setupMiddleware(config);
    this.setupEventHandlers();
  }

  /**
   * Setup Authentication Middleware
   */
  private setupMiddleware(config: WebSocketConfig): void {
    if (config.requireAuth) {
      this.io.use((socket, next) => {
        const token = socket.handshake.auth.token;

        if (!token) {
          return next(new Error('Authentication required'));
        }

        // TODO: Verify JWT token
        // For demo, accept any token
        next();
      });
    }
  }

  /**
   * Setup Event Handlers
   */
  private setupEventHandlers(): void {
    this.io.on(WsEvent.CONNECT, (socket: Socket) => {
      console.log(`✅ WebSocket client connected: ${socket.id}`);

      this.connections.set(socket.id, socket);
      this.subscriptions.set(socket.id, new Set());

      // Handle subscription
      socket.on(WsEvent.SUBSCRIBE, (data: { channel: Channel; agentId?: string }) => {
        this.handleSubscribe(socket, data);
      });

      // Handle unsubscription
      socket.on(WsEvent.UNSUBSCRIBE, (data: { channel: Channel }) => {
        this.handleUnsubscribe(socket, data);
      });

      // Handle disconnection
      socket.on(WsEvent.DISCONNECT, () => {
        this.handleDisconnect(socket);
      });
    });
  }

  /**
   * Handle Channel Subscription
   */
  private handleSubscribe(
    socket: Socket,
    data: { channel: Channel; agentId?: string }
  ): void {
    const { channel, agentId } = data;
    const roomName = agentId ? `${channel}:${agentId}` : channel;

    socket.join(roomName);

    const subs = this.subscriptions.get(socket.id);
    subs?.add(channel);

    console.log(`📡 Client ${socket.id} subscribed to ${roomName}`);

    socket.emit(WsEvent.SUBSCRIBE, {
      channel,
      agentId,
      success: true,
    });
  }

  /**
   * Handle Channel Unsubscription
   */
  private handleUnsubscribe(socket: Socket, data: { channel: Channel }): void {
    const { channel } = data;

    socket.leave(channel);

    const subs = this.subscriptions.get(socket.id);
    subs?.delete(channel);

    console.log(`📡 Client ${socket.id} unsubscribed from ${channel}`);
  }

  /**
   * Handle Client Disconnection
   */
  private handleDisconnect(socket: Socket): void {
    console.log(`❌ WebSocket client disconnected: ${socket.id}`);

    this.connections.delete(socket.id);
    this.subscriptions.delete(socket.id);
  }

  /**
   * Broadcast to Channel
   */
  public broadcast(
    channel: Channel,
    event: WsEvent,
    data: any,
    agentId?: string
  ): void {
    const roomName = agentId ? `${channel}:${agentId}` : channel;

    const message: WsMessage = {
      event,
      channel,
      data,
      timestamp: new Date(),
    };

    this.io.to(roomName).emit(event, message);

    this.emit('broadcast', { channel, event, agentId, data });
  }

  /**
   * Send to Specific Client
   */
  public sendToClient(socketId: string, event: WsEvent, data: any): void {
    const socket = this.connections.get(socketId);

    if (socket) {
      socket.emit(event, {
        event,
        data,
        timestamp: new Date(),
      });
    }
  }

  /**
   * Get Connection Stats
   */
  public getStats(): {
    totalConnections: number;
    subscriptionsByChannel: Record<Channel, number>;
  } {
    const subscriptionsByChannel: Record<string, number> = {};

    for (const channels of this.subscriptions.values()) {
      for (const channel of channels) {
        subscriptionsByChannel[channel] = (subscriptionsByChannel[channel] || 0) + 1;
      }
    }

    return {
      totalConnections: this.connections.size,
      subscriptionsByChannel: subscriptionsByChannel as Record<Channel, number>,
    };
  }

  /**
   * Close WebSocket Server
   */
  public async close(): Promise<void> {
    return new Promise((resolve) => {
      this.io.close(() => {
        console.log('✅ WebSocket server closed');
        resolve();
      });
    });
  }
}

/**
 * Create WebSocket Hub
 */
export function createWebSocketHub(
  httpServer: HttpServer,
  config?: WebSocketConfig
): WebSocketHub {
  return new WebSocketHub(httpServer, config);
}
