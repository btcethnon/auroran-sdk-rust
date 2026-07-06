//! EIP-712 dual-channel signing: L1 (msgpack connectionId) + User-Signed (field-by-field).
//!
//! Uses `alloy` for secp256k1 and `rmp-serde` for canonical msgpack encoding.

use alloy::{
    primitives::{keccak256, B256},
    signers::{local::PrivateKeySigner, SignerSync},
};

use crate::wire::eip712;
use crate::wire::{
    Action, Address20, RegisterAgentAction, RevokeAgentAction, WithdrawRequestAction,
};

/// L1 phantom primary type string (canonical — matches chain).
pub const L1_ACTION_TYPE: &str = "L1Action(string source,bytes32 connectionId)";

/// `Withdraw` primary type string.
pub const WITHDRAW_TYPE: &str =
    "Withdraw(string zeptoChain,address owner,string amount,string chain,uint64 nonce)";

/// `RegisterAgent` primary type string.
pub const REGISTER_AGENT_TYPE: &str =
    "RegisterAgent(string zeptoChain,address owner,address agentAddress,uint64 roleMask,uint64 expiresAtMs,uint64 nonce)";

/// `RevokeAgent` primary type string.
pub const REVOKE_AGENT_TYPE: &str =
    "RevokeAgent(string zeptoChain,address owner,address agentAddress,uint64 nonce)";

/// Rebuild L1-channel EIP-712 digest.
///
/// `domain_chain_id` seeds the EIP-712 domain separator (typically equals the
/// envelope `chain_id` but may differ for frontend wallets that see a different
/// EVM chain ID).
pub fn l1_digest(domain_chain_id: u64, network_tag: &str, nonce: u64, action: &Action) -> B256 {
    let connection_id = l1_connection_id(action, nonce);
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        L1_ACTION_TYPE,
        &[eip712::enc_string(network_tag), connection_id],
    );
    eip712::digest(&domain, &struct_hash)
}

/// Reconstruct `WithdrawRequest` EIP-712 digest (matches chain `withdraw_digest`).
pub fn withdraw_digest(domain_chain_id: u64, nonce: u64, action: &WithdrawRequestAction) -> B256 {
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        WITHDRAW_TYPE,
        &[
            eip712::enc_string(&action.network_name),
            eip712::enc_address(&action.owner),
            eip712::enc_string(action.amount.as_str()),
            eip712::enc_string(&action.chain),
            eip712::enc_uint(nonce),
        ],
    );
    eip712::digest(&domain, &struct_hash)
}

/// Reconstruct `RegisterAgent` EIP-712 digest.
pub fn register_agent_digest(
    domain_chain_id: u64,
    nonce: u64,
    action: &RegisterAgentAction,
) -> B256 {
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        REGISTER_AGENT_TYPE,
        &[
            eip712::enc_string(&action.network_name),
            eip712::enc_address(&action.owner),
            eip712::enc_address(&action.agent_address),
            eip712::enc_uint(action.role_mask),
            eip712::enc_uint(action.expires_at_ms),
            eip712::enc_uint(nonce),
        ],
    );
    eip712::digest(&domain, &struct_hash)
}

/// Reconstruct `RevokeAgent` EIP-712 digest.
pub fn revoke_agent_digest(domain_chain_id: u64, nonce: u64, action: &RevokeAgentAction) -> B256 {
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        REVOKE_AGENT_TYPE,
        &[
            eip712::enc_string(&action.network_name),
            eip712::enc_address(&action.owner),
            eip712::enc_address(&action.agent_address),
            eip712::enc_uint(nonce),
        ],
    );
    eip712::digest(&domain, &struct_hash)
}

/// Compute L1 `connectionId = keccak256(msgpack(action) ‖ nonce.to_be_bytes())`.
///
/// Uses `rmp_serde::to_vec_named` for canonical deterministic encoding,
/// matching the chain's `l1_connection_id` byte-for-byte.
pub fn l1_connection_id(action: &Action, nonce: u64) -> B256 {
    let mut buf = rmp_serde::to_vec_named(action).expect("L1 msgpack serialization must not fail");
    buf.extend_from_slice(&nonce.to_be_bytes());
    keccak256(&buf)
}

/// Sign an EIP-712 typed data hash with a secp256k1 private key.
///
/// Returns 65-byte `r||s||v` (`v = recid + 27`).
fn sign_eip712_hash(hash: B256, sk: &PrivateKeySigner) -> Vec<u8> {
    let sig = sk.sign_hash_sync(&hash).expect("secp256k1 sign_hash_sync");
    let sig = sig.normalized_s();
    let mut out = vec![0u8; 65];
    out[..32].copy_from_slice(&sig.r().to_be_bytes::<32>());
    out[32..64].copy_from_slice(&sig.s().to_be_bytes::<32>());
    let v_byte: u8 = sig.v().into();
    out[64] = v_byte + 27;
    out
}

