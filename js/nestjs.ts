/**
 * Integração com NestJS.
 *
 * Uso:
 *   import { RateLimitGuard } from "rust-node-rate-limit/nestjs";
 *   app.useGlobalGuards(new RateLimitGuard({ limit: 100, windowSeconds: 60 }));
 *
 * Importamos `HttpException`/`HttpStatus` em runtime (de `@nestjs/common`) para
 * responder corretamente com `429`. `@nestjs/common` é uma peerDependency
 * opcional — só é necessária para quem usa este subcaminho.
 */

import { HttpException, HttpStatus } from "@nestjs/common";
import type {
  CanActivate,
  ExecutionContext,
} from "@nestjs/common";
import { RateLimiter } from "../binding.js";

export interface RateLimitGuardOptions {
  /** Máximo de requisições por janela. */
  limit: number;
  /** Duração da janela, em segundos. */
  windowSeconds: number;
  /** Deriva a chave a partir da requisição HTTP. Padrão: `req.ip`. */
  keyGenerator?: (req: { ip?: string }) => string;
  /** Mensagem da exceção quando bloqueado. Padrão: "Too many requests". */
  message?: string;
  /** Reusar um `RateLimiter` existente (ignora `limit`/`windowSeconds`). */
  limiter?: RateLimiter;
}

/**
 * Guard NestJS de rate limiting. Consulta o core nativo, escreve os headers
 * `X-RateLimit-*` na resposta e lança `HttpException(429)` quando o limite é
 * excedido.
 */
export class RateLimitGuard implements CanActivate {
  private readonly limiter: RateLimiter;
  private readonly keyGenerator: (req: { ip?: string }) => string;
  private readonly message: string;

  constructor(options: RateLimitGuardOptions) {
    this.limiter =
      options.limiter ??
      new RateLimiter({
        limit: options.limit,
        windowSeconds: options.windowSeconds,
      });
    this.keyGenerator =
      options.keyGenerator ?? ((req) => req.ip ?? "global");
    this.message = options.message ?? "Too many requests";
  }

  canActivate(context: ExecutionContext): boolean {
    const http = context.switchToHttp();
    const req = http.getRequest<{ ip?: string }>();
    const res = http.getResponse<{
      setHeader?: (name: string, value: string | number) => void;
    }>();

    const result = this.limiter.check(this.keyGenerator(req));

    if (typeof res?.setHeader === "function") {
      res.setHeader("X-RateLimit-Limit", result.limit);
      res.setHeader("X-RateLimit-Remaining", Math.max(0, result.remaining));
      res.setHeader("X-RateLimit-Reset", result.resetAfter);
      if (!result.allowed) {
        res.setHeader("Retry-After", result.retryAfter);
      }
    }

    if (!result.allowed) {
      throw new HttpException(this.message, HttpStatus.TOO_MANY_REQUESTS);
    }

    return true;
  }
}
