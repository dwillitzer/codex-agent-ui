# Visual walkthrough and unload regression

The walkthrough started from `f009f81a116d3ce8d65b86c82bc6510d6d4d10a8` on Intel macOS.
The main agent received a read-only fixture task through Herdr; children used native
collaboration tools. Actual Warp window captures were inspected, in addition to terminal text.
The window was 1504 by 782 pixels. Full-window images remain local because the surrounding
application includes unrelated workspace names.

## First pass

In `codex-ui-contrast-review`, a forked-context source reviewer and a fresh-context backlog
reviewer used sleeps before reading their assigned fixtures. The resumed parent initially hit
the four-thread limit because older children were still loaded. Both new children eventually
ran and completed sequentially. This was not a clean simultaneous-dispatch demonstration.

Visual inspection covered:

- The highlighted status tab, with running counts and bounded assignment/model/context rows.
- Opening the running source child and observing its forked-thread banner.
- Keeping the solid viewed-session circle independent of the hollow keyboard-selected Main row.
- Returning to Main with the filter retained.
- The stop-turn confirmation defaulting to Cancel; cancellation left the child running.
- Done showing the completed backlog child and Failed showing an explicit empty result with Main.

## Issue found

After inspecting the completed source child and returning to Main, a `NotLoaded` thread-status
notification replaced its known `done` outcome with `unloaded`. The Done count dropped even
though the client had already observed successful completion.

The fix treats unload as a residency update, preserving an observed completed, failed, or
interrupted outcome. An actual new turn still replaces the old outcome. Unknown children are
not inferred successful merely because they unload; reconnect/lost-event handling is unchanged.
The regression exercises all three terminal outcomes, new-turn replacement, filter counts, and
an unloaded child without a known outcome.

The [status-filter specification](subagent-panel-status-filter-spec.md) includes this rule.

## Fresh recorded pass after the fix

The new `codex-ui-fresh-walkthrough` workspace (`w3D:p1`) used separate temporary HOME,
CODEX_HOME, and two fixture files. It started with no previous agent history. Exactly two
native children ran concurrently: `source_review` forked the parent context and
`backlog_review` started fresh. Both completed normally with the expected fixture findings.

The [90-second MP4](evidence/subagent-panel-walkthrough.mp4) is a captioned edit of an actual
240-second window recording. The [WebVTT transcript](evidence/subagent-panel-walkthrough.vtt)
provides chapter captions. Editing trims idle waits, crops the surrounding application, and
adds a caption band. A small lower-right area masks unrelated Herdr completion notifications;
the agent panel rows and interactions are unchanged. No UI states are simulated.

| Recorded interaction                   | Observed result                                                        |
| -------------------------------------- | ---------------------------------------------------------------------- |
| Native dispatch                        | Both children running in the same parent, with assignment labels       |
| F6, F, arrows                          | Active tab highlighted; row selection independent of the open session  |
| Enter on source                        | Existing running transcript opens with its forked-thread banner        |
| Highlight Main while viewing child     | Main has `>○`; viewed child retains `●`                                |
| Inspect backlog                        | Fresh-context transcript opens; parent-managed input remains disabled  |
| Leave unsent draft, inspect, return    | Draft and Active filter restored without submission                    |
| Done, inspect completed source, return | Source remains done and Done count stays 1 after unload                |
| Failed                                 | Explicit empty result; Main remains selectable                         |
| A on running backlog                   | Stop-turn dialog defaults to Cancel; no stop decision is sent          |
| Both children finish                   | All shows two done children, model/context, assignments, frozen timing |

The stop dialog remained open while backlog completed; it was then cancelled. The video
demonstrates the confirmation UI, not an actual interruption. The earlier first pass cancelled
the dialog while its child was still running. Live approvals, a failing child, pause, and
agent-lifetime termination are not demonstrated in this recording.

Both fixture hashes remained unchanged. Source reported that beta's zero retry count violates
the positive-retries requirement; backlog identified the P1 retry-count repair. No additional
model task was submitted for the unsent draft.

## Checks and submission material

- 76 distinct selected TUI tests passed under isolated Nextest processes: 56 panel, filter,
  approval, navigation, and rendering checks plus 20 existing thread-switch regressions.
  The selection includes the new unload regression.
- Strict all-target TUI Clippy passed with `--no-deps -- -D warnings`.
- The CLI build passed. The local signed companion-host prerequisite remains as documented
  in the earlier live evaluation; it is not distributed in this repository.
- This increment does not claim a new full-suite run or Linux/Windows UI validation.

The [machine-readable evidence](evidence/subagent-panel-visual-walkthrough.json) records
source/executable/media hashes, commands and exits, fixture hashes, and the recording edits.
The [upstream follow-up](subagent-panel-upstream-report.md) was
[published on the existing request](https://github.com/openai/codex/issues/43148#issuecomment-6961263780).
The [publication receipt](evidence/subagent-panel-upstream-publication.json) records the verified
comment and video attachment; the evaluation manifest above remains the pre-publication snapshot.
For an inline GitHub player, attach the MP4 in the issue editor; the repository file is the
durable download/reference copy.
