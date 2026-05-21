//! Declares a manifest with the wrong api_version. Used to assert
//! the host rejects it at load time.

#![no_main]
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::multiple_crate_versions,
    clippy::cargo_common_metadata,
    clippy::missing_errors_doc
)]

use hm_plugin_sdk::*;

register_plugin!(
    manifest = PluginManifest {
        api_version: 9999,
        name: "harmont-fixture-bad-api".into(),
        version: semver::Version::new(0, 1, 0),
        description: "always fails to load".into(),
        capabilities: vec![Capability::StepExecutor(StepExecutorSpec {
            runner: "bad".into(),
            default: false,
            step_schema: None,
        })],
        required_host_fns: vec![],
        config_schema: None,
        allowed_hosts: vec![],
    },
);
