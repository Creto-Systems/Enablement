import { Request, Response, NextFunction } from 'express';
import { IAgentService, CreateAgentDTO } from '../services/agent.service.interface';

export class AgentController {
  constructor(private agentService: IAgentService) {}

  createAgent = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      // Validate request body
      const validationError = this.validateCreateAgentDTO(req.body);
      if (validationError) {
        next(validationError);
        return;
      }

      const dto: CreateAgentDTO = req.body;
      const agent = await this.agentService.createAgent(dto);

      res.status(201).json(agent);
    } catch (error) {
      next(error);
    }
  };

  getAgent = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id } = req.params;
      const agent = await this.agentService.getAgent(id);

      if (!agent) {
        const error = new Error(`Agent with id ${id} not found`);
        (error as any).statusCode = 404;
        next(error);
        return;
      }

      res.status(200).json(agent);
    } catch (error) {
      next(error);
    }
  };

  terminateAgent = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Validate authorization
      if (!req.headers.authorization) {
        const error = new Error('Authorization header is required');
        (error as any).statusCode = 401;
        throw error;
      }

      const { id } = req.params;
      await this.agentService.terminateAgent(id);

      res.status(204).send();
    } catch (error) {
      next(error);
    }
  };

  private validateCreateAgentDTO(body: any): Error | null {
    const errors: string[] = [];

    if (!body.name || body.name.trim() === '') {
      errors.push('Name is required');
    }

    if (!body.type) {
      errors.push('Invalid agent type');
    }

    if (!body.userId) {
      errors.push('User ID is required');
    }

    if (!body.config || typeof body.config !== 'object') {
      errors.push('Config is required and must be an object');
    }

    if (errors.length > 0) {
      const error = new Error(`Validation failed: ${errors.join(', ')}`);
      (error as any).statusCode = 400;
      (error as any).errors = errors.map((msg, idx) => ({
        field: ['name', 'type', 'userId', 'config'][idx],
        message: msg,
      }));
      return error;
    }

    return null;
  }
}
