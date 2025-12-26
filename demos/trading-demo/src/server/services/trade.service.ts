import { randomUUID } from 'crypto';
import { MeteringService } from './metering.service';
import { OversightService } from './oversight.service';

export interface TradeRequest {
  agentId: string;
  symbol: string;
  quantity: number;
  side: 'buy' | 'sell';
  price: number;
}

export interface Trade {
  id: string;
  agentId: string;
  symbol: string;
  quantity: number;
  side: 'buy' | 'sell';
  price: number;
  status: 'pending_approval' | 'executed' | 'cancelled';
  createdAt: Date;
  executedAt?: Date;
  cancelledAt?: Date;
}

export class TradeService {
  private trades: Map<string, Trade> = new Map();

  constructor(
    private meteringService: MeteringService,
    private oversightService: OversightService
  ) {}

  async submitTrade(request: TradeRequest): Promise<Trade> {
    const amount = request.quantity * request.price;

    // Check quota
    const quotaCheck = await this.meteringService.checkQuota(
      request.agentId,
      amount
    );

    if (!quotaCheck.allowed) {
      throw new Error('Trade quota exceeded');
    }

    // Create trade
    const trade: Trade = {
      id: randomUUID(),
      agentId: request.agentId,
      symbol: request.symbol,
      quantity: request.quantity,
      side: request.side,
      price: request.price,
      status: 'executed',
      createdAt: new Date(),
    };

    // Check if oversight approval is required
    if (this.oversightService.requiresApproval(request)) {
      trade.status = 'pending_approval';

      // Create oversight request
      await this.oversightService.createRequest({
        tradeId: trade.id,
        agentId: request.agentId,
        symbol: request.symbol,
        quantity: request.quantity,
        side: request.side,
        price: request.price,
        amount,
      });
    } else {
      // Execute trade immediately
      trade.executedAt = new Date();

      // Record metering event
      await this.meteringService.recordEvent({
        agentId: request.agentId,
        eventType: 'trade_executed',
        amount,
        metadata: {
          symbol: request.symbol,
          quantity: request.quantity,
          side: request.side,
          amount,
        },
      });
    }

    this.trades.set(trade.id, trade);

    return trade;
  }

  async cancelTrade(tradeId: string): Promise<Trade> {
    const trade = this.trades.get(tradeId);

    if (!trade) {
      throw new Error('Trade not found');
    }

    if (trade.status === 'executed') {
      throw new Error('Cannot cancel executed trade');
    }

    trade.status = 'cancelled';
    trade.cancelledAt = new Date();

    this.trades.set(tradeId, trade);

    return trade;
  }

  async getTrade(tradeId: string): Promise<Trade | null> {
    return this.trades.get(tradeId) || null;
  }

  async executeTrade(tradeId: string): Promise<Trade> {
    const trade = this.trades.get(tradeId);

    if (!trade) {
      throw new Error('Trade not found');
    }

    trade.status = 'executed';
    trade.executedAt = new Date();

    const amount = trade.quantity * trade.price;

    // Record metering event
    await this.meteringService.recordEvent({
      agentId: trade.agentId,
      eventType: 'trade_executed',
      amount,
      metadata: {
        symbol: trade.symbol,
        quantity: trade.quantity,
        side: trade.side,
        amount,
      },
    });

    this.trades.set(tradeId, trade);

    return trade;
  }

  async listTrades(agentId?: string): Promise<Trade[]> {
    const trades = Array.from(this.trades.values());

    if (agentId) {
      return trades.filter(t => t.agentId === agentId);
    }

    return trades;
  }
}
