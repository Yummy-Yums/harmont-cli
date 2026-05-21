//! Minimal step-executor plugin. Records every `ExecutorInput` it
//! receives into a `Plugin`-scoped KV slot so tests can inspect it
//! after invocation.

#![no_main]
// Test fixtures: relax the workspace's pedantic/nursery lints so the
// manifest construction (`"...".into()`, `vec![...]`) and one-shot
// `serde_json::to_vec` reads cleanly.
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
struct NoopExec;

impl StepExecutor for NoopExec {
    fn run(&self, input: ExecutorInput) -> Result<StepResult, PluginError> {
        let key = format!("seen:{}", input.step.key);
        let val =
            serde_json::to_vec(&input).map_err(|e| PluginError::new("serde", e.to_string()))?;
        host::kv_set(KvScope::Plugin, &key, &val);
        host::log(Level::Info, &format!("noop ran step '{}'", input.step.key));
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
        name: "harmont-fixture-noop".into(),
        version: semver::Version::new(0, 1, 0),
        description: "Test fixture: records ExecutorInput, returns 0.".into(),
        capabilities: vec![Capability::StepExecutor(StepExecutorSpec {
            runner: "noop".into(),
            default: false,
            step_schema: None,
        })],
        required_host_fns: vec!["hm_log".into(), "hm_kv_set".into()],
        config_schema: None,
        allowed_hosts: vec![],
    },
    executor = NoopExec,
);
