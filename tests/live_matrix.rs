//! Live smoke tests against `rpc.auroran.io` (or `AURORAN_RPC_URL`).
//!
//! ```bash
//! cargo test live -- --ignored --nocapture
//! ```

use auroran_sdk_rust::{
    events_for_block_tip, fetch_all_block_events, parse_block_events_response, ws_topics,
    AuroranClient, WsClient, WsMessage,
};

fn rpc_url() -> String {
    std::env::var("AURORAN_RPC_URL").unwrap_or_else(|_| "https://rpc.auroran.io".into())
}

#[test]
#[ignore = "network; run: cargo test live -- --ignored"]
fn live_rpc_health_and_markets() {
    let client = AuroranClient::new(&rpc_url()).expect("client");
    let health = client.health_rest().expect("health_rest");
    assert_eq!(health.status, "ok");
    let markets = client.list_markets().expect("list_markets");
    assert!(!markets.is_empty());
    let rest = client.markets_rest().expect("markets_rest");
    assert_eq!(rest.len(), markets.len());
}

#[test]
#[ignore = "network; run: cargo test live -- --ignored"]
fn live_rpc_block_and_events() {
    let client = AuroranClient::new(&rpc_url()).expect("client");
    let block = client.block_latest().expect("block_latest");
    assert!(block.height > 0);

    let page = client
        .block_events(block.height, Some(0), Some(10))
        .expect("block_events");
    assert_eq!(page.height, block.height);
    let envelopes = parse_block_events_response(&page);
    assert_eq!(envelopes.len(), page.events.len());

    if block.height > 1 {
        let all = fetch_all_block_events(&client, block.height).expect("fetch_all");
        assert!(all.len() >= envelopes.len());
    }
}

#[test]
#[ignore = "network; run: cargo test live -- --ignored"]
fn live_ws_block_tip_and_events() {
    let rpc = rpc_url();
    let client = AuroranClient::new(&rpc).expect("client");
    let mut ws = WsClient::connect(&rpc).expect("ws connect");
    ws.subscribe(&[ws_topics::block()]).expect("subscribe");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if std::time::Instant::now() >= deadline {
            panic!("timed out waiting for block push");
        }
        match ws.recv() {
            Ok(WsMessage::Block(tip)) => {
                assert!(tip.height > 0);
                let events = events_for_block_tip(&client, &tip).expect("events_for_block_tip");
                assert_eq!(events.len(), tip.event_count);
                ws.close().ok();
                return;
            }
            Ok(WsMessage::Subscribed(_)) => {}
            Ok(_) => {}
            Err(e) => panic!("ws recv: {e}"),
        }
    }
}

#[cfg(feature = "async")]
#[test]
#[ignore = "network; run: cargo test live --features async -- --ignored"]
fn live_async_markets() {
    use auroran_sdk_rust::AsyncAuroranClient;
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let client = AsyncAuroranClient::new(&rpc_url()).expect("async client");
        let markets = client.list_markets().await.expect("list_markets");
        assert!(!markets.is_empty());
    });
}
