//! HTTP client 错误。

use serde::Deserialize;
use thiserror::Error;

/// JSON-RPC 2.0 error object (`error.code` / `error.message` / `error.data`).
#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn is_nonce_replay(&self) -> bool {
        self.code == -32001
    }

    pub fn is_resource_not_found(&self) -> bool {
        self.code == -32004
    }

    pub fn is_auth_failed(&self) -> bool {
        self.code == -32010
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("invalid URL: {0}")]
    Url(String),
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },
    #[error("JSON-RPC error {code}: {message}")]
    Rpc {
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    },
    #[error("JSON decode failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("transaction {tx_hash} rejected ({status})")]
    TxRejected {
        tx_hash: String,
        status: String,
        reason: Option<serde_json::Value>,
    },
}

impl ClientError {
    /// Parse a JSON-RPC error payload when present; otherwise wrap as generic API error.
    pub fn from_rpc_value(err: &serde_json::Value) -> Self {
        if let Ok(rpc) = serde_json::from_value::<JsonRpcError>(err.clone()) {
            Self::Rpc {
                code: rpc.code,
                message: rpc.message,
                data: rpc.data,
            }
        } else {
            Self::Api {
                status: 200,
                body: serde_json::to_string(err).unwrap_or_else(|_| format!("{err}")),
            }
        }
    }

    /// JSON-RPC error code when this is [`ClientError::Rpc`].
    pub fn rpc_code(&self) -> Option<i32> {
        match self {
            Self::Rpc { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// `-32004` resource not found (e.g. `getPosition` before any market config exists).
    pub fn is_resource_not_found(&self) -> bool {
        self.rpc_code() == Some(-32004)
    }

    /// Parsed business reject reason from [`ClientError::TxRejected`] or auth `-32010` RPC data.
    pub fn reject_reason(&self) -> Option<crate::events::RejectReason> {
        match self {
            Self::TxRejected { reason, .. } => reason
                .as_ref()
                .and_then(|v| crate::events::RejectReason::from_value(v).ok()),
            Self::Rpc { code, data, .. } if *code == -32010 => data.as_ref().and_then(|d| {
                d.get("reason")
                    .and_then(|v| crate::events::RejectReason::from_value(v).ok())
            }),
            _ => None,
        }
    }
}
