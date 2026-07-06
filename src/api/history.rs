use crate::wire::{Address20, MarketId, OrderId, Side};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::account::{AccountSummaryResponse, RestingOrderSummary};
use super::market::MarketListItem;
use super::meta::ActionsMetaResponse;
use super::oco::OcoPairResponse;
use super::orderbook::OrderbookLevelResponse;
use super::trigger::TriggerOrderResponse;

// ── History / query (layer 2) ──────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeResponse {
    pub block_height: u64,
    pub event_seq: u64,
    pub timestamp_ms: u64,
    pub market_id: MarketId,
    pub symbol: String,
    pub price: String,
    pub qty: String,
    pub notional: String,
    pub side: Side,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserFillResponse {
    pub block_height: u64,
    pub event_seq: u64,
    pub timestamp_ms: u64,
    pub market_id: MarketId,
    pub symbol: String,
    pub price: String,
    pub qty: String,
    pub notional: String,
    pub fee: String,
    pub is_taker: bool,
    pub aggressor_side: Side,
    #[serde(default)]
    pub order_id: Option<u64>,
    #[serde(default)]
    pub client_order_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrderLifecycleStatus {
    Open,
    Closed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseReason {
    Filled,
    IocExpired,
    Cancelled,
    GtdExpired,
    FokRejected,
    MarketDelisted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderStatusResponse {
    pub order_id: OrderId,
    pub status: OrderLifecycleStatus,
    #[serde(default)]
    pub market_id: Option<MarketId>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub side: Option<Side>,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub qty: Option<String>,
    #[serde(default)]
    pub remaining: Option<String>,
    #[serde(default)]
    pub filled: Option<String>,
    #[serde(default)]
    pub avg_price: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub close_reason: Option<CloseReason>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CandleResponse {
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub trades: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExchangeConfigResponse {
    pub action_version: u32,
    pub max_decimals: u32,
    pub max_tx_per_block: usize,
    pub settlement_paused: bool,
    #[serde(default)]
    pub global_rebate_ratio_bps: u32,
    pub market_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LiquidatablePosition {
    pub account: Address20,
    pub market_id: MarketId,
    pub symbol: String,
    pub size: String,
    pub entry_vwap: String,
    pub mark_price: String,
    pub equity: String,
    pub mm_required: String,
    pub shortfall: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EstimatedLiquidationResponse {
    pub symbol: String,
    pub size: String,
    pub entry_price: String,
    pub mark_price: String,
    pub leverage: u32,
    pub margin: String,
    pub liquidation_price: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRateLimitResponse {
    pub address: Address20,
    pub cum_vlm: String,
    pub n_requests_used: u64,
    pub n_requests_cap: u64,
    pub window_blocks: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserFeesResponse {
    pub address: Address20,
    pub custom_maker_fee_rate: Option<String>,
    pub custom_taker_fee_rate: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReferralResponse {
    pub address: Address20,
    pub referred_by_code: Option<String>,
    pub referral_code: Option<String>,
    pub n_referrals: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminAuditEntry {
    pub block_height: u64,
    pub event_seq: u64,
    pub timestamp_ms: u64,
    pub signer: Address20,
    pub event: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BookView {
    pub state_hash: String,
    pub bids: Vec<OrderbookLevelResponse>,
    pub asks: Vec<OrderbookLevelResponse>,
    #[serde(default)]
    pub spread: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BootstrapResponse {
    pub markets: Vec<MarketListItem>,
    #[serde(default)]
    pub account: Option<AccountSummaryResponse>,
    #[serde(default)]
    pub books: BTreeMap<String, BookView>,
    pub action_meta: ActionsMetaResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountListItem {
    pub address: Address20,
    pub balance: String,
    pub nonce: u64,
    pub role_mask: u64,
    pub position_count: usize,
}

/// Filters for [`AuroranClient::list_accounts_filtered`] / `listAccounts` RPC.
#[derive(Clone, Debug, Default)]
pub struct ListAccountsFilter {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub role: Option<String>,
    pub referral_code: Option<String>,
    pub referred_by_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AllBboItem {
    pub symbol: String,
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub spread: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketSummaryResponse {
    pub symbol: String,
    pub mark_price: String,
    pub best_bid: Option<String>,
    pub best_ask: Option<String>,
    pub spread: Option<String>,
    pub open_interest: Option<String>,
    pub open_interest_notional: Option<String>,
    pub fills_in_block: Option<u32>,
    pub bid_levels: usize,
    pub ask_levels: usize,
    pub open_orders: usize,
    #[serde(default)]
    pub prev_day_price: String,
    #[serde(default)]
    pub day_ntl_volume: String,
    #[serde(default)]
    pub day_base_volume: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopAccountItem {
    pub address: Address20,
    pub balance: String,
    pub account_value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalStatsResponse {
    pub account_count: usize,
    pub market_count: usize,
    pub deposit_count: usize,
    pub withdraw_count: usize,
    pub total_balance: String,
    pub total_open_interest_notional: String,
    pub settlement_paused: bool,
    pub open_order_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListAccountsResponse {
    pub offset: usize,
    pub total: usize,
    pub accounts: Vec<AccountListItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AllOpenOrdersResponse {
    pub offset: usize,
    pub total: usize,
    pub orders: Vec<RestingOrderSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AllTriggerOrdersResponse {
    pub offset: usize,
    pub total: usize,
    pub triggers: Vec<TriggerOrderResponse>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AllOcoPairsResponse {
    pub offset: usize,
    pub total: usize,
    pub pairs: Vec<OcoPairResponse>,
}
