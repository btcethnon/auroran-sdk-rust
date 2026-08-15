# auroran-sdk-rust

Standalone off-chain client and signing SDK for Auroran chain.

- Wire type mirrors and action builders
- EIP-712 dual-channel signing (L1 + User-Signed)
- JSON-RPC 2.0 HTTP client
- WebSocket streaming

## Installation

```toml
[dependencies]
auroran-sdk-rust = "0.1"
```

## Quick start

Read-only query (no private key):

```bash
cargo run --example query
```

Place a signed limit order:

```bash
AURORAN_PRIVATE_KEY=0x... AURORAN_SYMBOL=BTC-USDT cargo run --example place_order
```

```rust
use auroran_sdk_rust::{AuroranClient, place_order, Side, TimeInForce};

let client = AuroranClient::new("https://rpc.auroran.io")?;
let action = place_order(owner, "BTC-USDT", Side::Bid, "50000", "0.001", TimeInForce::Gtc);
let receipt = client.submit_signed_accepted(chain_id, network_tag, &signing_key, nonce, action)?;
```

## Account queries

| Method | Use for |
|--------|---------|
| `account()` | Balance, open positions (`size != 0` only), agents, margin summary |
| `try_position()` / `position()` | Per-market leverage, margin mode — **including flat markets after `SetLeverage`** |
| `account_orders()` | Open resting orders |

Important semantics:

- **`getAccount.positions`** only lists markets with **non-zero size**. Leverage set on a flat market does **not** appear here.
- Use **`try_position(address, symbol)`** to read leverage. Returns `Ok(None)` when the account has never configured that market (`-32004`).
- **`withdrawable`** is a display estimate (includes uPnL). On-chain withdraw / isolated transfer limits use **`cross_cash_available`** (no uPnL). Open/add admission uses **`cross_trading_available`**.

Submit helpers:

- `submit_signed` — returns receipt even on `kept-reject` (envelope kept in block).
- `submit_signed_accepted` / `TxReceiptResponse::ensure_accepted()` — fail with `ClientError::TxRejected` when the chain rejects execution.

## Helpers (`helpers` module)

High-level trading utilities:

- `flatten_account` — mass-cancel orders and market-close positions (idempotent)
- `wait_for_leverage`, `wait_for_account`, `wait_for_flat_orders`
- `set_leverage_if_needed`, `submit_accepted`
- `PollConfig` / `SigningConfig` / `poll_config_from_env()`

## Events (`events` module)

All **62** chain event variants (7 domains) have typed structs under [`events`](src/events/mod.rs) and matching `EventEnvelope::as_*()` accessors. Numeric fields are decimal strings (ADR-0026 projection). `Ops::MarketCancelled` is included; `Exec::OracleQuoteRejected` was removed with time-anchored quotes.

- Shell (crate root): [`EventEnvelope`], [`EventDomain`], [`EventKind`], reason enums, `parse_*`, `find_*`
- Typed tree: `ev.kind()` → `EventKind::Exec(ExecEventKind::Filled(...))`
- Payload types: `events::FilledEvent`, `events::LeverageUpdatedEvent`, …
- HTTP DTOs: `api::AccountSummaryResponse`, `api::BlockEventsResponse`, …
- Filters: `events_in_domain`, `events_with_path`
- Receipt helpers: `find_leverage_updated`, `find_filled`, `find_rejected`, `find_deposit_credited`, `find_withdraw_settled`

```rust
use auroran_sdk_rust::{parse_receipt_events, EventEnvelope};
use auroran_sdk_rust::events::FilledEvent;

for ev in parse_receipt_events(&receipt) {
    if let Some(fill) = ev.as_filled() { /* fill: &FilledEvent */ }
}
```

Duplicate variant names across domains (e.g. `OcoPairResolved` in `Oco` vs `Trigger`) use domain-specific struct/accessor names: `as_oco_pair_resolved()` vs `as_trigger_oco_pair_resolved()`.

### WebSocket → events

WS topics push derived views (book, order updates, …), not full `EventEnvelope` JSON. To stream typed chain events:

1. Subscribe to `block` via [`WsClient`] (or `blocks.live` / `candles.{symbol}.{interval}` for explorer pushes)
2. On each [`BlockTipPush`], call [`events_for_block_tip`] (sync) or [`events_for_block_tip_async`] (`feature = "async"`)

```rust
if let WsMessage::Block(tip) = ws.recv()? {
    let events = events_for_block_tip(&client, &tip)?;
}
```

[`OrderUpdateItem::done_reason`] / [`TriggerUpdateItem::cancel_reason`] parse WS terminal reason tags into typed enums.

## Account listing

`list_accounts_filtered` unifies pagination, role, and referral filters for `listAccounts`:

```rust
use auroran_sdk_rust::{AuroranClient, ListAccountsFilter};

let accounts = client.list_accounts_filtered(ListAccountsFilter {
    role: Some("market_maker".into()),
    limit: Some(50),
    ..Default::default()
})?;
```

Convenience wrappers: `list_accounts`, `accounts_by_role`, `search_accounts`.

## Async client (`feature = "async"`)

```toml
auroran-sdk-rust = { version = "0.1", features = ["async"] }
```

```rust
let client = AsyncAuroranClient::new("https://rpc.auroran.io")?;
let markets = client.list_markets().await?;
let receipt = client.submit_signed_accepted(chain_id, network_tag, &sk, nonce, action).await?;
```

`AsyncAuroranClient` mirrors every method on [`AuroranClient`] with native async `reqwest` (same signatures, `.await` instead of blocking).

Async trading helpers live in [`async_helpers`] (same names as `helpers`, async signatures):

```rust
use auroran_sdk_rust::async_helpers::{flatten_account, submit_accepted, PollConfig};
let (receipt, next_nonce) = submit_accepted(&client, &config, &sk, nonce, action).await?;
```

Bridge builders with full external-chain metadata:

- `record_deposit_with_meta(..., tx_hash, bsc_block, bsc_ts)`
- `withdraw_settle_with_tx(request_id, ExternalTxRef { ... })`

## Environment variables

| Variable | Description |
|----------|-------------|
| `AURORAN_RPC_URL` | Node base URL (default `https://rpc.auroran.io`) |
| `AURORAN_CHAIN_ID` | Chain network id |
| `AURORAN_NETWORK_TAG` | EIP-712 network tag |
| `AURORAN_CHAIN_NAME` | User-Signed action network name (wire field `zepto_chain`) |
| `AURORAN_PRIVATE_KEY` | Hex secp256k1 private key for signing |
| `AURORAN_SYMBOL` | Market symbol (examples) |
| `AURORAN_ADDRESS` | Account address (examples) |

See `examples/` for WebSocket and agent registration flows.

## Protocol compatibility

Some on-chain EIP-712 constants retain legacy names (`ZeptoSignTransaction`, `zeptoChain`) for wire compatibility. Rust API uses Auroran naming (`AuroranClient`, `network_name`).

## Development

```bash
cargo test
cargo test live -- --ignored --nocapture          # RPC + WS smoke (rpc.auroran.io)
cargo test live --features async -- --ignored     # includes async client
cargo test rest_markets_live -- --ignored --nocapture
cargo run --example block_events
cargo run --example async_query --features async
cargo run --example query
```

## License

MIT — see [LICENSE](LICENSE).
