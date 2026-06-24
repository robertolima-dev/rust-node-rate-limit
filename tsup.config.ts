import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["js/index.ts", "js/express.ts", "js/fastify.ts", "js/nestjs.ts"],
  outDir: "dist",
  format: ["cjs", "esm"],
  dts: true,
  clean: true,
  sourcemap: true,
  target: "node18",
  platform: "node",
  // O addon nativo e os frameworks são resolvidos em runtime, não empacotados.
  external: [
    "../binding.js",
    "../binding",
    "express",
    "fastify",
    "@nestjs/common",
  ],
});
