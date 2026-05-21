//! `hm cloud build list|show|cancel|watch`.

use std::collections::BTreeMap;

use hm_plugin_protocol::PluginError;
use hm_plugin_sdk::host;

use crate::api::types::{Build, BuildList};
use crate::cli::BuildCommand;
use crate::config::Config;
use crate::creds;
use crate::http::Client;
use crate::state::CloudState;

pub(crate) fn run(env: &BTreeMap<String, String>, cmd: BuildCommand) -> Result<(), PluginError> {
    let cfg = Config::from_env(env);
    let token = creds::load_token(&cfg.api_base, env).ok_or_else(not_logged_in)?;
    let client = Client::new(&cfg, Some(token));
    let org = active_org()?;

    match cmd {
        BuildCommand::List { pipeline } => list(&client, &org, &pipeline),
        BuildCommand::Show { pipeline, number } => show(&client, &org, &pipeline, number),
        BuildCommand::Cancel { pipeline, number } => cancel(&client, &org, &pipeline, number),
        BuildCommand::Watch { pipeline, number } => watch(&client, &org, &pipeline, number),
    }
}

fn list(client: &Client, org: &str, pipe: &str) -> Result<(), PluginError> {
    let builds: BuildList = client.get(&format!("/organizations/{org}/pipelines/{pipe}/builds"))?;
    for b in &builds.data {
        let line = format!(
            "#{:<5} {:<10} {}\n",
            b.number,
            b.state,
            b.message.as_deref().unwrap_or("")
        );
        host::write_stdout(line.as_bytes());
    }
    Ok(())
}

fn show(client: &Client, org: &str, pipe: &str, number: i64) -> Result<(), PluginError> {
    let b: Build = client.get(&format!(
        "/organizations/{org}/pipelines/{pipe}/builds/{number}"
    ))?;
    let json = serde_json::to_string_pretty(&b).unwrap_or_default();
    host::write_stdout(json.as_bytes());
    host::write_stdout(b"\n");
    Ok(())
}

fn cancel(client: &Client, org: &str, pipe: &str, number: i64) -> Result<(), PluginError> {
    let _: serde_json::Value = client.post(
        &format!("/organizations/{org}/pipelines/{pipe}/builds/{number}/cancel"),
        &serde_json::json!({}),
    )?;
    host::write_stderr(format!("build #{number} cancelled\n").as_bytes());
    Ok(())
}

fn watch(client: &Client, org: &str, pipe: &str, number: i64) -> Result<(), PluginError> {
    // Poll the build's state every 2 seconds; print state transitions
    // to stderr. Exit when terminal (passed/failed/canceled).
    //
    // TODO(plan-5+): replace this busy-wait with an `hm_sleep_ms` host
    // fn. WASM has no native sleep, so for now we spin while polling
    // `host::should_cancel`. Crude but adequate for short intervals.
    let mut last_state = String::new();
    loop {
        if host::should_cancel() {
            return Err(PluginError::new(
                "cloud_cancelled",
                "watch cancelled by user",
            ));
        }
        let b: Build = client.get(&format!(
            "/organizations/{org}/pipelines/{pipe}/builds/{number}"
        ))?;
        if b.state != last_state {
            host::write_stderr(format!("state: {last_state} -> {}\n", b.state).as_bytes());
            last_state = b.state.clone();
        }
        match b.state.as_str() {
            "passed" => return Ok(()),
            "failed" | "canceled" => {
                return Err(PluginError::new(
                    "cloud_build_failed",
                    format!("build {} ({})", b.state, number),
                ));
            }
            _ => {}
        }
        let start = std::time::SystemTime::now();
        while start.elapsed().map(|d| d.as_secs() < 2).unwrap_or(true) {
            if host::should_cancel() {
                return Err(PluginError::new(
                    "cloud_cancelled",
                    "watch cancelled by user",
                ));
            }
        }
    }
}

fn not_logged_in() -> PluginError {
    PluginError::new("cloud_not_logged_in", "not logged in; run `hm cloud login`")
}

fn active_org() -> Result<String, PluginError> {
    CloudState::load().active_org.ok_or_else(|| {
        PluginError::new(
            "cloud_no_active_org",
            "no active organization; run `hm cloud org switch <slug>`",
        )
    })
}
