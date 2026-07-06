use crate::wire::{Address20, WithdrawRequestId};
use serde::{Deserialize, Serialize};

// ── Bridge ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeSettlementResponse {
    pub settlement_paused: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum DepositStatus {
    Recorded,
    Credited,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DepositRecord {
    /// 链上自增序号（排序键 + 分页键）。
    pub seq: u64,
    /// 外部链标识（小写，如 `"bsc"` / `"admin"`）。
    pub chain: String,
    /// 外部链充值序号（去重键）。
    pub external_seq: u64,
    #[serde(default)]
    pub tx_hash: Option<serde_json::Value>,
    pub owner: Address20,
    pub amount: String,
    pub bsc_block: u64,
    pub bsc_ts: u64,
    pub status: DepositStatus,
    pub recorded_at_block: u64,
    pub recorded_at_ms: u64,
    pub credited_at_block: Option<u64>,
    pub credited_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeDepositResponse {
    pub deposit: DepositRecord,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum WithdrawStatus {
    Pending,
    Settled,
    Refunded,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WithdrawRecord {
    pub request_id: WithdrawRequestId,
    pub owner: Address20,
    pub amount: String,
    /// 用户选择的下提目标链（如 `"bsc"`）。
    pub chain: String,
    pub status: WithdrawStatus,
    #[serde(default)]
    pub settle_tx_hash: Option<serde_json::Value>,
    #[serde(default)]
    pub settle_bsc_block: Option<u64>,
    #[serde(default)]
    pub settle_bsc_ts: Option<u64>,
    #[serde(default)]
    pub reason_code: Option<u8>,
    #[serde(default)]
    pub requested_at_block: Option<u64>,
    #[serde(default)]
    pub requested_at_ms: Option<u64>,
    #[serde(default)]
    pub finalized_at_block: Option<u64>,
    #[serde(default)]
    pub finalized_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeWithdrawalResponse {
    pub withdrawal: WithdrawRecord,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeWithdrawalsListResponse {
    pub offset: usize,
    pub total: usize,
    pub withdrawals: Vec<WithdrawRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeDepositsListResponse {
    pub offset: usize,
    pub total: usize,
    pub deposits: Vec<DepositRecord>,
}
