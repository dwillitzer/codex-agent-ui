# Subagent panel evaluation

## Scope and provenance

- Repository: `https://github.com/openai/codex`
- Base commit: `42248f5ef988883cebce298c2279ce22cb536726`
- Branch: `feat/parent-agent-panel`
- Host: macOS, x86_64; Cargo/Rust toolchain 1.99.1.
- Installed Codex and its launch commands are not replaced by this checkout.
- Fixtures use synthetic thread IDs, models, prompts, and token counts. No account credentials,
  user conversations, or screenshots are included in the contribution artifacts.

## Acceptance matrix

| Behavior                                                                  | Evidence target                            |
| ------------------------------------------------------------------------- | ------------------------------------------ |
| No children: zero extra composer rows                                     | Render unit test                           |
| Parent and nested child membership; unrelated roots excluded from preview | App integration test                       |
| Model and context in persistent preview and picker                        | App integration test and render snapshot   |
| Latest context rather than lifetime tokens; compaction reduces usage      | App integration test                       |
| Missing/invalid context is unavailable; percentage stays bounded          | Unit test                                  |
| Idle, waiting, approval, completed, disconnected are distinguishable      | State and app tests                        |
| Older turn completion cannot finish the newer turn                        | App integration test                       |
| Completed children stay visible and inspectable                           | App and renderer tests                     |
| Compact/expand keeps membership and does not submit model input           | Command and render tests                   |
| Selecting child uses the existing thread-watch action                     | App action-routing test                    |
| Large teams stay bounded; active/current children prioritized             | Render unit test                           |
| Narrow terminals and clipped viewports                                    | Render unit tests                          |
| Return-to-parent and existing thread navigation                           | Existing agent navigation regression tests |
| Full TUI regression, formatting, Clippy                                   | Build/test commands below                  |
| Actual terminal startup and interactive child inspection                  | Local binary smoke test                    |

## Commands and results

The macOS TUI feature passed focused tests and live interaction. A self-contained Intel companion
build remains blocked by a V8 artifact prerequisite; see the
[live evaluation](subagent-panel-live-evaluation.md) for the signed companion used here.

| Check                                                                                  | Observed result                                                             |
| -------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Whitespace and touched-file nightly rustfmt                                            | Passed                                                                      |
| New application tests                                                                  | 8 passed                                                                    |
| New render and command tests                                                           | 7 passed                                                                    |
| Existing navigation tests                                                              | 10 passed                                                                   |
| Full shared-process TUI run, clean home                                                | 6,221 passed, 5 failed, 4 ignored                                           |
| Isolated Nextest rerun of all five failures plus the 25 feature/navigation cases       | **30 passed, 0 failed**, 3 slow; 49.460 seconds                             |
| Strict TUI Clippy, all targets, no dependency linting                                  | Passed with `--no-deps -- -D warnings`                                      |
| Strict Clippy including dependencies                                                   | Blocked by `collapsible_if` in unchanged `core/src/session/session.rs:1814` |
| Compiled CLI and version check                                                         | Passed; development version `0.0.0`                                         |
| Companion build from source on Intel macOS                                             | Blocked: V8 152.2.0 Intel binding download returned HTTP 404                |
| Explicit signed companion smoke                                                        | Passed                                                                      |
| Live Herdr TUI: children, active inspection, return to parent, context, compact/expand | Passed; see linked live evaluation                                          |

The shared-process run enumerated 6,230 library cases: 6,226 executed and four ignored. All five
failures were Insta rejecting inline snapshot assertions reused in that process, rather than
five differing product assertions. Each passed under the repository's recommended process-isolated
Nextest runner, together with all 15 new tests and 10 navigation tests. A redundant full Nextest
replay was stopped after identifying this cause; **one all-green full Nextest invocation is not
claimed**.

The isolated run tested implementation commit `4e0f53ae249aa5e2629c58793562d0f66870f66e`.
Subsequent changes concern docs and evaluation scripts, not the tested Rust implementation.

## Reproduction and isolation

Run `node scripts/evaluate-subagent-panel.mjs /path/to/output` for the complete workflow. It records
commands, exits, timestamps, and a diff digest. Tests use the upstream 8 MiB stack, Nextest's local
profile, and a Cargo runner isolating the test executable's home; compiler/Cargo caches retain
their normal environment. Build the companion first or use the explicit override documented in
the live evaluation. No installed helper is silently substituted.

`CODEX_AGENT_UI_EVAL_ONLY=runtime-host-help,cli-version` selects a narrow smoke check;
`CODEX_AGENT_UI_EVAL_FROM=clippy` resumes the later phase. Skipped checks are not counted as passed.

The default test stack initially overflowed; upstream CI's `RUST_MIN_STACK=8388608` resolved it.
Skill discovery also read the real home despite a temporary Codex directory, adding six warnings
to snapshots. The affected snapshot passed with the same executable and only a clean test-process
home. No golden snapshots were accepted to hide the warnings.

Raw pane captures, logs, and result JSON remain in ignored local `.dev/` directories.
Linux/Windows UI, the full workspace suite, and a source-built standalone Intel companion remain
unrun/unverified.

## Contribution route

Upstream `docs/contributing.md` currently does not accept external code contributions or PRs.
Prepare an issue-based implementation report with the final commit, test output, behavior examples,
and explicit remaining limits. The related issue is
[#43148](https://github.com/openai/codex/issues/43148). Do not describe this fork as merged, accepted,
or production-validated without corresponding evidence.
