# Agent-bar status filters

## User interaction

The focused agent bar has four status tabs, in order: **All, Active, Done, Failed**.
Press `f` (or `F`) once to advance, wrapping from Failed to All. Holding the key does not
advance repeatedly. Up/Down move through the filtered rows and Enter watches the selected
thread. Escape returns focus to the composer without interrupting a task. Outside panel focus,
`f` remains ordinary text input. Existing F6/empty-prompt Down focus and direct Option/Alt
Left/Right shortcuts remain available.

The list shrinks to matching child agents. Main remains selectable independently of the filter
and is excluded from tab counts. Empty filters show an explicit no-matches message and still
allow returning to Main or cycling to another tab. Selection uses stable thread IDs: retain a
matching selected row; otherwise select the next eligible row in spawn order, wrapping as
necessary. A status update never changes the viewed transcript or input target automatically.

The selected tab survives inspection and return within the same parent session. A different
parent session starts on All. Compact mode retains the selected filter and its label; focusing
expands the filtered list. Direct Option/Alt cycling retains its existing full-family traversal;
the filter controls the panel's Up/Down/Enter list.

## Status contract

| Tab    | Included children                                        |
| ------ | -------------------------------------------------------- |
| All    | Every discovered child in the current parent family      |
| Active | Reported running, starting, waiting, or pending approval |
| Done   | Explicit successful completion (`done`)                  |
| Failed | Explicit execution/system error (`error`)                |

Interrupted, idle, unloaded, closed, unavailable, and unknown/reconnecting states remain under
All unless a pending approval places the child under Active. They are not guessed to be
successful or failed. Elapsed time alone never implies stuck or failed. Failure evidence is
not replaced by a generic parent completion notice; an observed new turn may replace it.

Tab counts refer to children, not Main, and do not change the backing membership cache.
Hidden agents continue receiving lifecycle and metadata updates. All aggregate activity counts
remain visible independently of the selected filter.

## Approvals

A known pending child approval remains **Active**, with **Waiting for approval** on its row.
The parent sees an approval count/notice even when another filter hides that row, including
compact mode. Selecting Failed must never hide awareness of a required human decision.
The notice counts affected child agents; a child may have more than one pending request.

Existing request routing, pending-request storage, and approval UI remain authoritative. A
request is surfaced in the parent session; this does not grant the orchestration model new
permission to approve it. The user or configured approval policy decides. Resolving or
cancelling the request removes the indicator through the existing request lifecycle.

## Acceptance checks

- Four-tab cycle, wrap, and held-key behavior.
- Mixed-status membership and counts; interrupted/unknown are not Failed or Done.
- Approvals remain Active and visible in the parent across every filter and compact mode.
- Filtered arrow/Enter navigation, empty state/Main return, stable selection on updates.
- Filter retention across thread inspection and reset for a different parent.
- No filtering keystrokes reach model input; ordinary composer typing and modal priority hold.
- Existing draft, scroll bookmark, assignment, model/context, and turn-control regressions pass.

Pause/kill controls and automatic orchestration decisions are outside this increment.
