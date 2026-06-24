/**
 * Integração com Fastify.
 *
 * Uso:
 *   import { rateLimitPlugin } from "rust-node-rate-limit/fastify";
 *   fastify.register(rateLimitPlugin, { limit: 100, windowSeconds: 60 });
 */

import type {
  FastifyInstance,
  FastifyPluginOptions,
  FastifyReply,
  FastifyRequest,
} from "fastify";
import { RateLimiter } from "../binding.js";

export interface RateLimitPluginOptions extends FastifyPluginOptions {
  /** Máximo de requisições por janela. */
  limit: number;
  /** Duração da janela, em segundos. */
  windowSeconds: number;
  /** Deriva a chave a partir da requisição. Padrão: `request.ip`. */
  keyGenerator?: (request: FastifyRequest) => string;
  /** Corpo JSON retornado quando bloqueado. Padrão: "Too many requests". */
  message?: string;
  /** Status HTTP quando bloqueado. Padrão: 429. */
  statusCode?: number;
  /** Reusar um `RateLimiter` existente (ignora `limit`/`windowSeconds`). */
  limiter?: RateLimiter;
}

/**
 * Plugin Fastify de rate limiting. Usa o hook `onRequest` para verificar o
 * limite antes do handler, escrevendo os headers `X-RateLimit-*` e respondendo
 * `429` quando excedido.
 */
export async function rateLimitPlugin(
  fastify: FastifyInstance,
  options: RateLimitPluginOptions,
): Promise<void> {
  const limiter =
    options.limiter ??
    new RateLimiter({
      limit: options.limit,
      windowSeconds: options.windowSeconds,
    });
  const keyGenerator =
    options.keyGenerator ?? ((request: FastifyRequest) => request.ip);
  const message = options.message ?? "Too many requests";
  const statusCode = options.statusCode ?? 429;

  fastify.addHook(
    "onRequest",
    (request: FastifyRequest, reply: FastifyReply, done: () => void) => {
      const result = limiter.check(keyGenerator(request));

      reply.header("X-RateLimit-Limit", result.limit);
      reply.header("X-RateLimit-Remaining", Math.max(0, result.remaining));
      reply.header("X-RateLimit-Reset", result.resetAfter);

      if (!result.allowed) {
        reply.header("Retry-After", result.retryAfter);
        reply.code(statusCode).send({ message });
        return; // Resposta enviada: não chamamos done().
      }

      done();
    },
  );
}

// Desabilita o encapsulamento do Fastify para que o hook `onRequest` se aplique
// ao escopo onde o plugin é registrado (mesmo comportamento de envolver com
// `fastify-plugin`). Sem isso, rotas registradas como "irmãs" do plugin não
// passariam pelo rate limiter.
(rateLimitPlugin as unknown as Record<symbol, boolean>)[
  Symbol.for("skip-override")
] = true;
