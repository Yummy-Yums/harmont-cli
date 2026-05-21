//! `hm cloud job list|show|log`.

use std::collections::BTreeMap;

use hm_plugin_protocol::PluginError;
use hm_plugin_sdk::host;

use crate::api::types::{Job, JobList, JobLog};
use crate::cli::JobCommand;
use crate::config::Config;
use crate::creds;
use crate::http::Client;
use crate::state::CloudState;

pub(crate) fn run(env: &BTreeMap<String, String>, cmd: JobCommand) -> Result<(), PluginError> {
    let cfg = Config::from_env(env);
    let token = creds::load_token(&cfg.api_base, env).ok_or_else(not_logged_in)?;
    let client = Client::new(&cfg, Some(token));
    let org = active_org()?;

    match cmd {
        JobCommand::List { pipeline, build } => list(&client, &org, &pipeline, build),
        JobCommand::Show {
            pipeline,
            build,
            job_id,
        } => show(&client, &org, &pipeline, build, &job_id),
        JobCommand::Log {
            pipeline,
            build,
            job_id,
        } => log(&client, &org, &pipeline, build, &job_id),
    }
}

fn list(client: &Client, org: &str, pipe: &str, build: i64) -> Result<(), PluginError> {
    let jobs: JobList = client.get(&format!(
        "/organizations/{org}/pipelines/{pipe}/builds/{build}/jobs"
    ))?;
    for j in &jobs.data {
        let line = format!(
            "{}  {:<10}  {}\n",
            j.id,
            j.state,
            j.label.as_deref().unwrap_or("")
        );
        host::write_stdout(line.as_bytes());
    }
    Ok(())
}

fn show(client: &Client, org: &str, pipe: &str, build: i64, jid: &str) -> Result<(), PluginError> {
    let j: Job = client.get(&format!(
        "/organizations/{org}/pipelines/{pipe}/builds/{build}/jobs/{jid}"
    ))?;
    host::write_stdout(
        serde_json::to_string_pretty(&j)
            .unwrap_or_default()
            .as_bytes(),
    );
    host::write_stdout(b"\n");
    Ok(())
}

fn log(client: &Client, org: &str, pipe: &str, build: i64, jid: &str) -> Result<(), PluginError> {
    let log: JobLog = client.get(&format!(
        "/organizations/{org}/pipelines/{pipe}/builds/{build}/jobs/{jid}/log"
    ))?;
    for chunk in &log.data {
        host::write_stdout(chunk.line.as_bytes());
        host::write_stdout(b"\n");
    }
    Ok(())
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
