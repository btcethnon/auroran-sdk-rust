//! WebSocket subscribe: block + marks (+ optional book / account).
//!
//! ```bash
//! cargo run --example websocket
//!
//! AURORAN_SYMBOL=BTC-USDT cargo run --example websocket
//!
//! AURORAN_ADDRESS=0x1111... AURORAN_SYMBOL=BTC-USDT \
//!   cargo run --example websocket
//! ```

use std::time::{Duration, Instant};

use auroran_sdk_rust::{ws_topics, WsClient, WsMessage};

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = env("AURORAN_RPC_URL", "https://rpc.auroran.io");
    let symbol = env("AURORAN_SYMBOL", "BTC-USDT");
    let max_messages: usize = std::env::var("AURORAN_WS_MAX_MESSAGES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);

    let mut topics = vec![
        ws_topics::block(),
        ws_topics::marks(),
        ws_topics::book(&symbol),
    ];
    if let Ok(addr) = std::env::var("AURORAN_ADDRESS") {
        topics.push(ws_topics::account_hex(addr.trim_start_matches("0x")));
    }

    let mut ws = WsClient::connect(&rpc)?;
    let ack = ws.subscribe(&topics)?;
    println!("subscribed: {:?}", ack.topics);
    if !ack.rejected.is_empty() {
        eprintln!("rejected: {:?}", ack.rejected);
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut count = 0usize;
    while count < max_messages && Instant::now() < deadline {
        match ws.recv() {
            Ok(msg) => {
                count += 1;
                match msg {
                    WsMessage::Block(b) => {
                        println!("[block] height={} txs={}", b.height, b.envelope_count);
                    }
                    WsMessage::Marks(m) => {
                        println!("[marks] height={} symbols={}", m.height, m.marks.len());
                    }
                    WsMessage::Book(b) => {
                        println!(
                            "[book] {} height={} bids={} asks={}",
                            b.symbol,
                            b.height,
                            b.bids.len(),
                            b.asks.len()
                        );
                    }
                    WsMessage::Account(a) => {
                        println!(
                            "[account] {} balance={} nonce={}",
                            a.address, a.balance, a.nonce
                        );
                    }
                    WsMessage::Subscribed(_) | WsMessage::Error(_) => {}
                    other => println!("[push] {other:?}"),
                }
            }
            Err(e) if Instant::now() >= deadline => {
                eprintln!("timeout waiting for messages: {e}");
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    ws.close()?;
    Ok(())
}
