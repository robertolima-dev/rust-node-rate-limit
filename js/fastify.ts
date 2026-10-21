/**
 * Fastify integration.
 *
 * Usage:
 *   import { rateLimitPlugin } from "rust-node-rate-limit/fastify";
 *   fastify.register(rateLimitPlugin, { limit: 100, windowSeconds: 60 });
 */

import type {
  FastifyInstance,
  FastifyPluginOptions,
  FastifyReply,
  FastifyRequest,
} from "fastify";
import { RateLimiter } from "./limiter";

export interface RateLimitPluginOptions extends FastifyPluginOptions {
  /** Maximum requests per window. */
  limit: number;
  /** Window length, in seconds. */
  windowSeconds: number;
  /** Derives the key from the request. Default: `request.ip`. */
  keyGenerator?: (request: FastifyRequest) => string;
  /** JSON body returned when blocked. Default: "Too many requests". */
  message?: string;
  /** HTTP status when blocked. Default: 429. */
  statusCode?: number;
  /** Reuse an existing `RateLimiter` (ignores `limit`/`windowSeconds`). */
  limiter?: RateLimiter;
}

/**
 * Fastify rate limiting plugin. Uses the `onRequest` hook to check the limit
 * before the handler, writing the `X-RateLimit-*` headers and responding
 * `429` when it is exceeded.
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
        return; // Response already sent: done() is not called.
      }

      done();
    },
  );
}

// Disables Fastify encapsulation so the `onRequest` hook applies to the scope
// where the plugin is registered (the same as wrapping it with
// `fastify-plugin`). Without it, routes registered as "siblings" of the plugin
// would not go through the rate limiter.
(rateLimitPlugin as unknown as Record<symbol, boolean>)[
  Symbol.for("skip-override")
] = true;
