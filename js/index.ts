/**
 * rust-node-rate-limit — public API (TypeScript).
 *
 * This layer exports the `RateLimiter` class (a TypeScript wrapper over the
 * native class generated from Rust by napi-rs, see `./limiter`) and its types.
 * The framework adapters (Express, Fastify, NestJS) are imported from the
 * package subpaths.
 *
 * "Local rate limiting for Node.js with a Rust core."
 */

export type {
  CheckResult,
  RateLimitStats,
  RateLimiterOptions,
} from "../binding.js";

export { RateLimiter } from "./limiter";
