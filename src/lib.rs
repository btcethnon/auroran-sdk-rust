//! # auroran-sdk-rust — standalone off-chain client + signing for Auroran chain
#![allow(clippy::too_many_arguments)]
//!
//! Zero internal dependencies. Self-contained wire type mirrors, EIP-712
//! dual-channel signing (L1 + User-Signed), JSON-RPC 2.0 client, and
//! WebSocket streaming.
//!
//! ## Examples
//!
//! Runnable examples live under `examples/`:
//!
//! ```text
//! cargo run --example query          # read API (no key)
//! cargo run --example place_order    # L1 sign + submit
//! cargo run --example websocket      # WS subscribe
//! cargo run --example block_events   # WS block + RPC events
//! cargo run --example register_agent # User-Signed EIP-712
//! cargo run --example async_query --features async
//! ```
//!
//! Common env vars: `AURORAN_RPC_URL`, `AURORAN_CHAIN_ID`, `AURORAN_NETWORK_TAG`,
//! `AURORAN_PRIVATE_KEY`. See each example file for details.

pub mod api;
mod client_common;
pub mod builders;
pub mod client;
mod signed_action;
mod error;
pub mod events;
pub mod helpers;
mod routes;
pub mod signing;
pub mod wire;
mod ws;
mod ws_events;

#[cfg(feature = "async")]
pub mod async_helpers;

#[cfg(feature = "async")]
mod async_client;

// ── Wire mirrors ─────────────────────────────────────────────────────────

pub use wire::{
    eip712, format_decimal, parse_decimal, scale_int, AccountRole, Action, Address20,
    AddressParseError, ChainEnvelope, ChainId, ClientOrderId, DecimalStr, DepositId, DepositSeq,
    ExternalDepositRef, ExternalTxRef, IdemKey, MarginMode, MarginTier, MarketConfig, MarketId, MarketKind,
    MarketLifecycle, OcoExecution, OcoLimitLeg, OcoStopLimitLeg, OcoStopMarketLeg, OracleQuote,
    OrderId, PairId, Side, SigCredential, SignedActionEnvelope, StopTriggerLeg, TimeInForce,
    TriggerDirection, TriggerKind, TriggerOrderId, TriggerOrderPayload, WithdrawRequestId,
    ACTION_VERSION_V2, DECIMALS_6, MAX_DECIMALS, SCALE_6,
};
pub use wire::{
    AmendMarketConfigAction, AmendOrderAction, AmendTriggerOrderAction, BatchAmendTriggerAction,
    BatchModifyAction, BatchPlaceOrderAction, BatchSubmitOracleQuoteAction, CancelOcoAction,
    CancelOrderAction, CancelTriggerOrderAction, ClosePositionAction, CreateMarketAction,
    CreditDepositAction, LiquidateAction, MassCancelAction, MassCancelScopeAction, PlaceOcoAction,
    PlaceOrderAction, PlaceTriggerOrderAction, RecordDepositAction, RegisterAgentAction,
    RegisterReferrerAction, RevokeAgentAction, ScheduleCancelAction, SetAccountRebateRatioAction,
    SetAccountRoleAction, SetEmergencyHaltAction, SetFeeRecipientAction,
    SetGlobalRebateRatioAction, SetInviterKeepRatioAction, SetIsolatedMarginAction,
    SetLeverageAction, SetMarginModeAction, SetReferrerAction, SetSettlementPausedAction,
    SetUserFeeRateAction, SimpleMarketAction, SubmitOracleQuoteAction, WithdrawRefundAction,
    WithdrawRequestAction, WithdrawSettleAction,
};

// ── Signing ───────────────────────────────────────────────────────────────

#[cfg(feature = "test-support")]
pub use signing::test_keys::{
    test_agent_address, test_agent_signing_key, test_master_address, test_master_signing_key,
};
pub use signing::{
    address_from_verifying_key, generate_signing_key, l1_connection_id, l1_digest,
    register_agent_digest, revoke_agent_digest, secp256k1_from_hex, sign_action, sign_l1_action,
    sign_register_agent, sign_revoke_agent, sign_withdraw, signing_key_to_hex, withdraw_digest,
    L1_ACTION_TYPE, REGISTER_AGENT_TYPE, REVOKE_AGENT_TYPE, WITHDRAW_TYPE,
};

// ── Envelope ──────────────────────────────────────────────────────────────

pub use signed_action::{agent_signed_envelope, master_signed_envelope, master_signed_envelope_for};

// ── Client ────────────────────────────────────────────────────────────────

