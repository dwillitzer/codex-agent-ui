#!/usr/bin/env node
// Reproducible local evaluation. Does not launch an authenticated turn or replace installed Codex.
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  openSync,
  closeSync,
  writeFileSync,
  readFileSync,
  existsSync,
  realpathSync,
  symlinkSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const output = resolve(
  process.argv[2] ?? join(tmpdir(), `codex-agent-panel-eval-${Date.now()}`),
);
mkdirSync(output, { recursive: true });
const rustcVersion = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
const host = rustcVersion.stdout.match(/^host: (.+)$/m)?.[1];
if (!host) throw new Error("Could not determine the Rust host target");
const runtimeHost = join(
  root,
  "codex-rs",
  "target",
  "debug",
  process.platform === "win32"
    ? "codex-code-mode-host.exe"
    : "codex-code-mode-host",
);
const companionOverride = process.env.CODEX_AGENT_UI_HOST_PROGRAM;
let companion = { mode: "source-build" };
if (companionOverride) {
  const program = realpathSync(resolve(companionOverride));
  if (process.platform === "darwin") {
    const signature = spawnSync("codesign", ["--verify", "--strict", program], {
      encoding: "utf8",
    });
    if (signature.status !== 0)
      throw new Error(`Companion signature check failed: ${signature.stderr}`);
  }
  const help = spawnSync(program, ["--help"], {
    encoding: "utf8",
    timeout: 10000,
  });
  if (help.status !== 0 || !help.stdout.includes("grpc+stdio"))
    throw new Error(
      "Explicit companion does not advertise the required grpc+stdio transport",
    );
  if (existsSync(runtimeHost)) {
    if (realpathSync(runtimeHost) !== program)
      throw new Error(
        "A different runtime companion already exists; refusing to overwrite it",
      );
  } else {
    mkdirSync(dirname(runtimeHost), { recursive: true });
    symlinkSync(program, runtimeHost, "file");
  }
  companion = {
    mode: "explicit-companion",
    program,
    sha256: createHash("sha256").update(readFileSync(program)).digest("hex"),
  };
}
const testRunnerConfig = `target.${host}.runner=${JSON.stringify([
  process.execPath,
  join(root, "scripts", "subagent-panel-test-runner.mjs"),
])}`;
const git = (...args) => {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 10_000_000,
  });
  if (result.status !== 0) throw new Error(result.stderr);
  return result.stdout.trim();
};
const report = {
  base: git("rev-parse", "HEAD"),
  workingDiffSha256: createHash("sha256")
    .update(git("diff", "HEAD", "--binary"))
    .digest("hex"),
  workingStatus: git("status", "--short"),
  startedAt: new Date().toISOString(),
  rustMinStack: process.env.RUST_MIN_STACK ?? "8388608",
  testIsolation:
    "Temporary HOME for test executable only; Cargo/compiler environment unchanged",
  testThreads: 4,
  companion,
  checks: [],
};
const checks = [
  ["whitespace", "git", ["diff", "HEAD", "--check"], root],
  [
    "runtime-host-build",
    "cargo",
    ["build", "--locked", "-p", "codex-code-mode-host"],
    join(root, "codex-rs"),
  ],
  ["runtime-host-help", runtimeHost, ["--help"], root],
  [
    "new-app-tests",
    "cargo",
    [
      "test",
      "--locked",
      "-p",
      "codex-tui",
      "--lib",
      "app::tests::agent_panel_tests",
    ],
    join(root, "codex-rs"),
  ],
  [
    "panel-render-tests",
    "cargo",
    ["test", "--locked", "-p", "codex-tui", "--lib", "subagent_panel"],
    join(root, "codex-rs"),
  ],
  [
    "navigation-regression",
    "cargo",
    ["test", "--locked", "-p", "codex-tui", "--lib", "agents_navigation"],
    join(root, "codex-rs"),
  ],
  [
    "transcript-bookmark-regression",
    "cargo",
    ["test", "--locked", "-p", "codex-tui", "--lib", "transcript_view::bookmark"],
    join(root, "codex-rs"),
  ],
  [
    "thread-switch-regression",
    "cargo",
    [
      "nextest",
      "run",
      "--locked",
      "-p",
      "codex-tui",
      "--lib",
      "--profile",
      "local",
      "--test-threads",
      "4",
      "--no-fail-fast",
      "-E",
      "test(open_agent_picker) | test(select_uncached_agent_thread) | test(replay_thread_snapshot_restores_draft) | test(embedded_session_switch_routes_background_approval_and_restores_draft) | test(agents_overview_stop_) | test(transcript_view::bookmark) | test(replace_chat_widget)",
    ],
    join(root, "codex-rs"),
  ],
  [
    "tui-suite",
    "cargo",
    [
      "nextest",
      "run",
      "--locked",
      "-p",
      "codex-tui",
      "--lib",
      "--profile",
      "local",
      "--test-threads",
      "4",
      "--no-fail-fast",
    ],
    join(root, "codex-rs"),
  ],
  [
    "clippy",
    "cargo",
    [
      "clippy",
      "--locked",
      "-p",
      "codex-tui",
      "--all-targets",
      "--no-deps",
      "--",
      "-D",
      "warnings",
    ],
    join(root, "codex-rs"),
  ],
  [
    "cli-build",
    "cargo",
    ["build", "--locked", "-p", "codex-cli", "--bin", "codex"],
    join(root, "codex-rs"),
  ],
  [
    "cli-version",
    join(root, "codex-rs", "target", "debug", "codex"),
    ["--version"],
    root,
  ],
].filter(([name]) => !companionOverride || name !== "runtime-host-build");
const save = () =>
  writeFileSync(
    join(output, "results.json"),
    `${JSON.stringify(report, null, 2)}\n`,
  );
