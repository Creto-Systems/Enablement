/**
 * Express Application - Main HTTP Server
 *
 * Configures Express with middleware, security, routes, and error handling.
 * Serves both API endpoints and React frontend build.
 */

import express, { Express } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import rateLimit from 'express-rate-limit';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

import { createApiRouter, createHealthRouter, RouteConfig } from './routes/index';
import { errorHandler, notFoundHandler } from './middleware/error-handler';

/**
 * Get __dirname in ESM
 */
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Application Configuration
 */
export interface AppConfig {
  /** Enable CORS */
  cors?: boolean;

  /** CORS origin */
  corsOrigin?: string | string[];

  /** Enable rate limiting */
  rateLimit?: boolean;

  /** Rate limit max requests per window */
  rateLimitMax?: number;

  /** Rate limit window in milliseconds */
  rateLimitWindow?: number;

  /** Enable compression */
  compression?: boolean;

  /** Enable security headers */
  helmet?: boolean;

  /** Serve static files from build directory */
  serveStatic?: boolean;

  /** Static file directory */
  staticDir?: string;
}

/**
 * Default Configuration
 */
const defaultConfig: AppConfig = {
  cors: true,
  corsOrigin: process.env.CLIENT_URL || 'http://localhost:5173',
  rateLimit: true,
  rateLimitMax: 100,
  rateLimitWindow: 15 * 60 * 1000, // 15 minutes
  compression: true,
  helmet: true,
  serveStatic: process.env.NODE_ENV === 'production',
  staticDir: path.join(__dirname, '../../dist/client'),
};

/**
 * Create Express Application
 */
export function createApp(
  routeConfig: RouteConfig,
  config: Partial<AppConfig> = {}
): Express {
  const app = express();
  const finalConfig = { ...defaultConfig, ...config };

  // Trust proxy (for rate limiting behind reverse proxy)
  app.set('trust proxy', 1);

  // Apply Middleware
  setupMiddleware(app, finalConfig);

  // Mount Routes
  setupRoutes(app, routeConfig);

  // Serve Static Files (Production)
  if (finalConfig.serveStatic && finalConfig.staticDir) {
    setupStaticFiles(app, finalConfig.staticDir);
  }

  // Error Handling
  app.use(notFoundHandler);
  app.use(errorHandler);

  return app;
}

/**
 * Setup Middleware
 */
function setupMiddleware(app: Express, config: AppConfig): void {
  // Security Headers
  if (config.helmet) {
    app.use(
      helmet({
        contentSecurityPolicy: false, // Disable for development
      })
    );
  }

  // CORS
  if (config.cors) {
    app.use(
      cors({
        origin: config.corsOrigin,
        credentials: true,
      })
    );
  }

  // Compression
  if (config.compression) {
    app.use(compression());
  }

  // Body Parsing
  app.use(express.json({ limit: '10mb' }));
  app.use(express.urlencoded({ extended: true, limit: '10mb' }));

  // Rate Limiting
  if (config.rateLimit) {
    const limiter = rateLimit({
      windowMs: config.rateLimitWindow!,
      max: config.rateLimitMax!,
      message: 'Too many requests from this IP, please try again later.',
      standardHeaders: true,
      legacyHeaders: false,
    });

    app.use('/api', limiter);
  }

  // Request Logging (Development)
  if (process.env.NODE_ENV !== 'production') {
    app.use((req, res, next) => {
      console.log(`${req.method} ${req.path}`);
      next();
    });
  }
}

/**
 * Setup Routes
 */
function setupRoutes(app: Express, routeConfig: RouteConfig): void {
  // Health checks
  app.use('/', createHealthRouter());

  // API routes
  app.use('/api/v1', createApiRouter(routeConfig));

  // API version redirect
  app.get('/api', (req, res) => {
    res.json({
      version: '1.0.0',
      endpoints: {
        agents: '/api/v1/agents',
        trades: '/api/v1/agents/:agentId/trades',
        oversight: '/api/v1/oversight',
        metering: '/api/v1/metering',
      },
      health: {
        liveness: '/health/live',
        readiness: '/health/ready',
      },
    });
  });
}

/**
 * Setup Static File Serving (Production)
 */
function setupStaticFiles(app: Express, staticDir: string): void {
  // Serve static assets
  app.use(express.static(staticDir));

  // SPA fallback - serve index.html for all non-API routes
  app.get('*', (req, res) => {
    res.sendFile(path.join(staticDir, 'index.html'));
  });

  console.log(`✅ Serving static files from ${staticDir}`);
}
