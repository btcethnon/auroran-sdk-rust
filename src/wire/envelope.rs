//! Signed action envelope (wire mirror of `zepto_wire::SignedActionEnvelope`).

use super::action::Action;
use super::address::Address20;
use serde::{Deserialize, Serialize};

/// Unified signing channel version (V2: single secp256k1 credential + dual-channel digest).
/// Matches `zepto_wire::ACTION_VERSION_V2`. The chain (`verify_envelope`) only accepts V2.
pub const ACTION_VERSION_V2: u32 = 2;

/// Signature credential — unified secp256k1 (matches `zepto_wire::SigCredential`).
///
/// 65-byte wire `r||s||v` (`v = recid + 27`), JSON 为 byte array（与 `zepto-wire` 一致）。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigCredential {
    Secp256k1 { signature: Vec<u8> },
}

/// Signed action envelope — the top-level structure POSTed to `/api/v1/action`.
///
/// Generic parameter `A` is the `Action` enum. Matches `zepto_wire::SignedActionEnvelope<A>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedActionEnvelope<A = Action> {
    pub chain_id: u64,
    pub domain_chain_id: u64,
    pub action_version: u32,
    pub nonce: u64,
    pub signer: Address20,
    pub credential: SigCredential,
    pub action: A,
}

/// Legacy alias.
pub type ChainEnvelope = SignedActionEnvelope;
