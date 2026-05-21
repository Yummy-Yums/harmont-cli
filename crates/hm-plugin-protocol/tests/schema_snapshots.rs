//! JSON Schema snapshot test. Catches any unintentional change to the
//! wire format (field rename, type swap, required-vs-optional flip).
//! Run `cargo insta accept -p hm-plugin-protocol` to refresh after an
//! intended schema change.

#![allow(
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use hm_plugin_protocol::{
    DockerCommitArgs, DockerExecArgs, DockerExtractArgs, DockerStartArgs, PluginManifest,
};
use schemars::schema_for;

#[test]
fn plugin_manifest_schema_is_stable() {
    let schema = schema_for!(PluginManifest);
    insta::assert_json_snapshot!("plugin_manifest", schema);
}

#[test]
fn docker_start_args_schema_is_stable() {
    insta::assert_json_snapshot!("docker_start_args", schema_for!(DockerStartArgs));
}

#[test]
fn docker_exec_args_schema_is_stable() {
    insta::assert_json_snapshot!("docker_exec_args", schema_for!(DockerExecArgs));
}

#[test]
fn docker_commit_args_schema_is_stable() {
    insta::assert_json_snapshot!("docker_commit_args", schema_for!(DockerCommitArgs));
}

#[test]
fn docker_extract_args_schema_is_stable() {
    insta::assert_json_snapshot!("docker_extract_args", schema_for!(DockerExtractArgs));
}
