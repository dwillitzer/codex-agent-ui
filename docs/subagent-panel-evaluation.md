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

**Work in progress: feature tests pass; complete regression validation is not yet green.**

Run `node scripts/evaluate-subagent-panel.mjs /path/to/evaluation-output` to reproduce the checks.
The script records commands, exit status, timestamps, and the working diff digest. It uses the
upstream 8 MiB test-thread stack and a Cargo runner that isolates the **test executable's** home.
The compiler and Cargo caches keep their normal environment. The full suite uses Nextest's local
profile, process isolation, timeout/retry policy, and four concurrent tests.

| Check                                               | Observed result                                                             |
| --------------------------------------------------- | --------------------------------------------------------------------------- |
| Whitespace and touched-file nightly rustfmt         | Passed during implementation; final rerun pending                           |
| New application tests                               | **8 passed**, 0 failed                                                      |
| New render and command tests                        | **7 passed**, 0 failed; also passed through the isolated-home wrapper       |
| Existing navigation regression group                | **10 passed**, 0 failed with upstream's 8 MiB stack                         |
| Full shared-process libtest run with a clean home   | **6,221 passed, 5 failed, 4 ignored**; failures remain under investigation  |
| Canonical Nextest run with isolated test homes      | Queued behind the separate CLI build                                        |
| Standard strict Clippy including dependency linting | Blocked by `collapsible_if` in unchanged `core/src/session/session.rs:1814` |
| Strict TUI Clippy without dependency linting        | Pending                                                                     |
| Separate CLI build and terminal smoke               | Building; not yet validated                                                 |

The first default-stack navigation run aborted with stack overflow. The upstream Rust CI files
explicitly set `RUST_MIN_STACK=8388608`; that setting made all 10 navigation cases pass.
The first TUI compiler pass found an ambiguous test macro import and an omitted internal
token-usage field. Both were corrected before the passing new-feature tests.

An initial non-hermetic suite showed six skill/config warning banners in snapshots. Skill discovery
reads the real home even with a temporary Codex directory. The failing draft-handoff snapshot
passed with the **same compiled executable** and only a clean test-process home. No golden
snapshots were accepted to hide this difference. A fork-lifecycle test also passed when rerun
individually; this does not establish that the broader suite is green.

The five failures in the clean-home shared-process run were:

- `fork_releases_a_provisional_saved_chat_on_its_original_connection`
- `agents_overview_acknowledges_inactive_steer_before_interrupt`
- `saved_chat_replays_approval_after_a_stale_runtime_close`
- `sidebar_reopens_saved_side_chat_after_restart`
- `slash_daybreak_offers_an_application_when_unavailable`

Nextest results and any subsequent fixes must be recorded before marking the feature validated.
Raw local logs and result JSON files are retained under the checkout's ignored `.dev/` directory.
They are not included in the public fork.

The initial foreground build exceeded the tool transport's five-minute response timeout while
populating upstream git dependencies. It was stopped and restarted with a persistent log and exit
record. That transport timeout was not a compiler/test failure. The subsequent build progressed
through dependency download into Rust compilation.

## Contribution route

Upstream `docs/contributing.md` currently does not accept external code contributions or PRs.
Prepare an issue-based implementation report with the final commit, test output, behavior examples,
and explicit remaining limits. The related issue is
[#43148](https://github.com/openai/codex/issues/43148). Do not describe this fork as merged, accepted,
or production-validated without corresponding evidence.
