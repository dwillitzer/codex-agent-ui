# Agent UI fork interface scope

Version: **0.170.0-agent-ui.2** | Updated: 2026-11-18

## Public behavior

The CLI entry point remains `codex`. The `.2` candidate imports upstream completion,
navigation, and per-tool reviewer configuration; `.1` corrected build identity.
`/model` opens the available-model picker; `/model <model>` validates against that same
catalog. Selection is subject to the authenticated backend's model access rules.

The existing `codex_models_manager::client_version_to_whole() -> String` normalizes the
workspace's prerelease version to `0.170.0` for catalog requests and cache eligibility.
The normalized catalog version remains `0.170.0`; no authentication override is introduced.

## Imported per-tool app reviewer

```toml
[apps.calendar.tools.calendar_create_event]
approvals_reviewer = "user"
```

The tool setting precedes connected-account, app, app-default and session reviewer settings,
subject to managed requirements. Resolution matches the raw name before its title.
App-server config/read exposes the optional field. Candidate qualification is recorded separately.
This selects the reviewer when approval is needed; it does not itself enable the tool or
require every otherwise-allowed call to prompt.

## Fork interfaces

The maintenance command `node scripts/prepare-agent-ui-runtime.mjs` accepts `--target-dir PATH`,
`--cache-dir PATH`, `--replace`, and `--help`. It prepares the pinned code-mode helper without
changing the Codex configuration or model catalog. See the developer guide for qualification scope.

- `/subagents [compact|expand|focus]` controls the parent-family panel.
- F6, arrows, Enter, Escape, and focused `f` provide selection, inspection, and filters.
- `agent_ui.set_assignment` supplies bounded assignment metadata without sending child input.
- Optional assignment metadata extends retained activity records; existing records remain readable.

See [the behavior contract](subagent-panel.md) for model/context semantics, approval awareness,
and interruption limits. Native collaboration parameter signatures remain unchanged.
The upstream app-server protocol documentation and generated schemas remain authoritative
for the full server API; this document does not redefine unrelated workspace interfaces.

## Version history

0.170.0-agent-ui.2 imports upstream public configuration/schema changes. Earlier `.1`
corrected the version advertised by the existing build/catalog interfaces.
