//! `hm cloud login` — browser-loopback or paste-in flow.

use std::collections::BTreeMap;

use hm_plugin_protocol::PluginError;
use hm_plugin_sdk::host;

use crate::api::types::{CliExchangeRequest, CliExchangeResponse, User};
use crate::config::Config;
use crate::creds;
use crate::http::Client;

#[allow(
    dead_code,
    reason = "wired by `cli::dispatch` in the next cluster (Task 15)"
)]
pub(crate) fn run(env: &BTreeMap<String, String>, paste: bool) -> Result<(), PluginError> {
    let cfg = Config::from_env(env);
    let (verifier, challenge) = pkce_pair()?;

    if paste {
        login_paste(env, &cfg, &verifier, &challenge)
    } else {
        login_loopback(&cfg, &verifier, &challenge)
    }
}

fn login_loopback(cfg: &Config, verifier: &str, challenge: &str) -> Result<(), PluginError> {
    let handle = host::spawn_loopback(None).ok_or_else(|| {
        PluginError::new(
            "cloud_loopback_spawn",
            "host could not bind a loopback socket",
        )
    })?;
    let port = handle.0;
    let redirect = format!("http://127.0.0.1:{port}/cb");
    let auth_url = format!(
        "{}/cli/login?challenge={}&redirect_uri={}",
        cfg.api_base,
        challenge,
        urlencoding(&redirect),
    );

    host::log(
        hm_plugin_protocol::Level::Info,
        &format!("opening browser to {auth_url}"),
    );
    if !host::browser_open(&auth_url) {
        write_stderr(&format!(
            "couldn't auto-open the browser. Open this URL manually:\n  {auth_url}\n"
        ));
    }

    let data = host::loopback_recv(handle, 180_000).ok_or_else(|| {
        PluginError::new(
            "cloud_login_timeout",
            "browser callback did not arrive within 3 minutes",
        )
    })?;
    let code = data.query.get("code").cloned().ok_or_else(|| {
        PluginError::new(
            "cloud_login_missing_code",
            "callback had no 'code' query parameter",
        )
    })?;

    finalize(cfg, &code, verifier)
}

fn login_paste(
    env: &BTreeMap<String, String>,
    cfg: &Config,
    verifier: &str,
    challenge: &str,
) -> Result<(), PluginError> {
    let auth_url = format!(
        "{}/cli/login?challenge={}&redirect_uri=urn:ietf:wg:oauth:2.0:oob",
        cfg.api_base, challenge,
    );
    write_stderr(&format!(
        "Open this URL in your browser, then paste the code:\n  {auth_url}\n"
    ));
    let _ = host::browser_open(&auth_url);

    // Tests inject the code via `HARMONT_LOGIN_CODE` to avoid TTY.
    let code = if let Some(c) = env.get("HARMONT_LOGIN_CODE") {
        c.clone()
    } else {
        host::tty_prompt("code: ", false)
    };
    let code = code.trim().to_string();
    if code.is_empty() {
        return Err(PluginError::new("cloud_login_empty_code", "no code pasted"));
    }
    finalize(cfg, &code, verifier)
}

fn finalize(cfg: &Config, code: &str, verifier: &str) -> Result<(), PluginError> {
    let client = Client::anonymous(cfg);
    let resp: CliExchangeResponse = client.post(
        "/cli/exchange",
        &CliExchangeRequest {
            code: code.to_string(),
            verifier: verifier.to_string(),
        },
    )?;
    creds::save_token(&cfg.api_base, &resp.token);

    let auth_client = Client::new(cfg, Some(resp.token));
    let me: User = auth_client.get("/auth/me")?;
    write_stderr(&format!(
        "logged in as {} ({})\n",
        me.display_name.clone().unwrap_or_else(|| me.email.clone()),
        me.email,
    ));
    Ok(())
}

/// Generate a PKCE verifier + S256 challenge.
///
/// WASM has no entropy source, so we derive 32 bytes from the system
/// clock's nanos. This is INSECURE for production — replace with a
/// proper host fn `hm_random_bytes` in a follow-up.
///
/// TODO(plan-5): add `hm_random_bytes(len) -> Vec<u8>` host fn.
fn pkce_pair() -> Result<(String, String), PluginError> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use sha2::{Digest, Sha256};

    let mut seed = [0u8; 32];
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for (i, b) in seed.iter_mut().enumerate() {
        *b = ((now >> (i % 16)) & 0xFF) as u8;
    }
    let verifier = URL_SAFE_NO_PAD.encode(seed);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    Ok((verifier, challenge))
}

fn write_stderr(msg: &str) {
    host::write_stderr(msg.as_bytes());
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
