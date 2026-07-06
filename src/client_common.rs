//! Shared JSON-RPC / REST helpers for sync and async clients.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ClientError;

pub const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Serialize)]
pub struct RpcRequest<M: Serialize, P: Serialize> {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: M,
    pub params: P,
}

/// Thin wrapper around the unified `{height, data, page?}` read skeleton.
#[derive(Deserialize, Debug, Clone)]
pub struct QueryResult<T> {
    pub height: u64,
    pub data: T,
    #[serde(default)]
    pub page: Option<Page>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Page {
    pub offset: usize,
    pub limit: usize,
    /// 链上部分分页接口（如 getUserFills）可能返回 `null`。
    #[serde(default)]
    pub total: Option<usize>,
}

impl Page {
    pub fn total_or(&self, fallback: usize) -> usize {
        self.total.unwrap_or(fallback)
    }
}

pub fn page_from_query<T>(
    qr: &QueryResult<T>,
    offset: Option<usize>,
    limit: Option<usize>,
    data_len: usize,
) -> Page {
    qr.page.clone().unwrap_or(Page {
        offset: offset.unwrap_or(0),
        limit: limit.unwrap_or(data_len),
        total: None,
    })
}

pub fn format_http_error(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        return format!("network timeout: {err}");
    }
    if err.is_connect() {
        return format!("network connection failed: {err}");
    }
    if err.is_request() {
        return format!("network request failed: {err}");
    }
    format!("HTTP transport error: {err}")
}

pub fn bootstrap_params(
    address: Option<&str>,
    symbols: Option<&[String]>,
    book_depth: Option<usize>,
) -> Value {
    let mut params = serde_json::json!({});
    if let Some(a) = address {
        params["address"] = Value::from(a);
    }
    if let Some(s) = symbols {
        params["symbols"] = s.into();
    }
    if let Some(d) = book_depth {
        params["book_depth"] = Value::from(d);
    }
    params
}

pub fn parse_jsonrpc_result<T: serde::de::DeserializeOwned>(raw: Value) -> Result<T, ClientError> {
    if let Some(err) = raw.get("error") {
        return Err(ClientError::from_rpc_value(err));
    }
    let result = raw.get("result").cloned().unwrap_or(Value::Null);
    Ok(serde_json::from_value(result)?)
}
