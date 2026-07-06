//! Convenience constructors for all 44 `Action` variants.

use crate::wire::{
    AccountRole, Action, Address20, AmendMarketConfigAction, AmendOrderAction,
    AmendTriggerOrderAction, BatchAmendTriggerAction, BatchModifyAction,
    BatchSubmitOracleQuoteAction, CancelOcoAction, CancelOrderAction, CancelTriggerOrderAction,
    ClientOrderId, ClosePositionAction, CreateMarketAction, CreditDepositAction, DecimalStr,
    DepositSeq, ExternalDepositRef, ExternalTxRef, IdemKey, LiquidateAction, MarginMode, MarketConfig,
    MassCancelAction, MassCancelScopeAction, OcoExecution, PairId, PlaceOcoAction,
    PlaceOrderAction, PlaceTriggerOrderAction, RecordDepositAction, RegisterAgentAction,
    RegisterReferrerAction, RevokeAgentAction, ScheduleCancelAction, SetAccountRebateRatioAction,
    SetAccountRoleAction, SetEmergencyHaltAction, SetFeeRecipientAction,
    SetGlobalRebateRatioAction, SetInviterKeepRatioAction, SetIsolatedMarginAction,
    SetLeverageAction, SetMarginModeAction, SetReferrerAction, SetSettlementPausedAction,
    SetUserFeeRateAction, Side, SimpleMarketAction, SubmitOracleQuoteAction, TimeInForce,
    TriggerDirection, TriggerOrderId, TriggerOrderPayload, WithdrawRefundAction,
    WithdrawRequestAction, WithdrawRequestId, WithdrawSettleAction,
};

// ── Trading ────────────────────────────────────────────────────────────────

pub fn place_order(
    owner: Address20,
    symbol: impl Into<String>,
    side: Side,
    limit_price: impl Into<DecimalStr>,
    qty: impl Into<DecimalStr>,
    tif: TimeInForce,
) -> Action {
    Action::PlaceOrder(PlaceOrderAction {
        owner,
        symbol: symbol.into(),
        side,
        limit_price: limit_price.into(),
        qty: qty.into(),
        tif,
        client_order_id: None,
        reduce_only: false,
        expires_at_ms: None,
    })
}

pub fn place_order_full(action: PlaceOrderAction) -> Action {
    Action::PlaceOrder(action)
}

pub fn place_order_with_coid(
    owner: Address20,
    symbol: impl Into<String>,
    side: Side,
    limit_price: impl Into<DecimalStr>,
    qty: impl Into<DecimalStr>,
    tif: TimeInForce,
    client_order_id: impl Into<ClientOrderId>,
) -> Action {
    Action::PlaceOrder(PlaceOrderAction {
        owner,
        symbol: symbol.into(),
        side,
        limit_price: limit_price.into(),
        qty: qty.into(),
        tif,
        client_order_id: Some(client_order_id.into()),
        reduce_only: false,
        expires_at_ms: None,
    })
}

pub fn cancel_order(
    owner: Address20,
    symbol: Option<impl Into<String>>,
    order_id: Option<u64>,
    client_order_id: Option<ClientOrderId>,
) -> Action {
    Action::CancelOrder(CancelOrderAction {
        owner,
        symbol: symbol.map(|s| s.into()),
        order_id,
        client_order_id,
    })
}

pub fn amend_order(
    owner: Address20,
    symbol: impl Into<String>,
    order_id: Option<u64>,
    client_order_id: Option<ClientOrderId>,
    new_qty: Option<impl Into<DecimalStr>>,
) -> Action {
    Action::AmendOrder(AmendOrderAction {
        owner,
        symbol: symbol.into(),
        order_id,
        client_order_id,
        new_limit_price: None,
        new_qty: new_qty.map(|v| v.into()),
        new_tif: None,
        new_reduce_only: None,
    })
}

pub fn amend_order_full(action: AmendOrderAction) -> Action {
    Action::AmendOrder(action)
}

pub fn mass_cancel_owner(owner: Address20, symbol: impl Into<String>) -> Action {
    Action::MassCancel(MassCancelAction {
        owner,
        symbol: symbol.into(),
        scope: MassCancelScopeAction::Owner,
    })
}

