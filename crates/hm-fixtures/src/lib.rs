// Fixtures are pure binaries; nothing lives in the lib.
//
// schemars 0.8 pulls older indexmap and wit-bindgen via its transitive tree.
// Match the crate-level allows used in the sibling protocol/sdk crates so
// the workspace's `cargo` lint group doesn't drown out real issues.
#![allow(clippy::multiple_crate_versions, clippy::cargo_common_metadata)]
