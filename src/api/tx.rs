use crate::wire::{Address20};
use serde::{Deserialize, Serialize};

// ── Tx receipt ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxReceiptResponse {
    pub tx_hash: String,
    pub height: u64,
    pub envelope_idx: u32,
    pub signer: Address20,
    pub nonce: u64,
    pub action: serde_json::Value,
    pub status: String,
    #[serde(default)]
    pub reason: Option<serde_json::Value>,
    pub events: Vec<serde_json::Value>,
}

impl TxReceiptResponse {
    /// Node accepted the envelope into a block (`status == "accepted"`).
    pub fn is_accepted(&self) -> bool {
        self.status == "accepted"
    }

    /// Parsed kept-reject / auth reason when present (`status == "kept-reject"`).
    pub fn reject_reason(&self) -> Option<crate::events::RejectReason> {
        self.reason
            .as_ref()
            .and_then(|v| crate::events::RejectReason::from_value(v).ok())
    }

    /// Returns `Err` when the node kept the envelope but rejected execution (`kept-reject`).
    pub fn ensure_accepted(self) -> Result<Self, crate::error::ClientError> {
        if self.is_accepted() {
            Ok(self)
        } else {
            Err(crate::error::ClientError::TxRejected {
                tx_hash: self.tx_hash,
                status: self.status,
                reason: self.reason,
            })
        }
    }
}
