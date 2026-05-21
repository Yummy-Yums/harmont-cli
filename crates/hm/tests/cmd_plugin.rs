//! `hm plugin list` smoke. Real fixture-driven list/info tests live
//! in `plugin_registry.rs` once Phase F fixtures exist.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn plugin_list_with_no_plugins_prints_help() {
    let temp = tempfile::tempdir().unwrap();
    Command::cargo_bin("hm")
        .unwrap()
        .arg("plugin")
        .arg("list")
        // Point XDG at a clean dir so no user plugins leak in.
        // HOME is also set because some platforms ignore XDG_CONFIG_HOME
        // without a HOME pointing somewhere.
        .env("XDG_CONFIG_HOME", temp.path())
        .env("HOME", temp.path())
        // `project_plugins_dir()` uses cwd, so launch from the tempdir
        // too — otherwise a `.harmont/plugins/` next to the dev tree
        // would leak in.
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(contains("No plugins installed."));
}