pub fn mass_cancel_side(owner: Address20, symbol: impl Into<String>, side: Side) -> Action {
    Action::MassCancel(MassCancelAction {
        owner,
        symbol: symbol.into(),
        scope: MassCancelScopeAction::Side(side),
    })
}

pub fn mass_cancel_ids(owner: Address20, symbol: impl Into<String>, order_ids: Vec<u64>) -> Action {
    Action::MassCancel(MassCancelAction {
        owner,
        symbol: symbol.into(),
        scope: MassCancelScopeAction::Ids(order_ids),
    })
}

pub fn close_position_market(owner: Address20, symbol: impl Into<String>) -> Action {
    Action::ClosePosition(ClosePositionAction {
        owner,
        symbol: symbol.into(),
        qty: None,
        limit_price: None,
        tif: None,
        client_order_id: None,
    })
}

pub fn close_position_full(action: ClosePositionAction) -> Action {
    Action::ClosePosition(action)
}

pub fn schedule_cancel(owner: Address20, trigger_time_ms: Option<u64>) -> Action {
    Action::ScheduleCancel(ScheduleCancelAction {
        owner,
        trigger_time_ms,
    })
}

// ── Margin ─────────────────────────────────────────────────────────────────

pub fn set_leverage(owner: Address20, symbol: impl Into<String>, leverage: u32) -> Action {
    Action::SetLeverage(SetLeverageAction {
        owner,
        symbol: symbol.into(),
        leverage,
    })
}

pub fn set_margin_mode(owner: Address20, symbol: impl Into<String>, mode: MarginMode) -> Action {
    Action::SetMarginMode(SetMarginModeAction {
        owner,
        symbol: symbol.into(),
        margin_mode: mode,
    })
}

pub fn set_isolated_margin(
    owner: Address20,
    symbol: impl Into<String>,
    amount: impl Into<DecimalStr>,
) -> Action {
    Action::SetIsolatedMargin(SetIsolatedMarginAction {
        owner,
        symbol: symbol.into(),
        amount: amount.into(),
    })
}

// ── Market lifecycle / admin ───────────────────────────────────────────────

pub fn create_market(config: MarketConfig) -> Action {
    Action::CreateMarket(CreateMarketAction { config })
}

pub fn activate_market(symbol: impl Into<String>) -> Action {
    Action::ActivateMarket(SimpleMarketAction {
        symbol: symbol.into(),
    })
}

pub fn halt_market(symbol: impl Into<String>) -> Action {
    Action::HaltMarket(SimpleMarketAction {
        symbol: symbol.into(),
    })
}

pub fn resume_market(symbol: impl Into<String>) -> Action {
    Action::ResumeMarket(SimpleMarketAction {
        symbol: symbol.into(),
    })
}

pub fn request_delist(symbol: impl Into<String>) -> Action {
    Action::RequestDelist(SimpleMarketAction {
        symbol: symbol.into(),
    })
}

pub fn complete_delist(symbol: impl Into<String>) -> Action {
    Action::CompleteDelist(SimpleMarketAction {
        symbol: symbol.into(),
    })
}

pub fn set_fee_recipient(symbol: impl Into<String>, recipient: Address20) -> Action {
    Action::SetFeeRecipient(SetFeeRecipientAction {
        symbol: symbol.into(),
        recipient,
    })
}

pub fn amend_market_config(
    symbol: impl Into<String>,
    max_leverage: Option<u32>,
    maker_fee_rate: Option<impl Into<DecimalStr>>,
    taker_fee_rate: Option<impl Into<DecimalStr>>,
    margin_table: Option<Vec<crate::wire::MarginTier>>,
    mark_max_change_bps: Option<u32>,
) -> Action {
    Action::AmendMarketConfig(AmendMarketConfigAction {
        symbol: symbol.into(),
        max_leverage,
        maker_fee_rate: maker_fee_rate.map(|v| v.into()),
        taker_fee_rate: taker_fee_rate.map(|v| v.into()),
        margin_table,
        mark_max_change_bps,
    })
}

pub fn set_emergency_halt(symbol: impl Into<String>, halt: bool) -> Action {
    Action::SetEmergencyHalt(SetEmergencyHaltAction {
        symbol: symbol.into(),
        halt,
    })
}

