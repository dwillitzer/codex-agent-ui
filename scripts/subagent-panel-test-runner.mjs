#!/usr/bin/env node
// Cargo execution wrapper: preserve Cargo's test cwd/env and isolate runtime home discovery.
// The compiler and Cargo caches run outside this wrapper with their normal environment.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [binary, ...args] = process.argv.slice(2);
if (!binary) throw new Error("Expected a Cargo test executable");
const fixtureHome = mkdtempSync(join(tmpdir(), "codex-agent-ui-test-home-"));
console.log(`Isolated test-process home: ${fixtureHome}`);
const child = spawn(binary, args, {
  stdio: "inherit",
  env: {
    ...process.env,
    HOME: fixtureHome,
    ...(process.platform === "win32" ? { USERPROFILE: fixtureHome } : {}),
    RUST_MIN_STACK: process.env.RUST_MIN_STACK ?? "8388608",
  },
});
const timer = setTimeout(
  () => {
    console.error("Test process exceeded the 20-minute evaluation limit");
    child.kill("SIGTERM");
  },
  20 * 60 * 1000,
);
process.once("SIGTERM", () => child.kill("SIGTERM"));
child.once("error", (error) => {
  clearTimeout(timer);
  console.error(error);
  process.exitCode = 1;
});
child.once("exit", (code, signal) => {
  clearTimeout(timer);
  if (signal) console.error(`Test process terminated by ${signal}`);
  process.exitCode = code ?? 1;
});
