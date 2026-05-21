//! `hm cloud org switch <slug>` — pick the active organization.

use std::collections::BTreeMap;

use hm_plugin_protocol::PluginError;
use hm_plugin_sdk::host;

use crate::api::types::OrganizationList;
use crate::cli::OrgCommand;
use crate::config::Config;
use crate::creds;
use crate::http::Client;
use crate::state::CloudState;

pub(crate) fn run(env: &BTreeMap<String, String>, cmd: OrgCommand) -> Result<(), PluginError> {
    let cfg = Config::from_env(env);
    let token = creds::load_token(&cfg.api_base, env).ok_or_else(not_logged_in)?;
    let client = Client::new(&cfg, Some(token));

    match cmd {
        OrgCommand::Switch { slug } => switch(&client, &slug),
    }
}

fn switch(client: &Client, slug: &str) -> Result<(), PluginError> {
    let orgs: OrganizationList = client.get("/organizations")?;
    let found = orgs.data.iter().find(|o| o.slug == slug).ok_or_else(|| {
        PluginError::new(
            "cloud_org_not_found",
            format!("no organization with slug '{slug}'"),
        )
    })?;
    let mut state = CloudState::load();
    state.active_org = Some(found.slug.clone());
    state.save();
    host::write_stderr(
        format!("active organization: {} ({})\n", found.name, found.slug).as_bytes(),
    );
    Ok(())
}

fn not_logged_in() -> PluginError {
    PluginError::new("cloud_not_logged_in", "not logged in; run `hm cloud login`")
}