/// Reconstruct the L1-channel EIP-712 digest and sign it.
///
/// 1. `connectionId = keccak256(msgpack(action) ‖ nonce.to_be_bytes())`
/// 2. Domain = EIP-712 `ZeptoSignTransaction` with `domain_chain_id` (protocol constant)
/// 3. Struct = `L1Action(string source, bytes32 connectionId)`
/// 4. Final digest = `keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)`
///
/// Returns 65-byte secp256k1 signature bytes.
pub fn sign_l1_action(
    domain_chain_id: u64,
    network_tag: &str,
    nonce: u64,
    action: &Action,
    sk: &PrivateKeySigner,
) -> Vec<u8> {
    let connection_id = l1_connection_id(action, nonce);

    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        L1_ACTION_TYPE,
        &[eip712::enc_string(network_tag), connection_id],
    );
    let msg_hash = eip712::digest(&domain, &struct_hash);

    sign_eip712_hash(msg_hash, sk)
}

/// Reconstruct the `WithdrawRequest` EIP-712 digest and sign it.
pub fn sign_withdraw(
    domain_chain_id: u64,
    nonce: u64,
    action: &WithdrawRequestAction,
    sk: &PrivateKeySigner,
) -> Vec<u8> {
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        WITHDRAW_TYPE,
        &[
            eip712::enc_string(&action.network_name),
            eip712::enc_address(&action.owner),
            eip712::enc_string(action.amount.as_str()),
            eip712::enc_string(&action.chain),
            eip712::enc_uint(nonce),
        ],
    );
    let msg_hash = eip712::digest(&domain, &struct_hash);
    sign_eip712_hash(msg_hash, sk)
}

/// Reconstruct the `RegisterAgent` EIP-712 digest and sign it.
pub fn sign_register_agent(
    domain_chain_id: u64,
    nonce: u64,
    action: &RegisterAgentAction,
    sk: &PrivateKeySigner,
) -> Vec<u8> {
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        REGISTER_AGENT_TYPE,
        &[
            eip712::enc_string(&action.network_name),
            eip712::enc_address(&action.owner),
            eip712::enc_address(&action.agent_address),
            eip712::enc_uint(action.role_mask),
            eip712::enc_uint(action.expires_at_ms),
            eip712::enc_uint(nonce),
        ],
    );
    let msg_hash = eip712::digest(&domain, &struct_hash);
    sign_eip712_hash(msg_hash, sk)
}

/// Reconstruct the `RevokeAgent` EIP-712 digest and sign it.
pub fn sign_revoke_agent(
    domain_chain_id: u64,
    nonce: u64,
    action: &RevokeAgentAction,
    sk: &PrivateKeySigner,
) -> Vec<u8> {
    let domain = eip712::domain_separator(domain_chain_id);
    let struct_hash = eip712::hash_struct(
        REVOKE_AGENT_TYPE,
        &[
            eip712::enc_string(&action.network_name),
            eip712::enc_address(&action.owner),
            eip712::enc_address(&action.agent_address),
            eip712::enc_uint(nonce),
        ],
    );
    let msg_hash = eip712::digest(&domain, &struct_hash);
    sign_eip712_hash(msg_hash, sk)
}

/// Sign any action using the correct channel (L1 or User-Signed).
///
/// - L1 channel: all actions except `WithdrawRequest`, `RegisterAgent`, `RevokeAgent`
/// - User-Signed channel: `WithdrawRequest`, `RegisterAgent`, `RevokeAgent`
pub fn sign_action(
    domain_chain_id: u64,
    network_tag: &str,
    nonce: u64,
    action: &Action,
    sk: &PrivateKeySigner,
) -> Vec<u8> {
    match action {
        Action::WithdrawRequest(a) => sign_withdraw(domain_chain_id, nonce, a, sk),
        Action::RegisterAgent(a) => sign_register_agent(domain_chain_id, nonce, a, sk),
        Action::RevokeAgent(a) => sign_revoke_agent(domain_chain_id, nonce, a, sk),
        _ => sign_l1_action(domain_chain_id, network_tag, nonce, action, sk),
    }
}

// ── Key management ─────────────────────────────────────────────────────────

/// Derive an EVM-style `Address20` from a secp256k1 verifying key.
pub fn address_from_verifying_key(vk: &alloy::signers::local::PrivateKeySigner) -> Address20 {
    let addr = vk.address();
    let mut out = [0u8; 20];
    out.copy_from_slice(&addr[..]);
    Address20::from_bytes(out)
}

/// Generate a production-grade secp256k1 signing key.
///
/// Entropy is drawn exclusively from the OS CSPRNG via [`getrandom`] (e.g.
/// `/dev/urandom` on Unix, `BCryptGenRandom` on Windows). Values that are zero
/// or outside the curve order are rejected and retried with fresh entropy.
pub fn generate_signing_key() -> Result<PrivateKeySigner, String> {
    const MAX_ATTEMPTS: u32 = 8;
    for _ in 0..MAX_ATTEMPTS {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| format!("OS CSPRNG unavailable: {e}"))?;
        let hex_str = format!("0x{}", hex::encode(bytes));
        if let Ok(sk) = hex_str.parse::<PrivateKeySigner>() {
            return Ok(sk);
        }
    }
    Err("failed to sample a valid secp256k1 scalar from OS CSPRNG".into())
}

