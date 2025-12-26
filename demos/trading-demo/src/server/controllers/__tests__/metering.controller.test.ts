import { Request, Response, NextFunction } from 'express';
import { MeteringController } from '../metering.controller';
import { IMeteringService } from '../../services/metering.service.interface';

describe('MeteringController', () => {
  let controller: MeteringController;
  let mockMeteringService: jest.Mocked<IMeteringService>;
  let mockRequest: Partial<Request>;
  let mockResponse: Partial<Response>;
  let mockNext: jest.MockedFunction<NextFunction>;

  beforeEach(() => {
    mockMeteringService = {
      getUsage: jest.fn(),
      getQuota: jest.fn(),
      trackUsage: jest.fn(),
      resetQuota: jest.fn(),
    } as jest.Mocked<IMeteringService>;

    controller = new MeteringController(mockMeteringService);

    mockRequest = {
      body: {},
      params: {},
      headers: {},
      query: {},
    };

    mockResponse = {
      status: jest.fn().mockReturnThis(),
      json: jest.fn().mockReturnThis(),
      send: jest.fn().mockReturnThis(),
    };

    mockNext = jest.fn();
  });

  describe('GET /api/v1/agents/:id/usage', () => {
    it('should return usage summary', async () => {
      const agentId = 'agent-123';
      const usageSummary = {
        agentId,
        period: 'daily',
        totalTrades: 45,
        totalVolume: 125000,
        apiCalls: 320,
        computeTime: 1800, // seconds
        startDate: new Date('2024-01-01'),
        endDate: new Date('2024-01-01T23:59:59'),
      };

      mockRequest.params = { id: agentId };
      mockRequest.query = { period: 'daily' };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockMeteringService.getUsage.mockResolvedValue(usageSummary);

      await controller.getUsage(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call
      expect(mockMeteringService.getUsage).toHaveBeenCalledWith(agentId, {
        period: 'daily',
      });
      expect(mockMeteringService.getUsage).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith(usageSummary);
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should include breakdown by type', async () => {
      const agentId = 'agent-123';
      const detailedUsage = {
        agentId,
        period: 'daily',
        totalTrades: 45,
        totalVolume: 125000,
        breakdown: {
          trades: {
            buy: 25,
            sell: 20,
          },
          bySymbol: {
            'BTC/USD': 30,
            'ETH/USD': 15,
          },
          byType: {
            market: 35,
            limit: 10,
          },
        },
        apiCalls: 320,
        computeTime: 1800,
        startDate: new Date('2024-01-01'),
        endDate: new Date('2024-01-01T23:59:59'),
      };

      mockRequest.params = { id: agentId };
      mockRequest.query = { period: 'daily', includeBreakdown: 'true' };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockMeteringService.getUsage.mockResolvedValue(detailedUsage);

      await controller.getUsage(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify breakdown is included
      expect(mockMeteringService.getUsage).toHaveBeenCalledWith(agentId, {
        period: 'daily',
        includeBreakdown: true,
      });

      expect(mockResponse.json).toHaveBeenCalledWith(
        expect.objectContaining({
          breakdown: expect.any(Object),
        })
      );
    });

    it('should support different time periods', async () => {
      const agentId = 'agent-123';
      const weeklyUsage = {
        agentId,
        period: 'weekly',
        totalTrades: 315,
        totalVolume: 875000,
        apiCalls: 2240,
        computeTime: 12600,
        startDate: new Date('2024-01-01'),
        endDate: new Date('2024-01-07T23:59:59'),
      };

      mockRequest.params = { id: agentId };
      mockRequest.query = { period: 'weekly' };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockMeteringService.getUsage.mockResolvedValue(weeklyUsage);

      await controller.getUsage(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      expect(mockMeteringService.getUsage).toHaveBeenCalledWith(agentId, {
        period: 'weekly',
      });
    });
  });

  describe('GET /api/v1/agents/:id/quota', () => {
    it('should return quota status', async () => {
      const agentId = 'agent-123';
      const quotaStatus = {
        agentId,
        quotas: {
          daily: {
            trades: {
              limit: 100,
              used: 45,
              remaining: 55,
              resetAt: new Date('2024-01-02T00:00:00'),
            },
            volume: {
              limit: 500000,
              used: 125000,
              remaining: 375000,
              resetAt: new Date('2024-01-02T00:00:00'),
            },
          },
        },
      };

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockMeteringService.getQuota.mockResolvedValue(quotaStatus);

      await controller.getQuota(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call
      expect(mockMeteringService.getQuota).toHaveBeenCalledWith(agentId);
      expect(mockMeteringService.getQuota).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith(quotaStatus);
    });

    it('should include remaining budget', async () => {
      const agentId = 'agent-123';
      const quotaWithBudget = {
        agentId,
        quotas: {
          daily: {
            trades: {
              limit: 100,
              used: 45,
              remaining: 55,
              resetAt: new Date('2024-01-02T00:00:00'),
            },
            volume: {
              limit: 500000,
              used: 125000,
              remaining: 375000,
              resetAt: new Date('2024-01-02T00:00:00'),
            },
          },
          monthly: {
            budget: {
              limit: 10000,
              used: 2500,
              remaining: 7500,
              currency: 'USD',
              resetAt: new Date('2024-02-01T00:00:00'),
            },
          },
        },
      };

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockMeteringService.getQuota.mockResolvedValue(quotaWithBudget);

      await controller.getQuota(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify budget information is included
      expect(mockResponse.json).toHaveBeenCalledWith(
        expect.objectContaining({
          quotas: expect.objectContaining({
            monthly: expect.objectContaining({
              budget: expect.any(Object),
            }),
          }),
        })
      );
    });

    it('should return 404 for non-existent agent', async () => {
      const agentId = 'non-existent';

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      const notFoundError = new Error('Agent not found');
      (notFoundError as any).statusCode = 404;
      mockMeteringService.getQuota.mockRejectedValue(notFoundError);

      await controller.getQuota(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      expect(mockNext).toHaveBeenCalledWith(notFoundError);
    });

    it('should indicate when quota is exhausted', async () => {
      const agentId = 'agent-123';
      const exhaustedQuota = {
        agentId,
        quotas: {
          daily: {
            trades: {
              limit: 100,
              used: 100,
              remaining: 0,
              exhausted: true,
              resetAt: new Date('2024-01-02T00:00:00'),
            },
          },
        },
      };

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockMeteringService.getQuota.mockResolvedValue(exhaustedQuota);

      await controller.getQuota(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      expect(mockResponse.json).toHaveBeenCalledWith(
        expect.objectContaining({
          quotas: expect.objectContaining({
            daily: expect.objectContaining({
              trades: expect.objectContaining({
                exhausted: true,
                remaining: 0,
              }),
            }),
          }),
        })
      );
    });
  });
});
