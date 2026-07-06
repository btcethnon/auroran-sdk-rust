//! 20-byte account address (EVM-compatible wire mirror).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const ADDRESS_LEN: usize = 20;

/// A 20-byte account address compatible with EVM tooling.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address20([u8; ADDRESS_LEN]);

impl Serialize for Address20 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("0x{}", hex_encode(&self.0)))
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Address20 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            s.parse::<Address20>()
                .map_err(|e| D::Error::custom(format!("Address20: {e}")))
        } else {
            <[u8; ADDRESS_LEN]>::deserialize(deserializer).map(Self)
        }
    }
}

impl Default for Address20 {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl Address20 {
    pub const ZERO: Self = Self([0u8; ADDRESS_LEN]);

    #[inline]
    pub const fn from_bytes(bytes: [u8; ADDRESS_LEN]) -> Self {
        Self(bytes)
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; ADDRESS_LEN] {
        &self.0
    }

    pub fn from_keccak256(hash: &[u8; 32]) -> Self {
        let mut buf = [0u8; ADDRESS_LEN];
        buf.copy_from_slice(&hash[12..32]);
        Self(buf)
    }

    pub fn from_blake3(hash: &blake3::Hash) -> Self {
        let mut buf = [0u8; ADDRESS_LEN];
        buf.copy_from_slice(&hash.as_bytes()[..ADDRESS_LEN]);
        Self(buf)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; ADDRESS_LEN]
    }
}

impl fmt::Debug for Address20 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address20(0x{})", hex_encode(&self.0))
    }
}

impl fmt::Display for Address20 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex_encode(&self.0))
    }
}

/// Address parsing failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AddressParseError {
    #[error("address must be 0x-prefixed 20-byte hex")]
    InvalidLength,
    #[error("address contains non-hex character")]
    InvalidHex,
}

impl FromStr for Address20 {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"));
        let Some(hex) = hex else {
            return Err(AddressParseError::InvalidLength);
        };
        if hex.len() != ADDRESS_LEN * 2 {
            return Err(AddressParseError::InvalidLength);
        }
        let mut out = [0u8; ADDRESS_LEN];
        for (i, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
            let hi = hex_nibble(chunk[0])?;
            let lo = hex_nibble(chunk[1])?;
            out[i] = (hi << 4) | lo;
        }
        Ok(Self(out))
    }
}

// (Agent keys are now secp256k1 — no separate AgentPubkey type needed.)

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[usize::from(b >> 4)]);
        s.push(HEX_CHARS[usize::from(b & 0x0f)]);
    }
    s
}

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

fn hex_nibble(b: u8) -> Result<u8, AddressParseError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(AddressParseError::InvalidHex),
    }
}
