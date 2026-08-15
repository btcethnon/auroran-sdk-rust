//! Block / tx event envelope parsing (decimal-string projection from the node API).

mod kind;
mod reasons;
mod types;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{BlockEventsResponse, TxReceiptResponse};
use crate::client::AuroranClient;
use crate::error::ClientError;

pub use kind::EventKind;
pub use reasons::{DoneReason, OcoResolveReason, RejectReason, TriggerCancelReason};
pub use types::*;

/// Event domain tag (`Core`, `Exec`, `Bridge`, …).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventDomain {
    Core,
    Exec,
    Oco,
    Liquidation,
    Trigger,
    Bridge,
    Ops,
}

impl EventDomain {
    pub const ALL: [Self; 7] = [
        Self::Core,
        Self::Exec,
        Self::Oco,
        Self::Liquidation,
        Self::Trigger,
        Self::Bridge,
        Self::Ops,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::Exec => "Exec",
            Self::Oco => "Oco",
            Self::Liquidation => "Liquidation",
            Self::Trigger => "Trigger",
            Self::Bridge => "Bridge",
            Self::Ops => "Ops",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Core" => Some(Self::Core),
            "Exec" => Some(Self::Exec),
            "Oco" => Some(Self::Oco),
            "Liquidation" => Some(Self::Liquidation),
            "Trigger" => Some(Self::Trigger),
            "Bridge" => Some(Self::Bridge),
            "Ops" => Some(Self::Ops),
            _ => None,
        }
    }
}

/// Unified event shell (`getBlockEvents`, `getTx.events`, write `result.events`).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EventEnvelope {
    pub seq: u64,
    pub block_height: u64,
    pub envelope_idx: u32,
    pub kind: Value,
}

macro_rules! event_accessor {
    ($method:ident, $ty:ty, $domain:expr, $variant:expr) => {
        pub fn $method(&self) -> Option<$ty> {
            self.parse_at($domain, $variant)
        }
    };
}

impl EventEnvelope {
    /// Deserialize the full typed event tree (`Core` / `Exec` / … + payload).
    pub fn kind(&self) -> Option<EventKind> {
        serde_json::from_value(self.kind.clone()).ok()
    }

    /// Top-level domain tag, e.g. `"Exec"`, `"Core"`.
    pub fn domain(&self) -> Option<&str> {
        self.kind.as_object()?.keys().next().map(String::as_str)
    }

    pub fn domain_enum(&self) -> Option<EventDomain> {
        self.domain().and_then(EventDomain::parse)
    }

    /// Inner variant name within the domain, e.g. `"Filled"`, `"LeverageUpdated"`.
    pub fn variant(&self) -> Option<&str> {
        let outer = self.kind.as_object()?;
        let inner = outer.values().next()?;
        inner.as_object()?.keys().next().map(String::as_str)
    }

    /// `"Exec" / "Filled"`.
    pub fn path(&self) -> Option<(&str, &str)> {
        Some((self.domain()?, self.variant()?))
    }

    fn inner_payload(&self) -> Option<&Value> {
        let outer = self.kind.as_object()?;
        let inner = outer.values().next()?;
        let variant = self.variant()?;
        inner.get(variant)
    }

