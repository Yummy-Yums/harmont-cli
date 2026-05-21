//! Calls every host fn the spec defines (section 3.3) and reports back
//! what happened. Used by `tests/plugin_host_fns.rs` to assert each host
//! fn is wired up and produces the expected behaviour.

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
use serde::Serialize;
use serde_json::json;

#[derive(Default, Serialize)]
struct Report {
    log_ok: bool,
    kv_round_trip: bool,
    kv_isolated_per_scope: bool,
    fs_read_returns_none_for_missing: bool,
    keyring_round_trip: bool,
    should_cancel_default_false: bool,
}

#[derive(Default)]
struct Probe;

impl SubcommandPlugin for Probe {
    fn run(&self, _input: SubcommandInput) -> Result<ExitInfo, PluginError> {
        let mut r = Report::default();

        host::log(Level::Info, "probe: log");
        r.log_ok = true;

        host::kv_set(KvScope::Plugin, "k", b"v1");
        let v = host::kv_get(KvScope::Plugin, "k").unwrap_or_default();
        r.kv_round_trip = v == b"v1";

        host::kv_set(KvScope::Build, "k", b"v2");
        let p = host::kv_get(KvScope::Plugin, "k").unwrap_or_default();
        let b = host::kv_get(KvScope::Build, "k").unwrap_or_default();
        r.kv_isolated_per_scope = p == b"v1" && b == b"v2";

        r.fs_read_returns_none_for_missing = host::fs_read_config("does/not/exist").is_none();

        // Keyring uses a probe-scoped service+account so we don't
        // collide with the user's real secrets.
        host::keyring_set("harmont-probe", "test", "secret");
        r.keyring_round_trip =
            host::keyring_get("harmont-probe", "test").as_deref() == Some("secret");
        host::keyring_delete("harmont-probe", "test");

        r.should_cancel_default_false = !host::should_cancel();

        let json =
            serde_json::to_string(&r).map_err(|e| PluginError::new("serde", e.to_string()))?;
        Ok(ExitInfo {
            exit_code: 0,
            message: Some(json),
        })
    }
}

register_plugin!(
    manifest = PluginManifest {
        api_version: HM_PLUGIN_API_VERSION,
        name: "harmont-fixture-probe".into(),
        version: semver::Version::new(0, 1, 0),
        description: "Test fixture: exercises every host fn.".into(),
        capabilities: vec![Capability::Subcommand(SubcommandSpec {
            verb: "fixture-probe".into(),
            about: "Probe host-fn surface".into(),
            args_schema: json!({"args": []}),
            subcommands: vec![],
        })],
        required_host_fns: vec![
            "hm_log".into(),
            "hm_kv_get".into(),
            "hm_kv_set".into(),
            "hm_fs_read_config".into(),
            "hm_keyring_get".into(),
            "hm_keyring_set".into(),
            "hm_keyring_delete".into(),
            "hm_should_cancel".into(),
        ],
        config_schema: None,
        allowed_hosts: vec![],
    },
    subcommand = Probe,
);
