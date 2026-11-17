# Live TUI evaluation on macOS

The feature was exercised through Herdr against the compiled fork, not inferred from tool-call
acknowledgements. The installed Codex executable and existing Herdr workspaces were left intact.

## Fixture and runtime

- Implementation commit: `4e0f53ae249aa5e2629c58793562d0f66870f66e`.
- Host: Intel macOS; compiled CLI reports `codex-cli 0.0.0` (upstream development version).
- Herdr workspace: `codex-ui-eval`, workspace `w37`, pane `w37:p1`.
- Disposable cwd: `/private/tmp/codex-agent-ui-live-0Lq7St/workspace`.
- Separate home and Codex directory; read-only sandbox; no native configuration was copied.
- Existing account access token was reused locally without its refresh capability. No credentials
  or raw authentication records are in this repository.
- Model selected from the account's actual picker: `gpt-5.6-sol`, low effort. The initially requested
  `gpt-6.1-sol` was rejected by the account and was not used for successful tool evaluation.

The CLI was launched with `--no-daemon`. Herdr process information confirmed that the foreground
process was this checkout's `target/debug/codex`, in the disposable cwd. Each submitted instruction
was verified in the pane along with working state, rather than relying only on a CLI receipt.

## Companion prerequisite and limitation

A CLI-only build was insufficient: native tools require a sibling `codex-code-mode-host`.
Building that companion from the pinned source failed because the V8 152.2.0 Intel macOS binding
artifact returned HTTP 404. This is an unresolved source-build/distribution prerequisite, not a
passing standalone-package check.

An installed official `0.156.1` companion passed signature verification but failed the transport
probe. It was not used for successful evaluation. The signed companion bundled with ChatGPT's
`codex-cli 0.169.0-alpha.11.1` advertises `grpc+stdio` and passed the actual native-tool probes:

- SHA-256: `79a93adf3878978f076702e87fa3d91af490e00b8d8ebdc07770ef975c060879`.
- Native JavaScript evaluation printed `2` for `1 + 1`.
- Native child creation, shell execution, fixture reads, follow-up input, waiting, and completion
  worked in the scenarios below.

The companion was linked into this checkout's ignored build directory; the application bundle was
not changed or redistributed. Compatibility is established for the operations exercised here,
not every host protocol feature. A self-contained Intel macOS build remains unverified.

The evaluation script supports an explicit companion override, checks its signature on macOS,
checks the advertised transport, records its digest, and refuses to overwrite a different helper:

```sh
CODEX_AGENT_UI_HOST_PROGRAM=/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex-code-mode-host \
  node scripts/evaluate-subagent-panel.mjs /tmp/codex-agent-panel-evaluation
```

Without that override, it builds the companion from source. `V8_FROM_SOURCE=1` is the upstream
build's documented alternative when a prebuilt V8 artifact is unavailable; that full build was
not performed in this evaluation.

## Observed scenarios

| Scenario                                                                          | Observed result                                                                                                                          |
| --------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Two native children, each sleeping 45 seconds before reading its assigned fixture | Parent preview showed `Agents: 2 active / 2`, running state, models, and live clocks                                                     |
| Child context before the backend reports usage                                    | Displayed `ctx unavailable`, rather than a fabricated value                                                                              |
| Usage after reporting                                                             | Child rows and picker showed `gpt-5.6-sol` and `ctx 1% used (15.3K/258K)`                                                                |
| Both children finish                                                              | Preview retained both children with done state and frozen elapsed times                                                                  |
| Open a completed child                                                            | Its original task, `/bin/sleep 45`, fixture read, finding, and current-agent marker were visible                                         |
| Reuse an existing child with a 90-second sleep                                    | Count changed to `1 active / 2`; no new child was created                                                                                |
| Open that child while it was still running                                        | Its live background-terminal wait was visible; the row was marked current and running, with `ctx 4% used (21.7K/258K)`                   |
| Return to Main                                                                    | Original parent transcript and `Main [default]` footer were restored                                                                     |
| Follow-up finishes                                                                | Child returned exactly `navigation-check-complete`; row changed to done, context reached `21.9K/258K`, and elapsed time froze at `1m35s` |
| Compact and expand                                                                | Compact retained the count; expand restored both detailed rows                                                                           |

The backend assigned the native nicknames `Locke` and `Kuhn`; their task previews identified the
requested source-review and backlog-review assignments. The implementation displays backend
names/paths, rather than inventing names from a prompt.

The fixture findings were correct: `beta` had zero retries despite a positive-retry requirement,
and repairing it was the P1 backlog item. Both input files were compared byte-for-byte with their
original fixture contents after the run and were unchanged.

The pane was left idle on Main with the expanded completed-agent list for human inspection.
Raw pane captures and runtime provenance are retained locally under ignored `.dev/` paths.

## Remaining limits

Linux/Windows UI runs and a self-contained source-built Intel companion were not tested. The user
reported a macOS system-password dialog during initial attempts; its exact wording and cause could
not be recovered, so it is not attributed to a particular component. No macOS protection was
disabled as a workaround.
