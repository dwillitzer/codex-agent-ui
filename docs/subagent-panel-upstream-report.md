# Proposed report for openai/codex #43148

This is a prepared report, not a posted upstream comment or an external pull request.

Long-running delegated work is difficult to monitor from the parent TUI. Users have to reopen
`/subagents` to distinguish ongoing work from completed work, and the ordinary footer describes
the displayed thread rather than its team.

An implementation reference is available in
[dwillitzer/codex-agent-ui](https://github.com/dwillitzer/codex-agent-ui/tree/feat/parent-agent-panel).
The initial Rust feature commit is
[`4e0f53a`](https://github.com/dwillitzer/codex-agent-ui/commit/4e0f53ae249aa5e2629c58793562d0f66870f66e),
based on upstream `42248f5ef988883cebce298c2279ce22cb536726`.

The implementation adds a bounded persistent preview above the composer, scoped to the primary
thread's descendants. It shows active count, child names/paths, states, configured models, latest
reported context usage, and available elapsed timing. The existing `/subagents` picker exposes
the same metadata and still opens the child's existing transcript. F6, or Down from an empty
prompt, explicitly focuses inline selection. A cursor marks selection separately from the viewed
transcript, and a child-view banner makes direct-input restrictions explicit. Main remains
selectable to return to the parent, preserving its draft and reading bookmark. Escape leaves
panel focus without interrupting work. Compact/expand/focus commands do not submit model input.
No tmux, new server protocol, or periodic backend polling is required.

The panel also offers cancel-first confirmation before interrupting a selected child's current
turn. Parent membership and backend ownership are rechecked at confirmation time. This preserves
native interruption semantics: background terminals may continue, and Interrupted is resumable
rather than final for native agent wait. Agent-lifetime termination is not implemented through
archive/delete or subscription release.

Membership and lifecycle data come from existing spawn lineage, descendant discovery, and
app-server events. Missing context is explicit; cumulative tokens are not treated as current
context occupancy. Idle/unloaded does not imply completed. Direct-input restrictions remain owned
by the existing backend. The preview is bounded and yields to the existing modal/input layout.

Validation on Intel macOS:

- The final inline-navigation increment passed 55 distinct targeted tests, including app-level
  Enter/Escape ownership while reading history, key remaps, held-key draft protection, backend
  scoping, saved drafts/queues, background approvals, legacy/paginated interruption lookup, and
  transcript bookmarks. The 20-case Nextest regression selection passed in 24.376 seconds.
- The initial feature's full shared-process run enumerated 6,230 cases: 6,221 passed, five failed due to shared-process
  Insta inline-snapshot registry collisions, and four were ignored.
- Nextest isolated reruns of all five failures plus the 25 feature/navigation cases passed:
  30/30. That broad run predates the navigation increment; a new full-suite replay and a single
  all-green full Nextest invocation are not claimed.
- Strict TUI Clippy passed for all targets with `--no-deps -- -D warnings`. Dependency-inclusive
  linting encountered an existing warning in unchanged core code.
- The final CLI build passed. Real Herdr sessions exercised both legacy and parent-managed native
  children, model/context display, active-child inspection, exact parent draft restoration,
  reading-position restoration with new activity retained, normal completion, and confirmed
  turn interruption. Fixture inputs remained unchanged; owned test sleepers were cleaned up.

The local CLI required a compatible signed companion from the installed ChatGPT application:
the pinned V8 Intel artifact needed to build the host from source returned 404. This local runtime
prerequisite and the unrun Linux/Windows UI checks are documented in the
[evaluation record](subagent-panel-evaluation.md) and
[live evaluation](subagent-panel-live-evaluation.md). The
[navigation evaluation](subagent-panel-navigation-evaluation.md) and its machine-readable source
and executable hashes document the follow-up increment.

The intent is to share the behavior, implementation observations, and evidence through the issue
tracker in accordance with the current contribution policy. The fork is an implementation
reference, not a claim of upstream acceptance or production readiness.
