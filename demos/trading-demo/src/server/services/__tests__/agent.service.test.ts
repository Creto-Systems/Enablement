import { AgentService } from '../agent.service';
import { MeteringService } from '../metering.service';

describe('AgentService', () => {
  let agentService: AgentService;
  let mockMeteringService: jest.Mocked<MeteringService>;

  beforeEach(() => {
    // Mock the metering service
    mockMeteringService = {
      checkQuota: jest.fn().mockResolvedValue({ allowed: true, remaining: 1000 }),
      recordEvent: jest.fn().mockResolvedValue(undefined),
      getUsage: jest.fn().mockResolvedValue({ used: 0, limit: 10000 }),
    } as any;

    agentService = new AgentService(mockMeteringService);
  });

  describe('createAgent', () => {
    it('should create agent with valid config', async () => {
      const config = {
        name: 'test-agent',
        strategy: 'conservative' as const,
        budget: 50000,
        riskLevel: 0.5,
      };

      const agent = await agentService.createAgent(config);

      expect(agent).toMatchObject({
        name: 'test-agent',
        strategy: 'conservative',
        budget: 50000,
        riskLevel: 0.5,
        status: 'active',
      });
      expect(agent.id).toBeDefined();
      expect(agent.createdAt).toBeInstanceOf(Date);
    });

    it('should reject invalid budget (too low)', async () => {
      const config = {
        name: 'test-agent',
        strategy: 'conservative' as const,
        budget: 500, // Below minimum of 1000
        riskLevel: 0.5,
      };

      await expect(agentService.createAgent(config)).rejects.toThrow(
        'Budget must be between 1000 and 1000000'
      );
    });

    it('should reject invalid budget (too high)', async () => {
      const config = {
        name: 'test-agent',
        strategy: 'conservative' as const,
        budget: 2000000, // Above maximum of 1000000
        riskLevel: 0.5,
      };

      await expect(agentService.createAgent(config)).rejects.toThrow(
        'Budget must be between 1000 and 1000000'
      );
    });

    it('should initialize metering quota', async () => {
      const config = {
        name: 'test-agent',
        strategy: 'aggressive' as const,
        budget: 100000,
        riskLevel: 0.8,
      };

      await agentService.createAgent(config);

      expect(mockMeteringService.recordEvent).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: 'agent_created',
          metadata: expect.objectContaining({
            budget: 100000,
          }),
        })
      );
    });
  });

  describe('getAgent', () => {
    it('should return agent by id', async () => {
      const config = {
        name: 'test-agent',
        strategy: 'balanced' as const,
        budget: 50000,
        riskLevel: 0.5,
      };

      const created = await agentService.createAgent(config);
      const retrieved = await agentService.getAgent(created.id);

      expect(retrieved).toEqual(created);
    });

    it('should return null for non-existent agent', async () => {
      const retrieved = await agentService.getAgent('non-existent-id');

      expect(retrieved).toBeNull();
    });
  });

  describe('terminateAgent', () => {
    it('should mark agent as terminated', async () => {
      const config = {
        name: 'test-agent',
        strategy: 'conservative' as const,
        budget: 50000,
        riskLevel: 0.3,
      };

      const agent = await agentService.createAgent(config);
      const terminated = await agentService.terminateAgent(agent.id);

      expect(terminated.status).toBe('terminated');
      expect(terminated.terminatedAt).toBeInstanceOf(Date);
    });

    it('should throw error for non-existent agent', async () => {
      await expect(
        agentService.terminateAgent('non-existent-id')
      ).rejects.toThrow('Agent not found');
    });
  });
});
