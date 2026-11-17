# Parent-scoped subagent panel

This fork adds a persistent agent preview above the Codex CLI composer. It uses Codex's existing
thread navigation and app-server lifecycle events; tmux is not required.

## Using the panel

The panel appears once the current primary session has child agents. It shows an active count and
up to three children, prioritizing the currently viewed child and active children. Each row shows
its name/path, state, elapsed time, and a short assignment on one line. A second line keeps model
and context readable at ordinary terminal widths. Additional children remain in `/subagents`.

For example (synthetic data):

```text
Agents: 1 active / 2 · Alt+←/→ switch (empty prompt) · F6 list
 · /root/source-review · running · 1m14s — Audit retry validation
  review-model · ctx 50% used (60K/108K)
 · /root/tests · done · 0m42s — Run retry regression tests
  test-model · ctx 12% used (24K/108K)
```

- **Option/Alt + Left or Right**: switch directly between agent transcripts and Main from an
  empty prompt, without opening a list or pressing Enter. With text present, these keys retain
  their existing word-editing behavior.
- `/subagents`: inspect all discovered agents and select one with Enter to watch its existing
  conversation. Select `Main [default]` to return to the parent. The existing previous/next agent
  shortcuts also continue to work.
- `/subagents compact`: retain the count but collapse the preview rows.
- `/subagents expand`: restore the preview rows.
- F6 focuses the inline panel, including while a draft is present. Down also focuses it from an
  empty prompt. Popups, paste bursts, and explicit key remaps take precedence. `/subagents focus`
  provides a command alternative.
- While focused, Up/Down select in stable spawn order, Enter watches that thread, and Escape
  returns to the composer without interrupting work. Select Main to return to the parent. The
  `>` cursor marks selection; `●` marks the transcript currently viewed. The panel scrolls its
  bounded preview to keep the selected row visible, including when compacted.
- `a` on a child offers a cancel-first confirmation to stop its current turn. This preserves the
  agent and its history; it does not end its lifetime. The parent is excluded from this action.
  Native background terminals can remain running after turn interruption; use the existing `/stop`
  controls for those processes. Codex's native agent wait treats interruption as resumable rather
  than final, so interrupting a child does not complete a parent that is waiting for its final result.

Inspection preserves the backend's existing direct-input restrictions for parent-owned children.
It does not create another agent, resume an unrelated task, or send an instruction to the child.
Completed children remain inspectable until archived. Collapsing is session-local.
When viewing a child, a banner names the transcript and indicates where input is permitted.
Thread switching uses the existing per-thread draft state and saves transcript reading bookmarks;
returning restores the prior reading position, with unseen activity tracked by the native viewport.

## Data and lifecycle contract

Membership comes from the primary session's spawn events and the existing ancestor-filtered
thread-list request. Arbitrary loaded root sessions and attached side conversations do not become
children simply because their events are visible. Discovery is asynchronous and runs on initial
session attachment/resume; no extra periodic backend polling is introduced.

The model is the child's configured model from thread metadata/settings or spawn metadata, with
runtime model-reroute notifications applied when available. It is not independent proof of which
upstream provider actually served a request. No parent-model fallback is invented.

Assignment text comes from the parent's `agent_ui.set_assignment` metadata call after native v2
spawn and follow-up operations. It accepts the existing agent's task name or ID and a summary of
at most 80 characters. Model-reserved collaboration tool signatures remain unchanged. The producer
and UI normalize the summary to one bounded line; ordinary messages,
completion events, and older history pages do not replace the latest reported assignment.
Retained activity history restores summaries after resume. Older records without this metadata
use a saved thread title when available, otherwise `Assignment unavailable`. The panel does not
use the opening prompt as the assignment. The parent supplies the short title as part of its
ordinary workflow; updating this metadata does not wake the child or send it input.

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
The active count describes child-turn activity, not all background operating-system processes.

## Implementation

- `app/agent_panel.rs`: membership-scoped presentation cache, token formatting, notification updates.
- `app/agent_navigation.rs`: cache lifetime follows existing navigation state.
- `app/agent_picker.rs`: existing asynchronous descendant discovery also hydrates panel metadata.
- `app/session_lifecycle.rs`: picker descriptions expose child model/context/task.
- `bottom_pane/subagent_panel.rs`: bounded preview and explicit focus/selection state; IDs keep
  selection stable through live updates. The existing picker remains available for the full list.

No new server RPC or alternative agent executor is required. An optional activity metadata field
carries the assignment summary through live events and stored history; old records remain readable.
The feature affects this fork's terminal client, not the desktop app or IDE extension.

Ending an agent is a separate future backend capability. The existing app-server API exposes turn
interruption, archive/delete, and subscription release, but no dedicated agent-lifetime close
request. The UI deliberately offers only interruption until a scoped, non-destructive lifetime
operation can be implemented and tested in the backend.

## Evaluation and upstream contribution

See [the evaluation record](subagent-panel-evaluation.md) for commands, results, limits, and the
acceptance matrix, and [the live TUI evaluation](subagent-panel-live-evaluation.md) for the Herdr
run and companion-runtime prerequisite. [The inline navigation evaluation](subagent-panel-navigation-evaluation.md)
records the follow-up focus, draft, bookmark, and turn-interruption checks.
The [assignment evaluation](subagent-panel-assignment-evaluation.md) records direct cycling,
current one-line summaries, schema compatibility, and the latest tests and live checks.
The upstream contribution policy currently accepts issue-based findings and
design discussions rather than external code PRs. A tested fork and implementation reference can
accompany [the existing feature request](https://github.com/openai/codex/issues/43148).
The [prepared upstream report](subagent-panel-upstream-report.md) is ready for review; it has not
been posted to the issue tracker.
