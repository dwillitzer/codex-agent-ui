# Upstream integration and maintenance

## Candidate under evaluation

The `integration/upstream-20261118` branch imports the 50 commits between original upstream
base `42248f5ef988883cebce298c2279ce22cb536726` and captured upstream main
`333ea31947` on 2026-11-18. Candidate version: **0.170.0-agent-ui.2**.
The existing working branch and runtime are retained while the candidate is evaluated.

Imported changes include per-tool app approval reviewers, slash-command argument completion,
task-group navigation, internal thread-link navigation, stream-error handling, queued-agent
message preservation, sandbox-launch preparation, and upstream sidebar-agent discovery.
These include security-sensitive paths; a textual merge is not qualification.

## Maintenance assessment

This is feasible as a maintained fork, with ongoing work. Upstream moved by 50 commits in the
captured interval, so automatically adopting every main-branch change would be costly.
The first integration required manual resolution in three TUI files, the lockfile, and two
generated protocol archives. Cleanly merged areas also need semantic review and regression tests.

The merged-tree comparison against the captured upstream pin shows 51 differing Rust
files, about 3,006 added and 11 removed lines, including tests and five upstream fixture
constructor adaptations. It is mostly additive, but it
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

The official `@openai/codex@0.170.0-darwin-x64` package now provides a possible replacement:
its helper advertises gRPC+stdio. `runtime/companion-manifest.json` pins the official package
integrity and helper SHA-256, and `scripts/prepare-agent-ui-runtime.mjs` validates both before
using it. Four unit tests cover corrupted artifacts, unsafe recipes, and overwrite protection.
The candidate is being qualified with this public helper; the old `.1` runtime is unchanged.
If it passes, daily operation no longer needs a binary inside the installed ChatGPT app.

## Adoption gates

Candidate testing is in progress. It is not yet a recommendation to replace the working runtime.
The final record must name the upstream pin, conflict resolutions, generated artifacts, tests,
executable/helper hashes, live checks, and remaining platform/runtime limits.
