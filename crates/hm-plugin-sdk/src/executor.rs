use hm_plugin_protocol::{ExecutorInput, PluginError, StepResult};

/// Implemented by step-executor plugins. The host calls
/// [`StepExecutor::run`] exactly once per step; the plugin returns a
/// [`StepResult`] or a [`PluginError`].
///
/// During the call the plugin may stream logs via
/// [`crate::host::emit_step_log`] and check cancellation via
/// [`crate::host::should_cancel`].
pub trait StepExecutor {
    /// Execute a single step.
    ///
    /// # Errors
    /// Returns a [`PluginError`] describing the failure. The host
    /// converts errors into build events and a non-zero step exit.
    fn run(&self, input: ExecutorInput) -> Result<StepResult, PluginError>;
}
