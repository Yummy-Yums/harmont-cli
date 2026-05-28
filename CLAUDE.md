The `cli/` directory is a Cargo workspace.

- `crates/hm/` — the `hm` binary (today's CLI body).
- `crates/hm-pipeline-ir/` — pipeline IR schema (serde structs only, no runtime).
- `crates/hm-util/` — shared OS and filesystem utilities.
- `crates/hm-plugin-protocol/` — wire types (serde structs only).
- `crates/hm-plugin-sdk/` — authoring SDK for plugin writers.
Run `cargo build` from the workspace root. Build requires esbuild
(`npm ci` in `crates/hm-dsl-engine/harmont-ts/`).

For cross-cutting doctrine see [PRINCIPLES.md](../PRINCIPLES.md).

## DSLs

Both DSLs live inside `crates/hm-dsl-engine/` so they ship with the crate:

- `crates/hm-dsl-engine/harmont-py/` — the `harmont` Python package (pipeline DSL).
- `crates/hm-dsl-engine/harmont-ts/` — the `harmont` TypeScript package (pipeline DSL).
