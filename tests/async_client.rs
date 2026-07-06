#[cfg(feature = "async")]
#[tokio::test]
#[ignore = "network; run: cargo test async_markets_live --features async -- --ignored"]
async fn async_markets_live() {
    let rpc = std::env::var("AURORAN_RPC_URL").unwrap_or_else(|_| "https://rpc.auroran.io".into());
    let client = auroran_sdk_rust::AsyncAuroranClient::new(&rpc).expect("client");
    let markets = client.list_markets().await.expect("list_markets");
    assert!(!markets.is_empty());
    let rest = client.markets_rest().await.expect("markets_rest");
    assert_eq!(rest.len(), markets.len());
}
