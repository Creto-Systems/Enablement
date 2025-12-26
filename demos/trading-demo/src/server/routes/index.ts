/**
 * Route Configuration - API Endpoint Mapping
 *
 * Mounts all controllers and defines API route structure.
 * Follows RESTful conventions with versioned API paths.
 */

import { Router } from 'express';
import { AgentController } from '../controllers/agent.controller';
import { TradeController } from '../controllers/trade.controller';
import { OversightController } from '../controllers/oversight.controller';
import { MeteringController } from '../controllers/metering.controller';
import { PortfolioController } from '../controllers/portfolio.controller';

/**
 * API Route Configuration
 */
export interface RouteConfig {
  agentController: AgentController;
  tradeController: TradeController;
  oversightController: OversightController;
  meteringController: MeteringController;
  portfolioController: PortfolioController;
}

/**
 * Create API Router
 *
 * Mounts all controllers under /api/v1 prefix.
 */
export function createApiRouter(config: RouteConfig): Router {
  const router = Router();

  // Mount controllers
  mountAgentRoutes(router, config.agentController);
  mountTradeRoutes(router, config.tradeController);
  mountPortfolioRoutes(router, config.portfolioController);
  mountOversightRoutes(router, config.oversightController);
  mountMeteringRoutes(router, config.meteringController);

  return router;
}

/**
 * Agent Routes - /api/v1/agents
 */
function mountAgentRoutes(router: Router, controller: AgentController): void {
  // Create new agent
  router.post('/agents', controller.createAgent);

  // Get agent by ID
  router.get('/agents/:id', controller.getAgent);

  // Terminate agent
  router.delete('/agents/:id', controller.terminateAgent);

  // List all agents (optional)
  // router.get('/agents', controller.listAgents);
}

/**
 * Trade Routes - /api/v1/agents/:agentId/trades
 */
function mountTradeRoutes(router: Router, controller: TradeController): void {
  // Execute trade
  router.post('/agents/:agentId/trades', controller.executeTrade);

  // Get trade by ID
  router.get('/agents/:agentId/trades/:tradeId', controller.getTrade);

  // List agent trades
  router.get('/agents/:agentId/trades', controller.listTrades);

  // Cancel trade
  router.delete('/agents/:agentId/trades/:tradeId', controller.cancelTrade);
}

/**
 * Oversight Routes - /api/v1/oversight
 */
function mountOversightRoutes(router: Router, controller: OversightController): void {
  // List pending approvals
  router.get('/oversight/requests', controller.listRequests);

  // Get request details
  router.get('/oversight/requests/:requestId', controller.getRequest);

  // Approve request
  router.post('/oversight/requests/:requestId/approve', controller.approveRequest);

  // Reject request
  router.post('/oversight/requests/:requestId/reject', controller.rejectRequest);
}

/**
 * Portfolio Routes - /api/v1/portfolio
 */
function mountPortfolioRoutes(router: Router, controller: PortfolioController): void {
  // Get portfolio by agent ID
  router.get('/portfolio/:agentId', controller.getPortfolio);

  // Get portfolio history
  router.get('/portfolio/:agentId/history', controller.getHistory);

  // List positions
  router.get('/portfolio/:agentId/positions', controller.listPositions);
}

/**
 * Metering Routes - /api/v1/metering
 */
function mountMeteringRoutes(router: Router, controller: MeteringController): void {
  // Get usage metrics for agent
  router.get('/metering/agents/:agentId/usage', controller.getUsage);

  // Check quota
  router.get('/metering/agents/:agentId/quota', controller.checkQuota);

  // Record event (internal use)
  router.post('/metering/events', controller.recordEvent);
}

/**
 * Create Health Check Routes
 */
export function createHealthRouter(): Router {
  const router = Router();

  // Liveness probe
  router.get('/health/live', (req, res) => {
    res.status(200).json({
      status: 'ok',
      timestamp: new Date().toISOString(),
    });
  });

  // Readiness probe
  router.get('/health/ready', async (req, res) => {
    // TODO: Check database, gRPC connections
    res.status(200).json({
      status: 'ready',
      checks: {
        database: 'ok',
        grpc: 'ok',
      },
      timestamp: new Date().toISOString(),
    });
  });

  return router;
}
