use hm_plugin_protocol::{BuildEvent, PluginError};

/// Implemented by output-formatter plugins.
///
/// The host invokes [`OutputFormatter::on_event`] for every build event
/// in order, then once at the end calls [`OutputFormatter::finalize`]
/// for formatters that accumulate (`JUnit` XML, JSON arrays).
pub trait OutputFormatter {
    /// Handle a single build event.
    ///
    /// # Errors
    /// Returns a [`PluginError`] if the formatter cannot process the
    /// event (e.g. malformed input). The host renders the error and
    /// aborts the formatter; the build itself is unaffected.
    fn on_event(&self, event: BuildEvent) -> Result<(), PluginError>;

    /// Optional. Default returns empty bytes. Streaming formatters
    /// (human, json-lines) leave this alone; accumulating formatters
    /// (junit) return the full document here.
    ///
    /// # Errors
    /// Returns a [`PluginError`] if the formatter cannot serialise its
    /// accumulated state.
    fn finalize(&self) -> Result<Vec<u8>, PluginError> {
        Ok(Vec::new())
    }
}
