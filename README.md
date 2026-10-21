# rust-node-rate-limit

**Local rate limiting for Node.js with a Rust core.**

`rust-node-rate-limit` is a local, in-process rate limiter for Node.js. It offers
two algorithms — **Fixed Window** (default) and **Sliding Window** — with
millisecond windows, correct `X-RateLimit-*` / `Retry-After` headers, and
ready-made adapters for Express, Fastify and NestJS. A check costs about
0.2 µs (see [Performance](#performance)), far below anything an HTTP request
does.

It ships as a prebuilt native addon (via [napi-rs](https://napi.rs/)), so there
is **no compiler required at install time** on supported platforms.

[![CI](https://github.com/robertolima-dev/rust-node-rate-limit/actions/workflows/CI.yml/badge.svg)](https://github.com/robertolima-dev/rust-node-rate-limit/actions/workflows/CI.yml)
[![npm](https://img.shields.io/npm/v/rust-node-rate-limit.svg)](https://www.npmjs.com/package/rust-node-rate-limit)
![node](https://img.shields.io/badge/node-%3E%3D18-43853d)
![license](https://img.shields.io/badge/license-MIT-blue)

🌐 **Website:** [rust-node-rate-limit.vercel.app](https://rust-node-rate-limit.vercel.app/)

---

## Features

- 🪟 **Fixed Window** and **Sliding Window** algorithms — pick per limiter.
- ⏱️ **Sub-second windows** — `windowSeconds: 0.5` is 500 ms; results in seconds and ms.
- 📮 **Correct headers** — `X-RateLimit-Limit/Remaining/Reset` and a `Retry-After` that is never 0.
- 🔌 **Express** middleware, **Fastify** plugin and **NestJS** guard (Express and Fastify platforms).
- 🛡️ **Strict options** — invalid `limit` / `windowSeconds` throw instead of being truncated.
- 🧩 **TypeScript** support — generated `.d.ts`, dual CJS + ESM.

---

## Installation

```bash
npm install rust-node-rate-limit
```

Works with both **ESM** and **CommonJS**, and requires **Node.js 18+**.

---

## Supported platforms

Prebuilt binaries are published for:

| OS      | Architecture          | Triple                       |
| ------- | --------------------- | ---------------------------- |
| macOS   | arm64                 | `aarch64-apple-darwin`       |
| macOS   | x64                   | `x86_64-apple-darwin`        |
| Linux   | x64 (glibc)           | `x86_64-unknown-linux-gnu`   |
| Linux   | arm64 (glibc)         | `aarch64-unknown-linux-gnu`  |
| Linux   | x64 (musl, Alpine)    | `x86_64-unknown-linux-musl`  |
| Linux   | arm64 (musl, Alpine)  | `aarch64-unknown-linux-musl` |
| Windows | x64                   | `x86_64-pc-windows-msvc`     |

Every release is installed from the packed tarball and exercised on each of these platforms
in CI before it is published.

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

Opt into the **Sliding Window** algorithm to smooth bursts at the window
boundary (default is `"fixed"`):

```ts
const limiter = new RateLimiter({
  limit: 100,
  windowSeconds: 60,
  algorithm: "sliding",
});

limiter.algorithm; // "sliding"
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

The guard throws `HttpException` with status `429` when the limit is exceeded,
and writes the `X-RateLimit-*` / `Retry-After` headers on both
`@nestjs/platform-express` and `@nestjs/platform-fastify`.

---

## API Reference

### `new RateLimiter(options)`

| Option          | Type     | Description                                              |
| --------------- | -------- | ------------------------------------------------------- |
| `limit`         | `number` | Max requests allowed per window. A positive integer.     |
| `windowSeconds` | `number` | Window duration, in seconds. Fractions are allowed: `0.5` is a 500 ms window (minimum 1 ms). |
| `algorithm`     | `string` | `"fixed"` (default) or `"sliding"`. Optional.           |

Throws if `limit` is not a positive integer (`1.5` is rejected, not truncated), if
`windowSeconds` is not a positive number or is below `0.001`, if either is `NaN` or
`Infinity`, or if `algorithm` is not one of `"fixed"` / `"sliding"`.

```ts
// 10 requests per 500 ms
const burst = new RateLimiter({ limit: 10, windowSeconds: 0.5 });
```

### `algorithm: string` (getter)

The active algorithm — `"fixed"` or `"sliding"`.

### `allow(key): boolean`

Consumes one slot for `key`. Returns whether the request is allowed.

### `check(key): CheckResult`

Like `allow()`, but returns the detailed result:

```ts
interface CheckResult {
  allowed: boolean;
  limit: number;
  remaining: number;
  retryAfter: number; // whole seconds until you can retry, rounded up (0 when allowed)
  resetAfter: number; // whole seconds until the window resets, rounded up
  retryAfterMs: number; // the same in milliseconds (0 when allowed)
  resetAfterMs: number; // the same in milliseconds
}
```

`retryAfter` / `resetAfter` are what the framework adapters send in the `Retry-After`
and `X-RateLimit-Reset` headers. They round up, so a blocked client is never told to
retry in 0 seconds, even with sub-second windows.

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

**Sliding Window Rate Limiting** (`algorithm: "sliding"`). Fixed Window has a
known weakness: a client can send `limit` requests at the end of one window and
`limit` more at the start of the next, pushing `2 × limit` through in less than
a window. The sliding window counter keeps the current and previous aligned
windows and weights the previous one by how much of it still overlaps:

```txt
estimated = previousCount × weight + currentCount
weight    = (windowSeconds − elapsedInCurrentWindow) / windowSeconds
```

It is O(1) in time and memory per key (no per-request timestamp log) and removes
the boundary doubling, at the cost of being a smooth approximation rather than
an exact count.

Each key is stored in a concurrent `DashMap` (a hash map split into shards,
each with its own lock). The read-modify-write on a key happens under its
shard's lock, so counts stay exact.

---

## Performance

Measured on an Apple M2 with Node.js 24, 1,000 keys, 2 million calls:

| Call                                    | Time per call |
| --------------------------------------- | ------------- |
| `allow(key)`                            | ~0.13 µs      |
| `check(key)` (what the adapters use)    | ~0.18 µs      |
| The same fixed window in plain JS (`Map`) | ~0.05 µs    |

A rate limit check is tiny work — a hash lookup and an increment — and V8
compiles that to fast machine code too. Each call into Rust also pays a fixed
cost to cross the N-API boundary (copying the key into Rust, the call itself,
converting the result back), which is larger than the check. So this library is
**not faster than a plain JavaScript limiter**; it is fast enough that the limit
check is negligible next to an HTTP request (typically 100 µs or more).

What it gives you is the rest: two tested algorithms, sub-second windows,
correct headers, strict validation and the framework adapters.

`check()` builds its result object in JavaScript from numbers the native core
writes into a reused buffer; returning a 7-field object through N-API directly
cost ~1.1 µs per call (0.3.0 and earlier).

---

## Limitations

- **Local process only** — state lives in memory.
- **Not distributed** — use the upcoming Redis backend for that.
- **Multiple Node workers have separate counters** (one limiter per process).

---

## Roadmap

| Version | Feature                              | Status |
| ------- | ------------------------------------ | ------ |
| v0.1    | Fixed Window                         | ✅     |
| v0.2    | Sliding Window                       | ✅     |
| v0.3    | Nest guard headers on Fastify; `hello()` removed | ✅ |
| v0.4    | Token Bucket                         | 🔜     |
| v0.5    | Redis backend (distributed limits)   | 🔜     |
| v0.6    | Prometheus metrics                   | 🔜     |
| later   | ImmutableLog integration             | 🔜     |

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
