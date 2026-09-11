#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const mclocksRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const candidates = [
  process.env.MCLOCKS_E2E_ROOT,
  resolve(mclocksRoot, "mclocks-e2e"),
  resolve(mclocksRoot, "../mclocks-e2e"),
].filter(Boolean);

let e2eRoot;
for (const candidate of candidates) {
  if (existsSync(resolve(candidate, "run.mjs"))) {
    e2eRoot = candidate;
    break;
  }
}

if (!e2eRoot) {
  console.error("mclocks-e2e not found.");
  console.error("Clone https://github.com/bayashi/mclocks-e2e as a sibling of mclocks,");
  console.error("or place it at ./mclocks-e2e, or set MCLOCKS_E2E_ROOT.");
  process.exit(1);
}

const result = spawnSync(process.execPath, [resolve(e2eRoot, "run.mjs"), ...process.argv.slice(2)], {
  cwd: mclocksRoot,
  stdio: "inherit",
  env: {
    ...process.env,
    MCLOCKS_ROOT: mclocksRoot,
    MCLOCKS_E2E_ROOT: e2eRoot,
  },
});

process.exit(result.status ?? 1);
