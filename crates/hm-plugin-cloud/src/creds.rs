//! On-disk credential storage via the host's keyring host fns.

use std::collections::BTreeMap;

use hm_plugin_sdk::host;

const SERVICE: &str = "harmont-cli";

/// Stash `token` for `api_base`. Empty token clears the entry.
#[allow(dead_code, reason = "consumed by the `login` verb in a later cluster")]
pub(crate) fn save_token(api_base: &str, token: &str) {
    if token.is_empty() {
        host::keyring_delete(SERVICE, api_base);
    } else {
        host::keyring_set(SERVICE, api_base, token);
    }
}

/// Load the token for `api_base`. Prefers `HARMONT_API_TOKEN` from the
/// caller-provided env over the keyring entry.
#[allow(
    dead_code,
    reason = "consumed by the auth/verb modules in a later cluster"
)]
pub(crate) fn load_token(api_base: &str, env: &BTreeMap<String, String>) -> Option<String> {
    if let Some(t) = env.get("HARMONT_API_TOKEN")
        && !t.is_empty()
    {
        return Some(t.clone());
    }
    host::keyring_get(SERVICE, api_base)
}

#[allow(dead_code, reason = "consumed by the `logout` verb in a later cluster")]
pub(crate) fn clear_token(api_base: &str) {
    host::keyring_delete(SERVICE, api_base);
}
