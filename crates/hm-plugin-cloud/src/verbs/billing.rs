//! `hm cloud billing balance|transactions|usage|topup|redeem`.

use std::collections::BTreeMap;

use hm_plugin_protocol::PluginError;
use hm_plugin_sdk::host;

use crate::api::types::{
    Balance, RedeemRequest, RedeemResponse, TopupRequest, TopupResponse, TransactionList,
    UsageWindow,
};
use crate::cli::BillingCommand;
use crate::config::Config;
use crate::creds;
use crate::http::Client;
use crate::state::CloudState;

pub(crate) fn run(env: &BTreeMap<String, String>, cmd: BillingCommand) -> Result<(), PluginError> {
    let cfg = Config::from_env(env);
    let token = creds::load_token(&cfg.api_base, env).ok_or_else(not_logged_in)?;
    let client = Client::new(&cfg, Some(token));
    let org = active_org()?;

    match cmd {
        BillingCommand::Balance => balance(&client, &org),
        BillingCommand::Transactions { limit } => transactions(&client, &org, limit),
        BillingCommand::Usage { from, to } => usage(&client, &org, from.as_deref(), to.as_deref()),
        BillingCommand::Topup {
            amount_usd,
            no_browser,
        } => topup(&client, &org, amount_usd, no_browser),
        BillingCommand::Redeem { code } => redeem(&client, &org, &code),
    }
}

fn balance(client: &Client, org: &str) -> Result<(), PluginError> {
    let b: Balance = client.get(&format!("/organizations/{org}/billing/balance"))?;
    let dollars = b.credits_usd_cents as f64 / 100.0;
    host::write_stdout(format!("${dollars:.2}\n").as_bytes());
    Ok(())
}

fn transactions(client: &Client, org: &str, limit: u32) -> Result<(), PluginError> {
    let list: TransactionList = client.get(&format!(
        "/organizations/{org}/billing/transactions?limit={limit}"
    ))?;
    for t in &list.data {
        let line = format!(
            "{}  {:>10} {:<14} {}\n",
            t.at.format("%Y-%m-%d %H:%M:%S"),
            t.amount_cents,
            t.kind,
            t.memo.as_deref().unwrap_or("")
        );
        host::write_stdout(line.as_bytes());
    }
    Ok(())
}

fn usage(
    client: &Client,
    org: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<(), PluginError> {
    let mut q = vec![];
    if let Some(f) = from {
        q.push(format!("from={f}"));
    }
    if let Some(t) = to {
        q.push(format!("to={t}"));
    }
    let qs = if q.is_empty() {
        String::new()
    } else {
        format!("?{}", q.join("&"))
    };
    let u: UsageWindow = client.get(&format!("/organizations/{org}/billing/usage{qs}"))?;
    let line = format!(
        "{} -> {}: {:.2} min, ${:.2}\n",
        u.from.format("%Y-%m-%d"),
        u.to.format("%Y-%m-%d"),
        u.minutes_used,
        u.cents_used as f64 / 100.0
    );
    host::write_stdout(line.as_bytes());
    Ok(())
}

fn topup(client: &Client, org: &str, amount_usd: u32, no_browser: bool) -> Result<(), PluginError> {
    let r: TopupResponse = client.post(
        &format!("/organizations/{org}/billing/topup"),
        &TopupRequest {
            org_slug: org.to_string(),
            amount_cents: i64::from(amount_usd) * 100,
        },
    )?;
    if no_browser {
        host::write_stdout(r.checkout_url.as_bytes());
        host::write_stdout(b"\n");
    } else if !host::browser_open(&r.checkout_url) {
        host::write_stderr(b"couldn't open browser; URL:\n");
        host::write_stderr(r.checkout_url.as_bytes());
        host::write_stderr(b"\n");
    }
    Ok(())
}

fn redeem(client: &Client, org: &str, code: &str) -> Result<(), PluginError> {
    let r: RedeemResponse = client.post(
        &format!("/organizations/{org}/billing/redeem"),
        &RedeemRequest {
            org_slug: org.to_string(),
            code: code.to_string(),
        },
    )?;
    let dollars = r.credited_cents as f64 / 100.0;
    host::write_stderr(format!("credited ${dollars:.2}\n").as_bytes());
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