    /// Deserialize the inner variant payload when the envelope shape matches.
    pub fn try_payload<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.inner_payload()?.clone()).ok()
    }

    fn parse_at<T: DeserializeOwned>(&self, domain: &str, variant: &str) -> Option<T> {
        if self.path() != Some((domain, variant)) {
            return None;
        }
        self.try_payload()
    }

    event_accessor!(as_filled, FilledEvent, "Exec", "Filled");
    event_accessor!(as_leverage_updated, LeverageUpdatedEvent, "Exec", "LeverageUpdated");
    event_accessor!(as_order_accepted, OrderAcceptedEvent, "Exec", "OrderAccepted");
    event_accessor!(as_order_done, OrderDoneEvent, "Exec", "OrderDone");
    event_accessor!(as_order_resting, OrderRestingEvent, "Exec", "OrderResting");
    event_accessor!(as_position_updated, PositionUpdatedEvent, "Exec", "PositionUpdated");
    event_accessor!(as_rejected, RejectedEvent, "Core", "Rejected");
    event_accessor!(as_balance_changed, BalanceChangedEvent, "Core", "BalanceChanged");
    event_accessor!(as_rebate_paid, RebatePaidEvent, "Core", "RebatePaid");
    event_accessor!(as_deposit_credited, DepositCreditedEvent, "Bridge", "DepositCredited");
    event_accessor!(as_withdraw_settled, WithdrawSettledEvent, "Bridge", "WithdrawSettled");
    event_accessor!(as_liquidated, LiquidatedEvent, "Liquidation", "Liquidated");
    event_accessor!(as_trigger_activated, TriggerActivatedEvent, "Trigger", "TriggerActivated");
    event_accessor!(as_mark_updated, MarkUpdatedEvent, "Core", "MarkUpdated");
    event_accessor!(as_market_stats_snapshot, MarketStatsSnapshotEvent, "Core", "MarketStatsSnapshot");
    event_accessor!(as_batch_item_rejected, BatchItemRejectedEvent, "Core", "BatchItemRejected");
    event_accessor!(as_runtime_reject, RuntimeRejectEvent, "Core", "RuntimeReject");
    event_accessor!(as_bankruptcy, BankruptcyEvent, "Core", "Bankruptcy");
    event_accessor!(as_user_fee_rate_changed, UserFeeRateChangedEvent, "Core", "UserFeeRateChanged");
    event_accessor!(as_referrer_registered, ReferrerRegisteredEvent, "Core", "ReferrerRegistered");
    event_accessor!(as_referrer_bound, ReferrerBoundEvent, "Core", "ReferrerBound");
    event_accessor!(as_inviter_keep_ratio_changed, InviterKeepRatioChangedEvent, "Core", "InviterKeepRatioChanged");
    event_accessor!(as_dead_mans_switch_scheduled, DeadMansSwitchScheduledEvent, "Core", "DeadMansSwitchScheduled");
    event_accessor!(as_dead_mans_switch_triggered, DeadMansSwitchTriggeredEvent, "Core", "DeadMansSwitchTriggered");
    event_accessor!(as_position_flattened, PositionFlattenedEvent, "Exec", "PositionFlattened");
    event_accessor!(as_order_cancelled_by_position_flat, OrderCancelledByPositionFlatEvent, "Exec", "OrderCancelledByPositionFlat");
    event_accessor!(as_trigger_cancelled_by_position_flat, TriggerCancelledByPositionFlatEvent, "Exec", "TriggerCancelledByPositionFlat");
    event_accessor!(as_isolated_margin_updated, IsolatedMarginUpdatedEvent, "Exec", "IsolatedMarginUpdated");
    event_accessor!(as_margin_mode_updated, MarginModeUpdatedEvent, "Exec", "MarginModeUpdated");
    event_accessor!(as_position_force_closed_at_mark, PositionForceClosedAtMarkEvent, "Exec", "PositionForceClosedAtMark");
    event_accessor!(as_oracle_quote_accepted, OracleQuoteAcceptedEvent, "Exec", "OracleQuoteAccepted");
    event_accessor!(as_oracle_counter_depleted, OracleCounterDepletedEvent, "Exec", "OracleCounterDepleted");
    event_accessor!(as_market_halted_quote_stale, MarketHaltedQuoteStaleEvent, "Exec", "MarketHaltedQuoteStale");
    event_accessor!(as_market_resumed_after_quote, MarketResumedAfterQuoteEvent, "Exec", "MarketResumedAfterQuote");
    event_accessor!(as_quote_reject_storm, QuoteRejectStormEvent, "Exec", "QuoteRejectStorm");
    event_accessor!(as_order_cancelled_by_oco_sibling, OrderCancelledByOcoSiblingEvent, "Oco", "OrderCancelledByOcoSibling");
    event_accessor!(as_oco_trigger_order_cancelled, OcoTriggerOrderCancelledEvent, "Oco", "TriggerOrderCancelled");
    event_accessor!(as_oco_limit_trigger_order_cancelled, OcoLimitTriggerOrderCancelledEvent, "Oco", "LimitTriggerOrderCancelled");
    event_accessor!(as_oco_pair_resolved, OcoPairResolvedEvent, "Oco", "OcoPairResolved");
    event_accessor!(as_trigger_order_placed, TriggerOrderPlacedEvent, "Trigger", "TriggerOrderPlaced");
    event_accessor!(as_trigger_order_amended, TriggerOrderAmendedEvent, "Trigger", "TriggerOrderAmended");
    event_accessor!(as_trigger_order_cancelled, TriggerOrderCancelledEvent, "Trigger", "TriggerOrderCancelled");
    event_accessor!(as_trigger_limit_trigger_order_cancelled, TriggerLimitTriggerOrderCancelledEvent, "Trigger", "LimitTriggerOrderCancelled");
    event_accessor!(as_trigger_order_expired, TriggerOrderExpiredEvent, "Trigger", "TriggerOrderExpired");
    event_accessor!(as_trigger_fire_failed, TriggerFireFailedEvent, "Trigger", "TriggerFireFailed");
    event_accessor!(as_trigger_oco_pair_placed, TriggerOcoPairPlacedEvent, "Trigger", "OcoPairPlaced");
    event_accessor!(as_trigger_oco_pair_resolved, TriggerOcoPairResolvedEvent, "Trigger", "OcoPairResolved");
    event_accessor!(as_deposit_recorded, DepositRecordedEvent, "Bridge", "DepositRecorded");
    event_accessor!(as_withdraw_requested, WithdrawRequestedEvent, "Bridge", "WithdrawRequested");
    event_accessor!(as_withdraw_refunded, WithdrawRefundedEvent, "Bridge", "WithdrawRefunded");
    event_accessor!(as_settlement_paused_changed, SettlementPausedChangedEvent, "Bridge", "SettlementPausedChanged");
    event_accessor!(as_account_role_changed, AccountRoleChangedEvent, "Ops", "AccountRoleChanged");
    event_accessor!(as_agent_registered, AgentRegisteredEvent, "Ops", "AgentRegistered");
    event_accessor!(as_agent_revoked, AgentRevokedEvent, "Ops", "AgentRevoked");
    event_accessor!(as_emergency_halt_changed, EmergencyHaltChangedEvent, "Ops", "EmergencyHaltChanged");
    event_accessor!(as_market_created, MarketCreatedEvent, "Ops", "MarketCreated");
    event_accessor!(as_market_lifecycle_changed, MarketLifecycleChangedEvent, "Ops", "MarketLifecycleChanged");
    event_accessor!(as_market_cancelled, MarketCancelledEvent, "Ops", "MarketCancelled");
    event_accessor!(as_fee_recipient_changed, FeeRecipientChangedEvent, "Ops", "FeeRecipientChanged");
    event_accessor!(as_market_config_amended, MarketConfigAmendedEvent, "Ops", "MarketConfigAmended");
    event_accessor!(as_global_rebate_ratio_changed, GlobalRebateRatioChangedEvent, "Ops", "GlobalRebateRatioChanged");
    event_accessor!(as_account_rebate_ratio_changed, AccountRebateRatioChangedEvent, "Ops", "AccountRebateRatioChanged");
}

