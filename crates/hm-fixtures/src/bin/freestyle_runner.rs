//! Fixture: registers as `runner: "freestyle"` and records the step
//! key it was invoked with into `Plugin`-scoped KV under
//! `freestyle_called_with`. The host-side test asserts this KV value
//! to prove that a step declaring `runner: "freestyle"` actually
//! lands here (and not on the docker default) — the regression guard
//! for PR #22's runner-field-drop bug.

#![no_main]
// Test fixtures: relax the workspace's pedantic/nursery lints so the
// manifest construction (`"...".into()`, `vec![...]`) reads cleanly.
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::multiple_crate_versions,
    clippy::cargo_common_metadata,
    clippy::missing_errors_doc
)]

use hm_plugin_sdk::*;

#[derive(Default)]
struct Freestyle;

impl StepExecutor for Freestyle {
    fn run(&self, input: ExecutorInput) -> Result<StepResult, PluginError> {
        // Persistent (disk-backed) Plugin-scope KV: the host-side test
        // can read this back from `<XDG_CONFIG_HOME>/harmont/state/
        // harmont-fixture-freestyle.kv`. Build-scope KV is in-memory
        // and not host-accessible from tests.
        host::kv_set(
            KvScope::Plugin,
            "freestyle_called_with",
            input.step.key.as_bytes(),
        );
        Ok(StepResult {
            exit_code: 0,
            committed_snapshot: None,
            artifacts: vec![],
        })
    }
}

register_plugin!(
    manifest = PluginManifest {
        api_version: HM_PLUGIN_API_VERSION,
        name: "harmont-fixture-freestyle".into(),
        version: semver::Version::new(0, 1, 0),
        description: "Test fixture: records step key under runner=freestyle.".into(),
        capabilities: vec![Capability::StepExecutor(StepExecutorSpec {
            runner: "freestyle".into(),
            default: false,
            step_schema: None,
        })],
        required_host_fns: vec!["hm_kv_set".into()],
        config_schema: None,
        allowed_hosts: vec![],
    },
    executor = Freestyle,
);
