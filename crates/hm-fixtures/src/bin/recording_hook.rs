//! Records every `HookEvent` into a KV slot keyed by event kind.

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

#[derive(Default)]
struct RecHook;

impl LifecycleHook for RecHook {
    fn on_event(&self, event: HookEvent) -> Result<HookOutcome, PluginError> {
        let kind = match &event.event {
            BuildEvent::BuildStart { .. } => "build_start",
            BuildEvent::StepQueued { .. } => "step_queued",
            BuildEvent::StepStart { .. } => "step_start",
            BuildEvent::StepLog { .. } => "step_log",
            BuildEvent::StepCacheHit { .. } => "step_cache_hit",
            BuildEvent::StepEnd { .. } => "step_end",
            BuildEvent::BuildEnd { .. } => "build_end",
            BuildEvent::ChainFailed { .. } => "chain_failed",
        };
        let key = format!("hook:{kind}");
        let v = host::kv_get(KvScope::Plugin, &key).unwrap_or_default();
        let mut count: u64 = if v.is_empty() {
            0
        } else {
            String::from_utf8_lossy(&v).parse().unwrap_or(0)
        };
        count += 1;
        host::kv_set(KvScope::Plugin, &key, count.to_string().as_bytes());
        Ok(HookOutcome::Continue)
    }
}

register_plugin!(
    manifest = PluginManifest {
        api_version: HM_PLUGIN_API_VERSION,
        name: "harmont-fixture-rec-hook".into(),
        version: semver::Version::new(0, 1, 0),
        description: "Test fixture: counts HookEvents per kind.".into(),
        capabilities: vec![Capability::LifecycleHook(LifecycleHookSpec {
            events: vec![
                HookEventKind::BuildStart,
                HookEventKind::StepQueued,
                HookEventKind::StepStart,
                HookEventKind::StepLog,
                HookEventKind::StepCacheHit,
                HookEventKind::StepEnd,
                HookEventKind::BuildEnd,
            ],
            phase: HookPhase::After,
            timeout_ms: 5000,
        })],
        required_host_fns: vec!["hm_kv_get".into(), "hm_kv_set".into()],
        config_schema: None,
        allowed_hosts: vec![],
    },
    hook = RecHook,
);
