# rust-node-rate-limit

**Ultra-fast rate limiting for Node.js powered by Rust.**

`rust-node-rate-limit` is a local, in-process rate limiter for Node.js with a
core written in Rust. It uses a **Fixed Window** algorithm over a lock-free
[`DashMap`](https://docs.rs/dashmap), giving you predictable limits with very
low overhead.

It ships as a prebuilt native addon (via [napi-rs](https://napi.rs/)), so there
is **no compiler required at install time** on supported platforms.

[![CI](https://github.com/robertolima-dev/rust-node-rate-limit/actions/workflows/CI.yml/badge.svg)](https://github.com/robertolima-dev/rust-node-rate-limit/actions/workflows/CI.yml)
[![npm](https://img.shields.io/npm/v/rust-node-rate-limit.svg)](https://www.npmjs.com/package/rust-node-rate-limit)
![node](https://img.shields.io/badge/node-%3E%3D18-43853d)
![license](https://img.shields.io/badge/license-MIT-blue)

🌐 **Website:** [rust-node-rate-limit.vercel.app](https://rust-node-rate-limit.vercel.app/)

---

## Features

- 🦀 **Rust-powered** — the hot path is native code.
- ⚡ **Ultra-fast** — minimal allocation, near-zero overhead.
- 🪟 **Fixed Window** algorithm — simple and predictable.
- 🔌 **Express** middleware, **Fastify** plugin and **NestJS** guard.
- 🧩 **TypeScript** support — generated `.d.ts`, dual CJS + ESM.
- 🔒 **Thread-safe** — lock-free reads via `DashMap`.

---

## Installation

```bash
npm install rust-node-rate-limit
```

Works with both **ESM** and **CommonJS**, and requires **Node.js 18+**.

---

## Basic Usage

```ts
import { RateLimiter } from "rust-node-rate-limit";

const limiter = new RateLimiter({
  limit: 100,
  windowSeconds: 60,
});

limiter.allow("user:123"); // true / false
```

```js
// CommonJS
const { RateLimiter } = require("rust-node-rate-limit");

const limiter = new RateLimiter({ limit: 5, windowSeconds: 60 });
limiter.allow("ip:127.0.0.1");
```

Detailed result with `check()`:

```ts
limiter.check("user:123");
// {
//   allowed: true,
//   limit: 100,
//   remaining: 99,
//   retryAfter: 0,
//   resetAfter: 60
// }
```

---

## Express Integration

```ts
import express from "express";
import { rateLimitMiddleware } from "rust-node-rate-limit/express";

const app = express();

app.use(
  rateLimitMiddleware({
    limit: 100,
    windowSeconds: 60,
  }),
);
```

Blocked requests get `429` with:

```json
{ "message": "Too many requests" }
```

Response headers on every request:

```txt
X-RateLimit-Limit
X-RateLimit-Remaining
X-RateLimit-Reset
Retry-After        (only when blocked)
```

You can customize the key and the response:

```ts
app.use(
  rateLimitMiddleware({
    limit: 10,
    windowSeconds: 60,
    keyGenerator: (req) => req.headers["x-api-key"]?.toString() ?? req.ip,
    message: "Slow down!",
    statusCode: 429,
  }),
);
```

---

## Fastify Integration

```ts
import Fastify from "fastify";
import { rateLimitPlugin } from "rust-node-rate-limit/fastify";

const fastify = Fastify();

fastify.register(rateLimitPlugin, {
  limit: 100,
  windowSeconds: 60,
});
```

---

## NestJS Integration

```ts
import { RateLimitGuard } from "rust-node-rate-limit/nestjs";

app.useGlobalGuards(
  new RateLimitGuard({ limit: 100, windowSeconds: 60 }),
);
```

The guard throws `HttpException` with status `429` when the limit is exceeded.

---

## API Reference

### `new RateLimiter(options)`

| Option          | Type     | Description                          |
| --------------- | -------- | ------------------------------------ |
| `limit`         | `number` | Max requests allowed per window.     |
| `windowSeconds` | `number` | Window duration, in seconds.         |

Throws if `limit <= 0` or `windowSeconds <= 0`.

### `allow(key): boolean`

Consumes one slot for `key`. Returns whether the request is allowed.

### `check(key): CheckResult`

Like `allow()`, but returns the detailed result:

```ts
interface CheckResult {
  allowed: boolean;
  limit: number;
  remaining: number;
  retryAfter: number; // seconds until you can retry (0 when allowed)
  resetAfter: number; // seconds until the window resets
}
```

### `remaining(key): number`

Read-only peek at how many requests remain in the current window. Does **not**
consume a slot.

### `reset(key): void`

Clears the state of a single key.

### `clear(): void`

Clears the state of every key. Cumulative `stats()` counters are kept.

### `cleanup(): number`

Removes keys whose window has expired. Returns how many were removed. Useful
when you track many ephemeral keys (e.g. per-IP).

### `stats(): RateLimitStats`

```ts
interface RateLimitStats {
  allowed: number; // total allowed since creation
  blocked: number; // total blocked since creation
  checks: number; // allowed + blocked
  activeKeys: number; // keys currently tracked
}
```

---

## How It Works

**Fixed Window Rate Limiting.** Each key gets a window of `windowSeconds`.
Within the window up to `limit` requests are allowed; once the window expires it
"rolls over" and the count restarts.

```txt
limit = 3, windowSeconds = 60

t=0s   |#--|  allow  -> remaining 2
t=10s  |##-|  allow  -> remaining 1
t=20s  |###|  allow  -> remaining 0
t=30s  |###|  block  -> retryAfter 30
---- window resets at t=60s ----
t=61s  |#--|  allow  -> remaining 2
```

Each key is stored in a concurrent `DashMap`. The read-modify-write on a key is
atomic per shard, so the limiter is correct under heavy concurrency without a
global lock.

---

## Performance

- **Rust core** on the hot path.
- **`DashMap`** for sharded, concurrent key storage.
- **Lock-free reads** for `remaining()`.
- **Low allocation** — counters are plain integers, no per-request garbage.

---

## Limitations

- **Local process only** — state lives in memory.
- **Not distributed** — use the upcoming Redis backend for that.
- **Multiple Node workers have separate counters** (one limiter per process).

---

## Roadmap

| Version | Feature                              |
| ------- | ------------------------------------ |
| v0.1    | Fixed Window                         |
| v0.2    | Sliding Window                       |
| v0.3    | Token Bucket                         |
| v0.4    | Redis backend (distributed limits)   |
| v0.5    | Prometheus metrics                   |
| v0.6    | ImmutableLog integration             |

---

## Development

```bash
npm install
npm run build   # builds the native addon + the JS layer
npm test
```

Try the examples (after `npm run build`):

```bash
node examples/basic-example.mjs
node examples/express-example.mjs
node examples/fastify-example.mjs
```

---

## Publishing

```bash
npm version patch
git push --tags
```

The CI workflow builds binaries for all platforms and publishes a single npm
package; the loader picks the matching `.node` at runtime.

---

## License

MIT © [Roberto Lima](https://github.com/robertolima-dev)
