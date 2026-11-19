# Upstream integration and maintenance

## Qualified Intel trial candidate

The `integration/upstream-20261118` branch imports the 50 commits between original upstream
base `42248f5ef988883cebce298c2279ce22cb536726` and captured upstream main
`333ea31947` on 2026-11-18. Candidate version: **0.170.0-agent-ui.2**.
Selected Intel macOS qualification passed. The existing working branch and runtime are retained;
the candidate has not replaced the daily working runtime.

Imported changes include per-tool app approval reviewers, slash-command argument completion,
task-group navigation, internal thread-link navigation, stream-error handling, queued-agent
message preservation, sandbox-launch preparation, and upstream sidebar-agent discovery.
These include security-sensitive paths; a textual merge is not qualification.

## Maintenance assessment

This is feasible as a maintained fork, with ongoing work. Upstream moved by 50 commits in the
captured interval, so automatically adopting every main-branch change would be costly.
The first integration required manual resolution in three TUI files, the lockfile, and two
generated protocol archives. Cleanly merged areas also need semantic review and regression tests.

The merged-tree comparison against the captured upstream pin shows 54 differing Rust
files, about 3,051 added and 16 removed lines, including tests, five upstream fixture
constructor adaptations, version normalization, a pane-scoped test click target and failure diagnostics. It is mostly additive, but it
is larger than a small display plugin. Keep measuring this surface instead of assuming it stays thin.

The most expensive custom surfaces are assignment metadata across core/protocol/SDK and TUI
event/navigation integration. Keep the native executor and existing permission decisions.
As upstream adds subagent presentation, reuse its state and retire overlapping patches when
its behavior satisfies the tested contract. Avoid creating a second orchestration engine.

## Update procedure

1. Pin an upstream commit. Keep a known-good source revision and runtime.
2. Merge in an isolated worktree and candidate branch. Do not rewrite the daily working branch.
3. Resolve source semantics first. Regenerate stable/experimental protocol archives and SDK
   types from merged Rust definitions rather than choosing a side's generated files.
4. Run protocol/config/connector checks, relevant approval and lifecycle regressions, panel,
   thread-switch and model-selection tests, and strict scoped linting. Record omissions.
5. Build a separate CLI and exercise native tools, parent/child navigation, model/context,
   completion/unload behavior, and subscription discovery in an isolated test home.
6. Promote after review. Preserve the executable, helper, config and state needed for rollback;
   schema migrations can make a binary-only downgrade insufficient.

A weekly reviewed update window is a reasonable starting cadence, with targeted fixes sooner
when needed. This document does not configure recurring automation or unattended deployment.
Prefer published upstream release tags for routine adoption when they contain the required
features. Use a reviewed main-branch pin for a specific feature, as this candidate does, and
qualify its companion-runtime compatibility rather than assuming a matching version label.

Budget for build time as well as conflict resolution. In this Intel evaluation the selected
core test gate spent about 30 minutes compiling before its 12 tests passed in about 10 seconds.
Different Cargo feature combinations can require additional compilation. This is an observed
local cost, not a general benchmark or an estimate of every future update.

## Runtime and data boundaries

Use a separate command and dedicated CODEX_HOME for the fork. Keep official Codex installed.
Trial sandbox and approval changes in owned fixtures before unrelated workspaces or real apps.
Per-tool review behavior can be verified with synthetic fixtures before connected-service use.

The Intel demonstration relies on a compatible companion host. A private local copy
of the qualified `.1` CLI/helper pair is retained outside tracked files. That older runtime
used an installed-app helper. The `.2` candidate instead uses the public package described
below; no helper binaries or credentials are committed. A reproducible source-built host
remains a longer-term goal.

The official `@openai/codex@0.170.0-darwin-x64` package provides the qualified replacement:
its helper advertises gRPC+stdio. `runtime/companion-manifest.json` pins the official package
integrity and helper SHA-256, and `scripts/prepare-agent-ui-runtime.mjs` validates both before
using it. Four unit tests cover corrupted artifacts, unsafe recipes, and overwrite protection.
The public helper passed the live coding/subagent checks and the yielded code-mode spawn
regression. The qualified coding path no longer needs a binary inside the installed ChatGPT app.
The old `.1` runtime is unchanged. The test runner explicitly discovers this prepared sidecar
for fixtures because a downloaded companion is not a Cargo-built binary target.

## Adoption gates

The [machine-readable evidence](evidence/upstream-integration-20261118.json) records the upstream
pin, generated artifacts, source/binary/helper hashes, test commands, failed attempts and corrections.
Selected groups passed using the initial passing cases plus documented targeted rechecks:

| Area | Passing coverage |
| --- | --- |
| Protocol, config, connectors | 326, 430, 67 tests |
| Core approval configuration and routing | 12 unit cases and 3 integration cases |
| Parent panel/navigation and imported TUI features | 287 primary cases and 56 upstream cases, with overlap between groups |
| API and macOS sandbox | 211 and 123 tests; 3 child-helper entry points are invoked by parent tests |
| Shared sandbox launcher | 8 integration cases, none skipped |
| Models manager and model provider | 60 and 138 tests |
| Agent lifecycle and attribution | 10 lifecycle cases, 3 attribution variants, 1 metadata unit case |
| Python SDK contracts and runtime installer | 30 and 4 tests |
| Strict lint | TUI/connectors, all targets, warnings denied; final recheck passed |

The tool-reviewer fixture initially needed retries under parallel startup; its isolated run
and the final four-case attribution/reviewer run passed without retries after a scoped local
scheduling change. Three unrelated unused-import warnings remain in upstream app-server sleep tests.
Visual snapshot normalization changes only displayed build versions, preserving pane width and
the service-version mismatch. Functional assertions were retained.

The live Herdr trial exercised GPT-6.2 Sol, a forked-context child, a fresh-context GPT-6.1 Sol
child, assignment/model/context display, active/completed transcript navigation, and retained
Done counts after return. Fixture hashes remained unchanged. The CLI rebuilt by integration
testing also passed a read-only native-command smoke. The final normal CLI build is the exact
executable used by the interactive walkthrough.

This is a trial recommendation for the tested Darwin x64 coding path. The full monorepo and
other platforms were not tested; voice assets and a source-built companion remain unqualified.
The trial used a fresh, separate CODEX_HOME and does not validate migration of an existing daily
profile. Before daily promotion, review the candidate and preserve profile/config/state backups.
