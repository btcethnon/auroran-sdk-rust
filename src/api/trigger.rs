use crate::wire::{Address20, MarketId, Side, TimeInForce, TriggerDirection};
use serde::{Deserialize, Serialize};

// ── Triggers ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerOrderType {
    Market,
    Limit,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TriggerOrderResponse {
    pub trigger_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub symbol: String,
    pub side: Side,
    pub order_type: TriggerOrderType,
    pub qty: String,
    pub trigger_price: String,
    pub trigger_direction: TriggerDirection,
    pub limit_price: Option<String>,
    pub tif: TimeInForce,
    pub reduce_only: bool,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub created_at_block: u64,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TriggerOrdersResponse {
    pub address: Address20,
    pub triggers: Vec<TriggerOrderResponse>,
}
