//! Read-only JSON-RPC walkthrough (no private key required).
//!
//! ```bash
//! # default: https://rpc.auroran.io
//! cargo run --example query
//!
//! # optional account snapshot
//! AURORAN_ADDRESS=0x1111222233334444555566667777888899990000 \
//!   cargo run --example query
//!
//! AURORAN_RPC_URL=https://node.example.com cargo run --example query
//! ```

use auroran_sdk_rust::{AuroranClient, ClientError};

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn main() -> Result<(), ClientError> {
    let rpc = env("AURORAN_RPC_URL", "https://rpc.auroran.io");
    let client = AuroranClient::new(&rpc)?;

    let health = client.health_rest()?;
    println!("health: status={} height={}", health.status, health.height);

    let config = client.exchange_config()?;
    println!(
        "exchange: action_version={} markets={} settlement_paused={}",
        config.action_version, config.market_count, config.settlement_paused
    );

    let markets = client.list_markets()?;
    println!("markets ({}):", markets.len());
    for m in markets.iter().take(5) {
        println!(
            "  {} kind={} lifecycle={} mark={}",
            m.symbol, m.kind, m.lifecycle, m.mark_price
        );
    }
    if markets.len() > 5 {
        println!("  ...");
    }

    if let Some(first) = markets.first() {
        let book = client.orderbook_with_depth(&first.symbol, Some(5))?;
        println!(
            "orderbook {}: {} bids / {} asks @ height {}",
            book.symbol,
            book.bids.len(),
            book.asks.len(),
            book.height
        );
    }

    if let Ok(addr) = std::env::var("AURORAN_ADDRESS") {
        let acct = client.account(&addr)?;
        println!(
            "account {}: balance={} nonce={} positions={}",
            acct.address,
            acct.balance,
            acct.nonce,
            acct.positions.len()
        );
    }

    Ok(())
}
