import { Request, Response, NextFunction } from 'express';
import { ITradeService, SubmitTradeDTO, GetTradesOptions } from '../services/trade.service.interface';

export class TradeController {
  constructor(private tradeService: ITradeService) {}

  submitTrade = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id: agentId } = req.params;

      // Validate trade parameters
      const validationError = this.validateTradeDTO(req.body);
      if (validationError) {
        next(validationError);
        return;
      }

      const dto: SubmitTradeDTO = req.body;
      const result = await this.tradeService.submitTrade(agentId, dto);

      if (result.requiresOversight) {
        res.status(202).json({
          trade: result.trade,
          message: 'Trade submitted for approval required',
          reason: result.reason,
        });
      } else {
        res.status(201).json(result.trade);
      }
    } catch (error) {
      next(error);
    }
  };

  getTrades = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id: agentId } = req.params;

      // Parse query parameters
      const options: GetTradesOptions = {
        page: req.query.page ? parseInt(req.query.page as string, 10) : 1,
        limit: req.query.limit ? parseInt(req.query.limit as string, 10) : 20,
      };

      if (req.query.status) {
        options.status = req.query.status as any;
      }

      const result = await this.tradeService.getTrades(agentId, options);

      res.status(200).json(result);
    } catch (error) {
      next(error);
    }
  };

  private validateTradeDTO(body: any): Error | null {
    const errors: string[] = [];

    if (!body.symbol || body.symbol.trim() === '') {
      errors.push('Symbol is required');
    }

    if (!body.side || !['buy', 'sell'].includes(body.side)) {
      errors.push('Invalid trade side (must be buy or sell)');
    }

    if (!body.quantity || body.quantity <= 0) {
      errors.push('Quantity must be greater than 0');
    }

    if (!body.type || !['market', 'limit', 'stop'].includes(body.type)) {
      errors.push('Invalid trade type');
    }

    if (errors.length > 0) {
      const error = new Error(`Trade validation failed: ${errors.join(', ')}`);
      (error as any).statusCode = 400;
      (error as any).errors = errors;
      return error;
    }

    return null;
  }
}
