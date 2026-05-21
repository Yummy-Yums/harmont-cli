//! The `register_plugin!` macro generates the Extism plugin entry
//! points from a plugin's manifest and capability impls.
//!
//! A plugin can pass zero or more of `subcommand`, `executor`, `hook`,
//! `output` to register concrete implementations, in any order. Any
//! capability the plugin declares in its manifest but does not register
//! here is a compile-time omission — the host will call into an
//! unimplemented export at runtime and fail loudly.
//!
//! Two capability entries of the same kind (e.g. `executor = A,
//! executor = B`) are not detected by the macro itself, but each kind
//! emits a uniquely-named extern fn (`hm_executor_run`, etc.). Two of
//! the same kind therefore fails at type-check with a clean
//! "duplicate definition" error from rustc.

/// Generate `hm_manifest` + capability exports for a plugin.
///
/// # Example
///
/// ```ignore
/// register_plugin!(
///     manifest = ...,
///     executor = MyExec,
///     hook     = MyHook,
/// );
///
/// // Order-independent: this is equivalent.
/// register_plugin!(
///     manifest = ...,
///     hook     = MyHook,
///     executor = MyExec,
/// );
/// ```
#[macro_export]
macro_rules! register_plugin {
    (manifest = $manifest:expr $(, $($tail:tt)*)?) => {
        #[$crate::extism_pdk::plugin_fn]
        pub fn hm_manifest(_: ()) -> $crate::extism_pdk::FnResult<$crate::extism_pdk::Json<$crate::PluginManifest>> {
            Ok($crate::extism_pdk::Json($manifest))
        }

        $crate::__rp_dispatch!($($($tail)*)?);
    };
}

/// Dispatch loop for capability impls. Consumes one `key = $ty` pair
/// at a time and recurses on the tail. Order-independent because every
/// arm matches by keyword.
#[macro_export]
#[doc(hidden)]
macro_rules! __rp_dispatch {
    // Base case: nothing left (with or without trailing comma).
    () => {};
    (,) => {};

    (subcommand = $ty:ty $(, $($rest:tt)*)?) => {
        #[$crate::extism_pdk::plugin_fn]
        pub fn hm_subcommand_run(
            $crate::extism_pdk::Json(input): $crate::extism_pdk::Json<$crate::SubcommandInput>,
        ) -> $crate::extism_pdk::FnResult<$crate::extism_pdk::Json<$crate::ExitInfo>> {
            let plugin = <$ty as ::core::default::Default>::default();
            match $crate::SubcommandPlugin::run(&plugin, input) {
                Ok(info) => Ok($crate::extism_pdk::Json(info)),
                Err(e) => Err($crate::extism_pdk::WithReturnCode::new(e.into(), 1)),
            }
        }
        $crate::__rp_dispatch!($($($rest)*)?);
    };

    (executor = $ty:ty $(, $($rest:tt)*)?) => {
        #[$crate::extism_pdk::plugin_fn]
        pub fn hm_executor_run(
            $crate::extism_pdk::Json(input): $crate::extism_pdk::Json<$crate::ExecutorInput>,
        ) -> $crate::extism_pdk::FnResult<$crate::extism_pdk::Json<$crate::StepResult>> {
            let plugin = <$ty as ::core::default::Default>::default();
            match $crate::StepExecutor::run(&plugin, input) {
                Ok(r) => Ok($crate::extism_pdk::Json(r)),
                Err(e) => Err($crate::extism_pdk::WithReturnCode::new(e.into(), 1)),
            }
        }
        $crate::__rp_dispatch!($($($rest)*)?);
    };

    (hook = $ty:ty $(, $($rest:tt)*)?) => {
        #[$crate::extism_pdk::plugin_fn]
        pub fn hm_hook_on_event(
            $crate::extism_pdk::Json(event): $crate::extism_pdk::Json<$crate::HookEvent>,
        ) -> $crate::extism_pdk::FnResult<$crate::extism_pdk::Json<$crate::HookOutcome>> {
            let plugin = <$ty as ::core::default::Default>::default();
            match $crate::LifecycleHook::on_event(&plugin, event) {
                Ok(o) => Ok($crate::extism_pdk::Json(o)),
                Err(e) => Err($crate::extism_pdk::WithReturnCode::new(e.into(), 1)),
            }
        }
        $crate::__rp_dispatch!($($($rest)*)?);
    };

    (output = $ty:ty $(, $($rest:tt)*)?) => {
        #[$crate::extism_pdk::plugin_fn]
        pub fn hm_output_on_event(
            $crate::extism_pdk::Json(event): $crate::extism_pdk::Json<$crate::BuildEvent>,
        ) -> $crate::extism_pdk::FnResult<()> {
            let plugin = <$ty as ::core::default::Default>::default();
            $crate::OutputFormatter::on_event(&plugin, event)
                .map_err(|e| $crate::extism_pdk::WithReturnCode::new(e.into(), 1))?;
            Ok(())
        }

        #[$crate::extism_pdk::plugin_fn]
        pub fn hm_output_finalize(_: ()) -> $crate::extism_pdk::FnResult<Vec<u8>> {
            let plugin = <$ty as ::core::default::Default>::default();
            $crate::OutputFormatter::finalize(&plugin)
                .map_err(|e| $crate::extism_pdk::WithReturnCode::new(e.into(), 1))
        }
        $crate::__rp_dispatch!($($($rest)*)?);
    };
}
