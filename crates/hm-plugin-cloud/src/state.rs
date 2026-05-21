//! Persistent state (active organization slug) via the host's
//! `KvScope::Plugin` store.

use hm_plugin_sdk::{KvScope, host};
use serde::{Deserialize, Serialize};

const STATE_KEY: &str = "state";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct CloudState {
    pub active_org: Option<String>,
}

impl CloudState {
    #[allow(dead_code, reason = "consumed by the org/run verbs in a later cluster")]
    pub(crate) fn load() -> Self {
        let Some(bytes) = host::kv_get(KvScope::Plugin, STATE_KEY) else {
            return Self::default();
        };
        serde_json::from_slice(&bytes).unwrap_or_default()
    }

    #[allow(dead_code, reason = "consumed by the org/run verbs in a later cluster")]
    pub(crate) fn save(&self) {
        if let Ok(bytes) = serde_json::to_vec(self) {
            host::kv_set(KvScope::Plugin, STATE_KEY, &bytes);
        }
    }
}
