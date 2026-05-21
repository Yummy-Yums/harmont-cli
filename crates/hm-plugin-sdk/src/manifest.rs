//! Build helpers for plugin manifests. Today this file is a re-export
//! shim; future expansion will add a `manifest! {}` declarative macro.

pub use hm_plugin_protocol::{
    Capability, ClapJson, HookEventKind, HookPhase, JsonSchema, LifecycleHookSpec,
    OutputFormatterSpec, PluginManifest, StepExecutorSpec, SubcommandSpec,
};
