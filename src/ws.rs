//! WebSocket client and message types (`WS /api/v1/ws`).

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest, tungstenite::Message, MaybeTlsStream, WebSocketStream};

const DEFAULT_USER_AGENT: &str = crate::client_common::DEFAULT_USER_AGENT;

use crate::api::{
    MarketStatsRecord, OracleQuoteResponse, OrderbookLevelResponse, PositionRecord,
    TriggerOrderType,
};
use crate::error::ClientError;
use crate::events::{DoneReason, OcoResolveReason, RejectReason, TriggerCancelReason};
use crate::routes;
use crate::wire::{Address20, MarketId, Side, TimeInForce, TriggerDirection};

/// WebSocket topic helpers.
pub mod topics {
    pub fn block() -> String {
        "block".into()
    }
    pub fn blocks_live() -> String {
        "blocks.live".into()
    }
    pub fn candles(symbol: &str, interval: &str) -> String {
        format!("candles.{symbol}.{interval}")
    }
    pub fn book(symbol: &str) -> String {
        format!("book.{symbol}")
    }
    pub fn book_with_depth(symbol: &str, depth: usize) -> String {
        format!("book.{symbol}.{depth}")
    }
    pub fn trades(symbol: &str) -> String {
        format!("trades.{symbol}")
    }
    pub fn account_hex(address_hex: &str) -> String {
        format!("account.{address_hex}")
    }
    pub fn account(address: &crate::wire::Address20) -> String {
        account_hex(&hex::encode(address.as_bytes()))
    }
    pub fn external_quote(symbol: &str) -> String {
        format!("external_quote.{symbol}")
    }
    pub fn bbo(symbol: &str) -> String {
        format!("bbo.{symbol}")
    }
    pub fn user_fills_hex(address_hex: &str) -> String {
        format!("userFills.{address_hex}")
    }
    pub fn user_fills(address: &crate::wire::Address20) -> String {
        user_fills_hex(&hex::encode(address.as_bytes()))
    }
    pub fn order_updates_hex(address_hex: &str) -> String {
        format!("orderUpdates.{address_hex}")
    }
    pub fn order_updates(address: &crate::wire::Address20) -> String {
        order_updates_hex(&hex::encode(address.as_bytes()))
    }
    pub fn trigger_updates_hex(address_hex: &str) -> String {
        format!("triggerUpdates.{address_hex}")
    }
    pub fn trigger_updates(address: &crate::wire::Address20) -> String {
        trigger_updates_hex(&hex::encode(address.as_bytes()))
    }
    pub fn marks() -> String {
        "marks".into()
    }
}