/// Parse JSON event values into typed envelopes (skips invalid entries).
pub fn parse_events(values: &[Value]) -> Vec<EventEnvelope> {
    values
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect()
}

/// Parse tx receipt events.
pub fn parse_receipt_events(receipt: &TxReceiptResponse) -> Vec<EventEnvelope> {
    parse_events(&receipt.events)
}

/// Events whose domain matches `domain`.
pub fn events_in_domain(
    events: &[EventEnvelope],
    domain: EventDomain,
) -> Vec<&EventEnvelope> {
    let tag = domain.as_str();
    events
        .iter()
        .filter(|ev| ev.domain() == Some(tag))
        .collect()
}

/// Events whose `(domain, variant)` path matches.
pub fn events_with_path<'a>(
    events: &'a [EventEnvelope],
    domain: &str,
    variant: &str,
) -> Vec<&'a EventEnvelope> {
    events
        .iter()
        .filter(|ev| ev.path() == Some((domain, variant)))
        .collect()
}

/// Find the first matching event in a receipt.
pub fn find_leverage_updated(receipt: &TxReceiptResponse) -> Option<LeverageUpdatedEvent> {
    parse_receipt_events(receipt)
        .into_iter()
        .find_map(|ev| ev.as_leverage_updated())
}

pub fn find_filled(receipt: &TxReceiptResponse) -> Option<FilledEvent> {
    parse_receipt_events(receipt)
        .into_iter()
        .find_map(|ev| ev.as_filled())
}

