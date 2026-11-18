# Development-build model catalog identity

## Root cause measured on 2026-11-18

The custom CLI was built with the workspace placeholder `0.0.0`. That version is normalized
by `codex_models_manager::client_version_to_whole()` and sent by model-provider to the
authenticated Codex model catalog endpoint. The TUI validates `/model <name>` against the
resulting catalog before it attempts a model request.

An A/B request used the **same demo subscription credentials, endpoint, originator, and
user agent**, changing only the `client_version` query parameter:

| Client version query | HTTP status | Returned models | GPT-6.1 Sol | GPT-6.2 Sol/Astra |
| -------------------- | ----------- | --------------- | ----------- | ----------------- |
| `0.0.0`              | 200         | 6               | Absent      | Absent            |
| `0.170.0`            | 200         | 12              | Present     | Present           |

The six-model response included Astra and the legacy GPT-5.6 family. Both the normal-client
cache and the custom-client cache had been freshly fetched that morning, so an old fetch
timestamp did not explain the difference. Astra completed a live test in the original custom
session. `/model gpt-6.1-sol` was rejected locally as unavailable without a model turn.

This evidence establishes client-version-dependent discovery. It does not establish that
the subscription denied GPT-6.1, nor that merely selecting a catalog entry guarantees a
successful backend response.

## Fix

The Rust workspace is stamped **0.170.0-agent-ui.1**, rather than leaving its source-build
placeholder in the executable. The suffix identifies a development fork, not the official
release. Existing normalization sends `0.170.0` for catalog/cache compatibility.
The lockfile change consists solely of the corresponding local workspace version substitutions.

No model whitelist, credential, entitlement check, provider URL, or model-name validation is
removed. No normal-client cache is copied into the test client. Verification uses an empty-cache
temporary HOME/CODEX_HOME and a dedicated Herdr pane. Future model compatibility must be checked
when updating the fork; this version is not a promise of automatic access to every future model.

Historical recordings and manifests retain their original `0.0.0` identity and hashes.
See [architecture](ARCHITECTURE.md), [data flow](DATAFLOW.md), [API scope](API-SURFACE.md),
[build instructions](DEVELOPER.md), and [security boundaries](SECURITY.md).

## Verification

The rebuilt executable reports `codex-cli 0.170.0-agent-ui.1`. A fresh isolated Herdr workspace,
**codex-ui-model-compatibility** (`w3E:p1`), started without a model cache and with ChatGPT
authentication. API-key environment overrides were removed for that process. Its own normal
refresh wrote a catalog with client version `0.170.0` and twelve model entries.

`/model` displayed GPT-6.1 Sol/Terra and GPT-6.2 Sol/Astra. Direct `/model gpt-6.1-sol` and
`/model gpt-6.2-sol` selections succeeded. Each model executed one `/usr/bin/printf` command
and returned its exact expected marker: `MODEL_61_TOOL_OK` and `MODEL_62_TOOL_OK`.
Session records confirm the requested model on each turn and the assistant's final markers.
The workspace remained empty. The test pane is left on GPT-6.2 Sol for this session only.

Checks passed: 138 model-provider tests, 60 models-manager tests, 13 TUI model-selection tests,
the initial compile check, and the locked CLI build. This is 211 selected tests, not a new
full-workspace run. Live checks are compatibility smoke tests, not comprehensive validation
of every new-model capability. The signed companion-host prerequisite remains unchanged.

See the [machine-readable evidence](evidence/model-catalog-build-identity.json). Older running
custom CLI processes retain their compiled version and must restart to use the new build.
