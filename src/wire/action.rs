//! Action wire payloads — symbol-based wire format (mirrors Auroran chain protocol).

use super::address::Address20;
use super::decimal::DecimalStr;
use super::types::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceOrderAction {
    pub owner: Address20,
    pub symbol: String,
    pub side: Side,
    pub limit_price: DecimalStr,
    pub qty: DecimalStr,
    pub tif: TimeInForce,
    pub client_order_id: Option<ClientOrderId>,
    pub reduce_only: bool,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOrderAction {
    pub owner: Address20,
    pub symbol: Option<String>,
    pub order_id: Option<OrderId>,
    pub client_order_id: Option<ClientOrderId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetAccountRoleAction {
    pub target: Address20,
    pub role: AccountRole,
    pub granted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiquidateAction {
    pub target: Address20,
    pub symbol: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateMarketAction {
    pub config: MarketConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleMarketAction {
    pub symbol: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetFeeRecipientAction {
    pub symbol: String,
    pub recipient: Address20,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendMarketConfigAction {
    pub symbol: String,
    pub max_leverage: Option<u32>,
    pub maker_fee_rate: Option<DecimalStr>,
    pub taker_fee_rate: Option<DecimalStr>,
    pub margin_table: Option<Vec<MarginTier>>,
    pub mark_max_change_bps: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetEmergencyHaltAction {
    pub symbol: String,
    pub halt: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordDepositAction {
    /// 外部链充值引用（去重键）。链上自增分配真正的 `DepositSeq`。
    pub external_ref: ExternalDepositRef,
    pub tx_hash: Option<IdemKey>,
    pub account: Address20,
    pub amount: DecimalStr,
    pub bsc_block: u64,
    pub bsc_ts: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditDepositAction {
    pub seq: DepositSeq,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithdrawRequestAction {
    /// Logical network name bound in User-Signed EIP-712 (wire key `zepto_chain`).
    #[serde(rename = "zepto_chain")]
    pub network_name: String,
    pub owner: Address20,
    pub amount: DecimalStr,
    /// 用户选择的下提目标链（如 `"bsc"`），EIP-712 签名内绑定防跨链重放。
    pub chain: ChainId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithdrawSettleAction {
    pub request_id: WithdrawRequestId,
    pub external_tx: ExternalTxRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithdrawRefundAction {
    pub request_id: WithdrawRequestId,
    pub reason_code: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetSettlementPausedAction {
    pub paused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetUserFeeRateAction {
    pub owner: Address20,
    pub maker_fee_rate: Option<DecimalStr>,
    pub taker_fee_rate: Option<DecimalStr>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetReferrerAction {
    pub owner: Address20,
    pub code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterReferrerAction {
    pub owner: Address20,
    pub code: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetGlobalRebateRatioAction {
    pub ratio_bps: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetAccountRebateRatioAction {
    pub owner: Address20,
    pub ratio_bps: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetInviterKeepRatioAction {
    pub owner: Address20,
    pub ratio_bps: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceTriggerOrderAction {
    pub owner: Address20,
    pub symbol: String,
    pub trigger_price: DecimalStr,
    pub trigger_direction: TriggerDirection,
    pub payload: TriggerOrderPayload,
    pub expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelTriggerOrderAction {
    pub owner: Address20,
    pub trigger_id: TriggerOrderId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendTriggerOrderAction {
    pub owner: Address20,
    pub trigger_id: TriggerOrderId,
    pub new_trigger_price: Option<DecimalStr>,
    pub new_qty: Option<DecimalStr>,
    pub new_limit_price: Option<DecimalStr>,
    pub new_tif: Option<TimeInForce>,
    pub new_reduce_only: Option<bool>,
    /// `None` = unchanged; JSON `null` = clear GTD; positive ms = new expiry.
    #[serde(default)]
    pub new_expires_at_ms: Option<Option<u64>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchAmendTriggerAction {
    pub amends: Vec<AmendTriggerOrderAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceOcoAction {
    pub owner: Address20,
    pub execution: OcoExecution,
    pub client_pair_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOcoAction {
    pub owner: Address20,
    pub pair_id: PairId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitOracleQuoteAction {
    pub quoter: Address20,
    pub symbol: String,
    pub bid_price: DecimalStr,
    pub ask_price: DecimalStr,
    pub mark_price: DecimalStr,
    pub source_ts_ms: u64,
    pub sequence_id: u64,
    pub last_price: DecimalStr,
    pub volume: DecimalStr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendOrderAction {
    pub owner: Address20,
    pub symbol: String,
    pub order_id: Option<OrderId>,
    pub client_order_id: Option<ClientOrderId>,
    pub new_limit_price: Option<DecimalStr>,
    pub new_qty: Option<DecimalStr>,
    pub new_tif: Option<TimeInForce>,
    pub new_reduce_only: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MassCancelScopeAction {
    Owner,
    Side(Side),
    Ids(Vec<OrderId>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MassCancelAction {
    pub owner: Address20,
    pub symbol: String,
    pub scope: MassCancelScopeAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosePositionAction {
    pub owner: Address20,
    pub symbol: String,
    pub qty: Option<DecimalStr>,
    pub limit_price: Option<DecimalStr>,
    pub tif: Option<TimeInForce>,
    pub client_order_id: Option<ClientOrderId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetLeverageAction {
    pub owner: Address20,
    pub symbol: String,
    pub leverage: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetMarginModeAction {
    pub owner: Address20,
    pub symbol: String,
    pub margin_mode: MarginMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetIsolatedMarginAction {
    pub owner: Address20,
    pub symbol: String,
    pub amount: DecimalStr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterAgentAction {
    /// Logical network name bound in User-Signed EIP-712 (wire key `zepto_chain`).
    #[serde(rename = "zepto_chain")]
    pub network_name: String,
    pub owner: Address20,
    pub agent_address: Address20,
    pub role_mask: u64,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeAgentAction {
    /// Logical network name bound in User-Signed EIP-712 (wire key `zepto_chain`).
    #[serde(rename = "zepto_chain")]
    pub network_name: String,
    pub owner: Address20,
    pub agent_address: Address20,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleCancelAction {
    pub owner: Address20,
    pub trigger_time_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPlaceOrderAction {
    pub orders: Vec<PlaceOrderAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchModifyAction {
    pub modifies: Vec<AmendOrderAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchSubmitOracleQuoteAction {
    pub quotes: Vec<SubmitOracleQuoteAction>,
}

/// Aggregated business Action — mirrors Auroran chain protocol (44 variants).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    PlaceOrder(PlaceOrderAction),
    CancelOrder(CancelOrderAction),
    SetAccountRole(SetAccountRoleAction),
    Liquidate(LiquidateAction),
    CreateMarket(CreateMarketAction),
    ActivateMarket(SimpleMarketAction),
    HaltMarket(SimpleMarketAction),
    ResumeMarket(SimpleMarketAction),
    RequestDelist(SimpleMarketAction),
    CompleteDelist(SimpleMarketAction),
    SetFeeRecipient(SetFeeRecipientAction),
    AmendMarketConfig(AmendMarketConfigAction),
    SetEmergencyHalt(SetEmergencyHaltAction),
    RecordDeposit(RecordDepositAction),
    CreditDeposit(CreditDepositAction),
    WithdrawRequest(WithdrawRequestAction),
    WithdrawSettle(WithdrawSettleAction),
    WithdrawRefund(WithdrawRefundAction),
    SetSettlementPaused(SetSettlementPausedAction),
    SetUserFeeRate(SetUserFeeRateAction),
    SetReferrer(SetReferrerAction),
    RegisterReferrer(RegisterReferrerAction),
    SetGlobalRebateRatio(SetGlobalRebateRatioAction),
    SetAccountRebateRatio(SetAccountRebateRatioAction),
    SetInviterKeepRatio(SetInviterKeepRatioAction),
    PlaceTriggerOrder(PlaceTriggerOrderAction),
    CancelTriggerOrder(CancelTriggerOrderAction),
    AmendTriggerOrder(AmendTriggerOrderAction),
    PlaceOco(PlaceOcoAction),
    CancelOco(CancelOcoAction),
    SubmitOracleQuote(SubmitOracleQuoteAction),
    AmendOrder(AmendOrderAction),
    MassCancel(MassCancelAction),
    ClosePosition(ClosePositionAction),
    SetLeverage(SetLeverageAction),
    SetMarginMode(SetMarginModeAction),
    SetIsolatedMargin(SetIsolatedMarginAction),
    RegisterAgent(RegisterAgentAction),
    RevokeAgent(RevokeAgentAction),
    ScheduleCancel(ScheduleCancelAction),
    BatchPlaceOrder(BatchPlaceOrderAction),
    BatchModify(BatchModifyAction),
    BatchAmendTrigger(BatchAmendTriggerAction),
    BatchSubmitOracleQuote(BatchSubmitOracleQuoteAction),
}
