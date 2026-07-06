//! Blocking JSON-RPC 2.0 client (`AuroranClient`).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use alloy::signers::local::PrivateKeySigner;
use reqwest::blocking::Client as HttpClient;
use reqwest::Url;
use serde::Serialize;
use serde_json::Value;

use super::action::action_method_name;
use super::transport::{HttpAttempt, HttpExchange, HttpExchangeLogger};
use crate::api::*;
use crate::client_common::{
    bootstrap_params, page_from_query, parse_jsonrpc_result, RpcRequest, QueryResult,
    DEFAULT_USER_AGENT,
};
use crate::signed_action::{agent_signed_envelope, master_signed_envelope};
use crate::error::ClientError;
use crate::wire::{Action, Address20, SignedActionEnvelope};

/// Default HTTP timeout (seconds).
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// JSON-RPC 2.0 client for Auroran chain (blocking HTTP).
pub struct AuroranClient {
    base: Url,
    http: HttpClient,
    next_id: AtomicU64,
    http_log: Arc<RwLock<Option<HttpExchangeLogger>>>,
}

impl Clone for AuroranClient {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            http: self.http.clone(),
            next_id: AtomicU64::new(self.next_id.load(Ordering::Relaxed)),
            http_log: Arc::clone(&self.http_log),
        }
    }
}

impl AuroranClient {
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        Self::with_timeout(base_url, DEFAULT_TIMEOUT_SECS)
    }

    pub fn with_timeout(base_url: &str, timeout_secs: u64) -> Result<Self, ClientError> {
        let base = Url::parse(base_url).map_err(|e| ClientError::Url(e.to_string()))?;
        let http = HttpClient::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;
        Ok(Self {
            base,
            http,
            next_id: AtomicU64::new(1),
            http_log: Arc::new(RwLock::new(None)),
        })
    }

    pub fn set_http_logger<F>(&self, logger: F)
    where
        F: Fn(&HttpExchange) + Send + Sync + 'static,
    {
        *self.http_log.write().expect("http logger lock") = Some(Arc::new(logger));
    }

    fn http_attempt(&self, url: String, method: &'static str, request_json: Option<String>) -> HttpAttempt {
        HttpAttempt::new(url, method, request_json, Arc::clone(&self.http_log))
    }

    pub(crate) fn url(&self, path: &str) -> Result<Url, ClientError> {
        self.base
            .join(path.trim_start_matches('/'))
            .map_err(|e| ClientError::Url(e.to_string()))
    }

    pub(crate) fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let url = self.url(path)?;
        let url_str = url.to_string();
        let attempt = self.http_attempt(url_str, "GET", None);
        let resp = match self.http.get(url).send() {
            Ok(resp) => resp,
            Err(e) => return Err(attempt.log_transport_error(e)),
        };
        let status = resp.status().as_u16();
        let text = match resp.text() {
            Ok(text) => text,
            Err(e) => return Err(attempt.log_body_read_error(status, e)),
        };
        attempt.decode_json(status, text)
    }

    pub(crate) fn get_query_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<QueryResult<T>, ClientError> {
        self.get_json(path)
    }

    pub(crate) fn get_query_data<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, ClientError> {
        self.get_query_json(path).map(|qr| qr.data)
    }

    pub(crate) fn post_json<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ClientError> {
        let url = self.url(path)?;
        let url_str = url.to_string();
        let request_json = match serde_json::to_string(body) {
            Ok(json) => json,
            Err(e) => return Err(self.http_attempt(url_str, "POST", None).log_serialize_error(e)),
        };
        let attempt = self.http_attempt(url_str, "POST", Some(request_json));
        let resp = match self.http.post(url).json(body).send() {
            Ok(resp) => resp,
            Err(e) => return Err(attempt.log_transport_error(e)),
        };
        let status = resp.status().as_u16();
        let text = match resp.text() {
            Ok(text) => text,
            Err(e) => return Err(attempt.log_body_read_error(status, e)),
        };
        attempt.decode_json(status, text)
    }

    pub(crate) fn rpc_action(
        &self,
        method: &str,
        envelope: &SignedActionEnvelope,
    ) -> Result<TxReceiptResponse, ClientError> {
        let envelope_val = serde_json::to_value(envelope)?;
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method,
            params: serde_json::json!({ "envelope": envelope_val }),
        };
        let raw: Value = self.post_json("/api/v1/action", &req)?;
        parse_jsonrpc_result(raw)
    }

    pub(crate) fn rpc_query<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<QueryResult<T>, ClientError> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method,
            params,
        };
        let raw: Value = self.post_json("/api/v1/query", &req)?;
        parse_jsonrpc_result(raw)
    }

    pub fn submit_action(
        &self,
        envelope: &SignedActionEnvelope,
    ) -> Result<TxReceiptResponse, ClientError> {
        let method = action_method_name(&envelope.action);
        let resp = self.rpc_action(method, envelope)?;
        #[cfg(feature = "test-support")]
        super::tx_recorder::record(method, &resp.tx_hash, &resp.status);
        Ok(resp)
    }
}

crate::impl_auroran_client_methods!(blocking, AuroranClient);
