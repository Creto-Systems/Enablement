import { Request, Response, NextFunction } from 'express';
import { TradeController } from '../trade.controller';
import { ITradeService } from '../../services/trade.service.interface';

describe('TradeController', () => {
  let controller: TradeController;
  let mockTradeService: jest.Mocked<ITradeService>;
  let mockRequest: Partial<Request>;
  let mockResponse: Partial<Response>;
  let mockNext: jest.MockedFunction<NextFunction>;

  beforeEach(() => {
    mockTradeService = {
      submitTrade: jest.fn(),
      getTrades: jest.fn(),
      getTrade: jest.fn(),
      cancelTrade: jest.fn(),
    } as jest.Mocked<ITradeService>;

    controller = new TradeController(mockTradeService);

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

  describe('POST /api/v1/agents/:id/trades', () => {
    const validTradeDTO = {
      symbol: 'BTC/USD',
      side: 'buy' as const,
      quantity: 1.5,
      price: 45000,
      type: 'limit' as const,
    };

    it('should submit trade and return 201', async () => {
      const agentId = 'agent-123';
      const submittedTrade = {
        id: 'trade-123',
        agentId,
        ...validTradeDTO,
        status: 'pending',
        submittedAt: new Date(),
      };

      mockRequest.params = { id: agentId };
      mockRequest.body = validTradeDTO;
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockTradeService.submitTrade.mockResolvedValue({
        trade: submittedTrade,
        requiresOversight: false,
      });

      await controller.submitTrade(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service interaction
      expect(mockTradeService.submitTrade).toHaveBeenCalledWith(
        agentId,
        validTradeDTO
      );
      expect(mockTradeService.submitTrade).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(201);
      expect(mockResponse.json).toHaveBeenCalledWith(submittedTrade);
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should return 202 when oversight required', async () => {
      const agentId = 'agent-123';
      const tradeRequiringOversight = {
        id: 'trade-456',
        agentId,
        ...validTradeDTO,
        quantity: 100, // Large quantity triggers oversight
        status: 'pending_approval',
        submittedAt: new Date(),
      };

      mockRequest.params = { id: agentId };
      mockRequest.body = { ...validTradeDTO, quantity: 100 };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockTradeService.submitTrade.mockResolvedValue({
        trade: tradeRequiringOversight,
        requiresOversight: true,
        reason: 'Exceeds autonomous threshold',
      });

      await controller.submitTrade(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify response indicates oversight required
      expect(mockResponse.status).toHaveBeenCalledWith(202);
      expect(mockResponse.json).toHaveBeenCalledWith(
        expect.objectContaining({
          trade: tradeRequiringOversight,
          message: expect.stringContaining('approval required'),
        })
      );
    });

    it('should return 403 when quota exceeded', async () => {
      const agentId = 'agent-123';

      mockRequest.params = { id: agentId };
      mockRequest.body = validTradeDTO;
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      const quotaError = new Error('Daily quota exceeded');
      (quotaError as any).statusCode = 403;
      (quotaError as any).code = 'QUOTA_EXCEEDED';
      mockTradeService.submitTrade.mockRejectedValue(quotaError);

      await controller.submitTrade(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify error handling
      expect(mockNext).toHaveBeenCalledWith(quotaError);
    });

    it('should return 400 for invalid trade parameters', async () => {
      const agentId = 'agent-123';
      const invalidTrade = {
        symbol: '', // Invalid: empty symbol
        side: 'invalid', // Invalid: wrong side
        quantity: -1, // Invalid: negative quantity
      };

      mockRequest.params = { id: agentId };
      mockRequest.body = invalidTrade;
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      await controller.submitTrade(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify validation error
      expect(mockNext).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining('validation'),
        })
      );
      expect(mockTradeService.submitTrade).not.toHaveBeenCalled();
    });
  });

  describe('GET /api/v1/agents/:id/trades', () => {
    it('should return paginated trades', async () => {
      const agentId = 'agent-123';
      const trades = [
        {
          id: 'trade-1',
          agentId,
          symbol: 'BTC/USD',
          side: 'buy' as const,
          quantity: 1.0,
          price: 45000,
          status: 'completed',
        },
        {
          id: 'trade-2',
          agentId,
          symbol: 'ETH/USD',
          side: 'sell' as const,
          quantity: 10.0,
          price: 3000,
          status: 'completed',
        },
      ];

      mockRequest.params = { id: agentId };
      mockRequest.query = { page: '1', limit: '10' };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockTradeService.getTrades.mockResolvedValue({
        trades,
        total: 2,
        page: 1,
        limit: 10,
      });

      await controller.getTrades(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call with pagination
      expect(mockTradeService.getTrades).toHaveBeenCalledWith(agentId, {
        page: 1,
        limit: 10,
      });

      // Verify response structure
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith({
        trades,
        total: 2,
        page: 1,
        limit: 10,
      });
    });

    it('should filter by status', async () => {
      const agentId = 'agent-123';
      const pendingTrades = [
        {
          id: 'trade-3',
          agentId,
          symbol: 'BTC/USD',
          side: 'buy' as const,
          quantity: 1.0,
          price: 45000,
          status: 'pending',
        },
      ];

      mockRequest.params = { id: agentId };
      mockRequest.query = { status: 'pending', page: '1', limit: '10' };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockTradeService.getTrades.mockResolvedValue({
        trades: pendingTrades,
        total: 1,
        page: 1,
        limit: 10,
      });

      await controller.getTrades(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify filtering
      expect(mockTradeService.getTrades).toHaveBeenCalledWith(agentId, {
        status: 'pending',
        page: 1,
        limit: 10,
      });

      expect(mockResponse.json).toHaveBeenCalledWith(
        expect.objectContaining({
          trades: pendingTrades,
          total: 1,
        })
      );
    });

    it('should use default pagination when not specified', async () => {
      const agentId = 'agent-123';

      mockRequest.params = { id: agentId };
      mockRequest.query = {};
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockTradeService.getTrades.mockResolvedValue({
        trades: [],
        total: 0,
        page: 1,
        limit: 20, // Default limit
      });

      await controller.getTrades(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify default pagination
      expect(mockTradeService.getTrades).toHaveBeenCalledWith(agentId, {
        page: 1,
        limit: 20,
      });
    });
  });
});
