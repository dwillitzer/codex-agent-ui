# Parent-scoped subagent panel

This fork adds a persistent agent preview above the Codex CLI composer. It uses Codex's existing
thread navigation and app-server lifecycle events; tmux is not required.

## Using the panel

The panel appears once the current primary session has child agents. It shows an active count and
up to three children, prioritizing the currently viewed child and active children. Each row shows
its name/path, state, model, context usage, and elapsed time when available. A second line keeps
model and context readable at ordinary terminal widths; wider terminals also show the task
preview when the backend supplies one. Additional children remain in `/subagents`.

For example (synthetic data):

```text
Agents: 1 active / 2 · /subagents to inspect · /subagents compact
· /root/source-review · running · 1m14s
  review-model · ctx 50% used (60K/108K) · Review source and report findings
· /root/tests · done · 0m42s
  test-model · ctx 12% used (24K/108K) · Run regression tests
```

- `/subagents`: inspect all discovered agents and select one with Enter to watch its existing
  conversation. Select `Main [default]` to return to the parent. The existing previous/next agent
  shortcuts also continue to work.
- `/subagents compact`: retain the count but collapse the preview rows.
- `/subagents expand`: restore the preview rows.

Inspection preserves the backend's existing direct-input restrictions for parent-owned children.
It does not create another agent, resume an unrelated task, or send an instruction to the child.
Completed children remain inspectable until archived. Collapsing is session-local.

## Data and lifecycle contract

Membership comes from the primary session's spawn events and the existing ancestor-filtered
thread-list request. Arbitrary loaded root sessions and attached side conversations do not become
children simply because their events are visible. Discovery is asynchronous and runs on initial
session attachment/resume; no extra periodic backend polling is introduced.

The model is the child's configured model from thread metadata/settings or spawn metadata, with
runtime model-reroute notifications applied when available. It is not independent proof of which
upstream provider actually served a request. No parent-model fallback is invented.

Context uses the child's **latest response** token usage and reported context-window size. It does
not use cumulative session tokens. The percentage follows Codex's existing baseline-adjusted
context calculation; the accompanying counts show raw reported occupancy/window size. Missing or
invalid window data is displayed as `ctx unavailable`, not zero. Compaction can reduce occupancy.
Changing models invalidates the old context measurement until fresh usage arrives.

Elapsed time is shown only when the client observed a turn start, or when a completion supplies
duration. Live elapsed time is measured locally since that observed start; it is unavailable for a
running turn discovered after reconnect until the client has timing evidence. It freezes on
completion. A one-second repaint updates visible live clocks without polling the backend.

Idle/unloaded is not completed. Completion comes from a terminal turn or explicit child-completion
event. An older turn's completion cannot finish a newer observed turn. Disconnect/lag invalidates
live presentation to `unknown (reconnecting)` and clears context until fresh authoritative state
arrives. Resetting the primary session clears panel membership. Narrow terminals clip bounded rows
and omit task previews; the panel does not wrap unbounded task text into the composer.

## Implementation

- `app/agent_panel.rs`: membership-scoped presentation cache, token formatting, notification updates.
- `app/agent_navigation.rs`: cache lifetime follows existing navigation state.
- `app/agent_picker.rs`: existing asynchronous descendant discovery also hydrates panel metadata.
- `app/session_lifecycle.rs`: picker descriptions expose child model/context/task.
- `bottom_pane/subagent_panel.rs`: bounded render-only preview; selection remains in the existing
  picker, so composer typing keys keep their original behavior.

No app-server protocol or core agent-execution behavior changes are required by this implementation.
The feature affects this fork's terminal client, not the desktop app or IDE extension.

## Evaluation and upstream contribution

See [the evaluation record](subagent-panel-evaluation.md) for commands, results, limits, and the
acceptance matrix. The upstream contribution policy currently accepts issue-based findings and
design discussions rather than external code PRs. A tested fork and implementation reference can
accompany [the existing feature request](https://github.com/openai/codex/issues/43148).
