/**
 * Trading Demo Server - Main Entry Point
 *
 * Initializes all services, wires up dependency injection, and starts the server.
 * Coordinates database, gRPC clients, WebSocket server, and HTTP API.
 */

import http from 'http';
import { createApp } from './app';
import { initializeDatabase, closeDatabase } from './database/connection';
import { createGrpcClients } from './grpc/clients';
import { createWebSocketHub } from './websocket/hub';

// Import Services
import { AgentService } from './services/agent.service';
import { TradeService } from './services/trade.service';
import { MeteringService } from './services/metering.service';

// Import Controllers
import { AgentController } from './controllers/agent.controller';
import { TradeController } from './controllers/trade.controller';
import { OversightController } from './controllers/oversight.controller';
import { MeteringController } from './controllers/metering.controller';

/**
 * Server Configuration
 */
interface ServerConfig {
  port: number;
  host: string;
  nodeEnv: string;
}

/**
 * Get Server Configuration
 */
function getServerConfig(): ServerConfig {
  return {
    port: parseInt(process.env.PORT || '3000', 10),
    host: process.env.HOST || '0.0.0.0',
    nodeEnv: process.env.NODE_ENV || 'development',
  };
}

/**
 * Main Server Class
 *
 * Coordinates initialization and lifecycle of all server components.
 */
class TradingDemoServer {
  private httpServer: http.Server | null = null;
  private config: ServerConfig;

  constructor(config: ServerConfig) {
    this.config = config;
  }

  /**
   * Initialize and Start Server
   */
  async start(): Promise<void> {
    console.log('\n🚀 Starting Trading Demo Server...\n');

    try {
      // 1. Initialize Database
      console.log('📦 Initializing database...');
      await initializeDatabase();

      // 2. Initialize gRPC Clients
      console.log('🔌 Connecting to gRPC services...');
      const grpcFactory = createGrpcClients();
      const meteringClient = await grpcFactory.createMeteringClient();
      const oversightClient = await grpcFactory.createOversightClient();

      // 3. Initialize Services (Dependency Injection)
      console.log('⚙️  Initializing services...');
      const meteringService = new MeteringService(meteringClient);
      const agentService = new AgentService(meteringService);
      const tradeService = new TradeService(meteringService, oversightClient);

      // 4. Initialize Controllers
      console.log('🎮 Initializing controllers...');
      const agentController = new AgentController(agentService);
      const tradeController = new TradeController(tradeService);
      const oversightController = new OversightController(oversightClient);
      const meteringController = new MeteringController(meteringService);

      // 5. Create Express App
      console.log('🌐 Creating HTTP server...');
      const app = createApp({
        agentController,
        tradeController,
        oversightController,
        meteringController,
      });

      // 6. Create HTTP Server
      this.httpServer = http.createServer(app);

      // 7. Initialize WebSocket Server
      console.log('📡 Initializing WebSocket server...');
      const wsHub = createWebSocketHub(this.httpServer, {
        requireAuth: false, // Disable auth for demo
      });

      // Connect WebSocket events to services
      this.setupWebSocketIntegration(wsHub, meteringService);

      // 8. Start Server
      await this.listen();

      // 9. Setup Graceful Shutdown
      this.setupGracefulShutdown();

      console.log('\n✅ Trading Demo Server started successfully!\n');
      this.printServerInfo();
    } catch (error) {
      console.error('\n❌ Failed to start server:', error);
      await this.shutdown();
      process.exit(1);
    }
  }

  /**
   * Start HTTP Server Listening
   */
  private listen(): Promise<void> {
    return new Promise((resolve) => {
      this.httpServer!.listen(this.config.port, this.config.host, () => {
        resolve();
      });
    });
  }

  /**
   * Setup WebSocket Integration
   *
   * Connects service events to WebSocket broadcasts.
   */
  private setupWebSocketIntegration(wsHub: any, meteringService: MeteringService): void {
    // Example: Broadcast metering events
    // meteringService.on('event-recorded', (event) => {
    //   wsHub.broadcast('metering', 'event', event);
    // });

    // TODO: Connect more service events to WebSocket
  }

  /**
   * Setup Graceful Shutdown
   */
  private setupGracefulShutdown(): void {
    const signals: NodeJS.Signals[] = ['SIGTERM', 'SIGINT'];

    for (const signal of signals) {
      process.on(signal, async () => {
        console.log(`\n📡 Received ${signal}, starting graceful shutdown...`);
        await this.shutdown();
        process.exit(0);
      });
    }

    process.on('uncaughtException', async (error) => {
      console.error('💥 Uncaught Exception:', error);
      await this.shutdown();
      process.exit(1);
    });

    process.on('unhandledRejection', async (reason, promise) => {
      console.error('💥 Unhandled Rejection at:', promise, 'reason:', reason);
      await this.shutdown();
      process.exit(1);
    });
  }

  /**
   * Graceful Shutdown
   */
  private async shutdown(): Promise<void> {
    console.log('\n🛑 Shutting down server...');

    // Close HTTP server
    if (this.httpServer) {
      await new Promise<void>((resolve) => {
        this.httpServer!.close(() => {
          console.log('✅ HTTP server closed');
          resolve();
        });
      });
    }

    // Close database
    await closeDatabase();

    // Close gRPC clients
    // grpcFactory.close() - if needed

    console.log('✅ Server shutdown complete');
  }

  /**
   * Print Server Information
   */
  private printServerInfo(): void {
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('  Trading Demo Server');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`  🌐 HTTP Server:    http://${this.config.host}:${this.config.port}`);
    console.log(`  📡 WebSocket:      ws://${this.config.host}:${this.config.port}`);
    console.log(`  📚 API Docs:       http://${this.config.host}:${this.config.port}/api`);
    console.log(`  ❤️  Health Check:  http://${this.config.host}:${this.config.port}/health/live`);
    console.log(`  🔧 Environment:    ${this.config.nodeEnv}`);
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
  }
}

/**
 * Bootstrap Server
 */
async function bootstrap(): Promise<void> {
  const config = getServerConfig();
  const server = new TradingDemoServer(config);
  await server.start();
}

// Start Server
if (import.meta.url === `file://${process.argv[1]}`) {
  bootstrap().catch((error) => {
    console.error('💥 Fatal error during bootstrap:', error);
    process.exit(1);
  });
}

export { TradingDemoServer, bootstrap };
