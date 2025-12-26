export interface ITradeService {
  submitTrade(agentId: string, dto: SubmitTradeDTO): Promise<TradeSubmissionResult>;
  getTrades(agentId: string, options: GetTradesOptions): Promise<PaginatedTrades>;
  getTrade(tradeId: string): Promise<Trade | null>;
  cancelTrade(tradeId: string): Promise<void>;
}

export interface SubmitTradeDTO {
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price?: number;
  type: 'market' | 'limit' | 'stop';
}

export interface Trade {
  id: string;
  agentId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price?: number;
  type: 'market' | 'limit' | 'stop';
  status: TradeStatus;
  submittedAt: Date;
  executedAt?: Date;
}

export type TradeStatus =
  | 'pending'
  | 'pending_approval'
  | 'approved'
  | 'rejected'
  | 'executing'
  | 'completed'
  | 'cancelled'
  | 'failed';

export interface TradeSubmissionResult {
  trade: Trade;
  requiresOversight: boolean;
  reason?: string;
}

export interface GetTradesOptions {
  page?: number;
  limit?: number;
  status?: TradeStatus;
}

export interface PaginatedTrades {
  trades: Trade[];
  total: number;
  page: number;
  limit: number;
}
