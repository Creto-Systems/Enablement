import { Request, Response, NextFunction } from 'express';
import { AgentController } from '../agent.controller';
import { IAgentService } from '../../services/agent.service.interface';

describe('AgentController', () => {
  let controller: AgentController;
  let mockAgentService: jest.Mocked<IAgentService>;
  let mockRequest: Partial<Request>;
  let mockResponse: Partial<Response>;
  let mockNext: jest.MockedFunction<NextFunction>;

  beforeEach(() => {
    // Create mock service with all required methods
    mockAgentService = {
      createAgent: jest.fn(),
      getAgent: jest.fn(),
      terminateAgent: jest.fn(),
      listAgents: jest.fn(),
      updateAgentStatus: jest.fn(),
    } as jest.Mocked<IAgentService>;

    // Create controller with mock service
    controller = new AgentController(mockAgentService);

    // Create mock Express objects
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

  describe('POST /api/v1/agents', () => {
    const validAgentDTO = {
      name: 'TestAgent',
      type: 'trading',
      config: {
        tradingPair: 'BTC/USD',
        maxPositionSize: 10000,
      },
      userId: 'user-123',
    };

    it('should create agent and return 201', async () => {
      const createdAgent = {
        id: 'agent-123',
        ...validAgentDTO,
        status: 'active',
        createdAt: new Date(),
      };

      mockRequest.body = validAgentDTO;
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockAgentService.createAgent.mockResolvedValue(createdAgent);

      await controller.createAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service was called with correct data
      expect(mockAgentService.createAgent).toHaveBeenCalledWith(validAgentDTO);
      expect(mockAgentService.createAgent).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(201);
      expect(mockResponse.json).toHaveBeenCalledWith(createdAgent);
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should return 400 for invalid body', async () => {
      const invalidBody = {
        name: '', // Invalid: empty name
        type: 'invalid-type', // Invalid: wrong type
      };

      mockRequest.body = invalidBody;
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      const validationError = new Error('Validation failed');
      (validationError as any).statusCode = 400;
      (validationError as any).errors = [
        { field: 'name', message: 'Name is required' },
        { field: 'type', message: 'Invalid agent type' },
      ];

      await controller.createAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify next was called with error
      expect(mockNext).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining('validation'),
        })
      );
      expect(mockAgentService.createAgent).not.toHaveBeenCalled();
    });

    it('should return 401 for missing auth', async () => {
      mockRequest.body = validAgentDTO;
      mockRequest.headers = {}; // No authorization header

      await controller.createAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify next was called with auth error
      expect(mockNext).toHaveBeenCalledWith(
        expect.objectContaining({
          message: expect.stringContaining('Authorization'),
        })
      );
      expect(mockAgentService.createAgent).not.toHaveBeenCalled();
    });
  });

  describe('GET /api/v1/agents/:id', () => {
    it('should return agent details', async () => {
      const agentId = 'agent-123';
      const agentDetails = {
        id: agentId,
        name: 'TestAgent',
        type: 'trading',
        status: 'active',
        config: {
          tradingPair: 'BTC/USD',
          maxPositionSize: 10000,
        },
        createdAt: new Date(),
        userId: 'user-123',
      };

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockAgentService.getAgent.mockResolvedValue(agentDetails);

      await controller.getAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call
      expect(mockAgentService.getAgent).toHaveBeenCalledWith(agentId);
      expect(mockAgentService.getAgent).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(200);
      expect(mockResponse.json).toHaveBeenCalledWith(agentDetails);
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should return 404 for non-existent agent', async () => {
      const agentId = 'non-existent';

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockAgentService.getAgent.mockResolvedValue(null);

      await controller.getAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify next was called with 404 error
      expect(mockNext).toHaveBeenCalledWith(
        expect.objectContaining({
          statusCode: 404,
          message: expect.stringContaining('not found'),
        })
      );
    });
  });

  describe('DELETE /api/v1/agents/:id', () => {
    it('should terminate agent and return 204', async () => {
      const agentId = 'agent-123';

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };
      mockAgentService.terminateAgent.mockResolvedValue(undefined);

      await controller.terminateAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify service call
      expect(mockAgentService.terminateAgent).toHaveBeenCalledWith(agentId);
      expect(mockAgentService.terminateAgent).toHaveBeenCalledTimes(1);

      // Verify response
      expect(mockResponse.status).toHaveBeenCalledWith(204);
      expect(mockResponse.send).toHaveBeenCalled();
      expect(mockNext).not.toHaveBeenCalled();
    });

    it('should return 404 when terminating non-existent agent', async () => {
      const agentId = 'non-existent';

      mockRequest.params = { id: agentId };
      mockRequest.headers = { authorization: 'Bearer valid-token' };

      const notFoundError = new Error('Agent not found');
      (notFoundError as any).statusCode = 404;
      mockAgentService.terminateAgent.mockRejectedValue(notFoundError);

      await controller.terminateAgent(
        mockRequest as Request,
        mockResponse as Response,
        mockNext
      );

      // Verify next was called with error
      expect(mockNext).toHaveBeenCalledWith(notFoundError);
    });
  });
});