#[derive(Debug, Clone, Serialize)]
struct SubscribeRequest {
    op: &'static str,
    topics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeAck {
    pub op: String,
    pub topics: Vec<String>,
    #[serde(default)]
    pub rejected: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsError {
    pub op: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockTipPush {
    pub topic: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub digest: String,
    pub state_root: String,
    pub envelope_count: usize,
    pub event_count: usize,
}

/// Full block header inside a `blocks.live` frame (Explorer browser-layer push).
#[derive(Debug, Clone, Deserialize)]
pub struct BlocksLiveBlockHeader {
    pub height: u64,
    pub digest: String,
    #[serde(default)]
    pub parent: Option<String>,
    pub timestamp_ms: u64,
    pub state_root: String,
    #[serde(default)]
    pub signer: Option<String>,
    pub envelope_count: u64,
    pub event_count: u64,
}

/// Slimmed envelope view inside a `blocks.live` frame (`strip_for_push`).
#[derive(Debug, Clone, Deserialize)]
pub struct BlocksLiveEnvelope {
    pub tx_hash: String,
    pub height: u64,
    pub idx: u64,
    pub action_kind: String,
    pub signer: String,
    pub timestamp_ms: u64,
}

/// Full new-block push (`{"op":"subscribe","topics":["blocks.live"]}`).
#[derive(Debug, Clone, Deserialize)]
pub struct BlocksLivePush {
    pub topic: String,
    pub block: BlocksLiveBlockHeader,
    #[serde(default)]
    pub envelopes: Vec<BlocksLiveEnvelope>,
    pub watermark: u64,
    pub block_count: u64,
    #[serde(default)]
    pub node_tip: Option<u64>,
    #[serde(default)]
    pub behind: Option<u64>,
}

/// Hyperliquid-compatible candle payload inside a `candles.*` push.
#[derive(Debug, Clone, Deserialize)]
pub struct HlCandle {
    pub t: u64,
    pub i: String,
    pub o: String,
    pub c: String,
    pub h: String,
    pub l: String,
    pub v: String,
    pub n: u64,
}

/// Real-time candle push (`candles.{symbol}.{interval}` topic).
#[derive(Debug, Clone, Deserialize)]
pub struct CandlePush {
    pub topic: String,
    pub data: HlCandle,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookPush {
    pub topic: String,
    pub symbol: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub state_hash: String,
    pub bids: Vec<OrderbookLevelResponse>,
    pub asks: Vec<OrderbookLevelResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeItem {
    pub block_height: u64,
    pub event_seq: u64,
    #[serde(default)]
    pub timestamp_ms: u64,
    pub market_id: MarketId,
    pub price: String,
    pub qty: String,
    pub notional: String,
    pub side: Side,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradesPush {
    pub topic: String,
    pub symbol: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub trades: Vec<TradeItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountPush {
    pub topic: String,
    pub address: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub balance: String,
    pub nonce: u64,
    pub role_mask: u64,
    #[serde(default)]
    pub account_value: String,
    #[serde(default)]
    pub total_margin_used: String,
    #[serde(default)]
    pub total_notional: String,
    #[serde(default)]
    pub withdrawable: String,
    #[serde(default)]
    pub cross_cash_available: String,
    #[serde(default)]
    pub cross_trading_available: String,
    pub positions: std::collections::BTreeMap<MarketId, PositionRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalQuotePush {
    pub topic: String,
    pub symbol: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub quote: OracleQuoteResponse,
    #[serde(default)]
    pub market_stats: Option<MarketStatsRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BboPush {
    pub topic: String,
    pub symbol: String,
    pub height: u64,
    pub timestamp_ms: u64,
    #[serde(default)]
    pub best_bid: Option<String>,
    #[serde(default)]
    pub best_ask: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserFillItem {
    pub block_height: u64,
    pub event_seq: u64,
    pub market_id: MarketId,
    pub price: String,
    pub qty: String,
    pub notional: String,
    pub fee: String,
    pub is_taker: bool,
    pub aggressor_side: Side,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserFillsPush {
    pub topic: String,
    pub address: Address20,
    pub height: u64,
    pub timestamp_ms: u64,
    pub fills: Vec<UserFillItem>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderUpdateKind {
    Accepted,
    Resting,
    Done,
    Expired,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarksPush {
    pub topic: String,
    pub height: u64,
    pub timestamp_ms: u64,
    pub marks: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderUpdateItem {
    pub block_height: u64,
    pub event_seq: u64,
    pub order_id: u64,
    pub market_id: MarketId,
    #[serde(default)]
    pub symbol: Option<String>,
    pub kind: OrderUpdateKind,
    pub remaining: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub side: Option<Side>,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub qty: Option<String>,
    #[serde(default)]
    pub tif: Option<TimeInForce>,
    #[serde(default)]
    pub reduce_only: Option<bool>,
    #[serde(default)]
    pub client_order_id: Option<String>,
}

impl OrderUpdateItem {
    /// Parse `reason` for `done` / `expired` updates (`DoneReason` PascalCase tag).
    pub fn done_reason(&self) -> Option<DoneReason> {
        let tag = self.reason.as_ref()?;
        serde_json::from_value(serde_json::Value::String(tag.clone())).ok()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderUpdatesPush {
    pub topic: String,
    pub address: Address20,
    pub height: u64,
    pub timestamp_ms: u64,
    pub updates: Vec<OrderUpdateItem>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerUpdateKind {
    Placed,
    Amended,
    Activated,
    Cancelled,
    Expired,
    FireFailed,
    OcoPlaced,
    OcoResolved,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerUpdateItem {
    pub block_height: u64,
    pub event_seq: u64,
    pub kind: TriggerUpdateKind,
    pub market_id: MarketId,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub trigger_id: Option<u64>,
    #[serde(default)]
    pub pair_id: Option<u64>,
    #[serde(default)]
    pub reason: Option<serde_json::Value>,
    #[serde(default)]
    pub side: Option<Side>,
    #[serde(default)]
    pub order_type: Option<TriggerOrderType>,
    #[serde(default)]
    pub qty: Option<String>,
    #[serde(default)]
    pub trigger_price: Option<String>,
    #[serde(default)]
    pub trigger_direction: Option<TriggerDirection>,
    #[serde(default)]
    pub limit_price: Option<String>,
    #[serde(default)]
    pub tif: Option<TimeInForce>,
    #[serde(default)]
    pub reduce_only: Option<bool>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub expires_at_ms: Option<u64>,
    #[serde(default)]
    pub client_pair_id: Option<u64>,
}

impl TriggerUpdateItem {
    /// `cancelled` updates carry `TriggerCancelReason`.
    pub fn cancel_reason(&self) -> Option<TriggerCancelReason> {
        if !matches!(self.kind, TriggerUpdateKind::Cancelled) {
            return None;
        }
        let value = self.reason.as_ref()?;
        parse_pascal_case_tag(value)
    }

    /// `oco_resolved` updates carry `OcoResolveReason`.
    pub fn oco_resolve_reason(&self) -> Option<OcoResolveReason> {
        if !matches!(self.kind, TriggerUpdateKind::OcoResolved) {
            return None;
        }
        let value = self.reason.as_ref()?;
        parse_pascal_case_tag(value)
    }

    /// `fire_failed` updates carry projected [`RejectReason`] (ADR-0026 JSON object).
    pub fn fire_failed_reason(&self) -> Option<RejectReason> {
        if !matches!(self.kind, TriggerUpdateKind::FireFailed) {
            return None;
        }
        let value = self.reason.as_ref()?;
        if let Ok(reason) = RejectReason::from_value(value) {
            return Some(reason);
        }
        // Legacy string debug tag (pre-projection nodes).
        if let serde_json::Value::String(s) = value {
            let head = s.split([' ', '{']).next().unwrap_or(s.as_str());
            return RejectReason::from_value(&serde_json::json!({ head: null })).ok();
        }
        None
    }
}

/// Parse a PascalCase unit enum tag from WS `reason` (`"ByOwner"` or `{"ByOwner": null}`).
fn parse_pascal_case_tag<T: serde::de::DeserializeOwned>(value: &serde_json::Value) -> Option<T> {
    if let Ok(v) = serde_json::from_value(value.clone()) {
        return Some(v);
    }
    let tag = match value {
        serde_json::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_json::Value::Object(map) if map.len() == 1 => {
            let (tag, _) = map.iter().next()?;
            serde_json::Value::String(tag.clone())
        }
        _ => return None,
    };
    serde_json::from_value(tag).ok()
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerUpdatesPush {
    pub topic: String,
    pub address: Address20,
    pub height: u64,
    pub timestamp_ms: u64,
    pub updates: Vec<TriggerUpdateItem>,
}

#[derive(Debug, Clone)]
pub enum WsMessage {
    Subscribed(SubscribeAck),
    Error(WsError),
    Block(BlockTipPush),
    BlocksLive(BlocksLivePush),
    Candle(CandlePush),
    Book(BookPush),
    Trades(TradesPush),
    Account(AccountPush),
    ExternalQuote(ExternalQuotePush),
    Bbo(BboPush),
    UserFills(UserFillsPush),
    OrderUpdates(OrderUpdatesPush),
    TriggerUpdates(TriggerUpdatesPush),
    Marks(MarksPush),
}

pub fn parse_ws_message(text: &str) -> Result<WsMessage, ClientError> {
    let v: serde_json::Value = serde_json::from_str(text)?;
    if let Some(op) = v.get("op").and_then(|x| x.as_str()) {
        return match op {
            "subscribed" => Ok(WsMessage::Subscribed(serde_json::from_value(v)?)),
            "error" => Ok(WsMessage::Error(serde_json::from_value(v)?)),
            other => Err(ClientError::Api {
                status: 0,
                body: format!("unexpected ws op `{other}`"),
            }),
        };
    }
    let topic = v
        .get("topic")
        .and_then(|x| x.as_str())
        .ok_or_else(|| ClientError::Api {
            status: 0,
            body: "ws message missing topic/op".into(),
        })?;
    if topic == "block" {
        return Ok(WsMessage::Block(serde_json::from_value(v)?));
    }
    if topic == "blocks.live" {
        return Ok(WsMessage::BlocksLive(serde_json::from_value(v)?));
    }
    if topic.starts_with("candles.") {
        return Ok(WsMessage::Candle(serde_json::from_value(v)?));
    }
    if topic.starts_with("book.") {
        return Ok(WsMessage::Book(serde_json::from_value(v)?));
    }
    if topic.starts_with("trades.") {
        return Ok(WsMessage::Trades(serde_json::from_value(v)?));
    }
    if topic.starts_with("account.") {
        return Ok(WsMessage::Account(serde_json::from_value(v)?));
    }
    if topic.starts_with("external_quote.") {
        return Ok(WsMessage::ExternalQuote(serde_json::from_value(v)?));
    }
    if topic.starts_with("bbo.") {
        return Ok(WsMessage::Bbo(serde_json::from_value(v)?));
    }
    if topic.starts_with("userFills.") {
        return Ok(WsMessage::UserFills(serde_json::from_value(v)?));
    }
    if topic.starts_with("orderUpdates.") {
        return Ok(WsMessage::OrderUpdates(serde_json::from_value(v)?));
    }
    if topic.starts_with("triggerUpdates.") {
        return Ok(WsMessage::TriggerUpdates(serde_json::from_value(v)?));
    }
    if topic == "marks" {
        return Ok(WsMessage::Marks(serde_json::from_value(v)?));
    }
    Err(ClientError::Api {
        status: 0,
        body: format!("unknown ws topic `{topic}`"),
    })
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Blocking WebSocket client (internal tokio runtime).
pub struct WsClient {
    rt: tokio::runtime::Runtime,
    socket: WsStream,
}

impl WsClient {
    pub fn connect(base_url: &str) -> Result<Self, ClientError> {
        Self::connect_with_timeout(base_url, crate::client::DEFAULT_TIMEOUT_SECS)
    }

    pub fn connect_with_timeout(base_url: &str, timeout_secs: u64) -> Result<Self, ClientError> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| ClientError::Api {
                status: 0,
                body: format!("tokio runtime: {e}"),
            })?;
        let http = reqwest::Url::parse(base_url).map_err(|e| ClientError::Url(e.to_string()))?;
        let scheme = match http.scheme() {
            "http" => "ws",
            "https" => "wss",
            s if s == "ws" || s == "wss" => s,
            other => {
                return Err(ClientError::Api {
                    status: 0,
                    body: format!("unsupported scheme: {other}"),
                });
            }
        };
        let mut ws_url = http.clone();
        ws_url.set_scheme(scheme).map_err(|_| ClientError::Api {
            status: 0,
            body: "failed to set ws scheme".into(),
        })?;
        ws_url.set_path(routes::WS_ENTRY);

        let ws_url_str = ws_url.to_string();
        let mut request = ws_url_str
            .into_client_request()
            .map_err(|e| ClientError::Api {
                status: 0,
                body: format!("websocket request: {e}"),
            })?;
        request.headers_mut().insert(
            "User-Agent",
            tokio_tungstenite::tungstenite::http::HeaderValue::from_static(DEFAULT_USER_AGENT),
        );
        let socket = rt
            .block_on(async {
                tokio::time::timeout(
                    Duration::from_secs(timeout_secs),
                    connect_async(request),
                )
                .await
                .map_err(|_| ClientError::Api {
                    status: 0,
                    body: "websocket connect timeout".into(),
                })?
                .map_err(|e| ClientError::Api {
                    status: 0,
                    body: format!("websocket connect: {e}"),
                })
            })?
            .0;

        Ok(Self { rt, socket })
    }

    pub fn subscribe(&mut self, topics: &[String]) -> Result<SubscribeAck, ClientError> {
        let req = SubscribeRequest {
            op: "subscribe",
            topics: topics.to_vec(),
        };
        let text = serde_json::to_string(&req)?;
        self.rt
            .block_on(self.socket.send(Message::Text(text.into())))
            .map_err(|e| ClientError::Api {
                status: 0,
                body: format!("websocket send: {e}"),
            })?;
        match self.recv()? {
            WsMessage::Subscribed(ack) => Ok(ack),
            WsMessage::Error(err) => Err(ClientError::Api {
                status: 0,
                body: err.message,
            }),
            other => Err(ClientError::Api {
                status: 0,
                body: format!("expected subscribed ack, got {other:?}"),
            }),
        }
    }

    pub fn recv(&mut self) -> Result<WsMessage, ClientError> {
        loop {
            let msg = self
                .rt
                .block_on(self.socket.next())
                .ok_or_else(|| ClientError::Api {
                    status: 0,
                    body: "websocket closed".into(),
                })?
                .map_err(|e| ClientError::Api {
                    status: 0,
                    body: format!("websocket read: {e}"),
                })?;
            match msg {
                Message::Text(text) => return parse_ws_message(&text),
                Message::Ping(payload) => {
                    self.rt
                        .block_on(self.socket.send(Message::Pong(payload)))
                        .map_err(|e| ClientError::Api {
                            status: 0,
                            body: format!("websocket pong: {e}"),
                        })?;
                }
                Message::Close(_) => {
                    return Err(ClientError::Api {
                        status: 0,
                        body: "websocket closed by peer".into(),
                    });
                }
                Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => {}
            }
        }
    }

    pub fn close(mut self) -> Result<(), ClientError> {
        self.rt
            .block_on(self.socket.close(None))
            .map_err(|e| ClientError::Api {
                status: 0,
                body: format!("websocket close: {e}"),
            })
    }
}

impl std::fmt::Debug for WsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsClient").finish_non_exhaustive()
    }
}
