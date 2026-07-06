//! JSON-RPC 2.0 client for Auroran chain.
//!
//! Write: `POST /api/v1/action` with `{jsonrpc:"2.0", id, method, params:{envelope}}`.
//! Read:  `POST /api/v1/query` with `{jsonrpc:"2.0", id, method, params}`.
//! REST:  `GET /api/v1/health`, `GET /api/v1/markets`, etc. (cacheable aliases).

mod action;
mod blocking;
mod methods;
pub(crate) mod transport;

#[cfg(feature = "test-support")]
pub mod tx_recorder;

pub use blocking::{AuroranClient, DEFAULT_TIMEOUT_SECS};
pub use transport::HttpExchange;

pub use crate::api::{
    AccountListItem, AccountOrdersResponse, AccountSummaryResponse, ActionMetaItem,
    ActionsMetaResponse, AdminAuditEntry, AgentResponse, AllBboItem, AllOcoPairsResponse,
    AllOpenOrdersResponse, AllTriggerOrdersResponse, BlockEnvelopeView, BlockEventsResponse,
    BlockResponse, BookView, BootstrapResponse, BridgeDepositResponse, BridgeDepositsListResponse,
    BridgeSettlementResponse, BridgeWithdrawalResponse, BridgeWithdrawalsListResponse,
    CandleResponse, CloseReason, DepositRecord, DepositStatus, EstimatedLiquidationResponse,
    ExchangeConfigResponse, FillRecord, GlobalStatsResponse, HealthResponse, LiquidatablePosition,
    ListAccountsFilter, ListAccountsResponse, MarketDetailResponse, MarketListItem,
    MarketStatsHistoryResponse, MarketStatsRecord, MarketSummaryResponse, OcoLegs, OcoPairResponse,
    OcoPairsResponse, OcoStatus, OrderLifecycleStatus, OrderStatusResponse,
    OrderbookLevelResponse, OrderbookResponse, PositionRecord, QuoteHistorySample, ReferralResponse,
    RestingOrderSummary, StatsHistorySample, TopAccountItem, TradeResponse, TriggerOrderResponse,
    TriggerOrderType, TriggerOrdersResponse, TxReceiptResponse, UserFeesResponse, UserFillResponse,
    UserRateLimitResponse, WithdrawRecord, WithdrawStatus,
};

pub use crate::client_common::{Page, QueryResult};

pub(crate) use action::action_method_name;
