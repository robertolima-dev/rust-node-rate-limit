// Exemplo Express — execute com: node examples/express-example.mjs
// Requer build local: npm run build
import express from "express";
import { rateLimitMiddleware } from "rust-node-rate-limit/express";

const app = express();

// 100 requisições por minuto, por IP.
app.use(rateLimitMiddleware({ limit: 100, windowSeconds: 60 }));

app.get("/health", (_req, res) => {
  res.json({ ok: true });
});

app.listen(3000, () => {
  console.log("Express example on http://localhost:3000");
  console.log("Try hammering: /health");
});
