# Proposed report for openai/codex #43148

This is a prepared report, not a posted upstream comment or an external pull request.

Long-running delegated work is difficult to monitor from the parent TUI. Users have to reopen
`/subagents` to distinguish ongoing work from completed work, and the ordinary footer describes
the displayed thread rather than its team.

An implementation reference is available in
[dwillitzer/codex-agent-ui](https://github.com/dwillitzer/codex-agent-ui/tree/feat/parent-agent-panel).
The tested Rust implementation is commit
[`4e0f53a`](https://github.com/dwillitzer/codex-agent-ui/commit/4e0f53ae249aa5e2629c58793562d0f66870f66e),
based on upstream `42248f5ef988883cebce298c2279ce22cb536726`.

The implementation adds a bounded persistent preview above the composer, scoped to the primary
thread's descendants. It shows active count, child names/paths, states, configured models, latest
reported context usage, and available elapsed timing. The existing `/subagents` picker exposes
the same metadata and still opens the child's existing transcript. Main remains selectable to
return to the parent. Compact/expand commands preserve the team state without submitting model
input. No tmux, new server protocol, or periodic backend polling is required.

Membership and lifecycle data come from existing spawn lineage, descendant discovery, and
app-server events. Missing context is explicit; cumulative tokens are not treated as current
context occupancy. Idle/unloaded does not imply completed. Direct-input restrictions remain owned
by the existing backend. The preview is bounded and yields to the existing modal/input layout.

Validation on Intel macOS:

- All 15 new application/render/command tests passed, as did 10 existing navigation tests.
- A full shared-process run enumerated 6,230 cases: 6,221 passed, five failed due to shared-process
  Insta inline-snapshot registry collisions, and four were ignored.
- Nextest isolated reruns of all five failures plus the 25 feature/navigation cases passed:
  30/30. A single all-green full Nextest invocation is not claimed.
- Strict TUI Clippy passed for all targets with `--no-deps -- -D warnings`. Dependency-inclusive
  linting encountered an existing warning in unchanged core code.
- A real Herdr session exercised native child creation, model/context display, opening an active
  child during its shell wait, returning to Main, completion retention, and compact/expand.
  Fixture inputs remained unchanged.

The local CLI required a compatible signed companion from the installed ChatGPT application:
the pinned V8 Intel artifact needed to build the host from source returned 404. This local runtime
prerequisite and the unrun Linux/Windows UI checks are documented in the
[evaluation record](subagent-panel-evaluation.md) and
[live evaluation](subagent-panel-live-evaluation.md).

The intent is to share the behavior, implementation observations, and evidence through the issue
tracker in accordance with the current contribution policy. The fork is an implementation
reference, not a claim of upstream acceptance or production readiness.
