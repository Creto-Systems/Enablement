import { EventEmitter } from 'events';

export interface QuotaCheckResult {
  allowed: boolean;
  remaining: number;
  limit?: number;
  reason?: string;
}

export interface MeteringEvent {
  agentId: string;
  eventType: string;
  amount: number;
  metadata: Record<string, any>;
}

export interface Usage {
  used: number;
  limit: number;
}

export class MeteringService {
  private readonly QUOTA_WARNING_THRESHOLD = 0.8; // 80%

  constructor(
    private grpcClient: any,
    private eventEmitter?: EventEmitter
  ) {}

  async checkQuota(agentId: string, amount: number): Promise<QuotaCheckResult> {
    return new Promise((resolve, reject) => {
      this.grpcClient.checkQuota(
        { agentId, amount },
        (error: Error | null, response: QuotaCheckResult) => {
          if (error) {
            return reject(error);
          }

          // Check if we should emit a warning
          if (response.allowed && response.limit) {
            const used = response.limit - response.remaining;
            const usagePercentage = used / response.limit;

            if (usagePercentage >= this.QUOTA_WARNING_THRESHOLD && this.eventEmitter) {
              this.eventEmitter.emit('quota:warning', {
                agentId,
                used,
                limit: response.limit,
                percentage: Math.round(usagePercentage * 100),
              });
            }
          }

          resolve(response);
        }
      );
    });
  }

  async recordEvent(event: MeteringEvent): Promise<void> {
    return new Promise((resolve, reject) => {
      this.grpcClient.recordEvent(
        {
          agentId: event.agentId,
          eventType: event.eventType,
          amount: event.amount,
          metadata: JSON.stringify(event.metadata),
          timestamp: new Date().toISOString(),
        },
        (error: Error | null, response: any) => {
          if (error) {
            return reject(error);
          }
          resolve();
        }
      );
    });
  }

  async getUsage(agentId: string): Promise<Usage> {
    return new Promise((resolve, reject) => {
      this.grpcClient.getUsage(
        { agentId },
        (error: Error | null, response: Usage) => {
          if (error) {
            return reject(error);
          }
          resolve(response);
        }
      );
    });
  }
}
