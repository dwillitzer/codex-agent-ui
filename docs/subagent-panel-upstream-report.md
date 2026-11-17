# Published follow-up for openai/codex #43148

Posted as **dwillitzer** on 2026-11-17: [view the published comment](https://github.com/openai/codex/issues/43148#issuecomment-6961263780).
The inline video was verified in the browser, including playback.
See the [publication receipt](evidence/subagent-panel-upstream-publication.json) for verification details.

---

Building on this request and the earlier summary-row proposal, we explored a bounded interactive
agent panel in a Codex CLI fork. The use case is returning to a long-running parent session and
immediately seeing which children are working, what they were assigned, and which transcript is open.

I also saw the completed navigation-discoverability work in [#52853](https://github.com/openai/codex/issues/52853#issuecomment-6110703633), which exposes the existing shortcuts in `?` help. This proposal builds on that with persistent parent-scoped activity, filters, and inline child inspection.

### Recorded fresh-session walkthrough

https://github.com/user-attachments/assets/4772121c-c039-4dbf-9260-47a68f8332aa

**[Recorded walkthrough](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/evidence/subagent-panel-walkthrough.mp4)** ·
[Caption transcript](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/evidence/subagent-panel-walkthrough.vtt) ·
[Implementation branch](https://github.com/dwillitzer/codex-agent-ui/tree/feat/parent-agent-panel) ·
[Recorded source commit](https://github.com/dwillitzer/codex-agent-ui/commit/7a455536e48e8a7170133feeea52e2f52a9d99c7) ·
[Behavior and data contract](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/subagent-panel.md)

The recording is an actual fresh session with two native children: one forks parent context and
one starts fresh. Idle waits are trimmed, the surrounding application is cropped out, and captions
explain the keystrokes. Unrelated host notifications are masked in a small lower-right area.
It shows dispatch, inspection, draft restoration, status filters, a stop-turn
confirmation, and normal completion. It does not demonstrate pause or agent-lifetime termination.

The panel provides:

- A persistent active count and bounded child preview, scoped to the current parent family.
- Agent names, short current assignments, state, available elapsed time, configured model, and
  latest reported context occupancy. Compact mode retains a summary.
- A solid circle for the open transcript, hollow circles for other rows, and an independent
  navigation cursor. Open/highlighted rows are bold; other rows are dim.
- F6 (or Down from an empty prompt) to focus, arrows to select, Enter to inspect, and Escape to
  return to input. Option/Alt Left/Right switches directly from an empty prompt; draft word editing
  retains those keys. Main remains available for returning to the parent.
- All/Active/Done/Failed filters, cycled with `f`, with a clear active-tab highlight. Filters survive
  child inspection. Empty results retain Main, and filtering never changes the viewed transcript.
- Pending-approval awareness in the parent even when a filter hides the child. Approval remains
  an Active waiting state and follows existing user/policy decisions. Errors are Failed;
  interrupted, idle, and unknown states are not inferred successful or failed.

This uses existing thread navigation, lineage discovery, and lifecycle notifications; no tmux,
new server RPC, or periodic backend polling is required. Native collaboration signatures remain
unchanged. A separate metadata helper supplies the parent's bounded assignment caption without
waking the child or sending it input. Context uses latest-response occupancy rather than cumulative
session tokens. Missing data stays explicit, and the configured model is not proof of which provider
served a request.

One useful finding from walking the UI: leaving a completed child's transcript could release its
subscription and replace `done` with `unloaded`, removing it from Done. The fork now preserves an
observed terminal outcome across that residency update. A new observed turn still replaces the old
outcome; unknown unloaded children are not guessed completed. The fresh recording rechecks this path.

Validation on Intel macOS:

- The latest increment passed **76 distinct selected TUI tests** under Nextest process isolation:
  56 panel/lifecycle cases and 20 thread-switch regressions, including outcome preservation,
  filters, approvals, keyboard routing, rendering, and draft behavior.
- Strict all-target TUI Clippy and the CLI build passed. The recorded fresh run completed both
  read-only fixture reviews with fixture hashes unchanged.
- Earlier core/protocol/schema and SDK results are separate historical records,
  linked from the [evaluation index](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/subagent-panel.md#evaluation-and-upstream-contribution).
  A new full-suite or Linux/Windows UI run is not claimed.
- The local CLI still needs a compatible signed companion host from the installed ChatGPT app;
  the pinned Intel V8 source-build artifact was unavailable. The companion is not distributed.
- The stop action interrupts a turn after confirmation; background terminals can remain running.
  This is not a pause/kill/lifetime-close API. Live approval prompts and a failing child are not
  shown in the video; app-server fixtures cover those state transitions.

The [visual walkthrough notes](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/subagent-panel-visual-walkthrough.md),
[filter specification](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/subagent-panel-status-filter-spec.md), and [machine-readable evidence](https://github.com/dwillitzer/codex-agent-ui/blob/feat/parent-agent-panel/docs/evidence/subagent-panel-visual-walkthrough.json) record
the implementation observations and evaluation limits. The request is to make delegated work
persistently visible and inspectable from its parent, with this fork serving as a tested reference.
