export interface IMeteringService {
  getUsage(agentId: string, options: UsageOptions): Promise<UsageSummary>;
  getQuota(agentId: string): Promise<QuotaStatus>;
  trackUsage(agentId: string, usage: UsageRecord): Promise<void>;
  resetQuota(agentId: string, quotaType: QuotaType): Promise<void>;
}

export interface UsageOptions {
  period?: 'daily' | 'weekly' | 'monthly' | 'yearly';
  includeBreakdown?: boolean;
}

export interface UsageSummary {
  agentId: string;
  period: string;
  totalTrades: number;
  totalVolume: number;
  apiCalls: number;
  computeTime: number;
  startDate: Date;
  endDate: Date;
  breakdown?: UsageBreakdown;
}

export interface UsageBreakdown {
  trades: {
    buy: number;
    sell: number;
  };
  bySymbol: Record<string, number>;
  byType: Record<string, number>;
}

export interface QuotaStatus {
  agentId: string;
  quotas: {
    daily?: QuotaInfo;
    weekly?: QuotaInfo;
    monthly?: QuotaInfo;
  };
}

export interface QuotaInfo {
  trades?: QuotaDetail;
  volume?: QuotaDetail;
  budget?: BudgetDetail;
}

export interface QuotaDetail {
  limit: number;
  used: number;
  remaining: number;
  exhausted?: boolean;
  resetAt: Date;
}

export interface BudgetDetail extends QuotaDetail {
  currency: string;
}

export type QuotaType = 'daily' | 'weekly' | 'monthly';

export interface UsageRecord {
  type: 'trade' | 'api_call' | 'compute';
  amount: number;
  metadata?: Record<string, any>;
}
