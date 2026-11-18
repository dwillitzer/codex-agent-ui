# Agent UI fork interface scope

Version: **0.170.0-agent-ui.1** | Updated: 2026-11-18

## Public behavior

The CLI entry point remains `codex`. This version changes build identity, not command syntax.
`/model` opens the available-model picker; `/model <model>` validates against that same
catalog. Selection is subject to the authenticated backend's model access rules.

The existing `codex_models_manager::client_version_to_whole() -> String` normalizes the
workspace's prerelease version to `0.170.0` for catalog requests and cache eligibility.
No new configuration key or authentication override is introduced.

## Fork interfaces

- `/subagents [compact|expand|focus]` controls the parent-family panel.
- F6, arrows, Enter, Escape, and focused `f` provide selection, inspection, and filters.
- `agent_ui.set_assignment` supplies bounded assignment metadata without sending child input.
- Optional assignment metadata extends retained activity records; existing records remain readable.

See [the behavior contract](subagent-panel.md) for model/context semantics, approval awareness,
and interruption limits. Native collaboration parameter signatures remain unchanged.
The upstream app-server protocol documentation and generated schemas remain authoritative
for the full server API; this document does not redefine unrelated workspace interfaces.

## Version history

0.170.0-agent-ui.1 introduces no new public types, tools, traits, or endpoints. It corrects
the version advertised by the existing build and catalog interfaces.
