import { OversightService } from '../oversight.service';
import { TradeService } from '../trade.service';
import { EventEmitter } from 'events';

describe('OversightService', () => {
  let oversightService: OversightService;
  let mockTradeService: jest.Mocked<TradeService>;
  let eventEmitter: EventEmitter;

  beforeEach(() => {
    eventEmitter = new EventEmitter();

    mockTradeService = {
      executeTrade: jest.fn().mockResolvedValue({
        id: 'trade-123',
        status: 'executed',
      }),
      cancelTrade: jest.fn().mockResolvedValue({
        id: 'trade-123',
        status: 'cancelled',
      }),
    } as any;

    oversightService = new OversightService(mockTradeService, eventEmitter);
  });

  describe('requiresApproval', () => {
    it('should require approval for trades over threshold', () => {
      const trade = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
      };

      const requires = oversightService.requiresApproval(trade);

      expect(requires).toBe(true);
    });

    it('should not require approval for small trades', () => {
      const trade = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 10,
        side: 'buy' as const,
        price: 150.00,
      };

      const requires = oversightService.requiresApproval(trade);

      expect(requires).toBe(false);
    });

    it('should consider percentage of budget', () => {
      const trade = {
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 100,
        side: 'buy' as const,
        price: 150.00,
      };

      // Trade value: 15000
      // Assuming budget of 50000, this is 30% (over 25% threshold)
      const requires = oversightService.requiresApproval(trade, 50000);

      expect(requires).toBe(true);
    });
  });

  describe('createRequest', () => {
    it('should create pending request', async () => {
      const trade = {
        tradeId: 'trade-123',
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
        amount: 150000,
      };

      const request = await oversightService.createRequest(trade);

      expect(request).toMatchObject({
        tradeId: 'trade-123',
        agentId: 'agent-123',
        status: 'pending',
        amount: 150000,
      });
      expect(request.id).toBeDefined();
      expect(request.createdAt).toBeInstanceOf(Date);
    });

    it('should notify approvers', async () => {
      const notificationSpy = jest.fn();
      eventEmitter.on('oversight:request_created', notificationSpy);

      const trade = {
        tradeId: 'trade-123',
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
        amount: 150000,
      };

      const request = await oversightService.createRequest(trade);

      expect(notificationSpy).toHaveBeenCalledWith({
        requestId: request.id,
        agentId: 'agent-123',
        amount: 150000,
      });
    });
  });

  describe('processDecision', () => {
    it('should execute trade on approval', async () => {
      const trade = {
        tradeId: 'trade-123',
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
        amount: 150000,
      };

      const request = await oversightService.createRequest(trade);

      const decision = await oversightService.processDecision(request.id, {
        approved: true,
        approvedBy: 'admin-123',
        reason: 'Trade within risk parameters',
      });

      expect(decision.status).toBe('approved');
      expect(decision.approvedBy).toBe('admin-123');
      expect(mockTradeService.executeTrade).toHaveBeenCalledWith('trade-123');
    });

    it('should cancel trade on rejection', async () => {
      const trade = {
        tradeId: 'trade-123',
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
        amount: 150000,
      };

      const request = await oversightService.createRequest(trade);

      const decision = await oversightService.processDecision(request.id, {
        approved: false,
        approvedBy: 'admin-123',
        reason: 'Exceeds risk tolerance',
      });

      expect(decision.status).toBe('rejected');
      expect(decision.reason).toBe('Exceeds risk tolerance');
      expect(mockTradeService.cancelTrade).toHaveBeenCalledWith('trade-123');
    });

    it('should throw error for non-existent request', async () => {
      await expect(
        oversightService.processDecision('non-existent-id', {
          approved: true,
          approvedBy: 'admin-123',
        })
      ).rejects.toThrow('Oversight request not found');
    });

    it('should not process already decided request', async () => {
      const trade = {
        tradeId: 'trade-123',
        agentId: 'agent-123',
        symbol: 'AAPL',
        quantity: 1000,
        side: 'buy' as const,
        price: 150.00,
        amount: 150000,
      };

      const request = await oversightService.createRequest(trade);

      await oversightService.processDecision(request.id, {
        approved: true,
        approvedBy: 'admin-123',
      });

      await expect(
        oversightService.processDecision(request.id, {
          approved: false,
          approvedBy: 'admin-456',
        })
      ).rejects.toThrow('Request already processed');
    });
  });
});
