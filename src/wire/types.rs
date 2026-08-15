//! Value types wire mirror (order / market / OCO / bridge scalars).

use super::address::Address20;
use super::decimal::DecimalStr;
use serde::{Deserialize, Serialize};

pub type OrderId = u64;
pub type ClientOrderId = String;
pub type TriggerOrderId = u64;

/// Global decimal budget: `size_decimals + price_decimals() == MAX_DECIMALS`.
pub const MAX_DECIMALS: u32 = 6;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct MarketId(pub u32);

impl MarketId {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for MarketId {
    #[inline]
    fn from(v: u32) -> Self {
        Self(v)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PairId(pub u64);

impl PairId {
    #[inline]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct WithdrawRequestId(pub u64);

impl WithdrawRequestId {
    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct IdemKey(pub [u8; 32]);

impl IdemKey {
    pub const fn from_bytes(b: [u8; 32]) -> Self {
        Self(b)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct DepositSeq(pub u64);

impl DepositSeq {
    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

pub type DepositId = DepositSeq;

/// 外部链 / 目标链标识（如 `"bsc"`、`"polygon"`、`"admin"`）。镜像链 `zepto_types::ChainId`。
pub type ChainId = String;

/// 外部链充值引用（去重键），镜像链 `zepto_types::ExternalDepositRef`。
///
/// 序列化为 `{ "chain": ..., "seq": ... }`，与链 wire/EIP-712 / msgpack-named 一致。
/// 构造请走 [`ExternalDepositRef::new`]，确保 `chain` 统一小写。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExternalDepositRef {
    /// 外部链标识（统一小写，如 `"bsc"` / `"admin"`）。
    pub chain: ChainId,
    /// 该链上的充值业务序号。
    pub seq: u64,
}

impl ExternalDepositRef {
    /// 构造并规范化 `chain` 为小写。
    pub fn new(chain: impl Into<String>, seq: u64) -> Self {
        Self {
            chain: chain.into().to_lowercase(),
            seq,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalTxRef {
    pub tx_hash: [u8; 32],
    pub bsc_block: u64,
    pub bsc_ts: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl Side {
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
    PostOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TriggerDirection {
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MarginMode {
    #[default]
    Cross,
    Isolated,
}

/// Account role bit indices (`Trader=0` … `Quoter=5`). Mirrors `zepto-types::AccountRole`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum AccountRole {
    Trader = 0,
    OracleOperator = 1,
    SettlementOperator = 2,
    Admin = 3,
    Liquidator = 4,
    Quoter = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketKind {
    Native,
    ExternalPeg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketLifecycle {
    Created,
    Active,
    Halted,
    DelistPending,
    Delisted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerOrderPayload {
    Market {
        side: Side,
        qty: DecimalStr,
        reduce_only: bool,
        client_order_id: Option<ClientOrderId>,
    },
    Limit {
        side: Side,
        limit_price: DecimalStr,
        qty: DecimalStr,
        tif: TimeInForce,
        reduce_only: bool,
        client_order_id: Option<ClientOrderId>,
    },
}

impl TriggerOrderPayload {
    pub fn side(&self) -> Side {
        match self {
            TriggerOrderPayload::Market { side, .. } | TriggerOrderPayload::Limit { side, .. } => {
                *side
            }
        }
    }

    pub fn qty(&self) -> &DecimalStr {
        match self {
            TriggerOrderPayload::Market { qty, .. } | TriggerOrderPayload::Limit { qty, .. } => qty,
        }
    }

    pub fn reduce_only(&self) -> bool {
        match self {
            TriggerOrderPayload::Market { reduce_only, .. }
            | TriggerOrderPayload::Limit { reduce_only, .. } => *reduce_only,
        }
    }

    pub fn client_order_id(&self) -> Option<ClientOrderId> {
        match self {
            TriggerOrderPayload::Market {
                client_order_id, ..
            }
            | TriggerOrderPayload::Limit {
                client_order_id, ..
            } => client_order_id.clone(),
        }
    }

    pub fn tif(&self) -> Option<TimeInForce> {
        match self {
            TriggerOrderPayload::Market { .. } => None,
            TriggerOrderPayload::Limit { tif, .. } => Some(*tif),
        }
    }

    pub fn is_market(&self) -> bool {
        matches!(self, TriggerOrderPayload::Market { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcoLimitLeg {
    pub symbol: String,
    pub side: Side,
    pub limit_price: DecimalStr,
    pub qty: DecimalStr,
    pub tif: TimeInForce,
    pub reduce_only: bool,
    pub client_order_id: Option<ClientOrderId>,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcoStopMarketLeg {
    pub symbol: String,
    pub side: Side,
    pub qty: DecimalStr,
    pub trigger_price: DecimalStr,
    pub trigger_direction: TriggerDirection,
    pub reduce_only: bool,
    pub client_trigger_id: Option<u64>,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcoStopLimitLeg {
    pub symbol: String,
    pub side: Side,
    pub qty: DecimalStr,
    pub trigger_price: DecimalStr,
    pub trigger_direction: TriggerDirection,
    pub reduce_only: bool,
    pub limit_price: DecimalStr,
    pub limit_tif: TimeInForce,
    pub client_trigger_id: Option<u64>,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopTriggerLeg {
    Market(OcoStopMarketLeg),
    Limit(OcoStopLimitLeg),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TriggerKind {
    StopMarket = 0,
    StopLimit = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcoExecution {
    TwoLimits {
        primary: OcoLimitLeg,
        secondary: OcoLimitLeg,
    },
    StopAndLimit {
        stop_leg: StopTriggerLeg,
        limit_leg: OcoLimitLeg,
    },
    TwoTriggers {
        primary: StopTriggerLeg,
        secondary: StopTriggerLeg,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarginTier {
    pub max_notional: DecimalStr,
    pub im_rate: DecimalStr,
    pub mm_rate: DecimalStr,
    pub max_leverage: u32,
}

/// Market config wire mirror. `price_decimals` is derived via [`MarketConfig::price_decimals`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketConfig {
    pub market_id: MarketId,
    pub symbol: String,
    pub kind: MarketKind,
    pub lifecycle: MarketLifecycle,
    pub size_decimals: u32,
    pub max_leverage: u32,
    pub maker_fee_rate: DecimalStr,
    pub taker_fee_rate: DecimalStr,
    pub fee_recipient: Address20,
    pub margin_table: Vec<MarginTier>,
    pub mark_max_change_bps: u32,
    pub max_quote_lag_ms: u64,
    pub max_quote_change_bps: u32,
    pub max_spread_bps: u32,
    pub min_quote_interval_ms: u64,
    pub max_fills_per_quote: u32,
    pub price_floor: DecimalStr,
    pub price_ceil: DecimalStr,
}

impl MarketConfig {
    #[inline]
    pub const fn price_decimals(&self) -> u32 {
        MAX_DECIMALS - self.size_decimals
    }

    #[inline]
    pub const fn is_external_peg(&self) -> bool {
        matches!(self.kind, MarketKind::ExternalPeg)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleQuote {
    pub market_id: MarketId,
    pub bid_price: DecimalStr,
    pub ask_price: DecimalStr,
    pub mark_price: DecimalStr,
    pub source_ts_ms: u64,
    pub quoter: Address20,
    #[serde(default)]
    pub last_price: Option<DecimalStr>,
    #[serde(default)]
    pub volume: Option<DecimalStr>,
}
