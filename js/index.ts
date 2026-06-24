/**
 * rust-node-rate-limit — API pública (TypeScript).
 *
 * Esta camada re-exporta a classe nativa `RateLimiter` (gerada pelo Rust via
 * napi-rs) e seus tipos. Os adaptadores de framework (Express, Fastify, NestJS)
 * são importados pelos subcaminhos do pacote.
 *
 * "Ultra-fast rate limiting for Node.js powered by Rust."
 */

// Namespace import: funciona de forma idêntica em CommonJS e ESM ao consumir o
// addon nativo (que é um módulo CJS gerado pelo napi-rs).
import * as native from "../binding.js";

export type {
  CheckResult,
  RateLimitStats,
  RateLimiterOptions,
} from "../binding.js";

/** Sanity-check: retorna "Hello from Rust" vindo do core nativo. */
export const hello: () => string = native.hello;

/**
 * Rate limiter local (in-process) com algoritmo Fixed Window e core em Rust.
 *
 * ```ts
 * import { RateLimiter } from "rust-node-rate-limit";
 *
 * const limiter = new RateLimiter({ limit: 100, windowSeconds: 60 });
 * limiter.allow("user:123");      // true / false
 * limiter.check("user:123");      // { allowed, limit, remaining, retryAfter, resetAfter }
 * ```
 */
export const RateLimiter = native.RateLimiter;

/** Tipo da instância de {@link RateLimiter}. */
export type RateLimiter = InstanceType<typeof native.RateLimiter>;
