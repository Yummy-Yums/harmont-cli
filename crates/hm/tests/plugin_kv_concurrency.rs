//! Concurrent writers to `KvScope::Plugin` must all win — load → insert →
//! save without a lock loses writes. This test FAILS on the pre-fix
//! tree; Task C2 adds an advisory file lock that makes it pass.

#![allow(
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unsafe_code,
    reason = "test pokes XDG_CONFIG_HOME via std::env::set_var, which is unsafe in Rust 2024"
)]

use std::thread;

use harmont_cli::plugin::host_fns::{kv_set_impl, load_plugin_kv, set_current_plugin_name};
use hm_plugin_protocol::KvScope;

/// Drives N threads concurrently into the plugin-scope KV and asserts
/// every key persists. Without a lock around the RMW window the
/// second-writer's atomic save clobbers the first-writer's insert.
///
/// Ignored by default because:
/// 1. On the unfixed tree it would fail-spam the default test suite.
/// 2. After Task C2 fixes the race it passes — but `set_var` is
///    process-global and would race with other tests that touch
///    `XDG_CONFIG_HOME`. Run explicitly via `cargo test --test
///    plugin_kv_concurrency -- --ignored` after C2 lands.
#[test]
#[ignore = "reveals race; pre-C2 fails; post-C2 passes"]
fn concurrent_plugin_kv_writes_all_persist() {
    const PLUGIN: &str = "concurrency-test-plugin";
    const N: usize = 16;

    let tmp = tempfile::tempdir().unwrap();
    // SAFETY: process-global. The test is `#[ignore]`d so it's invoked
    // explicitly via --ignored and the user controls when it runs.
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
    }

    // Make payloads non-trivial so the save window widens enough for
    // the race to be reproducible.
    let payload = vec![0x42u8; 1024];

    let handles: Vec<_> = (0..N)
        .map(|i| {
            let payload = payload.clone();
            thread::spawn(move || {
                set_current_plugin_name(PLUGIN.into());
                kv_set_impl(KvScope::Plugin, &format!("key_{i}"), payload);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    set_current_plugin_name(PLUGIN.into());
    let kv = load_plugin_kv();
    let missing: Vec<usize> = (0..N)
        .filter(|i| !kv.contains_key(&format!("key_{i}")))
        .collect();
    assert!(
        missing.is_empty(),
        "lost writes for keys: {missing:?} (got {} of {N})",
        kv.len()
    );
}
