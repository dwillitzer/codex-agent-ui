# Agent-panel visual refinement

This follow-up to `aebe2cd53aa288f7651de3876951a34a0de6d697` changes presentation only:

- Other agents use a hollow circle (`○`); the viewed transcript retains `●`.
- The independent `>` cursor continues to identify keyboard selection.
- Viewed and keyboard-highlighted rows are bold in the terminal's normal foreground;
  other rows are dim. Moving the cursor does not move the viewed-session circle.
- The active status tab uses bold reverse-video cyan; inactive tabs are dim.
  Narrow layouts retain the same active-label highlight.

All 55 selected panel, navigation, and approval tests passed under isolated Nextest processes
on Intel macOS. The new rendering check inspects terminal-buffer colors and modifiers for
every active tab at widths 30 and 100. Plaintext snapshots alone cannot verify contrast.
The additional row-emphasis check covers bold/dim styling with the viewed transcript and keyboard
selection on different agents.

The first run passed 53 tests and exposed one old assertion expecting the previous dot glyph.
After correcting that expectation, the complete 54-test selection passed. Adding row-emphasis
coverage subsequently increased the passing selection to 55.

Strict TUI linting also passed with
`cargo clippy --locked -p codex-tui --all-targets --no-deps -- -D warnings`.
The CLI build passed with `cargo build --locked -p codex-cli --bin codex`.

An isolated copy of the completed demo was resumed in Herdr workspace
`codex-ui-contrast-review` (`w3C:p1`). Captured ANSI output verified bold Main with `●`,
a different bold keyboard-selected child with `>○`, and a dim remaining child with `○`.
Cycling to Active verified bold/reverse-video/cyan attributes on that tab. The panel was
returned to All with a child highlighted for review. No model task was submitted.
Resumed children were reported closed with unavailable context; this visual check does not
claim fresh lifecycle or context measurements. The original review pane was left untouched.

The [machine-readable evidence](evidence/subagent-panel-visual-refinement.json) records source
and executable hashes, check commands and exits, and the live terminal assertions. The
compatible companion-host prerequisite from the earlier live evaluation still applies.

The preceding [status-filter evaluation](subagent-panel-status-filter-evaluation.md) remains
the separate record for live lifecycle filtering and the 20 thread-switch regressions.
This increment does not claim a new full-suite or cross-platform run.
