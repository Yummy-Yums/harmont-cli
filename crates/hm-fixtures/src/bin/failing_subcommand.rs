//! A subcommand plugin that always exits non-zero. Lets the host
//! exercise `ExitInfo` plumbing.

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
use serde_json::json;

#[derive(Default)]
struct Failing;

impl SubcommandPlugin for Failing {
    fn run(&self, _input: SubcommandInput) -> Result<ExitInfo, PluginError> {
        Ok(ExitInfo {
            exit_code: 7,
            message: Some("intentional failure for tests".into()),
        })
    }
}

register_plugin!(
    manifest = PluginManifest {
        api_version: HM_PLUGIN_API_VERSION,
        name: "harmont-fixture-failing".into(),
        version: semver::Version::new(0, 1, 0),
        description: "Test fixture: always exits 7.".into(),
        capabilities: vec![Capability::Subcommand(SubcommandSpec {
            verb: "fixture-fail".into(),
            about: "Intentionally fails (test fixture)".into(),
            args_schema: json!({"args": []}),
            subcommands: vec![],
        })],
        required_host_fns: vec![],
        config_schema: None,
        allowed_hosts: vec![],
    },
    subcommand = Failing,
);