console.log(`Evaluation artifacts: ${output}`);
const requestedStart = process.env.CODEX_AGENT_UI_EVAL_FROM;
const only = process.env.CODEX_AGENT_UI_EVAL_ONLY?.split(",").filter(Boolean);
if (only?.some((name) => !checks.some(([check]) => check === name)))
  throw new Error("Unknown selected evaluation check");
const firstCheck = requestedStart
  ? checks.findIndex(([name]) => name === requestedStart)
  : 0;
if (firstCheck < 0)
  throw new Error(`Unknown evaluation check: ${requestedStart}`);
for (const [name, command, args, cwd] of checks.slice(firstCheck)) {
  if (only && !only.includes(name)) continue;
  const executedArgs =
    command === "cargo" && args[0] === "test"
      ? [...args, "--config", testRunnerConfig, "--", "--test-threads=4"]
      : command === "cargo" && args[0] === "nextest"
        ? [...args, "--config", testRunnerConfig]
        : args;
  const logfile = join(output, `${name}.log`);
  const fd = openSync(logfile, "w");
  const check = {
    name,
    command: [command, ...executedArgs],
    startedAt: new Date().toISOString(),
    logfile,
  };
  report.checks.push(check);
  save();
  console.log(`Starting ${name}`);
  try {
    check.exitCode = await new Promise((done, reject) => {
      const child = spawn(command, executedArgs, {
        cwd,
        // Match upstream rust-ci.yml: async TUI tests use an 8 MiB thread stack.
        env: { ...process.env, RUST_MIN_STACK: report.rustMinStack },
        stdio: ["ignore", fd, fd],
      });
      child.once("error", reject);
      child.once("exit", (code, signal) => {
        check.signal = signal;
        done(code);
      });
    });
  } catch (error) {
    check.error = String(error);
    check.exitCode = 1;
  } finally {
    closeSync(fd);
    check.finishedAt = new Date().toISOString();
    save();
  }
  console.log(`${name}: ${check.exitCode === 0 ? "passed" : "failed"}`);
  if (check.exitCode !== 0) {
    process.exitCode = 1;
    // New-feature failures stop the evaluation. Broader regression/lint failures still allow
    // independent checks to finish so the report does not imply those checks were run or passed.
    if (
      [
        "whitespace",
        "runtime-host-build",
        "runtime-host-help",
        "new-app-tests",
        "panel-render-tests",
      ].includes(name)
    )
      break;
  }
}
report.finishedAt = new Date().toISOString();
save();
