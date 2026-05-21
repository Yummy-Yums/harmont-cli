//! `hm cloud pipeline list|show`.

use std::collections::BTreeMap;

use hm_plugin_protocol::PluginError;
use hm_plugin_sdk::host;

use crate::api::types::{Pipeline, PipelineList};
use crate::cli::PipelineCommand;
use crate::config::Config;
use crate::creds;
use crate::http::Client;
use crate::state::CloudState;

pub(crate) fn run(env: &BTreeMap<String, String>, cmd: PipelineCommand) -> Result<(), PluginError> {
    let cfg = Config::from_env(env);
    let token = creds::load_token(&cfg.api_base, env).ok_or_else(not_logged_in)?;
    let client = Client::new(&cfg, Some(token));
    let org = active_org()?;

    match cmd {
        PipelineCommand::List => list(&client, &org),
        PipelineCommand::Show { slug } => show(&client, &org, &slug),
    }
}

fn list(client: &Client, org: &str) -> Result<(), PluginError> {
    let pipes: PipelineList = client.get(&format!("/organizations/{org}/pipelines"))?;
    for p in &pipes.data {
        let line = format!(
            "{:<24} {}\n",
            p.slug,
            p.label.as_deref().unwrap_or("(no label)")
        );
        host::write_stdout(line.as_bytes());
    }
    Ok(())
}

fn show(client: &Client, org: &str, slug: &str) -> Result<(), PluginError> {
    let p: Pipeline = client.get(&format!("/organizations/{org}/pipelines/{slug}"))?;
    let json = serde_json::to_string_pretty(&serde_json::json!({
        "id": p.id,
        "slug": p.slug,
        "label": p.label,
        "default_branch": p.default_branch,
    }))
    .unwrap_or_default();
    host::write_stdout(json.as_bytes());
    host::write_stdout(b"\n");
    Ok(())
}

fn active_org() -> Result<String, PluginError> {
    CloudState::load().active_org.ok_or_else(|| {
        PluginError::new(
            "cloud_no_active_org",
            "no active organization; run `hm cloud org switch <slug>`",
        )
    })
}

fn not_logged_in() -> PluginError {
    PluginError::new("cloud_not_logged_in", "not logged in; run `hm cloud login`")
}
