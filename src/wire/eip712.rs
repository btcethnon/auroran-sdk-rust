//! EIP-712 typed-data hashing primitives (zero upstream dependency).
//!
//! ## Domain
//!
//! ```text
//! EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)
//! name              = "ZeptoSignTransaction"  // protocol constant, unchanged on chain
//! version           = "1"
//! chainId           = ChainConfig.chain_id
//! verifyingContract = 0x0000000000000000000000000000000000000000
//! ```

use alloy::primitives::{keccak256, B256};

use super::address::Address20;

/// EIP-712 domain `name` (protocol constant on Auroran chain).
pub const EIP712_DOMAIN_NAME: &str = "ZeptoSignTransaction";
/// EIP-712 domain `version`.
pub const EIP712_DOMAIN_VERSION: &str = "1";
/// EIP-712 domain `verifyingContract` (zero address).
pub const EIP712_VERIFYING_CONTRACT: [u8; 20] = [0u8; 20];

/// `EIP712Domain` type string (canonical — changing breaks golden tests).
const EIP712_DOMAIN_TYPE: &[u8] =
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

/// Encode an EIP-712 `string` field → `keccak256(utf8)`.
#[inline]
pub fn enc_string(s: &str) -> B256 {
    keccak256(s.as_bytes())
}

/// Encode a `uintN` (N ≤ 256) field: 32-byte big-endian, right-aligned.
#[inline]
pub fn enc_uint(v: u64) -> B256 {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&v.to_be_bytes());
    B256::from(out)
}

/// Encode an `address` field: left-pad 12 zero bytes, right-align 20-byte address.
#[inline]
pub fn enc_address(addr: &Address20) -> B256 {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(addr.as_bytes());
    B256::from(out)
}

/// Encode a `bytes32` field (pass-through).
#[inline]
pub fn enc_bytes32(b: &[u8; 32]) -> B256 {
    B256::from(*b)
}

/// Compute the Auroran unified domain separator (binds `chain_id`).
pub fn domain_separator(chain_id: u64) -> B256 {
    let type_hash = keccak256(EIP712_DOMAIN_TYPE);
    let mut data = Vec::with_capacity(32 * 5);
    data.extend_from_slice(&type_hash[..]);
    data.extend_from_slice(&enc_string(EIP712_DOMAIN_NAME)[..]);
    data.extend_from_slice(&enc_string(EIP712_DOMAIN_VERSION)[..]);
    data.extend_from_slice(&enc_uint(chain_id)[..]);
    data.extend_from_slice(&enc_address(&Address20::from_bytes(EIP712_VERIFYING_CONTRACT))[..]);
    keccak256(&data)
}

/// `hashStruct(message) = keccak256(typeHash ‖ encoded_fields…)`.
///
/// `type_string` is the canonical type string (e.g.
/// `"Withdraw(address owner,string amount,uint64 nonce)"`); `encoded_fields`
/// are the 32-byte field words in declaration order.
pub fn hash_struct(type_string: &str, encoded_fields: &[B256]) -> B256 {
    let type_hash = keccak256(type_string.as_bytes());
    let mut data = Vec::with_capacity(32 * (1 + encoded_fields.len()));
    data.extend_from_slice(&type_hash[..]);
    for f in encoded_fields {
        data.extend_from_slice(&f[..]);
    }
    keccak256(&data)
}

/// EIP-712 v4 digest: `keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)`.
pub fn digest(domain_separator: &B256, struct_hash: &B256) -> B256 {
    let mut data = Vec::with_capacity(2 + 64);
    data.extend_from_slice(&[0x19, 0x01]);
    data.extend_from_slice(&domain_separator[..]);
    data.extend_from_slice(&struct_hash[..]);
    keccak256(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enc_uint_is_big_endian_right_aligned() {
        let e = enc_uint(0x0102);
        assert_eq!(e[31], 0x02);
        assert_eq!(e[30], 0x01);
        assert!(e[..30].iter().all(|&b| b == 0));
    }

    #[test]
    fn enc_address_left_pads_twelve_zero_bytes() {
        let a = Address20::from_bytes([0xAB; 20]);
        let e = enc_address(&a);
        assert!(e[..12].iter().all(|&b| b == 0));
        assert!(e[12..].iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn domain_separator_changes_with_chain_id() {
        assert_ne!(domain_separator(1), domain_separator(2));
    }

    #[test]
    fn digest_is_deterministic() {
        let ds = domain_separator(7);
        let sh = hash_struct("Foo(uint64 nonce)", &[enc_uint(42)]);
        assert_eq!(digest(&ds, &sh), digest(&ds, &sh));
    }
}