pub fn set_account_role(target: Address20, role: AccountRole, granted: bool) -> Action {
    Action::SetAccountRole(SetAccountRoleAction {
        target,
        role,
        granted,
    })
}

pub fn set_user_fee_rate(
    owner: Address20,
    maker_fee_rate: Option<impl Into<DecimalStr>>,
    taker_fee_rate: Option<impl Into<DecimalStr>>,
) -> Action {
    Action::SetUserFeeRate(SetUserFeeRateAction {
        owner,
        maker_fee_rate: maker_fee_rate.map(|v| v.into()),
        taker_fee_rate: taker_fee_rate.map(|v| v.into()),
    })
}

pub fn set_referrer(owner: Address20, code: impl Into<String>) -> Action {
    Action::SetReferrer(SetReferrerAction {
        owner,
        code: code.into(),
    })
}

pub fn register_referrer(owner: Address20, code: impl Into<String>) -> Action {
    Action::RegisterReferrer(RegisterReferrerAction {
        owner,
        code: code.into(),
    })
}

pub fn set_global_rebate_ratio(ratio_bps: u32) -> Action {
    Action::SetGlobalRebateRatio(SetGlobalRebateRatioAction { ratio_bps })
}

pub fn set_account_rebate_ratio(owner: Address20, ratio_bps: Option<u32>) -> Action {
    Action::SetAccountRebateRatio(SetAccountRebateRatioAction { owner, ratio_bps })
}

pub fn set_inviter_keep_ratio(owner: Address20, ratio_bps: u32) -> Action {
    Action::SetInviterKeepRatio(SetInviterKeepRatioAction { owner, ratio_bps })
}

// ── Liquidation ────────────────────────────────────────────────────────────

pub fn liquidate(target: Address20, symbol: impl Into<String>) -> Action {
    Action::Liquidate(LiquidateAction {
        target,
        symbol: symbol.into(),
    })
}

// ── Oracle ─────────────────────────────────────────────────────────────────

pub fn submit_oracle_quote(
    quoter: Address20,
    symbol: impl Into<String>,
    bid_price: impl Into<DecimalStr>,
    ask_price: impl Into<DecimalStr>,
    mark_price: impl Into<DecimalStr>,
    source_ts_ms: u64,
    sequence_id: u64,
    last_price: impl Into<DecimalStr>,
    volume: impl Into<DecimalStr>,
) -> Action {
    Action::SubmitOracleQuote(SubmitOracleQuoteAction {
        quoter,
        symbol: symbol.into(),
        bid_price: bid_price.into(),
        ask_price: ask_price.into(),
        mark_price: mark_price.into(),
        source_ts_ms,
        sequence_id,
        last_price: last_price.into(),
        volume: volume.into(),
    })
}

pub fn batch_submit_oracle_quote(quotes: Vec<SubmitOracleQuoteAction>) -> Action {
    Action::BatchSubmitOracleQuote(BatchSubmitOracleQuoteAction { quotes })
}

// ── Bridge ─────────────────────────────────────────────────────────────────

pub fn record_deposit(
    external_ref: ExternalDepositRef,
    account: Address20,
    amount: impl Into<DecimalStr>,
) -> Action {
    record_deposit_with_meta(external_ref, account, amount, None, 0, 0)
}

/// Full [`RecordDeposit`] builder with optional idempotency key and external-chain metadata.
pub fn record_deposit_with_meta(
    external_ref: ExternalDepositRef,
    account: Address20,
    amount: impl Into<DecimalStr>,
    tx_hash: Option<IdemKey>,
    bsc_block: u64,
    bsc_ts: u64,
) -> Action {
    Action::RecordDeposit(RecordDepositAction {
        external_ref,
        tx_hash,
        account,
        amount: amount.into(),
        bsc_block,
        bsc_ts,
    })
}

pub fn credit_deposit(seq: DepositSeq) -> Action {
    Action::CreditDeposit(CreditDepositAction { seq })
}

pub fn withdraw_request(
    network_name: impl Into<String>,
    owner: Address20,
    amount: impl Into<DecimalStr>,
    chain: impl Into<String>,
) -> Action {
    Action::WithdrawRequest(WithdrawRequestAction {
        network_name: network_name.into(),
        owner,
        amount: amount.into(),
        chain: chain.into().to_lowercase(),
    })
}

