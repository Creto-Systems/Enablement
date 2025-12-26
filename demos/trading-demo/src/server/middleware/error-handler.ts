/**
 * Centralized Error Handling Middleware
 *
 * Converts domain errors to HTTP responses following RFC 7807 Problem Details format.
 * Maps error types to appropriate HTTP status codes.
 */

import { Request, Response, NextFunction } from 'express';

/**
 * RFC 7807 Problem Details
 */
export interface ProblemDetails {
  /** Problem type URI */
  type: string;

  /** Short, human-readable summary */
  title: string;

  /** HTTP status code */
  status: number;

  /** Detailed explanation */
  detail: string;

  /** Instance URI identifying the specific occurrence */
  instance: string;

  /** Additional error-specific fields */
  [key: string]: any;
}

/**
 * Domain Error Types
 */
export enum ErrorType {
  VALIDATION_ERROR = 'validation-error',
  NOT_FOUND = 'not-found',
  UNAUTHORIZED = 'unauthorized',
  FORBIDDEN = 'forbidden',
  CONFLICT = 'conflict',
  QUOTA_EXCEEDED = 'quota-exceeded',
  INSUFFICIENT_FUNDS = 'insufficient-funds',
  APPROVAL_REQUIRED = 'approval-required',
  GRPC_ERROR = 'grpc-error',
  DATABASE_ERROR = 'database-error',
  INTERNAL_ERROR = 'internal-error',
}

/**
 * Error Status Code Mapping
 */
const errorStatusMap: Record<ErrorType, number> = {
  [ErrorType.VALIDATION_ERROR]: 400,
  [ErrorType.NOT_FOUND]: 404,
  [ErrorType.UNAUTHORIZED]: 401,
  [ErrorType.FORBIDDEN]: 403,
  [ErrorType.CONFLICT]: 409,
  [ErrorType.QUOTA_EXCEEDED]: 429,
  [ErrorType.INSUFFICIENT_FUNDS]: 402,
  [ErrorType.APPROVAL_REQUIRED]: 403,
  [ErrorType.GRPC_ERROR]: 503,
  [ErrorType.DATABASE_ERROR]: 500,
  [ErrorType.INTERNAL_ERROR]: 500,
};

/**
 * Domain Error Class
 */
export class DomainError extends Error {
  public readonly errorType: ErrorType;
  public readonly statusCode: number;
  public readonly details?: Record<string, any>;

  constructor(
    errorType: ErrorType,
    message: string,
    details?: Record<string, any>
  ) {
    super(message);
    this.name = 'DomainError';
    this.errorType = errorType;
    this.statusCode = errorStatusMap[errorType];
    this.details = details;

    Error.captureStackTrace(this, this.constructor);
  }
}

/**
 * Validation Error
 */
export class ValidationError extends DomainError {
  constructor(message: string, errors?: Array<{ field: string; message: string }>) {
    super(ErrorType.VALIDATION_ERROR, message, { errors });
  }
}

/**
 * Not Found Error
 */
export class NotFoundError extends DomainError {
  constructor(resource: string, id: string) {
    super(ErrorType.NOT_FOUND, `${resource} with id ${id} not found`);
  }
}

/**
 * Unauthorized Error
 */
export class UnauthorizedError extends DomainError {
  constructor(message: string = 'Unauthorized') {
    super(ErrorType.UNAUTHORIZED, message);
  }
}

/**
 * Quota Exceeded Error
 */
export class QuotaExceededError extends DomainError {
  constructor(resource: string, limit: number) {
    super(
      ErrorType.QUOTA_EXCEEDED,
      `Quota exceeded for ${resource}`,
      { resource, limit }
    );
  }
}

/**
 * Insufficient Funds Error
 */
export class InsufficientFundsError extends DomainError {
  constructor(required: number, available: number) {
    super(
      ErrorType.INSUFFICIENT_FUNDS,
      'Insufficient funds for this operation',
      { required, available }
    );
  }
}

/**
 * Convert Error to Problem Details
 */
function errorToProblemDetails(
  error: Error,
  requestPath: string
): ProblemDetails {
  if (error instanceof DomainError) {
    return {
      type: `https://api.trading-demo.com/problems/${error.errorType}`,
      title: error.errorType.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
      status: error.statusCode,
      detail: error.message,
      instance: requestPath,
      ...error.details,
    };
  }

  // Handle legacy error format from controllers
  if ((error as any).statusCode) {
    const statusCode = (error as any).statusCode;
    const errors = (error as any).errors;

    return {
      type: `https://api.trading-demo.com/problems/error`,
      title: 'Error',
      status: statusCode,
      detail: error.message,
      instance: requestPath,
      errors,
    };
  }

  // Unknown error - internal server error
  return {
    type: 'https://api.trading-demo.com/problems/internal-error',
    title: 'Internal Server Error',
    status: 500,
    detail: process.env.NODE_ENV === 'production'
      ? 'An unexpected error occurred'
      : error.message,
    instance: requestPath,
  };
}

/**
 * Error Handling Middleware
 */
export function errorHandler(
  error: Error,
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Log error
  console.error('Error occurred:', {
    path: req.path,
    method: req.method,
    error: error.message,
    stack: process.env.NODE_ENV !== 'production' ? error.stack : undefined,
  });

  // Convert to problem details
  const problem = errorToProblemDetails(error, req.path);

  // Send response
  res.status(problem.status).json(problem);
}

/**
 * Not Found Handler
 */
export function notFoundHandler(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const problem: ProblemDetails = {
    type: 'https://api.trading-demo.com/problems/not-found',
    title: 'Not Found',
    status: 404,
    detail: `Route ${req.method} ${req.path} not found`,
    instance: req.path,
  };

  res.status(404).json(problem);
}

/**
 * Async Handler Wrapper
 *
 * Wraps async route handlers to catch errors and pass to error middleware.
 */
export function asyncHandler<T = any>(
  fn: (req: Request, res: Response, next: NextFunction) => Promise<T>
) {
  return (req: Request, res: Response, next: NextFunction): void => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
}
