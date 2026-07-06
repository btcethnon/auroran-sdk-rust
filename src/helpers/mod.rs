//! High-level trading helpers: polling, account flatten, signed submit wrappers.

use std::thread;
use std::time::Duration;

use alloy::signers::local::PrivateKeySigner;

use crate::client::AuroranClient;
use crate::error::ClientError;
use crate::wire::{parse_decimal, Action};
use crate::{
    AccountOrdersResponse, AccountSummaryResponse, MarketListItem, TxReceiptResponse,
};

#[macro_use]
mod poll;

/// Chain signing parameters for [`submit_accepted`].
#[derive(Clone, Debug)]
pub struct SigningConfig {
    pub chain_id: u64,
    pub network_tag: String,
}

impl SigningConfig {
    pub fn new(chain_id: u64, network_tag: impl Into<String>) -> Self {
        Self {
            chain_id,
            network_tag: network_tag.into(),
        }
    }
}

/// Poll loop settings for index read retries.
#[derive(Clone, Debug)]
pub struct PollConfig {
    pub attempts: u64,
    pub interval: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            attempts: 12,
            interval: Duration::from_millis(500),
        }
    }
}

/// Build poll settings from `AURORAN_POLL_ATTEMPTS` / `AURORAN_POLL_INTERVAL_MS`.
pub fn poll_config_from_env() -> PollConfig {
    let attempts = std::env::var("AURORAN_POLL_ATTEMPTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);
    let interval_ms = std::env::var("AURORAN_POLL_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    PollConfig {
        attempts,
        interval: Duration::from_millis(interval_ms),
    }
}

/// Summary of a [`flatten_account`] run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlattenResult {
    pub skipped: bool,
    pub cancelled_symbols: Vec<String>,
    pub closed_symbols: Vec<String>,
}

pub(crate) fn sync_sleep(duration: Duration) {
    thread::sleep(duration);
}

/// Submit a signed action and advance nonce on success.
pub fn submit_accepted(
    client: &AuroranClient,
    config: &SigningConfig,
    sk: &PrivateKeySigner,
    nonce: u64,
    action: Action,
) -> Result<(TxReceiptResponse, u64), ClientError> {
    let receipt = client.submit_signed_accepted(
        config.chain_id,
        &config.network_tag,
        sk,
        nonce,
        action,
    )?;
    Ok((receipt, nonce + 1))
}

/// Returns `true` when parsed position size is non-zero.
pub fn position_is_open(size: &str, size_decimals: u32) -> bool {
    parse_decimal(size, size_decimals).is_some_and(|raw| raw != 0)
}

/// Lookup size decimals from a market list (defaults to 5 when symbol missing).
pub fn size_decimals_for(markets: &[MarketListItem], symbol: &str) -> u32 {
    markets
        .iter()
        .find(|m| m.symbol == symbol)
        .map(|m| m.size_decimals)
        .unwrap_or(5)
}

/// Symbols with non-zero open size in an account snapshot.
pub fn open_position_symbols(
    acct: &AccountSummaryResponse,
    markets: &[MarketListItem],
) -> Vec<String> {
    let mut symbols: Vec<String> = acct
        .positions
        .values()
        .filter_map(|pos| {
            let dec = size_decimals_for(markets, &pos.symbol);
            if position_is_open(&pos.size, dec) {
                Some(pos.symbol.clone())
            } else {
                None
            }
        })
        .collect();
    symbols.sort();
    symbols.dedup();
    symbols
}

/// Distinct symbols with open resting orders.
pub fn order_symbols(orders: &AccountOrdersResponse) -> Vec<String> {
    let mut symbols: Vec<String> = orders.orders.iter().map(|o| o.symbol.clone()).collect();
    symbols.sort();
    symbols.dedup();
    symbols
}

/// Returns true when `symbol` has a non-zero position in the account snapshot.
pub fn has_symbol_position(
    acct: &AccountSummaryResponse,
    symbol: &str,
    size_decimals: u32,
) -> bool {
    acct.positions.values().any(|pos| {
        pos.symbol == symbol && position_is_open(&pos.size, size_decimals)
    })
}

crate::impl_trading_poll_helpers!(sync, AuroranClient);
