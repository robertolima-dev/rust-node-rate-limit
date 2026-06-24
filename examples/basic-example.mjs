// Exemplo básico — execute com: node examples/basic-example.mjs
// Requer build local: npm run build
import { RateLimiter } from "rust-node-rate-limit";

const limiter = new RateLimiter({ limit: 5, windowSeconds: 60 });

for (let i = 1; i <= 7; i++) {
  const result = limiter.check("ip:127.0.0.1");
  console.log(
    `req ${i}: allowed=${result.allowed} remaining=${result.remaining} ` +
      `retryAfter=${result.retryAfter}s resetAfter=${result.resetAfter}s`,
  );
}

console.log("stats:", limiter.stats());