/// Hex-encode a signing key (`0x` + 64 lowercase hex chars).
pub fn signing_key_to_hex(sk: &PrivateKeySigner) -> String {
    format!("0x{}", hex::encode(sk.to_bytes().as_slice()))
}

/// Load a secp256k1 private key from 32-byte hex (optional `0x` prefix).
pub fn secp256k1_from_hex(hex_key: &str) -> Result<PrivateKeySigner, String> {
    let s = hex_key.trim().trim_start_matches("0x");
    let bytes = hex::decode(s).map_err(|e| format!("invalid hex: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!(
            "expected 32-byte private key, got {} bytes",
            bytes.len()
        ));
    }
    let hex_str = format!("0x{}", hex::encode(&bytes));
    hex_str
        .parse::<PrivateKeySigner>()
        .map_err(|e| format!("invalid secp256k1 key: {e}"))
}

#[cfg(feature = "test-support")]
pub mod test_keys {
    //! Reproducible test keys (mirrors chain `zepto_types::signing` seed labels).
    use alloy::signers::local::PrivateKeySigner;
    use rand::{rngs::StdRng, RngCore, SeedableRng};

    use crate::wire::Address20;

    fn seeded_rng(label: &[u8], seed: u8) -> StdRng {
        let mut material = [0u8; 32];
        let mut h = blake3::Hasher::new();
        h.update(label);
        h.update(&[seed]);
        material.copy_from_slice(h.finalize().as_bytes());
        StdRng::from_seed(material)
    }

    fn seeded_secp256k1(label: &[u8], seed: u8) -> PrivateKeySigner {
        let mut rng = seeded_rng(label, seed);
        loop {
            let mut bytes = [0u8; 32];
            rng.fill_bytes(&mut bytes);
            let hex_str = format!("0x{}", hex::encode(bytes));
            if let Ok(sk) = hex_str.parse::<PrivateKeySigner>() {
                return sk;
            }
        }
    }

    /// Reproducible secp256k1 master private key.
    pub fn test_master_signing_key(seed: u8) -> PrivateKeySigner {
        seeded_secp256k1(b"zepto-chain.test-master-signing-key.v0", seed)
    }

    /// Reproducible master address from seed.
    pub fn test_master_address(seed: u8) -> Address20 {
        let sk = test_master_signing_key(seed);
        super::address_from_verifying_key(&sk)
    }

    /// Reproducible secp256k1 agent (API-wallet) private key.
    pub fn test_agent_signing_key(seed: u8) -> PrivateKeySigner {
        seeded_secp256k1(b"zepto-chain.test-agent-signing-key.v0", seed)
    }

    /// Reproducible agent address from seed.
    pub fn test_agent_address(seed: u8) -> Address20 {
        let sk = test_agent_signing_key(seed);
        super::address_from_verifying_key(&sk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l1_connection_id_is_deterministic() {
        use crate::wire::*;
        let action = Action::CancelOrder(CancelOrderAction {
            owner: Address20::from_bytes([0x11; 20]),
            symbol: None,
            order_id: Some(42),
            client_order_id: None,
        });
        let cid1 = l1_connection_id(&action, 0);
        let cid2 = l1_connection_id(&action, 0);
        assert_eq!(cid1, cid2);
    }

    #[test]
    fn l1_connection_id_differs_by_nonce() {
        use crate::wire::*;
        let action = Action::CancelOrder(CancelOrderAction {
            owner: Address20::from_bytes([0x11; 20]),
            symbol: None,
            order_id: Some(42),
            client_order_id: None,
        });
        assert_ne!(l1_connection_id(&action, 0), l1_connection_id(&action, 1));
    }

    #[test]
    fn generate_signing_key_samples_distinct_valid_keys() {
        let k1 = generate_signing_key().expect("k1");
        let k2 = generate_signing_key().expect("k2");
        assert_ne!(k1.to_bytes(), k2.to_bytes());
        assert!(!address_from_verifying_key(&k1).is_zero());
        assert!(!address_from_verifying_key(&k2).is_zero());
        assert_eq!(signing_key_to_hex(&k1).len(), 66);
    }

    #[test]
    fn signing_key_round_trips_through_hex() {
        let sk = generate_signing_key().expect("sk");
        let hex = signing_key_to_hex(&sk);
        let loaded = secp256k1_from_hex(&hex).expect("load");
        assert_eq!(loaded.to_bytes(), sk.to_bytes());
        assert_eq!(
            address_from_verifying_key(&loaded),
            address_from_verifying_key(&sk),
        );
    }
}
