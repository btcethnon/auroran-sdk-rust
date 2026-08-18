//! Typed reject / done / cancel reason enums (mirrors zepto-chain wire tags).

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::wire::MarginMode;

/// Business reject reason (`Core::Rejected`, `BatchItemRejected`, `TriggerFireFailed`,
/// tx `result.reason`, WS `fire_failed`, auth `-32010`).
///
/// Numeric payload fields such as `InsufficientBalance.required` / `have` are ADR-0026
/// decimal strings everywhere on the public API.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum RejectReason {
    ChainIdMismatch,
    UnsupportedActionVersion,
    SignerNotFound,
    NonceMismatch,
    BadMasterSignature,
    AgentNotRegistered,
    AgentExpired,
    AgentRoleMissing,
    MasterOnlyAction,
    AccountRoleMissing,
    MarketNotFound,
    MarketNotActive,
    AccountNotFound,
    PriceNonPositive,
    QtyNonPositive,
    DecimalPrecisionExceeded,
    PriceQtyOverflow,
    InsufficientBalance {
        required: String,
        have: String,
    },
    NotionalOverflow,
    DustNotionalFill,
    FeeOverflow,
    NegativeFeeRate,
    OrderNotFound,
    CancelOwnerMismatch,
    RuntimeRejected,
    FokRejected,
    DuplicateClientOrderId,
    MarginTableEmpty,
    NotionalExceedsAllTiers,
    InitialMarginOverflow,
    ReduceOnlyWouldOpenOrFlip,
    LeverageOutOfRange,
    OpenOrdersOnMarket,
    PositionNotFlat,
    MarginModeAlreadySet(MarginMode),
    NotIsolatedMarginMode,
    IsolatedMarginNonPositive,
    IsolatedMarginInsufficientBalance,
    IsolatedMarginWithdrawTooLarge,
    LiquidateTargetNotUser,
    LiquidateNoPosition,
    LiquidateHealthy,
    MarkPriceUnavailable,
    UnknownRoleBit,
    MarketAlreadyExists,
    InvalidMarketConfig,
    LifecycleTransitionInvalid,
    DelistTimelockNotElapsed,
    InvalidMarketConfigAmend,
    ReferralCodeNotFound,
    ReferralCodeAlreadyExists,
    ReferralCodeInvalid,
    ReferralSelf,
    ReferrerAlreadyBound,
    ReferrerNotBound,
    ReferralCodeAlreadyRegistered,
    AmountNonPositive,
    DuplicateExternalDeposit,
    UnknownOrConsumedDepositId,
    SettlementPaused,
    WithdrawRequestNotFound,
    WithdrawRequestNotPending,
    WithdrawAvailableInsufficient,
    OwnerSignerMismatch,
    AgentAlreadyRegistered,
    AgentLimitExceeded,
    AgentRoleNotSubset,
    AgentNotFound,
    TriggerNotFound,
    TriggerOwnerMismatch,
    TriggerPriceNonPositive,
    TriggerWouldExecuteImmediately,
    TriggerCountCapExceeded,
    TriggerLimitUnreachableAtFire,
    GtdExpiryInPast,
    MarketEmergencyHalt,
    OcoCrossMarketUnsupported,
    OcoDuplicateLeg,
    OcoActivePairsCapExceeded,
    OcoLegValidationFailed,
    OcoPairNotFound,
    OcoNotOwner,
    OcoAlreadyResolved,
    BracketParentNotFound,
    BracketParentMismatch,
    BracketParentAlreadyLinked,
    BracketParentReduceOnly,
    BracketParentAmbiguous,
    BracketInvalidOcoKind,
    QuoteNotAvailable,
    QuoteBidAskInvalid,
    MarkPriceOutOfSpread,
    QuoteSequenceNotMonotonic,
    QuoteRateLimited,
    QuoteChangeTooLarge,
    QuotePriceOutOfBounds,
    QuoteSpreadTooWide,
    QuoteSourceTooStale,
    QuoteMarketNotExternalPeg,
    QuoteQuoterMismatch,
    AmendOrderNotResting,
    AmendNoChange,
    AmendTriggerNoChange,
    AmendTriggerPayloadMismatch,
    MassCancelTooLarge,
    ClosePositionNoPosition,
    ClosePositionQtyExceedsPosition,
    OrderTargetInvalid,
    BatchSizeInvalid,
    DmsTriggerTooSoon,
    ExecutionFault,
    RebateRatioOutOfRange {
        ratio_bps: u32,
    },
    RebateRatioOwnerNotReferrer,
    InviterKeepRatioOutOfRange {
        ratio_bps: u32,
    },
    MissingAccountForInviterKeepRatio,
    /// Forward-compatible fallback when the node adds a new reject tag.
    #[serde(skip)]
    Unknown(Value),
}

