use serde::{Deserialize, Serialize};

// ── Orderbook ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderbookLevelResponse {
    pub price: String,
    pub qty: String,
    #[serde(default)]
    pub cumulative_qty: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderbookResponse {
    pub symbol: String,
    pub height: u64,
    pub state_hash: String,
    pub source: String,
    pub bids: Vec<OrderbookLevelResponse>,
    pub asks: Vec<OrderbookLevelResponse>,
}
