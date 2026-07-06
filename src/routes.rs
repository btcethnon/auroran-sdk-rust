//! API endpoint paths (aligned with Auroran node HTTP API).

/// Write entry: JSON-RPC 2.0 `POST /api/v1/action`.
pub const POST_ACTION: &str = "/api/v1/action";

/// Read entry: JSON-RPC 2.0 `POST /api/v1/query`.
pub const QUERY: &str = "/api/v1/query";

/// Health check: `GET /api/v1/health`.
pub const GET_HEALTH: &str = "/api/v1/health";

/// Cacheable REST aliases (GET, for high-frequency market data).
pub const GET_MARKETS: &str = "/api/v1/markets";
pub const GET_MARKS: &str = "/api/v1/marks";
pub const GET_ORDERBOOK: &str = "/api/v1/orderbook";
pub const GET_ORDERBOOK_STATS: &str = "/api/v1/orderbook/{symbol}/stats";

/// WebSocket upgrade.
pub const WS_ENTRY: &str = "/api/v1/ws";
