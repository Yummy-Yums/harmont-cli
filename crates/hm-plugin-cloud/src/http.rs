//! Thin HTTP wrapper around extism-pdk's host-mediated `http_request`.
//! Bearer-token injection, JSON ser/de, status-code → stable error code
//! mapping.

use extism_pdk::{HttpRequest, HttpResponse, http::request};
use hm_plugin_protocol::PluginError;
use serde::{Serialize, de::DeserializeOwned};

use crate::config::Config;

pub(crate) struct Client {
    base: String,
    token: Option<String>,
}

impl Client {
    #[allow(
        dead_code,
        reason = "consumed by authenticated verbs in a later cluster"
    )]
    pub(crate) fn new(config: &Config, token: Option<String>) -> Self {
        Self {
            base: config.api_base.clone(),
            token,
        }
    }

    #[allow(dead_code, reason = "consumed by the `login` verb in a later cluster")]
    pub(crate) fn anonymous(config: &Config) -> Self {
        Self::new(config, None)
    }

    /// Issue a GET. Body deserialised as `O`.
    #[allow(dead_code, reason = "consumed by verbs in a later cluster")]
    pub(crate) fn get<O: DeserializeOwned>(&self, path: &str) -> Result<O, PluginError> {
        self.send::<(), O>("GET", path, None)
    }

    #[allow(dead_code, reason = "consumed by verbs in a later cluster")]
    pub(crate) fn post<I: Serialize, O: DeserializeOwned>(
        &self,
        path: &str,
        body: &I,
    ) -> Result<O, PluginError> {
        self.send::<I, O>("POST", path, Some(body))
    }

    #[allow(dead_code, reason = "consumed by verbs in a later cluster")]
    pub(crate) fn delete<O: DeserializeOwned>(&self, path: &str) -> Result<O, PluginError> {
        self.send::<(), O>("DELETE", path, None)
    }

    fn send<I, O>(&self, method: &str, path: &str, body: Option<&I>) -> Result<O, PluginError>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        let url = format!("{}{path}", self.base);
        let mut req = HttpRequest::new(&url).with_method(method);
        if let Some(token) = &self.token {
            req = req.with_header("Authorization", format!("Bearer {token}"));
        }
        req = req.with_header("Accept", "application/json");
        let body_bytes: Option<Vec<u8>> = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|e| PluginError::new("cloud_http_serialize", e.to_string()))?;
        if body_bytes.is_some() {
            req = req.with_header("Content-Type", "application/json");
        }
        let response: HttpResponse = request(&req, body_bytes.as_deref())
            .map_err(|e| PluginError::new("cloud_http_request", format!("{method} {url}: {e}")))?;
        let status = response.status_code();
        let body = response.body();
        if !(200..300).contains(&status) {
            let snippet = String::from_utf8_lossy(&body)
                .chars()
                .take(500)
                .collect::<String>();
            return Err(PluginError::new(
                map_status_code(status),
                format!("{method} {url} → HTTP {status}: {snippet}"),
            ));
        }
        if body.is_empty() {
            // Treat as unit type if `O` accepts `null` (e.g., `()`).
            return serde_json::from_slice(b"null")
                .map_err(|e| PluginError::new("cloud_http_decode", e.to_string()));
        }
        serde_json::from_slice(&body)
            .map_err(|e| PluginError::new("cloud_http_decode", e.to_string()))
    }
}

fn map_status_code(status: u16) -> &'static str {
    match status {
        401 | 403 => "cloud_auth",
        404 => "cloud_not_found",
        429 => "cloud_rate_limited",
        500..=599 => "cloud_server",
        _ => "cloud_http",
    }
}
