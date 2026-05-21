//! `hm version` — print version info, including the plugin API version
//! and the count of loaded plugins.

use anyhow::Result;
use hm_plugin_protocol::HM_PLUGIN_API_VERSION;

use crate::plugin::{PluginRegistry, RegistryConfig};

// User-facing output. This is the singular purpose of this fn.
#[allow(clippy::print_stdout)]
// `async` is required by the dispatcher signature in `commands::dispatch`.
#[allow(clippy::unused_async)]
/// Run the `hm version` subcommand.
///
/// # Errors
///
/// Returns an error if the plugin registry fails to load (e.g. an
/// invalid manifest on disk).
pub async fn run() -> Result<()> {
    let reg = PluginRegistry::load(RegistryConfig {
        auto_discover: true,
        ..Default::default()
    })?;
    println!("hm {}", env!("CARGO_PKG_VERSION"));
    println!("plugin api version: {HM_PLUGIN_API_VERSION}");
    let count = reg.manifests().count();
    println!("plugins loaded: {count}");
    Ok(())
}
