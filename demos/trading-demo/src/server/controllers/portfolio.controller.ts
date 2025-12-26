import { Request, Response, NextFunction } from 'express';
import { PortfolioService } from '../services/portfolio.service';

export class PortfolioController {
  constructor(private portfolioService: PortfolioService) {}

  getPortfolio = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { agentId } = req.params;
      const portfolio = await this.portfolioService.getPortfolio(agentId);

      if (!portfolio) {
        const error = new Error(`Portfolio for agent ${agentId} not found`);
        (error as any).statusCode = 404;
        next(error);
        return;
      }

      res.status(200).json(portfolio);
    } catch (error) {
      next(error);
    }
  };

  getHistory = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { agentId } = req.params;
      const days = req.query.days ? parseInt(req.query.days as string) : 30;

      // Validate days parameter
      if (isNaN(days) || days < 1 || days > 365) {
        const error = new Error('Invalid days parameter. Must be between 1 and 365');
        (error as any).statusCode = 400;
        next(error);
        return;
      }

      const history = await this.portfolioService.getPortfolioHistory(agentId, days);

      res.status(200).json({
        agentId,
        days,
        history,
      });
    } catch (error) {
      next(error);
    }
  };

  listPositions = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { agentId } = req.params;
      const portfolio = await this.portfolioService.getPortfolio(agentId);

      if (!portfolio) {
        const error = new Error(`Portfolio for agent ${agentId} not found`);
        (error as any).statusCode = 404;
        next(error);
        return;
      }

      res.status(200).json({
        agentId,
        positions: portfolio.positions,
        totalPositions: portfolio.positions.length,
      });
    } catch (error) {
      next(error);
    }
  };
}
