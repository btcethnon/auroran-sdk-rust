//! Bridge WebSocket block tips to JSON-RPC `getBlockEvents` → [`EventEnvelope`].

use crate::client::AuroranClient;
use crate::error::ClientError;
use crate::events::{fetch_all_block_events, EventEnvelope};
use crate::ws::BlockTipPush;

/// After a `WsMessage::Block` tip, load typed chain events for that height.
///
/// Returns an empty vec when `tip.event_count == 0` (skips RPC).
pub fn events_for_block_tip(
    client: &AuroranClient,
    tip: &BlockTipPush,
) -> Result<Vec<EventEnvelope>, ClientError> {
    if tip.event_count == 0 {
        return Ok(Vec::new());
    }
    fetch_all_block_events(client, tip.height)
}

#[cfg(feature = "async")]
pub async fn events_for_block_tip_async(
    client: &crate::AsyncAuroranClient,
    tip: &BlockTipPush,
) -> Result<Vec<EventEnvelope>, ClientError> {
    if tip.event_count == 0 {
        return Ok(Vec::new());
    }
    crate::events::fetch_all_block_events_async(client, tip.height).await
}
