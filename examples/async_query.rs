//! Async read-only query (`feature = "async"`).
//!
//! ```bash
//! cargo run --example async_query --features async
//! ```

use auroran_sdk_rust::AsyncAuroranClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = std::env::var("AURORAN_RPC_URL").unwrap_or_else(|_| "https://rpc.auroran.io".into());
    let client = AsyncAuroranClient::new(&rpc)?;

    let health = client.health_rest().await?;
    println!("health: status={} height={}", health.status, health.height);

    let markets = client.list_markets().await?;
    println!("markets ({}):", markets.len());
    for m in markets.iter().take(5) {
        println!("  {} mark={}", m.symbol, m.mark_price);
    }

    Ok(())
}
