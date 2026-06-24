// Exemplo Fastify — execute com: node examples/fastify-example.mjs
// Requer build local: npm run build
import Fastify from "fastify";
import { rateLimitPlugin } from "rust-node-rate-limit/fastify";

const fastify = Fastify();

// 100 requisições por minuto, por IP.
fastify.register(rateLimitPlugin, { limit: 100, windowSeconds: 60 });

fastify.get("/health", async () => ({ ok: true }));

fastify.listen({ port: 3001 }, (err, address) => {
  if (err) {
    console.error(err);
    process.exit(1);
  }
  console.log(`Fastify example on ${address}`);
  console.log("Try hammering: /health");
});
