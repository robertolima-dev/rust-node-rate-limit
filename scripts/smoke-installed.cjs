// Use the installed package through its public API and check that the native binary for the
// expected platform is the one that loaded. Run by scripts/smoke-installed.sh.
const assert = require("node:assert/strict");
const path = require("node:path");

const platform = process.argv[2];
const { RateLimiter } = require("rust-node-rate-limit");

const binary = Object.keys(require.cache).find((file) => file.endsWith(".node"));
assert.equal(path.basename(binary ?? "none"), `rust-node-rate-limit.${platform}.node`);

for (const algorithm of ["fixed", "sliding"]) {
  const limiter = new RateLimiter({ limit: 2, windowSeconds: 60, algorithm });
  assert.deepEqual([limiter.allow("ip"), limiter.allow("ip"), limiter.allow("ip")], [true, true, false]);
  assert.equal(limiter.check("ip").allowed, false);
  assert.equal(limiter.remaining("other"), 2);
}
assert.throws(() => new RateLimiter({ limit: 0, windowSeconds: 1 }), /limit/);

console.log(`ok: ${process.platform}/${process.arch} (node ${process.version}) loaded ${path.basename(binary)}`);
