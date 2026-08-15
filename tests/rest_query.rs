//! REST alias methods delegate to the JSON-RPC reads (`getMarkets` / `getAllMarks`
//! etc.) since the browser/explorer layer merged into the node and the plural
//! market GETs switched to the Explorer hot-read wrapper. The chain reads keep
//! the `{height, data, page?}` skeleton parsed below.

use auroran_sdk_rust::{
    AuroranClient, ClientError, MarketListItem, QueryResult, TxReceiptResponse, UserFillResponse,
};

#[test]
fn rest_markets_body_unwraps_data() {
    let json = r#"{
        "height": 1562394,
        "data": [{
            "symbol": "BTC",
            "market_id": 1,
            "kind": "ExternalPeg",
            "lifecycle": "Active",
            "emergency_halt": false,
            "price_decimals": 1,
            "size_decimals": 5,
            "max_leverage": 40,
            "mark_price": "62899.0",
            "prev_day_price": "63615.0",
            "day_ntl_volume": "23441.477320",
            "day_base_volume": "0.37118"
        }]
    }"#;
    let qr: QueryResult<Vec<MarketListItem>> = serde_json::from_str(json).unwrap();
    assert_eq!(qr.height, 1562394);
    assert_eq!(qr.data.len(), 1);
    assert_eq!(qr.data[0].symbol, "BTC");
    assert_eq!(qr.data[0].halt_reason, None);
    assert_eq!(qr.data[0].open_interest, "");
}

#[test]
fn market_list_item_reads_halt_reason_and_open_interest() {
    let json = r#"{
        "symbol": "BTC",
        "market_id": 1,
        "kind": "ExternalPeg",
        "lifecycle": "Halted",
        "emergency_halt": false,
        "halt_reason": "QuoteStale",
        "price_decimals": 1,
        "size_decimals": 5,
        "max_leverage": 40,
        "mark_price": "62899.0",
        "prev_day_price": "63615.0",
        "open_interest": "12.5",
        "open_interest_notional": "786237.5",
        "day_ntl_volume": "23441.477320",
        "day_base_volume": "0.37118"
    }"#;
    let item: MarketListItem = serde_json::from_str(json).unwrap();
    assert_eq!(item.halt_reason.as_deref(), Some("QuoteStale"));
    assert_eq!(item.open_interest, "12.5");
    assert_eq!(item.open_interest_notional, "786237.5");
}

#[test]
fn user_fill_reads_trigger_source() {
    let json = r#"{
        "block_height": 10,
        "event_seq": 2,
        "timestamp_ms": 1700000000000,
        "market_id": 1,
        "symbol": "BTC",
        "price": "100.0",
        "qty": "1.0",
        "notional": "100.0",
        "fee": "0.1",
        "is_taker": true,
        "aggressor_side": "Bid",
        "order_id": 7,
        "source_trigger_id": 3,
        "source_tx_hash": "0xabc"
    }"#;
    let fill: UserFillResponse = serde_json::from_str(json).unwrap();
    assert_eq!(fill.source_trigger_id, Some(3));
    assert_eq!(fill.source_tx_hash.as_deref(), Some("0xabc"));
}

#[test]
fn tx_receipt_ensure_accepted() {
    let accepted: TxReceiptResponse = serde_json::from_str(
        r#"{"tx_hash":"0xabc","height":1,"envelope_idx":0,"signer":"0x1111222233334444555566667777888899990000","nonce":1,"action":{},"status":"accepted","events":[]}"#,
    )
    .unwrap();
    assert!(accepted.clone().ensure_accepted().is_ok());

    let rejected: TxReceiptResponse = serde_json::from_str(
        r#"{"tx_hash":"0xdef","height":1,"envelope_idx":0,"signer":"0x1111222233334444555566667777888899990000","nonce":2,"action":{},"status":"kept-reject","reason":{"InsufficientMargin":{}},"events":[]}"#,
    )
    .unwrap();
    assert!(!rejected.is_accepted());
    assert!(rejected.ensure_accepted().is_err());
}

#[test]
fn client_error_resource_not_found() {
    let err = ClientError::Rpc {
        code: -32004,
        message: "position not found".into(),
        data: None,
    };
    assert!(err.is_resource_not_found());
    assert!(!ClientError::Rpc {
        code: -32001,
        message: "nonce".into(),
        data: None,
    }
    .is_resource_not_found());
}

#[test]
#[ignore = "network; run: cargo test rest_live -- --ignored"]
fn rest_markets_live() {
    let rpc = std::env::var("AURORAN_RPC_URL").unwrap_or_else(|_| "https://rpc.auroran.io".into());
    let client = AuroranClient::new(&rpc).expect("client");
    // RPC-backed wrapper methods (getMarkets / getAllMarks through `/api/v1/query`).
    let markets = client.markets_rest().expect("markets_rest");
    assert!(!markets.is_empty(), "expected at least one market from {rpc}");
    let marks = client.marks_rest().expect("marks_rest");
    assert!(!marks.is_empty(), "expected marks from {rpc}");
}