impl<'de> Deserialize<'de> for RejectReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::from_value(&value).map_err(serde::de::Error::custom)
    }
}

impl RejectReason {
    /// Parse externally-tagged reject JSON (`{"InsufficientBalance": {...}}` or `"NonceMismatch"`).
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let obj = value
            .as_object()
            .ok_or_else(|| "reject reason must be an object".to_string())?;
        if obj.len() != 1 {
            return Err(format!("reject reason object must have one key, got {}", obj.len()));
        }
        let (tag, payload) = obj.iter().next().expect("checked len");
        match tag.as_str() {
            "InsufficientBalance" => {
                let fields = payload
                    .as_object()
                    .ok_or_else(|| "InsufficientBalance payload must be object".to_string())?;
                Ok(Self::InsufficientBalance {
                    required: json_field_to_string(fields.get("required"), "required")?,
                    have: json_field_to_string(fields.get("have"), "have")?,
                })
            }
            "MarginModeAlreadySet" => {
                let mode: MarginMode = serde_json::from_value(payload.clone())
                    .map_err(|e| format!("MarginModeAlreadySet: {e}"))?;
                Ok(Self::MarginModeAlreadySet(mode))
            }
            "RebateRatioOutOfRange" => {
                let ratio_bps = payload
                    .get("ratio_bps")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| "RebateRatioOutOfRange.ratio_bps missing".to_string())?;
                Ok(Self::RebateRatioOutOfRange {
                    ratio_bps: ratio_bps as u32,
                })
            }
            "InviterKeepRatioOutOfRange" => {
                let ratio_bps = payload
                    .get("ratio_bps")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| "InviterKeepRatioOutOfRange.ratio_bps missing".to_string())?;
                Ok(Self::InviterKeepRatioOutOfRange {
                    ratio_bps: ratio_bps as u32,
                })
            }
            "ChainIdMismatch" => Ok(Self::ChainIdMismatch),
            "UnsupportedActionVersion" => Ok(Self::UnsupportedActionVersion),
            "SignerNotFound" => Ok(Self::SignerNotFound),
            "NonceMismatch" => Ok(Self::NonceMismatch),
            "BadMasterSignature" => Ok(Self::BadMasterSignature),
            "AgentNotRegistered" => Ok(Self::AgentNotRegistered),
            "AgentExpired" => Ok(Self::AgentExpired),
            "AgentRoleMissing" => Ok(Self::AgentRoleMissing),
            "MasterOnlyAction" => Ok(Self::MasterOnlyAction),
            "AccountRoleMissing" => Ok(Self::AccountRoleMissing),
            "MarketNotFound" | "SymbolNotFound" => Ok(Self::MarketNotFound),
            "MarketNotActive" => Ok(Self::MarketNotActive),
            "AccountNotFound" => Ok(Self::AccountNotFound),
            "PriceNonPositive" => Ok(Self::PriceNonPositive),
            "QtyNonPositive" => Ok(Self::QtyNonPositive),
            "DecimalPrecisionExceeded" => Ok(Self::DecimalPrecisionExceeded),
            "PriceQtyOverflow" => Ok(Self::PriceQtyOverflow),
            "NotionalOverflow" => Ok(Self::NotionalOverflow),
            "DustNotionalFill" => Ok(Self::DustNotionalFill),
            "FeeOverflow" => Ok(Self::FeeOverflow),
            "NegativeFeeRate" => Ok(Self::NegativeFeeRate),
            "OrderNotFound" => Ok(Self::OrderNotFound),
            "CancelOwnerMismatch" => Ok(Self::CancelOwnerMismatch),
            "RuntimeRejected" => Ok(Self::RuntimeRejected),
            "DuplicateClientOrderId" | "DuplicateLiveOrderId" => Ok(Self::DuplicateClientOrderId),
            "MarginTableEmpty" => Ok(Self::MarginTableEmpty),
            "NotionalExceedsAllTiers" => Ok(Self::NotionalExceedsAllTiers),
            "InitialMarginOverflow" => Ok(Self::InitialMarginOverflow),
            "ReduceOnlyRejected" | "ReduceOnlyWouldOpenOrFlip" => Ok(Self::ReduceOnlyWouldOpenOrFlip),
            "LeverageOutOfRange" => Ok(Self::LeverageOutOfRange),
            "OpenOrdersOnMarket" => Ok(Self::OpenOrdersOnMarket),
            "PositionNotFlat" | "PositionNotFound" => Ok(Self::PositionNotFlat),
            "NotIsolatedMarginMode" => Ok(Self::NotIsolatedMarginMode),
            "IsolatedMarginNonPositive" => Ok(Self::IsolatedMarginNonPositive),
            "IsolatedMarginInsufficientBalance" => Ok(Self::IsolatedMarginInsufficientBalance),
            "IsolatedMarginWithdrawTooLarge" => Ok(Self::IsolatedMarginWithdrawTooLarge),
            "LiquidateTargetNotUser" => Ok(Self::LiquidateTargetNotUser),
            "LiquidateNoPosition" => Ok(Self::LiquidateNoPosition),
            "LiquidateHealthy" => Ok(Self::LiquidateHealthy),
            "MarkPriceUnavailable" => Ok(Self::MarkPriceUnavailable),
            "UnknownRoleBit" => Ok(Self::UnknownRoleBit),
            "MarketAlreadyExists" => Ok(Self::MarketAlreadyExists),
            "InvalidMarketConfig" => Ok(Self::InvalidMarketConfig),
            "LifecycleTransitionInvalid" => Ok(Self::LifecycleTransitionInvalid),
            "DelistTimelockNotElapsed" => Ok(Self::DelistTimelockNotElapsed),
            "InvalidMarketConfigAmend" => Ok(Self::InvalidMarketConfigAmend),
            "ReferralCodeNotFound" => Ok(Self::ReferralCodeNotFound),
            "ReferralCodeAlreadyExists" => Ok(Self::ReferralCodeAlreadyExists),
            "ReferralCodeInvalid" => Ok(Self::ReferralCodeInvalid),
            "ReferralSelf" => Ok(Self::ReferralSelf),
            "ReferrerAlreadyBound" => Ok(Self::ReferrerAlreadyBound),
            "ReferrerNotBound" => Ok(Self::ReferrerNotBound),
            "ReferralCodeAlreadyRegistered" => Ok(Self::ReferralCodeAlreadyRegistered),
            "AmountNonPositive" => Ok(Self::AmountNonPositive),
            "DuplicateExternalDeposit" => Ok(Self::DuplicateExternalDeposit),
            "UnknownOrConsumedDepositId" => Ok(Self::UnknownOrConsumedDepositId),
            "SettlementPaused" => Ok(Self::SettlementPaused),
            "WithdrawRequestNotFound" => Ok(Self::WithdrawRequestNotFound),
            "WithdrawRequestNotPending" => Ok(Self::WithdrawRequestNotPending),
            "WithdrawAvailableInsufficient" => Ok(Self::WithdrawAvailableInsufficient),
            "OwnerSignerMismatch" => Ok(Self::OwnerSignerMismatch),
            "AgentAlreadyRegistered" => Ok(Self::AgentAlreadyRegistered),
            "AgentLimitExceeded" => Ok(Self::AgentLimitExceeded),
            "AgentRoleNotSubset" => Ok(Self::AgentRoleNotSubset),
            "AgentNotFound" => Ok(Self::AgentNotFound),
            "TriggerNotFound" => Ok(Self::TriggerNotFound),
            "TriggerOwnerMismatch" => Ok(Self::TriggerOwnerMismatch),
            "TriggerPriceNonPositive" => Ok(Self::TriggerPriceNonPositive),
            "TriggerWouldExecuteImmediately" => Ok(Self::TriggerWouldExecuteImmediately),
            "TriggerCountCapExceeded" => Ok(Self::TriggerCountCapExceeded),
            "TriggerLimitUnreachableAtFire" => Ok(Self::TriggerLimitUnreachableAtFire),
            "GtdExpiryInPast" => Ok(Self::GtdExpiryInPast),
            "MarketEmergencyHalt" => Ok(Self::MarketEmergencyHalt),
            "OcoCrossMarketUnsupported" => Ok(Self::OcoCrossMarketUnsupported),
            "OcoDuplicateLeg" => Ok(Self::OcoDuplicateLeg),
            "OcoActivePairsCapExceeded" => Ok(Self::OcoActivePairsCapExceeded),
            "OcoLegValidationFailed" => Ok(Self::OcoLegValidationFailed),
            "OcoPairNotFound" => Ok(Self::OcoPairNotFound),
            "OcoNotOwner" => Ok(Self::OcoNotOwner),
            "OcoAlreadyResolved" => Ok(Self::OcoAlreadyResolved),
            "BracketParentNotFound" => Ok(Self::BracketParentNotFound),
            "BracketParentMismatch" => Ok(Self::BracketParentMismatch),
            "BracketParentAlreadyLinked" => Ok(Self::BracketParentAlreadyLinked),
            "BracketParentReduceOnly" => Ok(Self::BracketParentReduceOnly),
            "BracketParentAmbiguous" => Ok(Self::BracketParentAmbiguous),
            "BracketInvalidOcoKind" => Ok(Self::BracketInvalidOcoKind),
            "QuoteNotAvailable" => Ok(Self::QuoteNotAvailable),
            "QuoteBidAskInvalid" => Ok(Self::QuoteBidAskInvalid),
            "MarkPriceOutOfSpread" => Ok(Self::MarkPriceOutOfSpread),
            "QuoteSequenceNotMonotonic" => Ok(Self::QuoteSequenceNotMonotonic),
            "QuoteRateLimited" => Ok(Self::QuoteRateLimited),
            "QuoteChangeTooLarge" => Ok(Self::QuoteChangeTooLarge),
            "QuotePriceOutOfBounds" => Ok(Self::QuotePriceOutOfBounds),
            "QuoteSpreadTooWide" => Ok(Self::QuoteSpreadTooWide),
            "QuoteSourceTooStale" => Ok(Self::QuoteSourceTooStale),
            "QuoteMarketNotExternalPeg" => Ok(Self::QuoteMarketNotExternalPeg),
            "QuoteQuoterMismatch" => Ok(Self::QuoteQuoterMismatch),
            "AmendOrderNotResting" => Ok(Self::AmendOrderNotResting),
            "AmendNoChange" => Ok(Self::AmendNoChange),
            "AmendTriggerNoChange" => Ok(Self::AmendTriggerNoChange),
            "AmendTriggerPayloadMismatch" => Ok(Self::AmendTriggerPayloadMismatch),
            "MassCancelTooLarge" | "BatchSizeExceeded" => Ok(Self::MassCancelTooLarge),
            "ClosePositionNoPosition" => Ok(Self::ClosePositionNoPosition),
            "ClosePositionQtyExceedsPosition" => Ok(Self::ClosePositionQtyExceedsPosition),
            "OrderTargetInvalid" => Ok(Self::OrderTargetInvalid),
            "BatchSizeInvalid" => Ok(Self::BatchSizeInvalid),
            "DmsTriggerTooSoon" => Ok(Self::DmsTriggerTooSoon),
            "ExecutionFault" => Ok(Self::ExecutionFault),
            "FokRejected" => Ok(Self::FokRejected),
            "RebateRatioOwnerNotReferrer" => Ok(Self::RebateRatioOwnerNotReferrer),
            "MissingAccountForInviterKeepRatio" => Ok(Self::MissingAccountForInviterKeepRatio),
            "PostOnlyRejected" | "InvalidCommand" | "EventBudgetExceeded" => {
                Ok(Self::Unknown(value.clone()))
            }
            other => Ok(Self::Unknown(serde_json::json!({ other: payload.clone() }))),
        }
    }

    pub fn tag(&self) -> &str {
        match self {
            Self::InsufficientBalance { .. } => "InsufficientBalance",
            Self::MarginModeAlreadySet(_) => "MarginModeAlreadySet",
            Self::RebateRatioOutOfRange { .. } => "RebateRatioOutOfRange",
            Self::InviterKeepRatioOutOfRange { .. } => "InviterKeepRatioOutOfRange",
            Self::Unknown(v) => v
                .as_object()
                .and_then(|o| o.keys().next())
                .map(String::as_str)
                .unwrap_or("Unknown"),
            Self::ChainIdMismatch => "ChainIdMismatch",
            Self::UnsupportedActionVersion => "UnsupportedActionVersion",
            Self::SignerNotFound => "SignerNotFound",
            Self::NonceMismatch => "NonceMismatch",
            Self::BadMasterSignature => "BadMasterSignature",
            Self::AgentNotRegistered => "AgentNotRegistered",
            Self::AgentExpired => "AgentExpired",
            Self::AgentRoleMissing => "AgentRoleMissing",
            Self::MasterOnlyAction => "MasterOnlyAction",
            Self::AccountRoleMissing => "AccountRoleMissing",
            Self::MarketNotFound => "MarketNotFound",
            Self::MarketNotActive => "MarketNotActive",
            Self::AccountNotFound => "AccountNotFound",
            Self::PriceNonPositive => "PriceNonPositive",
            Self::QtyNonPositive => "QtyNonPositive",
            Self::DecimalPrecisionExceeded => "DecimalPrecisionExceeded",
            Self::PriceQtyOverflow => "PriceQtyOverflow",
            Self::NotionalOverflow => "NotionalOverflow",
            Self::DustNotionalFill => "DustNotionalFill",
            Self::FeeOverflow => "FeeOverflow",
            Self::NegativeFeeRate => "NegativeFeeRate",
            Self::OrderNotFound => "OrderNotFound",
            Self::CancelOwnerMismatch => "CancelOwnerMismatch",
            Self::RuntimeRejected => "RuntimeRejected",
            Self::DuplicateClientOrderId => "DuplicateClientOrderId",
            Self::MarginTableEmpty => "MarginTableEmpty",
            Self::NotionalExceedsAllTiers => "NotionalExceedsAllTiers",
            Self::InitialMarginOverflow => "InitialMarginOverflow",
            Self::ReduceOnlyWouldOpenOrFlip => "ReduceOnlyWouldOpenOrFlip",
            Self::LeverageOutOfRange => "LeverageOutOfRange",
            Self::OpenOrdersOnMarket => "OpenOrdersOnMarket",
            Self::PositionNotFlat => "PositionNotFlat",
            Self::NotIsolatedMarginMode => "NotIsolatedMarginMode",
            Self::IsolatedMarginNonPositive => "IsolatedMarginNonPositive",
            Self::IsolatedMarginInsufficientBalance => "IsolatedMarginInsufficientBalance",
            Self::IsolatedMarginWithdrawTooLarge => "IsolatedMarginWithdrawTooLarge",
            Self::LiquidateTargetNotUser => "LiquidateTargetNotUser",
            Self::LiquidateNoPosition => "LiquidateNoPosition",
            Self::LiquidateHealthy => "LiquidateHealthy",
            Self::MarkPriceUnavailable => "MarkPriceUnavailable",
            Self::UnknownRoleBit => "UnknownRoleBit",
            Self::MarketAlreadyExists => "MarketAlreadyExists",
            Self::InvalidMarketConfig => "InvalidMarketConfig",
            Self::LifecycleTransitionInvalid => "LifecycleTransitionInvalid",
            Self::DelistTimelockNotElapsed => "DelistTimelockNotElapsed",
            Self::InvalidMarketConfigAmend => "InvalidMarketConfigAmend",
            Self::ReferralCodeNotFound => "ReferralCodeNotFound",
            Self::ReferralCodeAlreadyExists => "ReferralCodeAlreadyExists",
            Self::ReferralCodeInvalid => "ReferralCodeInvalid",
            Self::ReferralSelf => "ReferralSelf",
            Self::ReferrerAlreadyBound => "ReferrerAlreadyBound",
            Self::ReferrerNotBound => "ReferrerNotBound",
            Self::ReferralCodeAlreadyRegistered => "ReferralCodeAlreadyRegistered",
            Self::AmountNonPositive => "AmountNonPositive",
            Self::DuplicateExternalDeposit => "DuplicateExternalDeposit",
            Self::UnknownOrConsumedDepositId => "UnknownOrConsumedDepositId",
            Self::SettlementPaused => "SettlementPaused",
            Self::WithdrawRequestNotFound => "WithdrawRequestNotFound",
            Self::WithdrawRequestNotPending => "WithdrawRequestNotPending",
            Self::WithdrawAvailableInsufficient => "WithdrawAvailableInsufficient",
            Self::OwnerSignerMismatch => "OwnerSignerMismatch",
            Self::AgentAlreadyRegistered => "AgentAlreadyRegistered",
            Self::AgentLimitExceeded => "AgentLimitExceeded",
            Self::AgentRoleNotSubset => "AgentRoleNotSubset",
            Self::AgentNotFound => "AgentNotFound",
            Self::TriggerNotFound => "TriggerNotFound",
            Self::TriggerOwnerMismatch => "TriggerOwnerMismatch",
            Self::TriggerPriceNonPositive => "TriggerPriceNonPositive",
            Self::TriggerWouldExecuteImmediately => "TriggerWouldExecuteImmediately",
            Self::TriggerCountCapExceeded => "TriggerCountCapExceeded",
            Self::TriggerLimitUnreachableAtFire => "TriggerLimitUnreachableAtFire",
            Self::GtdExpiryInPast => "GtdExpiryInPast",
            Self::MarketEmergencyHalt => "MarketEmergencyHalt",
            Self::OcoCrossMarketUnsupported => "OcoCrossMarketUnsupported",
            Self::OcoDuplicateLeg => "OcoDuplicateLeg",
            Self::OcoActivePairsCapExceeded => "OcoActivePairsCapExceeded",
            Self::OcoLegValidationFailed => "OcoLegValidationFailed",
            Self::OcoPairNotFound => "OcoPairNotFound",
            Self::OcoNotOwner => "OcoNotOwner",
            Self::OcoAlreadyResolved => "OcoAlreadyResolved",
            Self::BracketParentNotFound => "BracketParentNotFound",
            Self::BracketParentMismatch => "BracketParentMismatch",
            Self::BracketParentAlreadyLinked => "BracketParentAlreadyLinked",
            Self::BracketParentReduceOnly => "BracketParentReduceOnly",
            Self::BracketParentAmbiguous => "BracketParentAmbiguous",
            Self::BracketInvalidOcoKind => "BracketInvalidOcoKind",
            Self::QuoteNotAvailable => "QuoteNotAvailable",
            Self::QuoteBidAskInvalid => "QuoteBidAskInvalid",
            Self::MarkPriceOutOfSpread => "MarkPriceOutOfSpread",
            Self::QuoteSequenceNotMonotonic => "QuoteSequenceNotMonotonic",
            Self::QuoteRateLimited => "QuoteRateLimited",
            Self::QuoteChangeTooLarge => "QuoteChangeTooLarge",
            Self::QuotePriceOutOfBounds => "QuotePriceOutOfBounds",
            Self::QuoteSpreadTooWide => "QuoteSpreadTooWide",
            Self::QuoteSourceTooStale => "QuoteSourceTooStale",
            Self::QuoteMarketNotExternalPeg => "QuoteMarketNotExternalPeg",
            Self::QuoteQuoterMismatch => "QuoteQuoterMismatch",
            Self::AmendOrderNotResting => "AmendOrderNotResting",
            Self::AmendNoChange => "AmendNoChange",
            Self::AmendTriggerNoChange => "AmendTriggerNoChange",
            Self::AmendTriggerPayloadMismatch => "AmendTriggerPayloadMismatch",
            Self::MassCancelTooLarge => "MassCancelTooLarge",
            Self::ClosePositionNoPosition => "ClosePositionNoPosition",
            Self::ClosePositionQtyExceedsPosition => "ClosePositionQtyExceedsPosition",
            Self::OrderTargetInvalid => "OrderTargetInvalid",
            Self::BatchSizeInvalid => "BatchSizeInvalid",
            Self::DmsTriggerTooSoon => "DmsTriggerTooSoon",
            Self::ExecutionFault => "ExecutionFault",
            Self::FokRejected => "FokRejected",
            Self::RebateRatioOwnerNotReferrer => "RebateRatioOwnerNotReferrer",
            Self::MissingAccountForInviterKeepRatio => "MissingAccountForInviterKeepRatio",
        }
    }
}

fn json_field_to_string(value: Option<&Value>, field: &str) -> Result<String, String> {
    match value {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(n.to_string()),
        Some(other) => Err(format!("{field} must be string or number, got {other}")),
        None => Err(format!("{field} missing")),
    }
}

/// `Exec::OrderDone.reason` (PascalCase wire tag).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DoneReason {
    Filled,
    IocExpired,
    Cancelled,
    GtdExpired,
    FokRejected,
    MarketDelisted,
    AmendClampedAtFilled,
}

/// Trigger / OCO trigger-leg cancel reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TriggerCancelReason {
    ByOwner,
    OnFailure,
    MarketDelisted,
    OcoSibling,
}

/// OCO pair resolution reason (`Oco` / `Trigger` domains).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OcoResolveReason {
    OrderResolved,
    TriggerFired,
    ManualCancel,
    PairCancelled,
    TriggerExpired,
}
