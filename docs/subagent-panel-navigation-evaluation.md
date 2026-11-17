# Inline agent navigation evaluation

This increment adds explicit focus to the parent-scoped panel, independently implemented in Rust
using Codex's existing thread switching, input state, transcript bookmarks, and interruption API.
No tmux session or alternative agent executor is involved.

## Automated checks

The final Rust changes passed 55 distinct targeted tests:

| Group | Result | Coverage |
| --- | --- | --- |
| `subagent_panel` | 15 passed | Bounded rendering, stable identity through completion/removal, keyboard focus, draft protection, held Enter/Escape, read-only navigation, key remaps, command routing |
| `app::tests::agent_panel_tests` | 10 passed | Parent membership, nested lineage, latest context, rerouting, stale events, backend-scoped turn interruption, panel Enter/Escape priority over a reading transcript |
| `agents_navigation` | 10 passed | Existing shortcuts, editor/Vim mappings, permission gates, safe local commands on parent-owned children |
| `thread-switch-regression` Nextest selection | 20 passed | Existing picker navigation, saved drafts/queues, background approvals, widget replacement, legacy/paginated interruption lookup, transcript bookmarks |

The three bookmark tests also passed as a standalone group; they overlap the Nextest selection.
Nextest used separate processes and four workers; its final 20-case run completed in 24.376 seconds.
Test executables used isolated temporary home directories and an 8 MiB thread stack. Native user
configuration and skill directories were excluded from test discovery.

Strict TUI lint also passed: `cargo clippy --locked -p codex-tui --all-targets --no-deps -- -D warnings`.
The final CLI build passed: `cargo build --locked -p codex-cli --bin codex`.

One earlier command-help assertion expected the old argument list. It was updated to include
`focus`, then all 15 panel/command cases passed. This increment has not replayed the entire TUI
suite; the prior broad-suite result and its snapshot-runner limitation remain documented in
[the original evaluation record](subagent-panel-evaluation.md).

The reproducible evaluator now includes `transcript-bookmark-regression` and
`thread-switch-regression`, alongside the existing panel, state, navigation, lint, and build checks.
Use `CODEX_AGENT_UI_EVAL_ONLY` to select those checks when rerunning a focused evaluation.

## Lifecycle boundary

`a` requests a cancel-first confirmation before interrupting the selected child's current turn.
The event handler verifies parent membership and backend ownership again at confirmation time.
The parent and unrelated sessions are excluded. State changes come from backend notifications,
not optimistic local completion markers.

This does not implement agent-lifetime termination. A dedicated backend operation is still needed
to expose that safely without substituting archive/delete or subscription release. See
[the user and implementation contract](subagent-panel.md).

## Interactive observations

The isolated Herdr session exercised F6 with an unsent draft, distinct selection/view markers,
opening the existing child transcript, returning to the exact parent draft, an active native
child shell wait, cancel-first interruption, and retained history after interruption. The same
child then accepted a new native follow-up and completed normally, with the parent receiving its
result. Its reported model and latest context remained visible throughout.

A separate unfocused workspace ran the final executable with two new native parent-managed
children. Their input-disabled banner was explicit, the parent draft survived a child visit,
and a reading position at inert fixture line 033 survived both panel Escape and a round trip
through a child transcript. Both children completed while the parent was reading; it retained
the same anchor and displayed New activity rather than following the new output automatically.

Turn interruption deliberately retains Codex's native semantics: an existing background terminal
may continue, and the core's `is_final` excludes `AgentStatus::Interrupted`. Consequently a parent
native wait does not complete just because the child's turn was interrupted. The test parent wait
was explicitly interrupted before the authorized new child continuation. This is not agent-lifetime
termination. The bounded test sleep finished and left no shell descendants in the test CLI process
tree.

The final executable also passed the confirmation path against the parent-managed child from
the parent panel: Cancel preserved the running work, and explicitly selecting Stop produced an
authoritative interrupted state. The evaluator then interrupted its own parent wait and terminated
only the owned test sleeper. That process cleanup was evaluation cleanup, not an effect claimed
for the turn-interruption button. Both fixture workspaces remained byte-identical.

## Evidence and limits

[The machine-readable record](evidence/subagent-panel-navigation.json) includes source-file hashes,
the executable hash, runtime-companion provenance, successful build/lint/test commands, and live
acceptance outcomes. The final Herdr workspace is `codex-ui-final-eval`, left idle on Main with
the panel expanded and no unsent draft or test shell process. Linux/Windows interactive checks
and a new full-suite replay remain unrun. Agent-lifetime termination remains a backend follow-up.
