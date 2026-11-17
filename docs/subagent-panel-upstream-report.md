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
reported context usage, available elapsed timing, and a short assignment on each main row.
Option/Alt + Left/Right switches directly from an empty prompt, in the same known parent family;
word editing retains those keys while a draft is present. The existing `/subagents` picker exposes
the same metadata and still opens the child's existing transcript. F6, or Down from an empty
prompt, explicitly focuses inline selection. A cursor marks selection separately from the viewed
transcript, and a child-view banner makes direct-input restrictions explicit. Main remains
selectable to return to the parent, preserving its draft and reading bookmark. Escape leaves
panel focus without interrupting work. Compact/expand/focus commands do not submit model input.
No tmux, new server RPC, or periodic backend polling is required.

The focused bar also offers All/Active/Done/Failed tabs, cycled with `f`. Arrow navigation uses
the filtered rows while Main remains selectable. The filter survives child inspection and
resets for a different parent. Pending approvals stay Active and retain a parent-visible count
even when another tab hides the child. Existing user/policy approval decisions remain unchanged.

The parent supplies a bounded one-line summary through `agent_ui.set_assignment` after dispatch
and follow-up. The native collaboration parameter schemas remain unchanged because the model API
reserves them. An optional activity metadata field carries the caption through live notifications
and persisted history. Older records remain readable and use a saved title or an explicit unknown
assignment. Historical activities cannot grant parent membership or overwrite newer live labels.

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

- The assignment increment passed 931 distinct selected tests: 388 protocol, 326 API/schema, 149 core
  agent/factory, 38 UI state/render/navigation, 20 existing thread-switch regressions, and 10 Python
  SDK contract cases. Stable/experimental exports and generated Python types were refreshed.
- The passing core selection included one Nextest process-output leak warning, documented by
  test name in the evidence. This does not imply a clean full-suite or cross-platform run.
- The initial feature's full shared-process run enumerated 6,230 cases: 6,221 passed, five failed due to shared-process
  Insta inline-snapshot registry collisions, and four were ignored.
- Nextest isolated reruns of all five failures plus the 25 feature/navigation cases passed:
  30/30. That broad run predates the navigation increment; a new full-suite replay and a single
  all-green full Nextest invocation are not claimed.
- Strict TUI Clippy passed for all targets with `--no-deps -- -D warnings`. Dependency-inclusive
  linting encountered an existing warning in unchanged core code; the latest core-only lint run
  explicitly allows that `collapsible_if` category while denying other warnings.
- The final CLI build passed. Real Herdr sessions exercised both legacy and parent-managed native
  children, model/context display, active-child inspection, exact parent draft restoration,
  reading-position restoration with new activity retained, normal completion, and confirmed
  turn interruption. Fixture inputs remained unchanged; owned test sleepers were cleaned up.
- The latest live session verified one-gesture direct switching, assignment replacement on a
  follow-up, caption retention after restart, automatic label reporting without a helper-specific
  user request, and preservation of word editing with an unsent draft.
- The subsequent status-filter increment passed 53 selected panel/navigation/approval tests and
  20 existing thread-switch regressions, strict TUI Clippy, and a CLI build. A separate live session
  verified mixed-status filtering, child inspection and return with filter retention, an empty
  Failed tab with Main available, and restoration of All. Approval and explicit-failure cases
  were exercised through app-server test fixtures, not live terminal prompts.

The local CLI required a compatible signed companion from the installed ChatGPT application:
the pinned V8 Intel artifact needed to build the host from source returned 404. This local runtime
prerequisite and the unrun Linux/Windows UI checks are documented in the
[evaluation record](subagent-panel-evaluation.md) and
[live evaluation](subagent-panel-live-evaluation.md). The
[navigation evaluation](subagent-panel-navigation-evaluation.md) and its machine-readable source
and executable hashes document the navigation increment. The latest
[assignment evaluation](subagent-panel-assignment-evaluation.md) records the metadata and direct
navigation changes with a separate source/binary manifest.
The [status-filter evaluation](subagent-panel-status-filter-evaluation.md) records the subsequent
filter behavior, limits, and its own source/binary manifest.

The intent is to share the behavior, implementation observations, and evidence through the issue
tracker in accordance with the current contribution policy. The fork is an implementation
reference, not a claim of upstream acceptance or production readiness.
