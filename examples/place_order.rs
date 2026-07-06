//! Sign and submit a limit order (L1 channel).
//!
//! Flow: `getAccount` → build `Action` → `submit_signed` → print tx receipt.
//!
//! ```bash
//! AURORAN_PRIVATE_KEY=0x... \
//! AURORAN_SYMBOL=BTC-USDT \
//!   cargo run --example place_order
//!
//! # chain config (must match the node you connect to)
//! AURORAN_RPC_URL=https://rpc.auroran.io \
//! AURORAN_CHAIN_ID=42 \
//! AURORAN_NETWORK_TAG=zepto-dev \
//! AURORAN_PRIVATE_KEY=0x... \
//!   cargo run --example place_order
//! ```

use auroran_sdk_rust::{
    address_from_verifying_key, place_order, secp256k1_from_hex, AuroranClient, Side, TimeInForce,
};

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = env("AURORAN_RPC_URL", "https://rpc.auroran.io");
    let chain_id = env_u64("AURORAN_CHAIN_ID", 42);
    let network_tag = env("AURORAN_NETWORK_TAG", "zepto-dev");
    let symbol = env("AURORAN_SYMBOL", "BTC-USDT");
    let key_hex = std::env::var("AURORAN_PRIVATE_KEY")
        .map_err(|_| "set AURORAN_PRIVATE_KEY (32-byte secp256k1 hex)")?;

    let sk = secp256k1_from_hex(&key_hex)?;
    let owner = address_from_verifying_key(&sk);
    let owner_hex = format!("0x{}", hex::encode(owner.as_bytes()));

    let client = AuroranClient::new(&rpc)?;
    let acct = client.account(&owner_hex)?;
    let nonce = acct.nonce;
    println!("signer={owner_hex} nonce={nonce} symbol={symbol}");

    // Post-only far from market — adjust price/qty for your environment.
    let action = place_order(
        owner,
        &symbol,
        Side::Bid,
        "1.00",
        "0.00100",
        TimeInForce::PostOnly,
    );

    let receipt = client
        .submit_signed(chain_id, &network_tag, &sk, nonce, action)?
        .ensure_accepted()?;
    println!(
        "status={} tx_hash={} height={}",
        receipt.status, receipt.tx_hash, receipt.height
    );

    Ok(())
}
