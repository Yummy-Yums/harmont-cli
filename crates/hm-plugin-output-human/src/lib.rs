//! Built-in human-readable output formatter for the hm CLI.
//!
//! Subscribes to the orchestrator's BuildEvent stream via the
//! `hm_output_on_event` capability export; writes prefixed step logs
//! and brief status lines to stderr.

#![allow(unsafe_code, reason = "extism-pdk host_fn imports require unsafe")]
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::multiple_crate_versions,
    clippy::cargo_common_metadata,
    clippy::missing_errors_doc,
    reason = "matches the test-fixtures allow-list; plugin authoring crate"
)]

mod render;

use hm_plugin_sdk::*;

#[derive(Default)]
struct Human;

impl OutputFormatter for Human {
    fn on_event(&self, event: BuildEvent) -> Result<(), PluginError> {
        let bytes = render::render(&event);
        if !bytes.is_empty() {
            host::write_stderr(&bytes);
        }
        Ok(())
    }
}

register_plugin!(
    manifest = PluginManifest {
        api_version: HM_PLUGIN_API_VERSION,
        name: "harmont-output-human".into(),
        version: semver::Version::new(0, 1, 0),
        description: "Human-readable build output formatter.".into(),
        capabilities: vec![Capability::OutputFormatter(OutputFormatterSpec {
            name: "human".into(),
            mime: "text/plain".into(),
        })],
        required_host_fns: vec!["hm_write_stderr".into()],
        config_schema: None,
        allowed_hosts: vec![],
    },
    output = Human,
);
