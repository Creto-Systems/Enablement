import { Request, Response, NextFunction } from 'express';
import { IMeteringService, UsageOptions } from '../services/metering.service.interface';

export class MeteringController {
  constructor(private meteringService: IMeteringService) {}

  getUsage = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id: agentId } = req.params;

      const options: UsageOptions = {
        period: (req.query.period as any) || 'daily',
      };

      if (req.query.includeBreakdown === 'true') {
        options.includeBreakdown = true;
      }

      const usage = await this.meteringService.getUsage(agentId, options);

      res.status(200).json(usage);
    } catch (error) {
      next(error);
    }
  };

  getQuota = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id: agentId } = req.params;

      const quota = await this.meteringService.getQuota(agentId);

      res.status(200).json(quota);
    } catch (error) {
      next(error);
    }
  };
}
