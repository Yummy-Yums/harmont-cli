//! Locates fixture `.wasm` files for tests.
//!
//! We do not depend on the `hm-fixtures` crate as a normal
//! dependency because its target is `wasm32-wasip1`. Instead, tests
//! invoke `cargo build --target wasm32-wasip1 -p hm-fixtures`
//! lazily and read the output from
//! `cli/target/wasm32-wasip1/debug/<name>.wasm`.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

static BUILT: OnceLock<()> = OnceLock::new();

/// Build the `hm-fixtures` crate for `wasm32-wasip1` if it hasn't been
/// built in this test process yet. Idempotent across threads.
///
/// # Panics
///
/// Panics if `cargo build` cannot be invoked or returns a non-zero
/// exit. Tests can't proceed without the artifacts, so failing loudly
/// is the right behaviour.
pub fn ensure_built() {
    BUILT.get_or_init(|| {
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip1", "-p", "hm-fixtures"])
            .current_dir(workspace_root())
            .status()
            .expect("invoke cargo build for hm-fixtures");
        assert!(status.success(), "hm-fixtures wasm build failed");
    });
}

/// Path to the compiled `.wasm` for a given fixture bin name (e.g.
/// `"noop_executor"`). Triggers `ensure_built` on first call.
#[must_use]
pub fn fixture_path(name: &str) -> PathBuf {
    ensure_built();
    workspace_root()
        .join("target")
        .join("wasm32-wasip1")
        .join("debug")
        .join(format!("{name}.wasm"))
}

fn workspace_root() -> PathBuf {
    // cli/crates/hm/tests/common/fixtures.rs → cli/
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/hm → crates
    p.pop(); // crates    → cli
    p
}