pub fn find_rejected(receipt: &TxReceiptResponse) -> Option<RejectedEvent> {
    parse_receipt_events(receipt)
        .into_iter()
        .find_map(|ev| ev.as_rejected())
}

pub fn find_deposit_credited(receipt: &TxReceiptResponse) -> Option<DepositCreditedEvent> {
    parse_receipt_events(receipt)
        .into_iter()
        .find_map(|ev| ev.as_deposit_credited())
}

pub fn find_withdraw_settled(receipt: &TxReceiptResponse) -> Option<WithdrawSettledEvent> {
    parse_receipt_events(receipt)
        .into_iter()
        .find_map(|ev| ev.as_withdraw_settled())
}

/// Strict parse of one event value.
pub fn parse_event(value: &Value) -> Result<EventEnvelope, ClientError> {
    serde_json::from_value(value.clone()).map_err(ClientError::Json)
}

/// Parse a `getBlockEvents` page into typed envelopes (skips invalid entries).
pub fn parse_block_events_response(resp: &BlockEventsResponse) -> Vec<EventEnvelope> {
    parse_events(&resp.events)
}

/// Fetch one page of block events as typed envelopes.
pub fn fetch_block_events(
    client: &AuroranClient,
    height: u64,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>, ClientError> {
    Ok(parse_block_events_response(&client.block_events(height, offset, limit)?))
}

/// Fetch all events for a block (follows pagination until `total` exhausted).
pub fn fetch_all_block_events(
    client: &AuroranClient,
    height: u64,
) -> Result<Vec<EventEnvelope>, ClientError> {
    const PAGE: usize = 500;
    let mut out = Vec::new();
    let mut offset = 0usize;
    loop {
        let page = client.block_events(height, Some(offset), Some(PAGE))?;
        let batch = parse_block_events_response(&page);
        let batch_len = batch.len();
        out.extend(batch);
        if batch_len == 0 || offset + batch_len >= page.total {
            break;
        }
        offset += batch_len;
    }
    Ok(out)
}

/// Async variant of [`fetch_all_block_events`].
#[cfg(feature = "async")]
pub async fn fetch_all_block_events_async(
    client: &crate::AsyncAuroranClient,
    height: u64,
) -> Result<Vec<EventEnvelope>, ClientError> {
    const PAGE: usize = 500;
    let mut out = Vec::new();
    let mut offset = 0usize;
    loop {
        let page = client.block_events(height, Some(offset), Some(PAGE)).await?;
        let batch = parse_block_events_response(&page);
        let batch_len = batch.len();
        out.extend(batch);
        if batch_len == 0 || offset + batch_len >= page.total {
            break;
        }
        offset += batch_len;
    }
    Ok(out)
}

/// Fetch one page of block events as typed envelopes (async).
#[cfg(feature = "async")]
pub async fn fetch_block_events_async(
    client: &crate::AsyncAuroranClient,
    height: u64,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>, ClientError> {
    Ok(parse_block_events_response(
        &client.block_events(height, offset, limit).await?,
    ))
}
