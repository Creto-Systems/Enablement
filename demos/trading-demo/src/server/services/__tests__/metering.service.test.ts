import { MeteringService } from '../metering.service';
import { EventEmitter } from 'events';

describe('MeteringService', () => {
  let meteringService: MeteringService;
  let mockGrpcClient: any;
  let eventEmitter: EventEmitter;

  beforeEach(() => {
    eventEmitter = new EventEmitter();

    // Mock gRPC client
    mockGrpcClient = {
      checkQuota: jest.fn((request, callback) => {
        callback(null, {
          allowed: true,
          remaining: 5000,
          limit: 10000,
        });
      }),
      recordEvent: jest.fn((request, callback) => {
        callback(null, { success: true });
      }),
      getUsage: jest.fn((request, callback) => {
        callback(null, {
          used: 1000,
          limit: 10000,
        });
      }),
    };

    meteringService = new MeteringService(mockGrpcClient, eventEmitter);
  });

  describe('checkQuota', () => {
    it('should return allowed when under limit', async () => {
      const result = await meteringService.checkQuota('agent-123', 1000);

      expect(result.allowed).toBe(true);
      expect(result.remaining).toBe(5000);
      expect(mockGrpcClient.checkQuota).toHaveBeenCalledWith(
        expect.objectContaining({
          agentId: 'agent-123',
          amount: 1000,
        }),
        expect.any(Function)
      );
    });

    it('should return denied when over limit', async () => {
      mockGrpcClient.checkQuota.mockImplementation((request: any, callback: any) => {
        callback(null, {
          allowed: false,
          remaining: 0,
          limit: 10000,
          reason: 'Quota exceeded',
        });
      });

      const result = await meteringService.checkQuota('agent-123', 15000);

      expect(result.allowed).toBe(false);
      expect(result.reason).toBe('Quota exceeded');
    });

    it('should emit warning at 80% usage', async () => {
      mockGrpcClient.checkQuota.mockImplementation((request: any, callback: any) => {
        callback(null, {
          allowed: true,
          remaining: 1500, // 15% remaining
          limit: 10000,
        });
      });

      const warningSpy = jest.fn();
      eventEmitter.on('quota:warning', warningSpy);

      await meteringService.checkQuota('agent-123', 1000);

      expect(warningSpy).toHaveBeenCalledWith({
        agentId: 'agent-123',
        used: 8500,
        limit: 10000,
        percentage: 85,
      });
    });
  });

  describe('recordEvent', () => {
    it('should send event to metering service', async () => {
      const event = {
        agentId: 'agent-123',
        eventType: 'trade_executed',
        amount: 1500,
        metadata: {
          symbol: 'AAPL',
          quantity: 10,
        },
      };

      await meteringService.recordEvent(event);

      expect(mockGrpcClient.recordEvent).toHaveBeenCalledWith(
        expect.objectContaining({
          agentId: 'agent-123',
          eventType: 'trade_executed',
          amount: 1500,
        }),
        expect.any(Function)
      );
    });

    it('should handle service unavailable', async () => {
      mockGrpcClient.recordEvent.mockImplementation((request: any, callback: any) => {
        callback(new Error('Service unavailable'), null);
      });

      const event = {
        agentId: 'agent-123',
        eventType: 'trade_executed',
        amount: 1500,
        metadata: {},
      };

      // Should not throw, but log error
      await expect(meteringService.recordEvent(event)).rejects.toThrow(
        'Service unavailable'
      );
    });
  });

  describe('getUsage', () => {
    it('should return current usage statistics', async () => {
      const usage = await meteringService.getUsage('agent-123');

      expect(usage).toEqual({
        used: 1000,
        limit: 10000,
      });
      expect(mockGrpcClient.getUsage).toHaveBeenCalledWith(
        expect.objectContaining({
          agentId: 'agent-123',
        }),
        expect.any(Function)
      );
    });
  });
});