pub use client::{
    AccountListItem, AccountOrdersResponse, AccountSummaryResponse, ActionMetaItem,
    ActionsMetaResponse, AdminAuditEntry, AgentResponse, AllBboItem, AllOcoPairsResponse,
    AllOpenOrdersResponse, AllTriggerOrdersResponse, AuroranClient, BlockEnvelopeView,
    BlockEventsResponse, BlockResponse, BookView, BootstrapResponse, BridgeDepositResponse,
    BridgeDepositsListResponse, BridgeSettlementResponse, BridgeWithdrawalResponse,
    BridgeWithdrawalsListResponse, CandleResponse, CloseReason, DepositRecord, DepositStatus,
    EstimatedLiquidationResponse, ExchangeConfigResponse, FillRecord, GlobalStatsResponse,
    HealthResponse, HttpExchange, LiquidatablePosition, ListAccountsFilter, ListAccountsResponse, MarketDetailResponse,
    MarketListItem, MarketStatsHistoryResponse, MarketStatsRecord, MarketSummaryResponse, OcoLegs,
    OcoPairResponse, OcoPairsResponse, OcoStatus, OrderLifecycleStatus, OrderStatusResponse,
    OrderbookLevelResponse, OrderbookResponse, Page, PositionRecord, QueryResult,
    QuoteHistorySample, ReferralResponse, RestingOrderSummary, StatsHistorySample, TopAccountItem,
    TradeResponse, TriggerOrderResponse, TriggerOrderType, TriggerOrdersResponse,
    TxReceiptResponse, UserFeesResponse, UserFillResponse, UserRateLimitResponse, WithdrawRecord,
    WithdrawStatus, DEFAULT_TIMEOUT_SECS,
};

/// 测试构建专用：进程级「已提交交易」内存日志（每笔 `submit_action` 自动记录）。
#[cfg(feature = "test-support")]
pub use client::tx_recorder;

// ── Builders ──────────────────────────────────────────────────────────────

pub use builders::{
    activate_market, amend_market_config, amend_order, amend_order_full, amend_trigger_order,
    amend_trigger_order_full, batch_amend_trigger, batch_modify, batch_place_order,
    batch_submit_oracle_quote, cancel_oco, cancel_order, cancel_trigger_order, close_position_full,
    close_position_market, complete_delist, create_market, credit_deposit, halt_market, liquidate,
    mass_cancel_ids, mass_cancel_owner, mass_cancel_side, place_oco, place_order, place_order_full,
    place_order_with_coid, place_trigger_order, record_deposit, record_deposit_with_meta,
    register_agent, register_referrer,
    request_delist, resume_market, revoke_agent, schedule_cancel, set_account_rebate_ratio,
    set_account_role, set_emergency_halt, set_fee_recipient, set_global_rebate_ratio,
    set_inviter_keep_ratio, set_isolated_margin, set_leverage, set_margin_mode, set_referrer,
    set_settlement_paused, set_user_fee_rate, submit_oracle_quote, withdraw_refund,
    withdraw_request, withdraw_settle, withdraw_settle_with_tx,
};

// ── Helpers ───────────────────────────────────────────────────────────────

pub use helpers::{
    flatten_account, has_symbol_position, open_position_symbols, order_symbols, poll_config_from_env,
    position_is_open, set_leverage_if_needed, size_decimals_for, submit_accepted, symbol_leverage,
    wait_for_account, wait_for_flat_orders, wait_for_flat_positions, wait_for_leverage,
    FlattenResult, PollConfig, SigningConfig,
};

// ── Events ────────────────────────────────────────────────────────────────

pub use events::{
    events_in_domain, events_with_path, fetch_all_block_events, fetch_block_events,
    find_deposit_credited, find_filled, find_leverage_updated, find_rejected, find_withdraw_settled,
    parse_block_events_response, parse_event, parse_events, parse_receipt_events, DoneReason,
    EventDomain, EventEnvelope, EventKind, OcoResolveReason, RejectReason, TriggerCancelReason,
};

#[cfg(feature = "async")]
pub use events::{fetch_all_block_events_async, fetch_block_events_async};

pub use ws_events::events_for_block_tip;

#[cfg(feature = "async")]
pub use ws_events::events_for_block_tip_async;

#[cfg(feature = "async")]
pub use async_client::AsyncAuroranClient;

pub use routes::{
    GET_HEALTH, GET_MARKETS, GET_MARKS, GET_ORDERBOOK, GET_ORDERBOOK_STATS, POST_ACTION, QUERY,
    WS_ENTRY,
};

// ── Error ─────────────────────────────────────────────────────────────────

pub use error::{ClientError, JsonRpcError};

// ── WebSocket ─────────────────────────────────────────────────────────────

pub use ws::{
    parse_ws_message, topics as ws_topics, AccountPush, BboPush, BlockTipPush, BookPush,
    ExternalQuotePush, MarksPush, OrderUpdateItem, OrderUpdateKind, OrderUpdatesPush, SubscribeAck,
    TradeItem, TradesPush, TriggerUpdateItem, TriggerUpdateKind, TriggerUpdatesPush, UserFillItem,
    UserFillsPush, WsClient, WsError, WsMessage,
};
