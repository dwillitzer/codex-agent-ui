# Agent UI fork developer guide

Version: **0.170.0-agent-ui.2** | Updated: 2026-11-18

## Build and test

Use the Rust toolchain pinned by `codex-rs/rust-toolchain.toml` and the workspace lockfile.

```sh
cd codex-rs
cargo test --locked -p codex-models-manager -p codex-model-provider --lib
cargo build --locked -p codex-cli --bin codex
target/debug/codex --version
```

The executable should identify the fork as `0.170.0-agent-ui.2`. Do not install it over the
normal Codex binary for evaluation. Use a separate temporary HOME and CODEX_HOME, and run
`target/debug/codex --no-daemon` so an older shared server cannot supply the catalog.

The pinned companion code-mode host must be available. From the repository root, candidate
`.2` has a public runtime recipe:

```sh
node scripts/prepare-agent-ui-runtime.mjs
node --test scripts/tests/prepare-agent-ui-runtime.test.mjs
```

The script pins the official npm artifact and extracted helper hashes, caches the helper, and
places a symlink beside the debug CLI. Use `--target-dir codex-rs/target/release` for a release
build, `--cache-dir PATH` for an alternate cache, and `--replace` to replace a different symlink.
It refuses to overwrite a different regular helper file. Only Darwin x64 has a recipe at present;
candidate runtime qualification is still in progress. Voice assets are not prepared by this script.

The earlier Intel demonstration used
the compatible signed host from the installed ChatGPT application because its pinned V8
source-build artifact was unavailable; that host is not redistributed by this fork. The public
recipe is intended to remove the installed-app dependency after qualification; a standalone
source-built host is still a separate build target and has not been revalidated in this increment.

## Model troubleshooting

The `0.0.0` workspace placeholder is unsuitable for an account-backed catalog evaluation:
the server uses the normalized client-version query to choose compatible catalog entries.
The meaningful fork version makes normal refresh and cache invalidation work together.
Do not solve a missing model by removing name validation, editing credentials, or borrowing
another account's model cache. Compare catalog responses and verify an actual model turn.

This does not guarantee every offered model supports every feature or service tier. Report
selection errors separately from backend request errors. Keep original demo/evaluation records
at their original version and hashes; add a new record for a newly built executable.

The [model-catalog evaluation](model-catalog-build-identity.md) records the earlier `.1` build's
211 selected tests and subscription-backed GPT-6.1 Sol/GPT-6.2 Sol tool calls.
The `.2` candidate requires its own [qualification gates](upstream-maintenance.md).

The `scripts/subagent-panel-test-runner.mjs` Cargo runner isolates test HOME and defaults
to a 32 MiB test-thread stack for large debug navigation futures. It does not alter the
shipped runtime stack. Use the `local` nextest profile: its exact nine-scenario thread-link
case has a three-minute limit after an isolated Intel run passed in about 75 seconds.

## References and version history

[Architecture](ARCHITECTURE.md), [data flow](DATAFLOW.md), [API scope](API-SURFACE.md), and
[security](SECURITY.md) describe this increment. Version `0.170.0-agent-ui.1` corrected development
build identity; `.2` integrates the captured upstream update and adds a pinned public helper recipe.
Earlier 0.0.0 recordings remain historical evidence of those earlier builds.
