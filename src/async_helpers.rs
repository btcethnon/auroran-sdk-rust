//! Async trading helpers (`feature = "async"`).

use std::time::Duration;

use tokio::time::sleep;

use alloy::signers::local::PrivateKeySigner;

use crate::async_client::AsyncAuroranClient;
use crate::error::ClientError;
use crate::helpers::SigningConfig;
use crate::wire::Action;
use crate::TxReceiptResponse;

pub(crate) async fn async_sleep(duration: Duration) {
    sleep(duration).await;
}

/// Submit a signed action and advance nonce on success.
pub async fn submit_accepted(
    client: &AsyncAuroranClient,
    config: &SigningConfig,
    sk: &PrivateKeySigner,
    nonce: u64,
    action: Action,
) -> Result<(TxReceiptResponse, u64), ClientError> {
    let receipt = client
        .submit_signed_accepted(
            config.chain_id,
            &config.network_tag,
            sk,
            nonce,
            action,
        )
        .await?;
    Ok((receipt, nonce + 1))
}

crate::impl_trading_poll_helpers!(async, AsyncAuroranClient);
