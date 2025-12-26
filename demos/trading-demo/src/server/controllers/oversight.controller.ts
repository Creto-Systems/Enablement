import { Request, Response, NextFunction } from 'express';
import { IOversightService, OversightFilters } from '../services/oversight.service.interface';

export class OversightController {
  constructor(private oversightService: IOversightService) {}

  getPendingRequests = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const filters: OversightFilters = {};

      if (req.query.agentId) {
        filters.agentId = req.query.agentId as string;
      }

      const requests = await this.oversightService.getPendingRequests(filters);

      res.status(200).json({
        requests,
        total: requests.length,
      });
    } catch (error) {
      next(error);
    }
  };

  approveRequest = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id } = req.params;
      const userId = (req as any).user?.id || 'user-123'; // Get from auth middleware

      const approvedRequest = await this.oversightService.approveRequest(id, userId);

      res.status(200).json(approvedRequest);
    } catch (error) {
      next(error);
    }
  };

  rejectRequest = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id } = req.params;
      const { reason } = req.body;

      // Validate rejection reason
      if (!reason || reason.trim() === '') {
        const error = new Error('Rejection reason is required');
        (error as any).statusCode = 400;
        next(error);
        return;
      }

      const userId = (req as any).user?.id || 'user-123'; // Get from auth middleware

      const rejectedRequest = await this.oversightService.rejectRequest(id, userId, reason);

      res.status(200).json(rejectedRequest);
    } catch (error) {
      next(error);
    }
  };
}
