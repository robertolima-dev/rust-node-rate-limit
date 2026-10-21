# Project: rust-node-rate-limit

## Shipped — v0.2.2

- ✅ **No silent truncation** — `limit` and `windowSeconds` arrived as integers through
  napi, so `limit: 1.5` became 1, `windowSeconds: 1.5` became 1 s, and `windowSeconds: 0.5`
  failed with "must be greater than 0". Options are now validated as numbers: `limit` must be
  a positive integer, `NaN`/`Infinity` are rejected, with messages that show the value.
- ✅ **Sub-second windows** — time is tracked in milliseconds; `windowSeconds: 0.5` is a
  500 ms window (minimum 1 ms).
- ✅ **`retryAfterMs` / `resetAfterMs`** on `check()`. `retryAfter` / `resetAfter` stay in
  seconds (for the HTTP headers) and now round up, so a blocked client never sees 0.

## Shipped — v0.2.1

- ✅ **Alpine / musl binaries** — `linux-x64-musl` and `linux-arm64-musl` are bundled; before
  this the package failed to load on `node:alpine` (2026-10-21).
- ✅ **Release CI installs what it ships** — the tarball is packed once, installed and exercised
  (CJS + ESM) on Linux glibc x64/arm64, Alpine x64/arm64, macOS arm64/x64 and Windows x64, and
  that exact tarball is published. The publish job checks the tag matches `package.json`.
- ✅ **English API docs** — the JSDoc shipped in the `.d.ts` files (from Rust `///` and TS
  comments) is now in English, matching the README.
- ✅ Rust lint job (fmt, clippy, cargo test) added to CI, as in the other libs.

---

## Goal

Build a production-ready Node.js rate limiting library powered by Rust.

The package name will be:

```txt
rust-node-rate-limit
```

The library must be written using:

* Rust
* napi-rs
* TypeScript
* Node.js
* GitHub Actions
* npm

The project must follow the exact architecture, workflow, build strategy, release strategy, CI/CD structure and npm publishing approach already used in the reference project:

```txt
rust_node_monitor
```

Use that repository as the implementation reference for:

* folder structure
* package.json organization
* Cargo.toml organization
* GitHub Actions
* npm release workflow
* napi-rs setup
* TypeScript definitions
* binary packaging
* multi-platform builds
* README layout

---

# Product Vision

Provide ultra-fast local rate limiting for:

* Express
* Fastify
* NestJS
* Hono
* plain Node.js applications

with Rust performance and minimal overhead.

Tagline:

```txt
Ultra-fast rate limiting for Node.js powered by Rust.
```

---

# MVP Features

## Core Fixed Window Rate Limiter

Support:

```ts
import { RateLimiter } from "rust-node-rate-limit";

const limiter = new RateLimiter({
  limit: 100,
  windowSeconds: 60,
});

limiter.allow("user:123");
```

---

## allow()

```ts
limiter.allow(key);
```

Returns:

```ts
true
```

or

```ts
false
```

---

## check()

```ts
limiter.check(key);
```

Returns:

```ts
{
  allowed: true,
  limit: 100,
  remaining: 99,
  retryAfter: 0,
  resetAfter: 60
}
```

---

## remaining()

```ts
limiter.remaining(key);
```

---

## reset()

```ts
limiter.reset(key);
```

---

## clear()

```ts
limiter.clear();
```

---

## stats()

```ts
limiter.stats();
```

Returns:

```ts
{
  allowed: 1500,
  blocked: 42,
  checks: 1542,
  activeKeys: 73
}
```

---

# Express Middleware

Example:

```ts
import express from "express";
import { rateLimitMiddleware } from "rust-node-rate-limit/express";

const app = express();

app.use(
  rateLimitMiddleware({
    limit: 100,
    windowSeconds: 60,
  })
);
```

Blocked response:

```json
{
  "message": "Too many requests"
}
```

Status:

```txt
429
```

Headers:

```txt
X-RateLimit-Limit
X-RateLimit-Remaining
X-RateLimit-Reset
Retry-After
```

---

# Fastify Plugin

Example:

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

# NestJS Support

Provide:

```ts
import { RateLimitGuard } from "rust-node-rate-limit/nestjs";
```

---

# Rust Architecture

Folder structure:

```txt
rust-node-rate-limit/
├── Cargo.toml
├── package.json
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs
│   ├── rate_limiter.rs
│   ├── fixed_window.rs
│   ├── entry.rs
│   ├── stats.rs
│   ├── errors.rs
│   └── cleanup.rs
├── js/
│   ├── express.ts
│   ├── fastify.ts
│   ├── nestjs.ts
│   └── index.ts
├── test/
│   ├── rate_limit.test.ts
│   ├── middleware.test.ts
│   └── stats.test.ts
├── npm/
├── examples/
└── .github/
```

---

# Internal Rust Structures

```rust
pub struct RateLimitEntry {
    pub count: u64,
    pub window_start: u64,
}
```

```rust
pub struct RateLimiter {
    limit: u64,
    window_seconds: u64,
    entries: DashMap<String, RateLimitEntry>,
}
```

---

# Rust Dependencies

```toml
napi
napi-derive
dashmap
serde
serde_json
thiserror
```

---

# npm API

```ts
import { RateLimiter } from "rust-node-rate-limit";

const limiter = new RateLimiter({
  limit: 5,
  windowSeconds: 60,
});

limiter.allow("ip:127.0.0.1");
```

---

# Testing Requirements

Create tests for:

1. Allow requests under limit
2. Block after limit
3. Reset after window expiration
4. Remaining requests
5. Multiple independent keys
6. Reset key
7. Clear limiter
8. Stats counters
9. Express middleware
10. Fastify plugin
11. High concurrency simulation
12. Invalid configuration

Use:

```txt
vitest
```

---

# CI/CD

Must follow the exact workflow used by rust-node-monitor.

Include:

## Pull Request

* cargo fmt
* cargo clippy
* cargo test
* npm test

## Main Branch

Build:

```txt
Linux x64
Linux ARM64
macOS Intel
macOS ARM64
Windows x64
```

Generate binaries via napi-rs.

---

# npm Publishing

Follow rust-node-monitor publishing strategy.

Support:

```bash
npm publish
```

Generated packages:

```txt
linux-x64
linux-arm64
darwin-x64
darwin-arm64
win32-x64-msvc
```

---

# README.md (FULLY IN ENGLISH)

Create a professional README.

Sections:

# rust-node-rate-limit

Short description.

---

## Features

* Rust-powered
* Ultra-fast
* Fixed Window algorithm
* Express middleware
* Fastify plugin
* NestJS support
* TypeScript support
* Thread-safe

---

## Installation

```bash
npm install rust-node-rate-limit
```

---

## Basic Usage

Full examples.

---

## Express Integration

Full examples.

---

## Fastify Integration

Full examples.

---

## NestJS Integration

Full examples.

---

## API Reference

Document:

* constructor
* allow()
* check()
* remaining()
* reset()
* clear()
* stats()

---

## How It Works

Explain:

```txt
Fixed Window Rate Limiting
```

with diagrams.

---

## Performance

Explain:

```txt
Rust core
DashMap
Lock-free reads
Low allocation
```

---

## Limitations

* Local process only
* Not distributed
* Multiple Node workers have separate counters

---

## Roadmap

### v0.1

Fixed Window

### v0.2

Sliding Window

### v0.3

Token Bucket

### v0.4

Redis Backend

### v0.5

Prometheus Metrics

### v0.6

ImmutableLog Integration

---

## Development

Commands:

```bash
npm install
npm run build
npm test
```

---

## Publishing

Commands:

```bash
npm version patch
git push --tags
```

---

## License

MIT

---

# Future ImmutableLog Integration

Create architecture notes for future support:

```ts
limiter.onBlocked((event) => {
  immutablelog.send(event);
});
```

Event:

```json
{
  "event_type": "rate_limit_blocked",
  "key": "user:123",
  "limit": 100,
  "window_seconds": 60
}
```

---

# Development Rules

Act as senior Rust + Node.js engineer.

Advance step-by-step.

Always:

* explain Rust code
* explain napi-rs
* explain ownership
* explain concurrency
* explain DashMap
* create tests before advancing
* keep API simple
* follow rust-node-monitor conventions exactly
* keep README production quality

Start with:

**Step 1**
Analyze rust-node-monitor structure and reproduce the same foundation for rust-node-rate-limit before implementing any rate limiting logic.

