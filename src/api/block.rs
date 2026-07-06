use crate::wire::{Address20};
use serde::{Deserialize, Serialize};

// ── Block ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockResponse {
    pub parent: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub digest: String,
    pub envelope_count: usize,
    pub event_count: usize,
    pub state_root: String,
    #[serde(default)]
    pub envelopes: Vec<BlockEnvelopeView>,
}

/// Per-tx receipt in `getBlock.envelopes[]` (no per-tx events — use `getBlockEvents` or `getTx`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockEnvelopeView {
    pub tx_hash: String,
    pub envelope_idx: u32,
    pub signer: Address20,
    pub nonce: u64,
    pub action: serde_json::Value,
    pub status: String,
    #[serde(default)]
    pub reason: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockEventsResponse {
    pub height: u64,
    pub offset: usize,
    pub total: usize,
    pub events: Vec<serde_json::Value>,
}
