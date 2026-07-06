use crate::wire::{Address20, MarginMode, MarketId, OrderId, Side, TimeInForce};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Account ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PositionRecord {
    #[serde(default)]
    pub symbol: String,
    pub size: String,
    pub entry_vwap: String,
    pub mark_price: String,
    pub margin_mode: MarginMode,
    pub leverage: u32,
    pub isolated_margin: String,
    #[serde(default)]
    pub margin_used: String,
    pub unrealized_pnl: String,
    pub notional: String,
    pub liquidation_price: Option<String>,
    pub roe: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentResponse {
    pub address: Address20,
    pub role_mask: u64,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountSummaryResponse {
    pub address: Address20,
    pub balance: String,
    pub nonce: u64,
    pub role_mask: u64,
    pub positions: BTreeMap<MarketId, PositionRecord>,
    #[serde(default)]
    pub account_value: String,
    #[serde(default)]
    pub total_margin_used: String,
    #[serde(default)]
    pub total_notional: String,
    #[serde(default)]
    pub withdrawable: String,
    /// Cross cash available for withdraw / isolated transfer (SCALE_6, excludes uPnL).
    #[serde(default)]
    pub cross_cash_available: String,
    /// Cross trading headroom for open/add admission (SCALE_6, includes Cross uPnL).
    #[serde(default)]
    pub cross_trading_available: String,
    #[serde(default)]
    pub agents: Vec<AgentResponse>,
    #[serde(default)]
    pub dms_deadline_ms: Option<u64>,
    #[serde(default)]
    pub inviter_rebate_ratio_bps: Option<u32>,
    #[serde(default)]
    pub inviter_keep_ratio_bps: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RestingOrderSummary {
    pub order_id: OrderId,
    pub owner: Address20,
    pub market_id: MarketId,
    pub symbol: String,
    pub side: Side,
    pub price: String,
    pub qty: String,
    pub remaining: String,
    pub filled: String,
    pub tif: TimeInForce,
    pub reduce_only: bool,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub placed_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub order_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountOrdersResponse {
    pub address: Address20,
    pub orders: Vec<RestingOrderSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FillRecord {
    pub block_height: u64,
    pub event_seq: u64,
    pub market_id: MarketId,
    pub price: String,
    pub qty: String,
    pub notional: String,
    pub is_taker: bool,
}