---

# Website (Landing Page)

## Goal

Build the landing page / marketing site for `rust-node-rate-limit` in the
directory:

```txt
/Users/robertolima/Documents/projects/rust/study/rust_node_rate_limit_website
```

## Reference

Use the existing site as the structural and visual reference:

```txt
/Users/robertolima/Documents/projects/rust/study/rust_node_monitor_website
```

> Note: the originally cited path `rust_node_rate_monitor` does not exist.
> The actual reference is `rust_node_monitor_website`.

It is a **static site** — plain HTML / CSS / JS, **no build step** — that
deploys on Vercel with zero configuration. Reproduce the exact same approach,
file layout, dark theme (Rust/Node accents), and interaction patterns, adapting
all copy and code examples to the rate limiting product.

## File layout (mirror the reference)

```txt
rust_node_rate_limit_website/
├── index.html      # the page
├── styles.css      # dark theme, Rust/Node accents
├── script.js       # copy button + usage tabs
├── vercel.json     # cleanUrls + security headers
└── README.md       # local preview + Vercel deploy instructions
```

## Tasks

### 1. Scaffold

* [ ] Copy `vercel.json` verbatim (cleanUrls, trailingSlash:false, the
  `X-Content-Type-Options` and `Referrer-Policy` headers).
* [ ] Adapt `README.md` (static site, `npx serve .` preview, Vercel deploy via
  CLI and Git import — framework preset **Other**, no build command, output `.`).
* [ ] Reuse `script.js` (copy-to-clipboard button + tabbed code panels).
* [ ] Reuse `styles.css` as the base; keep the dark theme and 🦀 accents.

### 2. Page content (`index.html`)

Mirror the reference sections, with rate-limit copy:

* [ ] **Head / meta** — title, description, OG tags, 🦀 favicon. Title e.g.
  `rust-node-rate-limit — Ultra-fast rate limiting for Node.js powered by Rust`.
* [ ] **Nav** — brand + links (Features, Usage, Integrations, Roadmap) + GitHub
  CTA. Point links to the `rust-node-rate-limit` repo and npm package.
* [ ] **Hero** — tagline "Ultra-fast rate limiting for Node.js powered by Rust",
  version pill, install command `npm install rust-node-rate-limit` with copy
  button, "Get started" / "View on npm" buttons, and a hero code card showing
  the `RateLimiter` + `check()` snapshot:

  ```ts
  import { RateLimiter } from "rust-node-rate-limit";

  const limiter = new RateLimiter({ limit: 100, windowSeconds: 60 });
  limiter.check("user:123");
  // { allowed: true, limit: 100, remaining: 99, retryAfter: 0, resetAfter: 60 }
  ```

* [ ] **Features grid** — adapt cards to: ultra-fast Rust core, Fixed Window
  algorithm, lock-free `DashMap` / thread-safe, prebuilt binaries, TS-first API,
  framework ready (Express / Fastify / NestJS).
* [ ] **Usage tabs** — tabs for `RateLimiter` (allow/check), `stats()`, and
  Express middleware (mirroring the reference's three-tab pattern).
* [ ] **Integrations** — three cards: Express middleware, Fastify plugin,
  NestJS guard, using the import paths from the MVP API
  (`rust-node-rate-limit/express`, `/fastify`, `/nestjs`).
* [ ] **Supported platforms** — chips for darwin-arm64, darwin-x64,
  linux-x64, linux-arm64, win32-x64-msvc.
* [ ] **Roadmap** — Sliding Window (v0.2), Token Bucket (v0.3), Redis backend
  (v0.4), Prometheus metrics (v0.5), ImmutableLog integration (v0.6).
* [ ] **Footer** — brand line, GitHub + npm links, MIT © Roberto Lima.

### 3. Polish & QA

* [ ] Responsive layout verified on mobile and desktop.
* [ ] All copy in English (matches package README).
* [ ] Copy buttons and usage tabs work.
* [ ] Local preview works (`npx serve .` / `python3 -m http.server`).
* [ ] No console errors; valid HTML.

### 4. Deploy

* [ ] Deploy to Vercel (preset **Other**, empty build command, output `.`).
* [ ] Verify production URL and update links if a custom domain is used.
