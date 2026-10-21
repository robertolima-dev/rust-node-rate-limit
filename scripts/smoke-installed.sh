#!/bin/sh
# Install the packed tarball into an empty project and use the public API, the way a user does.
#
# Usage (from the repository root, next to the .tgz made by `npm pack`):
#   sh scripts/smoke-installed.sh <platform>      e.g. linux-x64-musl
set -eu

platform="${1:?usage: smoke-installed.sh <platform>, e.g. linux-x64-musl}"
root="$(pwd)"
tarball="$(ls "$root"/rust-node-rate-limit-*.tgz | head -n 1)"
work="$(mktemp -d)"

cp "$tarball" "$work/package.tgz"
cp "$root/scripts/smoke-installed.cjs" "$work/"
cd "$work"
npm init -y >/dev/null
npm install --no-audit --no-fund ./package.tgz

node smoke-installed.cjs "$platform"
node --input-type=module -e '
  import { RateLimiter } from "rust-node-rate-limit";
  const limiter = new RateLimiter({ limit: 1, windowSeconds: 60 });
  if (limiter.allow("k") !== true || limiter.allow("k") !== false) throw new Error("ESM import returned the wrong value");
  console.log("ESM import ok");
'
