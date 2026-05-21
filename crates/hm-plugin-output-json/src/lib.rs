//! Built-in JSON-lines output formatter.
//!
//! Each `BuildEvent` is serialised to JSON on a single line and
//! written to stdout. Stderr is reserved for plugin/host diagnostics.

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

use hm_plugin_sdk::*;

#[derive(Default)]
struct Json;

impl OutputFormatter for Json {
    fn on_event(&self, event: BuildEvent) -> Result<(), PluginError> {
        let mut bytes = serde_json::to_vec(&event)
            .map_err(|e| PluginError::new("output_json_serde", e.to_string()))?;
        bytes.push(b'\n');
        host::write_stdout(&bytes);
        Ok(())
    }
}

register_plugin!(
    manifest = PluginManifest {
        api_version: HM_PLUGIN_API_VERSION,
        name: "harmont-output-json".into(),
        version: semver::Version::new(0, 1, 0),
        description: "JSON-lines build output formatter.".into(),
        capabilities: vec![Capability::OutputFormatter(OutputFormatterSpec {
            name: "json".into(),
            mime: "application/x-ndjson".into(),
        })],
        required_host_fns: vec!["hm_write_stdout".into()],
        config_schema: None,
        allowed_hosts: vec![],
    },
    output = Json,
);
