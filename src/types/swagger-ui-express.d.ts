/**
 * Type definitions for swagger-ui-express
 * Temporary type definitions until @types/swagger-ui-express is properly installed
 */

declare module 'swagger-ui-express' {
  import { RequestHandler } from 'express';

  export function setup(
    swaggerDoc: Record<string, unknown>,
    opts?: Record<string, unknown>
  ): RequestHandler;

  export function serve(
    req: unknown,
    res: unknown,
    next: unknown
  ): void;

  export function serveFiles(
    swaggerDoc: Record<string, unknown>,
    opts?: Record<string, unknown>
  ): RequestHandler[];
}
