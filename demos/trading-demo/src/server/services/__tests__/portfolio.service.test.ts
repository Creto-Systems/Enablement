import { PortfolioService } from '../portfolio.service';

describe('PortfolioService', () => {
  let portfolioService: PortfolioService;

  beforeEach(() => {
    portfolioService = new PortfolioService();
  });

  describe('calculateValue', () => {
    it('should sum position values', async () => {
      const agentId = 'agent-123';

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: 10,
        averagePrice: 150.00,
      });

      await portfolioService.updatePosition(agentId, {
        symbol: 'TSLA',
        quantity: 5,
        averagePrice: 200.00,
      });

      const marketPrices = {
        AAPL: 155.00,
        TSLA: 210.00,
      };

      const value = await portfolioService.calculateValue(agentId, marketPrices);

      // AAPL: 10 * 155 = 1550
      // TSLA: 5 * 210 = 1050
      // Total: 2600
      expect(value).toBe(2600);
    });

    it('should handle empty portfolio', async () => {
      const value = await portfolioService.calculateValue('agent-123', {});

      expect(value).toBe(0);
    });

    it('should calculate P&L correctly', async () => {
      const agentId = 'agent-123';

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: 10,
        averagePrice: 150.00,
      });

      const marketPrices = { AAPL: 160.00 };

      const pnl = await portfolioService.calculatePnL(agentId, marketPrices);

      // Cost: 10 * 150 = 1500
      // Value: 10 * 160 = 1600
      // P&L: 1600 - 1500 = 100
      expect(pnl).toBe(100);
    });
  });

  describe('updatePosition', () => {
    it('should add new position', async () => {
      const agentId = 'agent-123';
      const position = {
        symbol: 'AAPL',
        quantity: 10,
        averagePrice: 150.00,
      };

      await portfolioService.updatePosition(agentId, position);
      const portfolio = await portfolioService.getPortfolio(agentId);

      expect(portfolio.positions).toHaveLength(1);
      expect(portfolio.positions[0]).toMatchObject(position);
    });

    it('should update existing position', async () => {
      const agentId = 'agent-123';

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: 10,
        averagePrice: 150.00,
      });

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: 5,
        averagePrice: 160.00,
      });

      const portfolio = await portfolioService.getPortfolio(agentId);

      expect(portfolio.positions).toHaveLength(1);
      // New average: (10*150 + 5*160) / 15 = 153.33
      expect(portfolio.positions[0].quantity).toBe(15);
      expect(portfolio.positions[0].averagePrice).toBeCloseTo(153.33, 2);
    });

    it('should remove zero-quantity position', async () => {
      const agentId = 'agent-123';

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: 10,
        averagePrice: 150.00,
      });

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: -10,
        averagePrice: 160.00,
      });

      const portfolio = await portfolioService.getPortfolio(agentId);

      expect(portfolio.positions).toHaveLength(0);
    });
  });

  describe('getPortfolio', () => {
    it('should return portfolio for agent', async () => {
      const agentId = 'agent-123';

      await portfolioService.updatePosition(agentId, {
        symbol: 'AAPL',
        quantity: 10,
        averagePrice: 150.00,
      });

      const portfolio = await portfolioService.getPortfolio(agentId);

      expect(portfolio.agentId).toBe(agentId);
      expect(portfolio.positions).toHaveLength(1);
    });

    it('should return empty portfolio for new agent', async () => {
      const portfolio = await portfolioService.getPortfolio('new-agent');

      expect(portfolio.agentId).toBe('new-agent');
      expect(portfolio.positions).toHaveLength(0);
    });
  });
});
