//! Async JSON-RPC client (`feature = "async"`).
//!
//! Native [`reqwest`] async HTTP — shared RPC methods with [`crate::client::AuroranClient`].

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use alloy::signers::local::PrivateKeySigner;
use reqwest::Client as HttpClient;
use reqwest::Url;
use serde::Serialize;
use serde_json::Value;

use crate::api::*;
use crate::client::transport::{HttpAttempt, HttpExchange, HttpExchangeLogger};
use crate::client::{action_method_name, DEFAULT_TIMEOUT_SECS};
use crate::client_common::{
    bootstrap_params, page_from_query, parse_jsonrpc_result, RpcRequest, QueryResult,
    DEFAULT_USER_AGENT,
};
use crate::signed_action::{agent_signed_envelope, master_signed_envelope};
use crate::error::ClientError;
use crate::wire::{Action, Address20, SignedActionEnvelope};

/// Async JSON-RPC 2.0 client for Auroran chain.
pub struct AsyncAuroranClient {
    base: Url,
    http: HttpClient,
    next_id: AtomicU64,
    http_log: Arc<RwLock<Option<HttpExchangeLogger>>>,
}

impl Clone for AsyncAuroranClient {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            http: self.http.clone(),
            next_id: AtomicU64::new(self.next_id.load(Ordering::Relaxed)),
            http_log: Arc::clone(&self.http_log),
        }
    }
}

impl AsyncAuroranClient {
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        Self::with_timeout(base_url, DEFAULT_TIMEOUT_SECS)
    }

    pub fn with_timeout(base_url: &str, timeout_secs: u64) -> Result<Self, ClientError> {
        let base = Url::parse(base_url).map_err(|e| ClientError::Url(e.to_string()))?;
        let http = HttpClient::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .timeout(Duration::from_secs(timeout_secs))
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

    pub(crate) async fn get_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        path: &str,
    ) -> Result<T, ClientError> {
        let url = self.url(path)?;
        let url_str = url.to_string();
        let attempt = self.http_attempt(url_str, "GET", None);
        let resp = match self.http.get(url).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(attempt.log_transport_error(e)),
        };
        let status = resp.status().as_u16();
        let text = match resp.text().await {
            Ok(text) => text,
            Err(e) => return Err(attempt.log_body_read_error(status, e)),
        };
        attempt.decode_json(status, text)
    }

    pub(crate) async fn get_query_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        path: &str,
    ) -> Result<QueryResult<T>, ClientError> {
        self.get_json(path).await
    }

    pub(crate) async fn get_query_data<T: serde::de::DeserializeOwned + Send>(
        &self,
        path: &str,
    ) -> Result<T, ClientError> {
        self.get_query_json(path).await.map(|qr| qr.data)
    }

    pub(crate) async fn post_json<T: serde::de::DeserializeOwned + Send, B: Serialize + Sync>(
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
        let resp = match self.http.post(url).json(body).send().await {
            Ok(resp) => resp,
            Err(e) => return Err(attempt.log_transport_error(e)),
        };
        let status = resp.status().as_u16();
        let text = match resp.text().await {
            Ok(text) => text,
            Err(e) => return Err(attempt.log_body_read_error(status, e)),
        };
        attempt.decode_json(status, text)
    }

    pub(crate) async fn rpc_action(
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
        let raw: Value = self.post_json("/api/v1/action", &req).await?;
        parse_jsonrpc_result(raw)
    }

    pub(crate) async fn rpc_query<T: serde::de::DeserializeOwned + Send>(
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
        let raw: Value = self.post_json("/api/v1/query", &req).await?;
        parse_jsonrpc_result(raw)
    }

    pub async fn submit_action(
        &self,
        envelope: &SignedActionEnvelope,
    ) -> Result<TxReceiptResponse, ClientError> {
        let method = action_method_name(&envelope.action);
        let resp = self.rpc_action(method, envelope).await?;
        #[cfg(feature = "test-support")]
        crate::client::tx_recorder::record(method, &resp.tx_hash, &resp.status);
        Ok(resp)
    }
}

crate::impl_auroran_client_methods!(async, AsyncAuroranClient);
