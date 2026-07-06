use crate::wire::{Address20, MarketConfig, MarketId};
use serde::{Deserialize, Serialize};

// ── Market ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketListItem {
    pub symbol: String,
    pub market_id: MarketId,
    pub kind: String,
    pub lifecycle: String,
    pub emergency_halt: bool,
    pub price_decimals: u32,
    pub size_decimals: u32,
    pub max_leverage: u32,
    pub mark_price: String,
    #[serde(default)]
    pub prev_day_price: String,
    #[serde(default)]
    pub day_ntl_volume: String,
    #[serde(default)]
    pub day_base_volume: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketStatsRecord {
    pub long_size: String,
    pub short_size: String,
    pub net_size: String,
    pub oracle_counter_pnl: String,
    pub open_interest: String,
    pub open_interest_notional: String,
    pub fills_in_block: u32,
    pub block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OracleQuoteResponse {
    pub bid_price: String,
    pub ask_price: String,
    pub mark_price: String,
    pub source_ts_ms: u64,
    pub sequence_id: u64,
    pub quoter: Address20,
    /// 外部参考市场最新成交价（仅 WS tick 携带；HTTP 查询 latest_quote 不含）。
    #[serde(default)]
    pub last_price: Option<String>,
    /// 外部参考市场成交量增量（仅 WS tick 携带）。
    #[serde(default)]
    pub volume: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketDetailResponse {
    pub config: MarketConfig,
    pub emergency_halt: bool,
    pub latest_quote: Option<OracleQuoteResponse>,
    pub last_stats: Option<MarketStatsRecord>,
    pub mark_price: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuoteHistorySample {
    pub block_height: u64,
    pub event_seq: u64,
    pub timestamp_ms: u64,
    pub quote: OracleQuoteResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatsHistorySample {
    pub block_height: u64,
    pub event_seq: u64,
    pub timestamp_ms: u64,
    pub stats: MarketStatsRecord,
    pub oracle_counter_balance: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketTradeSample {
    pub block_height: u64,
    pub event_seq: u64,
    pub timestamp_ms: u64,
    pub price: String,
    pub qty: String,
    pub notional: String,
    pub taker_owner: Address20,
    pub is_taker_buy: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketStatsHistoryResponse {
    pub symbol: String,
    pub height: u64,
    pub latest_quote: Option<OracleQuoteResponse>,
    pub last_stats: Option<MarketStatsRecord>,
    pub oracle_counter_balance: String,
    pub quotes: Vec<QuoteHistorySample>,
    pub stats: Vec<StatsHistorySample>,
    pub trades: Vec<MarketTradeSample>,
}