pub fn withdraw_settle(request_id: WithdrawRequestId, tx_hash: [u8; 32]) -> Action {
    withdraw_settle_with_tx(
        request_id,
        ExternalTxRef {
            tx_hash,
            bsc_block: 0,
            bsc_ts: 0,
        },
    )
}

/// Full [`WithdrawSettle`] builder with external-chain tx metadata.
pub fn withdraw_settle_with_tx(
    request_id: WithdrawRequestId,
    external_tx: ExternalTxRef,
) -> Action {
    Action::WithdrawSettle(WithdrawSettleAction {
        request_id,
        external_tx,
    })
}

pub fn withdraw_refund(request_id: WithdrawRequestId, reason_code: u8) -> Action {
    Action::WithdrawRefund(WithdrawRefundAction {
        request_id,
        reason_code,
    })
}

pub fn set_settlement_paused(paused: bool) -> Action {
    Action::SetSettlementPaused(SetSettlementPausedAction { paused })
}

// ── Triggers ───────────────────────────────────────────────────────────────

pub fn place_trigger_order(
    owner: Address20,
    symbol: impl Into<String>,
    trigger_price: impl Into<DecimalStr>,
    trigger_direction: TriggerDirection,
    payload: TriggerOrderPayload,
    expires_at_ms: Option<u64>,
) -> Action {
    Action::PlaceTriggerOrder(PlaceTriggerOrderAction {
        owner,
        symbol: symbol.into(),
        trigger_price: trigger_price.into(),
        trigger_direction,
        payload,
        expires_at_ms,
    })
}

pub fn cancel_trigger_order(owner: Address20, trigger_id: TriggerOrderId) -> Action {
    Action::CancelTriggerOrder(CancelTriggerOrderAction { owner, trigger_id })
}

pub fn amend_trigger_order_full(action: AmendTriggerOrderAction) -> Action {
    Action::AmendTriggerOrder(action)
}

pub fn amend_trigger_order(
    owner: Address20,
    trigger_id: TriggerOrderId,
    new_trigger_price: Option<DecimalStr>,
    new_qty: Option<DecimalStr>,
    new_limit_price: Option<DecimalStr>,
    new_tif: Option<TimeInForce>,
    new_reduce_only: Option<bool>,
    new_expires_at_ms: Option<Option<u64>>,
) -> Action {
    Action::AmendTriggerOrder(AmendTriggerOrderAction {
        owner,
        trigger_id,
        new_trigger_price,
        new_qty,
        new_limit_price,
        new_tif,
        new_reduce_only,
        new_expires_at_ms,
    })
}

pub fn batch_amend_trigger(amends: Vec<AmendTriggerOrderAction>) -> Action {
    Action::BatchAmendTrigger(BatchAmendTriggerAction { amends })
}

pub fn place_oco(owner: Address20, execution: OcoExecution, client_pair_id: Option<u64>) -> Action {
    Action::PlaceOco(PlaceOcoAction {
        owner,
        execution,
        client_pair_id,
    })
}

pub fn cancel_oco(owner: Address20, pair_id: PairId) -> Action {
    Action::CancelOco(CancelOcoAction { owner, pair_id })
}

// ── Agent ──────────────────────────────────────────────────────────────────

pub fn register_agent(
    network_name: impl Into<String>,
    owner: Address20,
    agent_address: Address20,
    role_mask: u64,
    expires_at_ms: u64,
) -> Action {
    Action::RegisterAgent(RegisterAgentAction {
        network_name: network_name.into(),
        owner,
        agent_address,
        role_mask,
        expires_at_ms,
    })
}

pub fn revoke_agent(
    network_name: impl Into<String>,
    owner: Address20,
    agent_address: Address20,
) -> Action {
    Action::RevokeAgent(RevokeAgentAction {
        network_name: network_name.into(),
        owner,
        agent_address,
    })
}

// ── Batch ────────────────────────────────────────────────────────────────────

pub fn batch_place_order(orders: Vec<PlaceOrderAction>) -> Action {
    Action::BatchPlaceOrder(crate::wire::BatchPlaceOrderAction { orders })
}

pub fn batch_modify(modifies: Vec<AmendOrderAction>) -> Action {
    Action::BatchModify(BatchModifyAction { modifies })
}
