use hm_plugin_protocol::{HookEvent, HookOutcome, PluginError};

/// Implemented by lifecycle-hook plugins.
pub trait LifecycleHook {
    /// React to a lifecycle event.
    ///
    /// # Errors
    /// Returns a [`PluginError`] describing the failure. The host
    /// converts errors into build events; whether the build aborts
    /// depends on the hook's declared `phase`.
    fn on_event(&self, event: HookEvent) -> Result<HookOutcome, PluginError>;
}
