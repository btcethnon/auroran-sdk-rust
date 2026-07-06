//! EIP-712 signing golden vectors — byte-identical with Auroran chain protocol fixtures.

use alloy::primitives::B256;
use auroran_sdk_rust::{
    l1_digest, register_agent_digest, revoke_agent_digest, withdraw_digest, Action, Address20,
    CancelOrderAction, DecimalStr, RegisterAgentAction, RevokeAgentAction, WithdrawRequestAction,
};

const CHAIN_ID: u64 = 42;
const NONCE: u64 = 7;
const NETWORK_TAG: &str = "zepto-dev";

const GOLDEN_L1_CANCEL_DIGEST: &str =
    "03337b62ded4f7dccee51cf053ae81182934cf31d27facb660e96eb0cb164f3c";
const GOLDEN_WITHDRAW_DIGEST: &str =
    "45557ed44d54d8f1989e413dda8bcb77fcd5e05ef3a9d41b8de08b47b0a0a3ed";
const GOLDEN_REGISTER_DIGEST: &str =
    "62a464fbed3e1591a021da4927038ca5cb02474c005c1e41af28383559398db1";
const GOLDEN_REVOKE_DIGEST: &str =
    "13a3b09f2ce578bb8e02232acf45af8cc4d6aec591cec7ba8e479a6acaf25914";

fn owner() -> Address20 {
    Address20::from_bytes([0x11; 20])
}

fn agent_addr() -> Address20 {
    Address20::from_bytes([0x22; 20])
}

fn hex_digest(d: B256) -> String {
    hex::encode(d.as_slice())
}

#[test]
fn l1_cancel_order_digest_matches_chain() {
    let action = Action::CancelOrder(CancelOrderAction {
        owner: owner(),
        symbol: None,
        order_id: Some(42),
        client_order_id: None,
    });
    let d = l1_digest(CHAIN_ID, NETWORK_TAG, NONCE, &action);
    assert_eq!(hex_digest(d), GOLDEN_L1_CANCEL_DIGEST);
}

#[test]
fn withdraw_digest_matches_chain() {
    let action = WithdrawRequestAction {
        network_name: "zepto-dev".into(),
        owner: owner(),
        // ADR-0026 §D：用户所签即 canonical 小数串 "100.500000"（100.5 USDC）。
        amount: DecimalStr::new("100.500000"),
        // 下提目标链（小写），EIP-712 签名内绑定。
        chain: "bsc".into(),
    };
    let d = withdraw_digest(CHAIN_ID, NONCE, &action);
    assert_eq!(hex_digest(d), GOLDEN_WITHDRAW_DIGEST);
}

#[test]
fn register_agent_digest_matches_chain() {
    let action = RegisterAgentAction {
        network_name: "zepto-dev".into(),
        owner: owner(),
        agent_address: agent_addr(),
        role_mask: 1, // TRADER_ONLY
        expires_at_ms: 0,
    };
    let d = register_agent_digest(CHAIN_ID, NONCE, &action);
    assert_eq!(hex_digest(d), GOLDEN_REGISTER_DIGEST);
}

#[test]
fn revoke_agent_digest_matches_chain() {
    let action = RevokeAgentAction {
        network_name: "zepto-dev".into(),
        owner: owner(),
        agent_address: agent_addr(),
    };
    let d = revoke_agent_digest(CHAIN_ID, NONCE, &action);
    assert_eq!(hex_digest(d), GOLDEN_REVOKE_DIGEST);
}
