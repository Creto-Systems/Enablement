import { randomUUID } from 'crypto';
import { getDatabase } from '../database/connection';
import { MeteringService } from './metering.service';
import { Agent, AgentStatus, AgentConfig, TradingStrategy } from '../../types/models';

export interface CreateAgentRequest {
  name: string;
  strategy: TradingStrategy;
  budget: number;
  riskTolerance: number;
  oversightThreshold?: number;
  autoApprovalEnabled?: boolean;
  autoApprovalLimit?: number;
}

export class AgentService {
  constructor(private meteringService: MeteringService) {}

  async createAgent(request: CreateAgentRequest): Promise<Agent> {
    // Validate budget
    if (request.budget < 1000 || request.budget > 1000000) {
      throw new Error('Budget must be between $1,000 and $1,000,000');
    }

    const db = getDatabase();
    const agentId = randomUUID();
    const portfolioId = randomUUID();
    const now = new Date().toISOString();

    const config: AgentConfig = {
      strategy: request.strategy,
      riskTolerance: request.riskTolerance,
      maxPositionSize: 0.3,
      dailyTradeLimit: 100,
      oversightThreshold: request.oversightThreshold || 50000,
      autoApprovalEnabled: request.autoApprovalEnabled ?? true,
      autoApprovalLimit: request.autoApprovalLimit || 10000,
    };

    // Insert agent
    const insertAgent = db.prepare(`
      INSERT INTO agents (id, name, status, budget, available_budget, created_at, last_active_at, config, portfolio_id)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    insertAgent.run(
      agentId,
      request.name,
      AgentStatus.ACTIVE,
      request.budget,
      request.budget,
      now,
      now,
      JSON.stringify(config),
      portfolioId
    );

    // Create portfolio
    const metrics = {
      totalReturn: 0,
      annualizedReturn: 0,
      sharpeRatio: 0,
      maxDrawdown: 0,
      winRate: 0,
      avgWin: 0,
      avgLoss: 0,
      totalTrades: 0,
    };

    const insertPortfolio = db.prepare(`
      INSERT INTO portfolios (id, agent_id, total_value, cash_balance, daily_change, daily_change_percent, realized_pnl, unrealized_pnl, metrics, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    insertPortfolio.run(
      portfolioId,
      agentId,
      request.budget,
      request.budget,
      0,
      0,
      0,
      0,
      JSON.stringify(metrics),
      now
    );

    // Initialize quota
    await this.meteringService.recordEvent({
      agentId,
      eventType: 'agent_created',
      amount: 0,
      metadata: {
        budget: request.budget,
        strategy: request.strategy,
      },
    });

    return await this.getAgent(agentId) as Agent;
  }

  async getAgent(id: string): Promise<Agent | null> {
    const db = getDatabase();
    const stmt = db.prepare('SELECT * FROM agents WHERE id = ?');
    const row = stmt.get(id) as any;

    if (!row) {
      return null;
    }

    return {
      id: row.id,
      name: row.name,
      status: row.status as AgentStatus,
      budget: row.budget,
      availableBudget: row.available_budget,
      createdAt: new Date(row.created_at),
      lastActiveAt: new Date(row.last_active_at),
      config: JSON.parse(row.config),
      portfolioId: row.portfolio_id,
    };
  }

  async listAgents(): Promise<Agent[]> {
    const db = getDatabase();
    const stmt = db.prepare('SELECT * FROM agents ORDER BY created_at DESC');
    const rows = stmt.all() as any[];

    return rows.map(row => ({
      id: row.id,
      name: row.name,
      status: row.status as AgentStatus,
      budget: row.budget,
      availableBudget: row.available_budget,
      createdAt: new Date(row.created_at),
      lastActiveAt: new Date(row.last_active_at),
      config: JSON.parse(row.config),
      portfolioId: row.portfolio_id,
    }));
  }

  async terminateAgent(id: string): Promise<Agent> {
    const db = getDatabase();
    const agent = await this.getAgent(id);

    if (!agent) {
      throw new Error('Agent not found');
    }

    const stmt = db.prepare('UPDATE agents SET status = ? WHERE id = ?');
    stmt.run(AgentStatus.TERMINATED, id);

    return await this.getAgent(id) as Agent;
  }

  async updateLastActive(id: string): Promise<void> {
    const db = getDatabase();
    const stmt = db.prepare('UPDATE agents SET last_active_at = ? WHERE id = ?');
    stmt.run(new Date().toISOString(), id);
  }

  async deductBudget(id: string, amount: number): Promise<void> {
    const db = getDatabase();
    const stmt = db.prepare('UPDATE agents SET available_budget = available_budget - ? WHERE id = ?');
    stmt.run(amount, id);
  }
}
