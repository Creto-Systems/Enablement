import { randomUUID } from 'crypto';
import { TradeService } from './trade.service';
import { EventEmitter } from 'events';

export interface OversightRequest {
  id: string;
  tradeId: string;
  agentId: string;
  symbol: string;
  quantity: number;
  side: 'buy' | 'sell';
  price: number;
  amount: number;
  status: 'pending' | 'approved' | 'rejected';
  createdAt: Date;
  processedAt?: Date;
  approvedBy?: string;
  reason?: string;
}

export interface OversightDecision {
  approved: boolean;
  approvedBy: string;
  reason?: string;
}

export class OversightService {
  private requests: Map<string, OversightRequest> = new Map();
  private readonly APPROVAL_THRESHOLD = 10000; // $10,000
  private readonly BUDGET_PERCENTAGE_THRESHOLD = 0.25; // 25%

  constructor(
    private tradeService: TradeService,
    private eventEmitter?: EventEmitter
  ) {}

  requiresApproval(
    trade: { quantity: number; price: number },
    budget?: number
  ): boolean {
    const amount = trade.quantity * trade.price;

    // Check absolute threshold
    if (amount >= this.APPROVAL_THRESHOLD) {
      return true;
    }

    // Check percentage of budget
    if (budget) {
      const percentage = amount / budget;
      if (percentage >= this.BUDGET_PERCENTAGE_THRESHOLD) {
        return true;
      }
    }

    return false;
  }

  async createRequest(trade: {
    tradeId: string;
    agentId: string;
    symbol: string;
    quantity: number;
    side: 'buy' | 'sell';
    price: number;
    amount: number;
  }): Promise<OversightRequest> {
    const request: OversightRequest = {
      id: randomUUID(),
      tradeId: trade.tradeId,
      agentId: trade.agentId,
      symbol: trade.symbol,
      quantity: trade.quantity,
      side: trade.side,
      price: trade.price,
      amount: trade.amount,
      status: 'pending',
      createdAt: new Date(),
    };

    this.requests.set(request.id, request);

    // Notify approvers
    if (this.eventEmitter) {
      this.eventEmitter.emit('oversight:request_created', {
        requestId: request.id,
        agentId: trade.agentId,
        amount: trade.amount,
      });
    }

    return request;
  }

  async processDecision(
    requestId: string,
    decision: OversightDecision
  ): Promise<OversightRequest> {
    const request = this.requests.get(requestId);

    if (!request) {
      throw new Error('Oversight request not found');
    }

    if (request.status !== 'pending') {
      throw new Error('Request already processed');
    }

    request.status = decision.approved ? 'approved' : 'rejected';
    request.processedAt = new Date();
    request.approvedBy = decision.approvedBy;
    request.reason = decision.reason;

    if (decision.approved) {
      // Execute the trade
      await this.tradeService.executeTrade(request.tradeId);
    } else {
      // Cancel the trade
      await this.tradeService.cancelTrade(request.tradeId);
    }

    this.requests.set(requestId, request);

    return request;
  }

  async getRequest(requestId: string): Promise<OversightRequest | null> {
    return this.requests.get(requestId) || null;
  }

  async listRequests(status?: 'pending' | 'approved' | 'rejected'): Promise<OversightRequest[]> {
    const requests = Array.from(this.requests.values());

    if (status) {
      return requests.filter(r => r.status === status);
    }

    return requests;
  }
}
