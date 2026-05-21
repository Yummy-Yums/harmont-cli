#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod fixtures;

/// Construct a Command pointing at the freshly-built `hm` binary.
///
/// # Panics
///
/// Panics if the `hm` binary has not been built — `assert_cmd::cargo`
/// resolves the path lazily and only fails when the binary is genuinely
/// missing from `target/`.
#[must_use]
pub fn hm_bin() -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("hm").expect("binary 'hm' not found");
    // Integration tests must never touch the developer's real OS keyring.
    // Pinning to the file backend keeps credentials confined to the per-test
    // HOME tempdir and matches the headless-Linux path we ship in CI.
    cmd.env("HARMONT_CREDENTIAL_STORE", "file");
    cmd
}

/// Build an `hm` command wired against a wiremock `MockServer`.
///
/// The harness sets:
/// - `HARMONT_API_URL`  → the mock server's URI (random localhost port)
/// - `HARMONT_API_TOKEN` → a fake bearer (`test-token`) so `require_auth`
///   passes without touching the OS keyring; the mock accepts any value
/// - `HARMONT_ORG` → the supplied slug, so subcommands that need an org
///   resolve it without reading a config file
///
/// The returned `Command` is ready to `.assert()`. Callers can chain
/// extra `.env(...)` for case-specific overrides (e.g. `NO_COLOR`).
#[must_use]
pub fn hm_command(server: &wiremock::MockServer, org: &str, args: &[&str]) -> assert_cmd::Command {
    let mut cmd = hm_bin();
    cmd.env("HARMONT_API_URL", server.uri())
        .env("HARMONT_API_TOKEN", "test-token")
        .env("HARMONT_ORG", org)
        .args(args);
    cmd
}
