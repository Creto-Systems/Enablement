import { Request, Response, NextFunction } from 'express';
import { OversightController } from '../oversight.controller';
import { IOversightService } from '../../services/oversight.service.interface';

describe('OversightController', () => {
  let controller: OversightController;
  let mockOversightService: jest.Mocked<IOversightService>;
  let mockRequest: Partial<Request>;
  let mockResponse: Partial<Response>;
  let mockNext: jest.MockedFunction<NextFunction>;

  beforeEach(() => {
    mockOversightService = {
      getPendingRequests: jest.fn(),
      getRequest: jest.fn(),
      approveRequest: jest.fn(),
      rejectRequest: jest.fn(),
      createRequest: jest.fn(),
    } as jest.Mocked<IOversightService>;

    controller = new OversightController(mockOversightService);

    mockRequest = {
      body: {},
      params: {},
      headers: {},
      query: {},
      user: { id: 'user-123', role: 'supervisor' },
    };

    mockResponse = {
      status: jest.fn().mockReturnThis(),
      json: jest.fn().mockReturnThis(),
      send: jest.fn().mockReturnThis(),
    };

    mockNext = jest.fn();
  });

  describe('GET /api/v1/oversight/requests', () => {
    it('should return pending requests', async () => {
      const pendingRequests = [
        {
          id: 'req-1',
          agentId: 'agent-123',
          tradeId: 'trade-456',
          reason: 'Exceeds autonomous threshold',
          status: 'pending',
          createdAt: new Date(),
        },
        {
          id: 'req-2',
          agentId: 'agent-789',
          tradeId: 'trade-012',
          reason: 'High risk symbol',
          status: 'pending',
          createdAt: new Date(),
        },
      ];

      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockOversightService.getPendingRequests.mockResolvedValue(pendingRequests);

      await controller.getPendingRequests(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call
      expect(mockOversightService.getPendingRequests).toHaveBeenCalledWith({});
      expect(mockOversightService.getPendingRequests).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith({
        requests: pendingRequests,
        total: pendingRequests.length,
      });
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should filter by agent', async () => {
      const agentId = 'agent-123';
      const agentRequests = [
        {
          id: 'req-1',
          agentId,
          tradeId: 'trade-456',
          reason: 'Exceeds autonomous threshold',
          status: 'pending',
          createdAt: new Date(),
        },
      ];

      mockRequest.query = { agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockOversightService.getPendingRequests.mockResolvedValue(agentRequests);

      await controller.getPendingRequests(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify filtering by agent
      expect(mockOversightService.getPendingRequests).toHaveBeenCalledWith({
        agentId,
      });

      expect(mockResponse.json).toHaveBeenCalledWith({
        requests: agentRequests,
        total: 1,
      });
    });

    it('should return empty array when no pending requests', async () => {
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockOversightService.getPendingRequests.mockResolvedValue([]);

      await controller.getPendingRequests(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      expect(mockResponse.json).toHaveBeenCalledWith({
        requests: [],
        total: 0,
      });
    });
  });

  describe('POST /api/v1/oversight/requests/:id/approve', () => {
    it('should approve request', async () => {
      const requestId = 'req-123';
      const approvedRequest = {
        id: requestId,
        agentId: 'agent-123',
        tradeId: 'trade-456',
        status: 'approved',
        approvedBy: 'user-123',
        approvedAt: new Date(),
      };

      mockRequest.params = { id: requestId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockOversightService.approveRequest.mockResolvedValue(approvedRequest);

      await controller.approveRequest(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service interaction
      expect(mockOversightService.approveRequest).toHaveBeenCalledWith(
        requestId,
        'user-123'
      );
      expect(mockOversightService.approveRequest).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith(approvedRequest);
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should return 404 for invalid request', async () => {
      const requestId = 'non-existent';

      mockRequest.params = { id: requestId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      const notFoundError = new Error('Oversight request not found');
      (notFoundError as any).statusCode = 404;
      mockOversightService.approveRequest.mockRejectedValue(notFoundError);

      await controller.approveRequest(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify error handling
      expect(mockNext).toHaveBeenCalledWith(notFoundError);
    });

    it('should return 403 when user lacks permission', async () => {
      const requestId = 'req-123';

      mockRequest.params = { id: requestId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockRequest.user = { id: 'user-123', role: 'viewer' }; // Non-supervisor role

      const permissionError = new Error('Insufficient permissions');
      (permissionError as any).statusCode = 403;
      mockOversightService.approveRequest.mockRejectedValue(permissionError);

      await controller.approveRequest(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      expect(mockNext).toHaveBeenCalledWith(permissionError);
    });
  });

  describe('POST /api/v1/oversight/requests/:id/reject', () => {
    it('should reject request with reason', async () => {
      const requestId = 'req-123';
      const rejectionReason = 'Trade parameters outside acceptable risk range';
      const rejectedRequest = {
        id: requestId,
        agentId: 'agent-123',
        tradeId: 'trade-456',
        status: 'rejected',
        rejectedBy: 'user-123',
        rejectedAt: new Date(),
        rejectionReason,
      };

      mockRequest.params = { id: requestId };
      mockRequest.body = { reason: rejectionReason };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockOversightService.rejectRequest.mockResolvedValue(rejectedRequest);

      await controller.rejectRequest(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call with reason
      expect(mockOversightService.rejectRequest).toHaveBeenCalledWith(
        requestId,
        'user-123',
        rejectionReason
      );
      expect(mockOversightService.rejectRequest).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith(rejectedRequest);
    });

    it('should require rejection reason', async () => {
      const requestId = 'req-123';

      mockRequest.params = { id: requestId };
      mockRequest.body = {}; // Missing reason
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      await controller.rejectRequest(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify validation error
      expect(mockNext).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining('reason'),
        })
      );
      expect(mockOversightService.rejectRequest).not.toHaveBeenCalled();
    });

    it('should return 404 for invalid request', async () => {
      const requestId = 'non-existent';

      mockRequest.params = { id: requestId };
      mockRequest.body = { reason: 'Invalid trade' };
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      const notFoundError = new Error('Oversight request not found');
      (notFoundError as any).statusCode = 404;
      mockOversightService.rejectRequest.mockRejectedValue(notFoundError);

      await controller.rejectRequest(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      expect(mockNext).toHaveBeenCalledWith(notFoundError);
    });
  });
});
