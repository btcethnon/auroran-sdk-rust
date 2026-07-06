use crate::wire::{Address20, MarketId, OrderId, TriggerKind, TriggerOrderId};
use serde::{Deserialize, Serialize};

// ── OCO ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum OcoStatus {
    Active,
    Resolved,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum OcoLegs {
    TwoLimits {
        primary_order_id: OrderId,
        secondary_order_id: OrderId,
    },
    StopMarketAndLimit {
        stop_trigger_id: TriggerOrderId,
        limit_order_id: OrderId,
    },
    StopLimitAndLimit {
        stop_trigger_id: TriggerOrderId,
        limit_order_id: OrderId,
    },
    TwoTriggers {
        primary_trigger_id: TriggerOrderId,
        primary_kind: TriggerKind,
        secondary_trigger_id: TriggerOrderId,
        secondary_kind: TriggerKind,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OcoPairResponse {
    pub pair_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub symbol: String,
    pub status: OcoStatus,
    pub legs: OcoLegs,
    pub placed_at_block: u64,
    #[serde(default)]
    pub client_pair_id: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OcoPairsResponse {
    pub address: Address20,
    pub pairs: Vec<OcoPairResponse>,
}
