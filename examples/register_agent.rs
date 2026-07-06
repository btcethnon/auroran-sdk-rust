//! User-Signed action: register an API-wallet agent (EIP-712 channel).
//!
//! Master signs with `domain_chain_id` = the EVM network the wallet is connected to
//! (e.g. Ethereum mainnet = 1). `chain_id` is the Auroran network id from node config.
//!
//! ```bash
//! AURORAN_PRIVATE_KEY=0x... \
//! AURORAN_AGENT_ADDRESS=0xaaaa... \
//!   cargo run --example register_agent
//!
//! # wallet on BSC (domain_chain_id = 56)
//! AURORAN_DOMAIN_CHAIN_ID=56 \
//! AURORAN_PRIVATE_KEY=0x... \
//! AURORAN_AGENT_ADDRESS=0xaaaa... \
//!   cargo run --example register_agent
//! ```

use auroran_sdk_rust::{
    address_from_verifying_key, register_agent, secp256k1_from_hex, AuroranClient,
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

fn parse_address20(hex_addr: &str) -> Result<auroran_sdk_rust::Address20, String> {
    let s = hex_addr.trim().trim_start_matches("0x");
    let bytes = hex::decode(s).map_err(|e| format!("invalid hex: {e}"))?;
    if bytes.len() != 20 {
        return Err(format!(
            "expected 20-byte address, got {} bytes",
            bytes.len()
        ));
    }
    let mut out = [0u8; 20];
    out.copy_from_slice(&bytes);
    Ok(auroran_sdk_rust::Address20::from_bytes(out))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = env("AURORAN_RPC_URL", "https://rpc.auroran.io");
    let chain_id = env_u64("AURORAN_CHAIN_ID", 42);
    let domain_chain_id = env_u64("AURORAN_DOMAIN_CHAIN_ID", chain_id);
    let network_tag = env("AURORAN_NETWORK_TAG", "zepto-dev");
    let network_name = env("AURORAN_CHAIN_NAME", "zepto-dev");

    let key_hex = std::env::var("AURORAN_PRIVATE_KEY")
        .map_err(|_| "set AURORAN_PRIVATE_KEY (master secp256k1 hex)")?;
    let agent_hex = std::env::var("AURORAN_AGENT_ADDRESS")
        .map_err(|_| "set AURORAN_AGENT_ADDRESS (agent API-wallet address)")?;

    let sk = secp256k1_from_hex(&key_hex)?;
    let owner = address_from_verifying_key(&sk);
    let agent = parse_address20(&agent_hex)?;
    let owner_hex = format!("0x{}", hex::encode(owner.as_bytes()));

    let client = AuroranClient::new(&rpc)?;
    let nonce = client.account(&owner_hex)?.nonce;
    println!(
        "master={owner_hex} agent={agent_hex} nonce={nonce} chain_id={chain_id} domain_chain_id={domain_chain_id}"
    );

    // role_mask = 1 → Trader
    let action = register_agent(&network_name, owner, agent, 1, 0);

    let receipt = client
        .submit_signed_with_domain(
            chain_id,
            domain_chain_id,
            &network_tag,
            &sk,
            nonce,
            action,
        )?
        .ensure_accepted()?;
    println!("status={} tx_hash={}", receipt.status, receipt.tx_hash);

    Ok(())
}
