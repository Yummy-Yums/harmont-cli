use hm_plugin_protocol::{ExitInfo, PluginError};

pub use hm_plugin_protocol::SubcommandInput;

pub trait SubcommandPlugin {
    /// Run the subcommand.
    ///
    /// # Errors
    /// Returns a [`PluginError`] describing the failure. The host
    /// renders the error and exits the process with code 1.
    fn run(&self, input: SubcommandInput) -> Result<ExitInfo, PluginError>;
}
