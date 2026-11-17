# Direct navigation and assignment summaries

This increment makes the one-gesture Option/Alt + Left/Right switch visible in the panel and
limits its known-family traversal to the same parent group. The composer retains word motion
while a draft is present. F6 remains an explicit list-selection alternative.

Each agent's main row now ends with a bounded one-line assignment. Model and context occupy the
second line. The full opening prompt is not used as the assignment text.

## Metadata contract

The parent reports a short title with `agent_ui.set_assignment(target, summary)` after dispatch
and follow-up. The companion tool validates the target through the calling agent tree and reads
metadata without sending input or resuming a task. The summary is normalized to one line and at
most 80 Unicode characters. Native spawn guidance tells the parent to keep this label current.

The existing native collaboration signatures remain unchanged. A live API check caught that
the model reserves those signatures and rejects extra parameters. The companion namespace avoids
that constraint. Regression assertions prevent adding `task_summary` to the reserved spawn and
follow-up schemas.

An optional `task_summary` on persisted core activities and `taskSummary` on the app-server item
carries the label to the UI and through resume. Old records deserialize without it and serialize
without an extra null field. Live updates supersede saved titles; older history pages cannot
overwrite newer reported assignments. Historical activities alone do not establish membership,
because a fork can retain another parent's history. Existing lineage and ancestor-filtered
discovery remain the membership authority.

Older records use a saved thread title when available, otherwise `Assignment unavailable`.
Metadata updates can fail safely when the runtime cannot resolve an unloaded target; existing
saved captions remain visible without waking the child.

## Automated validation

- 388 protocol unit tests passed.
- 326 app-server protocol/schema tests passed; the schema-writing test is ignored by that normal
  run and was separately executed for both stable and experimental exports.
- Stable and experimental exports and the Python SDK model were regenerated successfully.
- 10 Python SDK contract tests passed, including generated-artifact consistency.
- 149 selected core agent/tool-factory tests passed under isolated Nextest processes. Nextest
  marked `build_agent_resume_config_clears_base_instructions` as leaky during the passing run;
  this process-output-handle warning is retained in the evidence rather than hidden.
- 38 distinct UI checks passed: 15 panel/command, 13 app state/assignment, and 10 existing
  navigation tests. A separate 3-test assignment filter repeated cases in the app group.
- The revised CLI built successfully.
- 20 existing thread-switch regressions passed under isolated Nextest processes (45.884 seconds).
- Strict TUI Clippy passed with warnings denied. Core Clippy passed with the base checkout's
  existing `collapsible_if` warning category allowed; that exception is explicit in the command.
- The final executable build and version check passed. The selected groups total 931 distinct
  passing tests, excluding repeated assignment-filter cases and the separate schema-writer runs.

## Interactive validation

- The live parent showed `Audit source retries` and `Prioritize retry backlog` on the respective
  rows, with configured model and latest context below.
- Alt+Right opened a running native parent-managed child directly; Alt+Left returned to Main.
  Neither action required F6, list selection, or Enter.
- A follow-up on the same source-review child replaced its label with `Count retry entries`.
- A fresh final-binary session received an ordinary one-child review request without mentioning
  the summary helper. The parent automatically reported a concise assignment using its built-in
  guidance. With a draft present, Alt+Right retained the draft and stayed on Main; the test draft
  was then cleared without submission.
- After process exit and resume, both captions persisted while the children correctly showed
  closed and context unavailable. A metadata-only request that could not resolve the unloaded
  target stopped safely, without starting work.

## Evidence and limits

[The machine-readable evidence](evidence/subagent-panel-assignments.json) records source and
binary hashes, commands, test counts, compatibility checks, and live acceptance results.
The isolated `codex-ui-summary-eval` Herdr workspace is left idle on Main with no unsent draft.
Fixture files remained byte-identical. It uses a `/tmp` home and access-token-only fixture
authentication; installed Codex configuration remains untouched.

The full core/TUI suites and Linux/Windows interactive matrix were not rerun for this increment.
The compatible signed local runtime companion is still needed for the Intel live run; the pinned
V8 source-host artifact remains unavailable as documented in the original evaluation.
Agent-lifetime pause/kill controls are deferred; the existing confirmed turn interruption is
unchanged. Historical navigation evidence reflects its earlier implementation commit.
