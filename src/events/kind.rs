//! Typed `EventKind` tree mirroring wire `{Domain: {Variant: payload}}`.

use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CoreEventKind {
    BalanceChanged(BalanceChangedEvent),
    MarkUpdated(MarkUpdatedEvent),
    MarketStatsSnapshot(MarketStatsSnapshotEvent),
    Rejected(RejectedEvent),
    BatchItemRejected(BatchItemRejectedEvent),
    RuntimeReject(RuntimeRejectEvent),
    Bankruptcy(BankruptcyEvent),
    UserFeeRateChanged(UserFeeRateChangedEvent),
    ReferrerRegistered(ReferrerRegisteredEvent),
    ReferrerBound(ReferrerBoundEvent),
    InviterKeepRatioChanged(InviterKeepRatioChangedEvent),
    RebatePaid(RebatePaidEvent),
    DeadMansSwitchScheduled(DeadMansSwitchScheduledEvent),
    DeadMansSwitchTriggered(DeadMansSwitchTriggeredEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExecEventKind {
    OrderAccepted(OrderAcceptedEvent),
    Filled(FilledEvent),
    OrderResting(OrderRestingEvent),
    OrderDone(OrderDoneEvent),
    PositionUpdated(PositionUpdatedEvent),
    PositionFlattened(PositionFlattenedEvent),
    OrderCancelledByPositionFlat(OrderCancelledByPositionFlatEvent),
    TriggerCancelledByPositionFlat(TriggerCancelledByPositionFlatEvent),
    IsolatedMarginUpdated(IsolatedMarginUpdatedEvent),
    LeverageUpdated(LeverageUpdatedEvent),
    MarginModeUpdated(MarginModeUpdatedEvent),
    PositionForceClosedAtMark(PositionForceClosedAtMarkEvent),
    OracleQuoteAccepted(OracleQuoteAcceptedEvent),
    OracleCounterDepleted(OracleCounterDepletedEvent),
    MarketHaltedQuoteStale(MarketHaltedQuoteStaleEvent),
    MarketResumedAfterQuote(MarketResumedAfterQuoteEvent),
    QuoteRejectStorm(QuoteRejectStormEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OcoEventKind {
    OrderCancelledByOcoSibling(OrderCancelledByOcoSiblingEvent),
    TriggerOrderCancelled(OcoTriggerOrderCancelledEvent),
    LimitTriggerOrderCancelled(OcoLimitTriggerOrderCancelledEvent),
    OcoPairResolved(OcoPairResolvedEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LiquidationEventKind {
    Liquidated(LiquidatedEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TriggerEventKind {
    TriggerOrderPlaced(TriggerOrderPlacedEvent),
    TriggerOrderAmended(TriggerOrderAmendedEvent),
    TriggerOrderCancelled(TriggerOrderCancelledEvent),
    LimitTriggerOrderCancelled(TriggerLimitTriggerOrderCancelledEvent),
    TriggerOrderExpired(TriggerOrderExpiredEvent),
    TriggerActivated(TriggerActivatedEvent),
    TriggerFireFailed(TriggerFireFailedEvent),
    OcoPairPlaced(TriggerOcoPairPlacedEvent),
    OcoPairResolved(TriggerOcoPairResolvedEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BridgeEventKind {
    DepositRecorded(DepositRecordedEvent),
    DepositCredited(DepositCreditedEvent),
    WithdrawRequested(WithdrawRequestedEvent),
    WithdrawSettled(WithdrawSettledEvent),
    WithdrawRefunded(WithdrawRefundedEvent),
    SettlementPausedChanged(SettlementPausedChangedEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OpsEventKind {
    AccountRoleChanged(AccountRoleChangedEvent),
    AgentRegistered(AgentRegisteredEvent),
    AgentRevoked(AgentRevokedEvent),
    EmergencyHaltChanged(EmergencyHaltChangedEvent),
    MarketCreated(MarketCreatedEvent),
    MarketLifecycleChanged(MarketLifecycleChangedEvent),
    MarketCancelled(MarketCancelledEvent),
    FeeRecipientChanged(FeeRecipientChangedEvent),
    MarketConfigAmended(MarketConfigAmendedEvent),
    GlobalRebateRatioChanged(GlobalRebateRatioChangedEvent),
    AccountRebateRatioChanged(AccountRebateRatioChangedEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Core(CoreEventKind),
    Exec(ExecEventKind),
    Oco(OcoEventKind),
    Liquidation(LiquidationEventKind),
    Trigger(TriggerEventKind),
    Bridge(BridgeEventKind),
    Ops(OpsEventKind),
}

impl EventKind {
    pub fn domain(&self) -> super::EventDomain {
        match self {
            Self::Core(_) => super::EventDomain::Core,
            Self::Exec(_) => super::EventDomain::Exec,
            Self::Oco(_) => super::EventDomain::Oco,
            Self::Liquidation(_) => super::EventDomain::Liquidation,
            Self::Trigger(_) => super::EventDomain::Trigger,
            Self::Bridge(_) => super::EventDomain::Bridge,
            Self::Ops(_) => super::EventDomain::Ops,
        }
    }

    pub fn variant(&self) -> &'static str {
        match self {
            Self::Core(inner) => match inner {
                CoreEventKind::BalanceChanged(_) => "BalanceChanged",
                CoreEventKind::MarkUpdated(_) => "MarkUpdated",
                CoreEventKind::MarketStatsSnapshot(_) => "MarketStatsSnapshot",
                CoreEventKind::Rejected(_) => "Rejected",
                CoreEventKind::BatchItemRejected(_) => "BatchItemRejected",
                CoreEventKind::RuntimeReject(_) => "RuntimeReject",
                CoreEventKind::Bankruptcy(_) => "Bankruptcy",
                CoreEventKind::UserFeeRateChanged(_) => "UserFeeRateChanged",
                CoreEventKind::ReferrerRegistered(_) => "ReferrerRegistered",
                CoreEventKind::ReferrerBound(_) => "ReferrerBound",
                CoreEventKind::InviterKeepRatioChanged(_) => "InviterKeepRatioChanged",
                CoreEventKind::RebatePaid(_) => "RebatePaid",
                CoreEventKind::DeadMansSwitchScheduled(_) => "DeadMansSwitchScheduled",
                CoreEventKind::DeadMansSwitchTriggered(_) => "DeadMansSwitchTriggered",
            },
            Self::Exec(inner) => match inner {
                ExecEventKind::OrderAccepted(_) => "OrderAccepted",
                ExecEventKind::Filled(_) => "Filled",
                ExecEventKind::OrderResting(_) => "OrderResting",
                ExecEventKind::OrderDone(_) => "OrderDone",
                ExecEventKind::PositionUpdated(_) => "PositionUpdated",
                ExecEventKind::PositionFlattened(_) => "PositionFlattened",
                ExecEventKind::OrderCancelledByPositionFlat(_) => "OrderCancelledByPositionFlat",
                ExecEventKind::TriggerCancelledByPositionFlat(_) => "TriggerCancelledByPositionFlat",
                ExecEventKind::IsolatedMarginUpdated(_) => "IsolatedMarginUpdated",
                ExecEventKind::LeverageUpdated(_) => "LeverageUpdated",
                ExecEventKind::MarginModeUpdated(_) => "MarginModeUpdated",
                ExecEventKind::PositionForceClosedAtMark(_) => "PositionForceClosedAtMark",
                ExecEventKind::OracleQuoteAccepted(_) => "OracleQuoteAccepted",
                ExecEventKind::OracleCounterDepleted(_) => "OracleCounterDepleted",
                ExecEventKind::MarketHaltedQuoteStale(_) => "MarketHaltedQuoteStale",
                ExecEventKind::MarketResumedAfterQuote(_) => "MarketResumedAfterQuote",
                ExecEventKind::QuoteRejectStorm(_) => "QuoteRejectStorm",
            },
            Self::Oco(inner) => match inner {
                OcoEventKind::OrderCancelledByOcoSibling(_) => "OrderCancelledByOcoSibling",
                OcoEventKind::TriggerOrderCancelled(_) => "TriggerOrderCancelled",
                OcoEventKind::LimitTriggerOrderCancelled(_) => "LimitTriggerOrderCancelled",
                OcoEventKind::OcoPairResolved(_) => "OcoPairResolved",
            },
            Self::Liquidation(inner) => match inner {
                LiquidationEventKind::Liquidated(_) => "Liquidated",
            },
            Self::Trigger(inner) => match inner {
                TriggerEventKind::TriggerOrderPlaced(_) => "TriggerOrderPlaced",
                TriggerEventKind::TriggerOrderAmended(_) => "TriggerOrderAmended",
                TriggerEventKind::TriggerOrderCancelled(_) => "TriggerOrderCancelled",
                TriggerEventKind::LimitTriggerOrderCancelled(_) => "LimitTriggerOrderCancelled",
                TriggerEventKind::TriggerOrderExpired(_) => "TriggerOrderExpired",
                TriggerEventKind::TriggerActivated(_) => "TriggerActivated",
                TriggerEventKind::TriggerFireFailed(_) => "TriggerFireFailed",
                TriggerEventKind::OcoPairPlaced(_) => "OcoPairPlaced",
                TriggerEventKind::OcoPairResolved(_) => "OcoPairResolved",
            },
            Self::Bridge(inner) => match inner {
                BridgeEventKind::DepositRecorded(_) => "DepositRecorded",
                BridgeEventKind::DepositCredited(_) => "DepositCredited",
                BridgeEventKind::WithdrawRequested(_) => "WithdrawRequested",
                BridgeEventKind::WithdrawSettled(_) => "WithdrawSettled",
                BridgeEventKind::WithdrawRefunded(_) => "WithdrawRefunded",
                BridgeEventKind::SettlementPausedChanged(_) => "SettlementPausedChanged",
            },
            Self::Ops(inner) => match inner {
                OpsEventKind::AccountRoleChanged(_) => "AccountRoleChanged",
                OpsEventKind::AgentRegistered(_) => "AgentRegistered",
                OpsEventKind::AgentRevoked(_) => "AgentRevoked",
                OpsEventKind::EmergencyHaltChanged(_) => "EmergencyHaltChanged",
                OpsEventKind::MarketCreated(_) => "MarketCreated",
                OpsEventKind::MarketLifecycleChanged(_) => "MarketLifecycleChanged",
                OpsEventKind::MarketCancelled(_) => "MarketCancelled",
                OpsEventKind::FeeRecipientChanged(_) => "FeeRecipientChanged",
                OpsEventKind::MarketConfigAmended(_) => "MarketConfigAmended",
                OpsEventKind::GlobalRebateRatioChanged(_) => "GlobalRebateRatioChanged",
                OpsEventKind::AccountRebateRatioChanged(_) => "AccountRebateRatioChanged",
            },
        }
    }
}
