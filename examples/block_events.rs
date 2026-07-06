//! WebSocket `block` tip + JSON-RPC `getBlockEvents` → typed [`EventEnvelope`] stream.
//!
//! ```bash
//! cargo run --example block_events
//!
//! AURORAN_WS_MAX_BLOCKS=3 cargo run --example block_events
//! ```

use std::time::{Duration, Instant};

use auroran_sdk_rust::{events_for_block_tip, ws_topics, AuroranClient, WsClient, WsMessage};

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = env("AURORAN_RPC_URL", "https://rpc.auroran.io");
    let max_blocks: usize = std::env::var("AURORAN_WS_MAX_BLOCKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let client = AuroranClient::new(&rpc)?;
    let mut ws = WsClient::connect(&rpc)?;
    let ack = ws.subscribe(&[ws_topics::block()])?;
    println!("subscribed: {:?}", ack.topics);

    let deadline = Instant::now() + Duration::from_secs(45);
    let mut seen = 0usize;
    while seen < max_blocks && Instant::now() < deadline {
        match ws.recv() {
            Ok(WsMessage::Block(tip)) => {
                seen += 1;
                println!(
                    "[block] height={} envelopes={} events={}",
                    tip.height, tip.envelope_count, tip.event_count
                );
                let events = events_for_block_tip(&client, &tip)?;
                println!("  fetched {} typed event(s)", events.len());
                for ev in events.iter().take(5) {
                    if let Some((domain, variant)) = ev.path() {
                        println!("    {domain}::{variant}");
                    }
                }
                if events.len() > 5 {
                    println!("    ...");
                }
            }
            Ok(WsMessage::Subscribed(_) | WsMessage::Error(_)) => {}
            Ok(other) => println!("unexpected push: {other:?}"),
            Err(e) if Instant::now() >= deadline => {
                eprintln!("timeout: {e}");
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    ws.close()?;
    Ok(())
}
