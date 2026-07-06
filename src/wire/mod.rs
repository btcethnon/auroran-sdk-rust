//! Wire contract mirrors (byte-identical serde with on-chain types).
//!
//! All types in this module are self-contained copies of the upstream
//! Auroran chain wire format mirrors. The SDK has zero dependency on upstream crates.

pub mod eip712;

mod action;
mod address;
mod decimal;
mod envelope;

mod types;

pub use action::{
    Action, AmendMarketConfigAction, AmendOrderAction, AmendTriggerOrderAction,
    BatchAmendTriggerAction, BatchModifyAction, BatchPlaceOrderAction,
    BatchSubmitOracleQuoteAction, CancelOcoAction, CancelOrderAction, CancelTriggerOrderAction,
    ClosePositionAction, CreateMarketAction, CreditDepositAction, LiquidateAction,
    MassCancelAction, MassCancelScopeAction, PlaceOcoAction, PlaceOrderAction,
    PlaceTriggerOrderAction, RecordDepositAction, RegisterAgentAction, RegisterReferrerAction,
    RevokeAgentAction, ScheduleCancelAction, SetAccountRebateRatioAction, SetAccountRoleAction,
    SetEmergencyHaltAction, SetFeeRecipientAction, SetGlobalRebateRatioAction,
    SetInviterKeepRatioAction, SetIsolatedMarginAction, SetLeverageAction, SetMarginModeAction,
    SetReferrerAction, SetSettlementPausedAction, SetUserFeeRateAction, SimpleMarketAction,
    SubmitOracleQuoteAction, WithdrawRefundAction, WithdrawRequestAction, WithdrawSettleAction,
};
pub use address::{Address20, AddressParseError};
pub use decimal::{format_decimal, parse_decimal, scale_int, DecimalStr, DECIMALS_6, SCALE_6};
pub use envelope::{ChainEnvelope, SigCredential, SignedActionEnvelope, ACTION_VERSION_V2};
pub use types::{
    AccountRole, ChainId, ClientOrderId, DepositId, DepositSeq, ExternalDepositRef, ExternalTxRef,
    IdemKey, MarginMode, MarginTier, MarketConfig, MarketId, MarketKind, MarketLifecycle,
    OcoExecution, OcoLimitLeg, OcoStopLimitLeg, OcoStopMarketLeg, OracleQuote, OrderId, PairId,
    Side, StopTriggerLeg, TimeInForce, TriggerDirection, TriggerKind, TriggerOrderId,
    TriggerOrderPayload, WithdrawRequestId, MAX_DECIMALS,
};
