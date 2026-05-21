//! In-process plugin host.
//!
//! Loads `.wasm` plugins via Extism, validates their manifests, exposes the
//! host-fn surface from the design spec (see
//! `docs/superpowers/specs/2026-05-18-hm-local-first-redesign-design.md` §3.3).

pub mod embedded;
pub mod host;
pub mod host_fns;
pub mod install;
pub mod manifest;
pub mod paths;
pub mod pool;
pub mod registry;
pub mod signal;

pub use host::LoadedPlugin;
pub use registry::{PluginRegistry, RegistryConfig};
