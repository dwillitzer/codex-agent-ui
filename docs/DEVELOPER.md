# Agent UI fork developer guide

Version: **0.170.0-agent-ui.1** | Updated: 2026-11-18

## Build and test

Use the Rust toolchain pinned by `codex-rs/rust-toolchain.toml` and the workspace lockfile.

```sh
cd codex-rs
cargo test --locked -p codex-models-manager -p codex-model-provider --lib
cargo build --locked -p codex-cli --bin codex
target/debug/codex --version
```

The executable should identify the fork as `0.170.0-agent-ui.1`. Do not install it over the
normal Codex binary for evaluation. Use a separate temporary HOME and CODEX_HOME, and run
`target/debug/codex --no-daemon` so an older shared server cannot supply the catalog.

The pinned companion code-mode host must be available. The earlier Intel demonstration used
the compatible signed host from the installed ChatGPT application because its pinned V8
source-build artifact was unavailable; that host is not redistributed by this fork.

## Model troubleshooting

The `0.0.0` workspace placeholder is unsuitable for an account-backed catalog evaluation:
the server uses the normalized client-version query to choose compatible catalog entries.
The meaningful fork version makes normal refresh and cache invalidation work together.
Do not solve a missing model by removing name validation, editing credentials, or borrowing
another account's model cache. Compare catalog responses and verify an actual model turn.

This does not guarantee every offered model supports every feature or service tier. Report
selection errors separately from backend request errors. Keep original demo/evaluation records
at their original version and hashes; add a new record for a newly built executable.

The [model-catalog evaluation](model-catalog-build-identity.md) records the 211 passing selected
tests and real subscription-backed GPT-6.1 Sol/GPT-6.2 Sol tool calls for this build.

## References and version history

[Architecture](ARCHITECTURE.md), [data flow](DATAFLOW.md), [API scope](API-SURFACE.md), and
[security](SECURITY.md) describe this increment. Version 0.170.0-agent-ui.1 corrects development
build identity. Earlier 0.0.0 recordings remain historical evidence of those earlier builds.
