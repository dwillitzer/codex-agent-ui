# Status-filter evaluation

The [status-filter specification](subagent-panel-status-filter-spec.md) is implemented on
`feat/parent-agent-panel`, following assignment/navigation commit
`8a1bd4d1c9fc865754c7691e3f9839d1fe7bb489`. Evidence was collected on Intel macOS on
2026-11-17. The [machine-readable record](evidence/subagent-panel-status-filters.json)
records commands, exit codes, source hashes, executable hash, and live assertions.

## Automated checks

- 53 selected TUI tests passed under Nextest process isolation: panel rendering, filter
  classification and navigation, approval aggregation, and existing agent-navigation cases.
- 20 existing thread-switch regressions passed, including draft restoration, approval routing,
  transcript bookmarks, and stop-turn behavior.
- Strict TUI Clippy passed: `cargo clippy --locked -p codex-tui --all-targets --no-deps -- -D warnings`.
- CLI build passed: `cargo build --locked -p codex-cli --bin codex`.

Tests exercise fresh-press cycling and repeat suppression, empty results/Main return, stable
selection, filter persistence across widget replacement, reset for a new parent, and rejection
of stale parent-bound filter events. App-server request fixtures exercise a viewed child's
pending approval and cancellation while Failed is selected. Explicit failure remains Failed
after a generic parent completion event. Narrow/compact rendering retains approval awareness.

These are 73 selected passing tests, not a full-suite replay or a Linux/Windows UI matrix.
Earlier assignment/core/schema evaluation results remain separate historical records.

## Live TUI check

An isolated Herdr workspace, `codex-ui-filter-review` (`w3B:p1`), ran the built CLI with
separate temporary HOME and CODEX_HOME directories. Exactly two native children reviewed
read-only fixtures after different sleeps. Both completed normally.

- With one child running and one done, Active showed only the running child plus Main.
- Enter opened that child's existing transcript; the Active filter survived inspection.
- When the viewed child completed, Active became empty without changing the viewed transcript.
- Returning to Main retained Active. Done then displayed both completed children.
- Failed displayed an explicit empty result and retained selectable Main.
- All restored both children, their short assignments, configured model, and context usage.
- The review was left focused on All, ready for `f` cycling. Both fixture hashes were unchanged.

Live terminal approval prompts and a real failing child were not generated in this increment;
their state transitions are covered by the automated app-server fixtures above. No installed
Codex binary or user configuration was replaced.

The CLI uses the compatible signed companion host from the installed ChatGPT application,
as qualified in the [earlier live evaluation](subagent-panel-live-evaluation.md). The pinned
Intel V8 artifact still prevents claiming a self-contained source-built host. The companion
is a local prerequisite and is not distributed with this fork.
