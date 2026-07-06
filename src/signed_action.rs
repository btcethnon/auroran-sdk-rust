//! Master / agent signed envelope construction.
//!
//! Wires the EIP-712 dual-channel signing module into `SignedActionEnvelope`s
//! ready to POST to `/api/v1/action`.

use alloy::signers::local::PrivateKeySigner;

use crate::signing::{address_from_verifying_key, sign_action};
use crate::wire::{Action, Address20, SigCredential, SignedActionEnvelope, ACTION_VERSION_V2};

/// Build a master-signed envelope.
///
/// `sk` is the secp256k1 private key whose derived address is the `signer`.
/// The signing channel (L1 vs User-Signed) is auto-selected based on the action variant.
pub fn master_signed_envelope(
    chain_id: u64,
    domain_chain_id: u64,
    network_tag: &str,
    sk: &PrivateKeySigner,
    nonce: u64,
    action: Action,
) -> SignedActionEnvelope {
    let signer = address_from_verifying_key(sk);
    master_signed_envelope_for(
        chain_id,
        domain_chain_id,
        network_tag,
        sk,
        signer,
        nonce,
        action,
    )
}

/// Build a master-signed envelope with an explicit `signer` address.
/// The `signer` must match the address derived from `sk`, or the chain will reject.
pub fn master_signed_envelope_for(
    chain_id: u64,
    domain_chain_id: u64,
    network_tag: &str,
    sk: &PrivateKeySigner,
    signer: Address20,
    nonce: u64,
    action: Action,
) -> SignedActionEnvelope {
    let signature = sign_action(domain_chain_id, network_tag, nonce, &action, sk);
    SignedActionEnvelope {
        chain_id,
        domain_chain_id,
        action_version: ACTION_VERSION_V2,
        nonce,
        signer,
        credential: SigCredential::Secp256k1 { signature },
        action,
    }
}

/// Build an agent-signed envelope.
///
/// `signer` is the **master** account address. `agent_sk` is the secp256k1
/// private key of the registered API-wallet agent. The agent must be registered
/// and unexpired on-chain.
///
/// Agents sign via the L1 channel (User-Signed actions like `WithdrawRequest`,
/// `RegisterAgent`, `RevokeAgent` are `is_master_only` and cannot be delegated).
pub fn agent_signed_envelope(
    chain_id: u64,
    domain_chain_id: u64,
    network_tag: &str,
    signer: Address20,
    agent_sk: &PrivateKeySigner,
    nonce: u64,
    action: Action,
) -> SignedActionEnvelope {
    let signature =
        crate::signing::sign_l1_action(domain_chain_id, network_tag, nonce, &action, agent_sk);
    SignedActionEnvelope {
        chain_id,
        domain_chain_id,
        action_version: ACTION_VERSION_V2,
        nonce,
        signer,
        credential: SigCredential::Secp256k1 { signature },
        action,
    }
}
