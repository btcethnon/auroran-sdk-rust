//! Typed event payloads (decimal-string projection from the node API).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::OcoLegs;
use crate::wire::{
    AccountRole, Address20, ChainId, DepositSeq, ExternalDepositRef, MarginMode, MarketId,
    MarketKind, MarketLifecycle, PairId, Side, TimeInForce, TriggerDirection, TriggerOrderId,
    TriggerOrderPayload, WithdrawRequestId,
};

use super::reasons::{DoneReason, OcoResolveReason, RejectReason, TriggerCancelReason};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FilledEvent {
    pub taker_order_id: u64,
    pub maker_order_id: u64,
    pub market_id: MarketId,
    pub taker_owner: Address20,
    pub maker_owner: Address20,
    pub price: String,
    pub qty: String,
    pub notional: String,
    #[serde(default)]
    pub taker_fee: String,
    #[serde(default)]
    pub maker_fee: String,
    pub aggressor_side: Side,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LeverageUpdatedEvent {
    pub owner: Address20,
    pub market_id: MarketId,
    pub leverage: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OrderAcceptedEvent {
    pub order_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub side: Side,
    pub limit_price: String,
    pub qty: String,
    pub tif: TimeInForce,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OrderDoneEvent {
    pub order_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub reason: DoneReason,
    pub remaining: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RestingOrderSnapshot {
    pub price: String,
    pub qty: String,
    pub remaining: String,
    pub im_reserved: String,
    #[serde(default)]
    pub prepaid_maker_fee: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OrderRestingEvent {
    pub order_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub resting: RestingOrderSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PositionUpdatedEvent {
    pub owner: Address20,
    pub market_id: MarketId,
    pub old_size: String,
    pub new_size: String,
    #[serde(default)]
    pub new_entry_vwap: String,
    #[serde(default)]
    pub realized_pnl: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RejectedEvent {
    pub action_kind: String,
    pub reason: RejectReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BalanceChangedEvent {
    pub owner: Address20,
    pub delta: String,
    pub new_balance: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RebatePaidEvent {
    pub trader: Address20,
    pub inviter: Address20,
    pub referral_code: String,
    pub total_amount: String,
    pub inviter_share: String,
    pub invitee_share: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DepositCreditedEvent {
    pub seq: DepositSeq,
    pub owner: Address20,
    pub amount: String,
    pub new_balance: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WithdrawSettledEvent {
    pub request_id: WithdrawRequestId,
    pub owner: Address20,
    pub amount: String,
    pub tx_hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LiquidatedEvent {
    pub target: Address20,
    pub market_id: MarketId,
    pub closed_via_book: String,
    pub force_closed_at_mark: String,
    pub total_realized_pnl: String,
    pub mark_price: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerActivatedEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub mark_price: String,
    pub trigger_price: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarkUpdatedEvent {
    pub market_id: MarketId,
    pub mark_price: String,
    pub previous_mark_price: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketStatsSnapshotEvent {
    pub market_id: MarketId,
    pub long_size: String,
    pub short_size: String,
    pub net_size: String,
    pub oracle_counter_pnl: String,
    #[serde(default)]
    pub fills_in_block: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BatchItemRejectedEvent {
    pub index: u32,
    pub action_kind: String,
    pub reason: RejectReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeRejectEvent {
    pub order_id: Option<u64>,
    pub engine_reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BankruptcyEvent {
    pub target_owner: Address20,
    pub market_id: MarketId,
    pub shortfall_amount: String,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserFeeRateChangedEvent {
    pub owner: Address20,
    pub maker_fee_rate: Option<String>,
    pub taker_fee_rate: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReferrerRegisteredEvent {
    pub owner: Address20,
    pub code: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReferrerBoundEvent {
    pub owner: Address20,
    pub code: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InviterKeepRatioChangedEvent {
    pub owner: Address20,
    pub old_ratio_bps: u32,
    pub new_ratio_bps: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DeadMansSwitchScheduledEvent {
    pub owner: Address20,
    pub trigger_time_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DeadMansSwitchTriggeredEvent {
    pub owner: Address20,
    pub cancelled: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PositionFlattenedEvent {
    pub owner: Address20,
    pub market_id: MarketId,
    pub old_size: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OrderCancelledByPositionFlatEvent {
    pub order_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub remaining: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerCancelledByPositionFlatEvent {
    pub trigger_id: TriggerOrderId,
    pub owner: Address20,
    pub market_id: MarketId,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IsolatedMarginUpdatedEvent {
    pub owner: Address20,
    pub market_id: MarketId,
    pub new_isolated_margin: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarginModeUpdatedEvent {
    pub owner: Address20,
    pub market_id: MarketId,
    pub margin_mode: MarginMode,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PositionForceClosedAtMarkEvent {
    pub target: Address20,
    pub market_id: MarketId,
    pub size: String,
    pub mark_price: String,
    pub realized_pnl: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OracleQuoteAcceptedEvent {
    pub market_id: MarketId,
    pub quoter: Address20,
    pub bid_price: String,
    pub ask_price: String,
    pub mark_price: String,
    pub last_price: String,
    pub volume: String,
    pub source_ts_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OracleCounterDepletedEvent {
    pub market_id: MarketId,
    pub required: String,
    pub available: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketHaltedQuoteStaleEvent {
    pub market_id: MarketId,
    pub last_quote_ts_ms: u64,
    pub block_ts_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketResumedAfterQuoteEvent {
    pub market_id: MarketId,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct QuoteRejectStormEvent {
    pub market_id: MarketId,
    pub quoter: Address20,
    pub streak: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OrderCancelledByOcoSiblingEvent {
    pub order_id: u64,
    pub owner: Address20,
    pub market_id: MarketId,
    pub remaining: String,
    pub pair_id: PairId,
    pub winning_leg: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OcoTriggerOrderCancelledEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub reason: TriggerCancelReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OcoLimitTriggerOrderCancelledEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub reason: TriggerCancelReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OcoPairResolvedEvent {
    pub pair_id: PairId,
    pub owner: Address20,
    pub market_id: MarketId,
    pub winner_leg: Option<Value>,
    pub reason: OcoResolveReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerOrderPlacedEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub trigger_price: String,
    pub trigger_direction: TriggerDirection,
    pub payload: TriggerOrderPayload,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerOrderAmendedEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub trigger_price: String,
    pub trigger_direction: TriggerDirection,
    pub payload: TriggerOrderPayload,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerOrderCancelledEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub reason: TriggerCancelReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerLimitTriggerOrderCancelledEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub reason: TriggerCancelReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerOrderExpiredEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerFireFailedEvent {
    pub trigger_id: TriggerOrderId,
    pub market_id: MarketId,
    pub owner: Address20,
    pub reason: RejectReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerOcoPairPlacedEvent {
    pub pair_id: PairId,
    pub owner: Address20,
    pub market_id: MarketId,
    pub legs: OcoLegs,
    pub client_pair_id: Option<u64>,
    pub parent_order_id: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TriggerOcoPairResolvedEvent {
    pub pair_id: PairId,
    pub owner: Address20,
    pub market_id: MarketId,
    pub winner_leg: Option<Value>,
    pub reason: OcoResolveReason,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DepositRecordedEvent {
    pub seq: DepositSeq,
    pub external_ref: ExternalDepositRef,
    pub tx_hash: Option<String>,
    pub account: Address20,
    pub amount: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WithdrawRequestedEvent {
    pub request_id: WithdrawRequestId,
    pub owner: Address20,
    pub amount: String,
    pub chain: ChainId,
    pub new_balance: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WithdrawRefundedEvent {
    pub request_id: WithdrawRequestId,
    pub owner: Address20,
    pub amount: String,
    pub reason_code: u8,
    pub new_balance: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SettlementPausedChangedEvent {
    pub paused: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AccountRoleChangedEvent {
    pub target: Address20,
    pub role: AccountRole,
    pub granted: bool,
    pub new_mask: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AgentRegisteredEvent {
    pub owner: Address20,
    pub agent_address: Address20,
    pub role_mask: u64,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AgentRevokedEvent {
    pub owner: Address20,
    pub agent_address: Address20,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EmergencyHaltChangedEvent {
    pub market_id: MarketId,
    pub halted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketCreatedEvent {
    pub market_id: MarketId,
    pub kind: MarketKind,
    pub lifecycle: MarketLifecycle,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketLifecycleChangedEvent {
    pub market_id: MarketId,
    pub from: MarketLifecycle,
    pub to: MarketLifecycle,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketCancelledEvent {
    pub market_id: MarketId,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FeeRecipientChangedEvent {
    pub market_id: MarketId,
    pub old_recipient: Address20,
    pub new_recipient: Address20,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MarketConfigAmendedEvent {
    pub market_id: MarketId,
    pub max_leverage: Option<u32>,
    pub maker_fee_rate: Option<Value>,
    pub taker_fee_rate: Option<Value>,
    pub margin_table_len: Option<u32>,
    pub mark_max_change_bps: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GlobalRebateRatioChangedEvent {
    pub old_ratio_bps: u32,
    pub new_ratio_bps: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AccountRebateRatioChangedEvent {
    pub owner: Address20,
    pub ratio_bps: Option<u32>,
}
