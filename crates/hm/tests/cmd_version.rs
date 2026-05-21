//! `hm version` should exit 0 and print the version + API version.

#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn version_prints_api_version() {
    Command::cargo_bin("hm")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(contains("plugin api version: 1"));
}
