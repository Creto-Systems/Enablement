import { TradeService } from '../trade.service';
import { MeteringService } from '../metering.service';
import { OversightService } from '../oversight.service';

describe('TradeService', () => {
  let tradeService: TradeService;
  let mockMeteringService: jest.Mocked<MeteringService>;
  let mockOversightService: jest.Mocked<OversightService>;

  beforeEach(() => {
    mockMeteringService = {
      checkQuota: jest.fn().mockResolvedValue({ allowed: true, remaining: 5000 }),
      recordEvent: jest.fn().mockResolvedValue(undefined),
      getUsage: jest.fn().mockResolvedValue({ used: 1000, limit: 10000 }),
    } as any;

    mockOversightService = {
      requiresApproval: jest.fn().mockReturnValue(false),
      createRequest: jest.fn().mockResolvedValue({ id: 'req-123', status: 'pending' }),
      processDecision: jest.fn().mockResolvedValue(undefined),
    } as any;

    tradeService = new TradeService(mockMeteringService, mockOversightService);
  });

  describe('submitTrade', () => {
    it('should execute trade within budget', async () => {
      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 10,
        side: 'buy' as const,
        price: 150.00,
      };

      const trade = await tradeService.submitTrade(tradeRequest);

      expect(trade).toMatchObject({
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 10,
        side: 'buy',
        price: 150.00,
        status: 'executed',
      });
      expect(trade.id).toBeDefined();
      expect(trade.executedAt).toBeInstanceOf(Date);
      expect(mockMeteringService.recordEvent).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: 'trade_executed',
        })
      );
    });

    it('should trigger oversight for large trades', async () => {
      mockOversightService.requiresApproval.mockReturnValue(true);

      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000, // Large quantity
        side: 'buy' as const,
        price: 150.00,
      };

      const trade = await tradeService.submitTrade(tradeRequest);

      expect(trade.status).toBe('pending_approval');
      expect(mockOversightService.createRequest).toHaveBeenCalledWith(
        expect.objectContaining({
          tradeId: trade.id,
          amount: 150000, // 1000 * 150
        })
      );
    });

    it('should reject trade exceeding quota', async () => {
      mockMeteringService.checkQuota.mockResolvedValue({
        allowed: false,
        remaining: 0,
        reason: 'Quota exceeded',
      });

      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 10,
        side: 'buy' as const,
        price: 150.00,
      };

      await expect(tradeService.submitTrade(tradeRequest)).rejects.toThrow(
        'Trade quota exceeded'
      );
    });

    it('should record metering event', async () => {
      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'TSLA',
        quantity: 5,
        side: 'sell' as const,
        price: 200.00,
      };

      await tradeService.submitTrade(tradeRequest);

      expect(mockMeteringService.recordEvent).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: 'trade_executed',
          agentId: 'agent-123',
          metadata: expect.objectContaining({
            symbol: 'TSLA',
            amount: 1000, // 5 * 200
          }),
        })
      );
    });
  });

  describe('cancelTrade', () => {
    it('should cancel pending trade', async () => {
      mockOversightService.requiresApproval.mockReturnValue(true);

      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
      };

      const trade = await tradeService.submitTrade(tradeRequest);
      const cancelled = await tradeService.cancelTrade(trade.id);

      expect(cancelled.status).toBe('cancelled');
      expect(cancelled.cancelledAt).toBeInstanceOf(Date);
    });

    it('should not cancel executed trade', async () => {
      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 10,
        side: 'buy' as const,
        price: 150.00,
      };

      const trade = await tradeService.submitTrade(tradeRequest);

      await expect(tradeService.cancelTrade(trade.id)).rejects.toThrow(
        'Cannot cancel executed trade'
      );
    });
  });

  describe('getTrade', () => {
    it('should return trade by id', async () => {
      const tradeRequest = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 10,
        side: 'buy' as const,
        price: 150.00,
      };

      const created = await tradeService.submitTrade(tradeRequest);
      const retrieved = await tradeService.getTrade(created.id);

      expect(retrieved).toEqual(created);
    });

    it('should return null for non-existent trade', async () => {
      const retrieved = await tradeService.getTrade('non-existent-id');

      expect(retrieved).toBeNull();
    });
  });
});
