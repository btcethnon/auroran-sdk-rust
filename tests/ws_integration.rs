//! Unit tests for WS ↔ events integration helpers (no network).

use auroran_sdk_rust::{
    parse_block_events_response, parse_ws_message, BlockEventsResponse, DoneReason, MarketId,
    OrderUpdateItem, OrderUpdateKind, RejectReason, TriggerCancelReason, TriggerUpdateItem,
    TriggerUpdateKind, WsMessage,
};

#[test]
fn order_update_done_reason_parses_pascal_case_tag() {
    let item = OrderUpdateItem {
        block_height: 1,
        event_seq: 1,
        order_id: 42,
        market_id: MarketId(1),
        symbol: None,
        kind: OrderUpdateKind::Done,
        remaining: "0".into(),
        reason: Some("Filled".into()),
        side: None,
        price: None,
        qty: None,
        tif: None,
        reduce_only: None,
        client_order_id: None,
    };
    assert_eq!(item.done_reason(), Some(DoneReason::Filled));
}

#[test]
fn trigger_update_cancel_reason_parses_string_tag() {
    let item = TriggerUpdateItem {
        block_height: 1,
        event_seq: 1,
        kind: TriggerUpdateKind::Cancelled,
        market_id: MarketId(1),
        symbol: None,
        trigger_id: Some(7),
        pair_id: None,
        reason: Some(serde_json::json!("ByOwner")),
        side: None,
        order_type: None,
        qty: None,
        trigger_price: None,
        trigger_direction: None,
        limit_price: None,
        tif: None,
        reduce_only: None,
        client_order_id: None,
        expires_at_ms: None,
        client_pair_id: None,
    };
    assert_eq!(item.cancel_reason(), Some(TriggerCancelReason::ByOwner));
}

#[test]
fn trigger_update_cancel_reason_parses_externally_tagged_object() {
    let item = TriggerUpdateItem {
        block_height: 1,
        event_seq: 1,
        kind: TriggerUpdateKind::Cancelled,
        market_id: MarketId(1),
        symbol: None,
        trigger_id: Some(7),
        pair_id: None,
        reason: Some(serde_json::json!({"ByOwner": null})),
        side: None,
        order_type: None,
        qty: None,
        trigger_price: None,
        trigger_direction: None,
        limit_price: None,
        tif: None,
        reduce_only: None,
        client_order_id: None,
        expires_at_ms: None,
        client_pair_id: None,
    };
    assert_eq!(item.cancel_reason(), Some(TriggerCancelReason::ByOwner));
}

#[test]
fn trigger_fire_failed_reason_parses_string_debug_tag() {
    let item = TriggerUpdateItem {
        block_height: 1,
        event_seq: 1,
        kind: TriggerUpdateKind::FireFailed,
        market_id: MarketId(1),
        symbol: None,
        trigger_id: Some(7),
        pair_id: None,
        reason: Some(serde_json::json!("NonceMismatch")),
        side: None,
        order_type: None,
        qty: None,
        trigger_price: None,
        trigger_direction: None,
        limit_price: None,
        tif: None,
        reduce_only: None,
        client_order_id: None,
        expires_at_ms: None,
        client_pair_id: None,
    };
    assert_eq!(
        item.fire_failed_reason().map(|r| r.tag().to_string()),
        Some("NonceMismatch".to_string())
    );
}

#[test]
fn trigger_fire_failed_reason_parses_projected_reject_object() {
    let item = TriggerUpdateItem {
        block_height: 1,
        event_seq: 1,
        kind: TriggerUpdateKind::FireFailed,
        market_id: MarketId(1),
        symbol: None,
        trigger_id: Some(7),
        pair_id: None,
        reason: Some(serde_json::json!({
            "InsufficientBalance": { "required": "10.000000", "have": "1.000000" }
        })),
        side: None,
        order_type: None,
        qty: None,
        trigger_price: None,
        trigger_direction: None,
        limit_price: None,
        tif: None,
        reduce_only: None,
        client_order_id: None,
        expires_at_ms: None,
        client_pair_id: None,
    };
    let reason = item.fire_failed_reason().expect("projected reject");
    assert!(matches!(
        reason,
        RejectReason::InsufficientBalance { required, have }
        if required == "10.000000" && have == "1.000000"
    ));
}

#[test]
fn parse_trigger_updates_push_with_fire_failed_object_reason() {
    let text = serde_json::json!({
        "topic": "triggerUpdates.0x1111222233334444555566667777888899990000",
        "address": "0x1111222233334444555566667777888899990000",
        "height": 42,
        "timestamp_ms": 1_700_000_000_000u64,
        "updates": [{
            "block_height": 42,
            "event_seq": 3,
            "kind": "fire_failed",
            "market_id": 1,
            "symbol": "BTC-USDT",
            "trigger_id": 7,
            "reason": {
                "InsufficientBalance": { "required": "5.000000", "have": "0.000000" }
            }
        }]
    })
    .to_string();

    let msg = parse_ws_message(&text).expect("triggerUpdates frame");
    let WsMessage::TriggerUpdates(push) = msg else {
        panic!("expected TriggerUpdates");
    };
    assert_eq!(push.updates.len(), 1);
    let reason = push.updates[0]
        .fire_failed_reason()
        .expect("fire_failed reason");
    assert!(matches!(reason, RejectReason::InsufficientBalance { .. }));
}

#[test]
fn parse_block_events_response_skips_invalid() {
    let resp = BlockEventsResponse {
        height: 100,
        offset: 0,
        total: 2,
        events: vec![
            serde_json::json!({
                "seq": 1,
                "block_height": 100,
                "envelope_idx": 0,
                "kind": {
                    "Exec": {
                        "LeverageUpdated": {
                            "owner": "0x1111222233334444555566667777888899990000",
                            "market_id": 1,
                            "leverage": 10
                        }
                    }
                }
            }),
            serde_json::json!({"not": "an event"}),
        ],
    };
    let parsed = parse_block_events_response(&resp);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].path(), Some(("Exec", "LeverageUpdated")));
}

#[test]
fn parse_blocks_live_push() {
    let text = serde_json::json!({
        "topic": "blocks.live",
        "block": {
            "height": 10,
            "digest": "0xabc",
            "timestamp_ms": 1_700_000_000_000u64,
            "state_root": "0xdef",
            "envelope_count": 2,
            "event_count": 5
        },
        "envelopes": [{
            "tx_hash": "0x1111",
            "height": 10,
            "idx": 0,
            "action_kind": "PlaceOrder",
            "signer": "0x1111222233334444555566667777888899990000",
            "timestamp_ms": 1_700_000_000_000u64
        }],
        "watermark": 10,
        "block_count": 1
    })
    .to_string();
    let msg = parse_ws_message(&text).expect("blocks.live frame");
    let WsMessage::BlocksLive(push) = msg else {
        panic!("expected BlocksLive");
    };
    assert_eq!(push.block.height, 10);
    assert_eq!(push.envelopes.len(), 1);
    assert_eq!(push.envelopes[0].action_kind, "PlaceOrder");
}

#[test]
fn parse_candle_push() {
    let text = serde_json::json!({
        "topic": "candles.BTC-USDT.1m",
        "data": {
            "t": 1_700_000_000_000u64,
            "i": "1m",
            "o": "100.0",
            "c": "101.0",
            "h": "102.0",
            "l": "99.0",
            "v": "3.5",
            "n": 4
        }
    })
    .to_string();
    let msg = parse_ws_message(&text).expect("candles frame");
    let WsMessage::Candle(push) = msg else {
        panic!("expected Candle");
    };
    assert_eq!(push.topic, "candles.BTC-USDT.1m");
    assert_eq!(push.data.i, "1m");
    assert_eq!(push.data.c, "101.0");
}

#[test]
fn parse_user_fills_push_keeps_symbol_and_client_order_id() {
    let text = serde_json::json!({
        "topic": "userFills.0x1111222233334444555566667777888899990000",
        "address": "0x1111222233334444555566667777888899990000",
        "height": 12346,
        "timestamp_ms": 1_717_200_001_000u64,
        "fills": [{
            "block_height": 12346,
            "event_seq": 3,
            "timestamp_ms": 1_717_200_001_000u64,
            "market_id": 1,
            "symbol": "BTC-USDT",
            "order_id": 1001,
            "client_order_id": "my-cloid-1",
            "price": "97234.50",
            "qty": "0.50000",
            "notional": "48617.250000",
            "fee": "24.308625",
            "is_taker": true,
            "aggressor_side": "Bid"
        }]
    })
    .to_string();
    let msg = parse_ws_message(&text).expect("userFills frame");
    let WsMessage::UserFills(push) = msg else {
        panic!("expected UserFills");
    };
    assert_eq!(push.fills.len(), 1);
    let fill = &push.fills[0];
    assert_eq!(fill.symbol, "BTC-USDT");
    assert_eq!(fill.order_id, 1001);
    assert_eq!(fill.client_order_id.as_deref(), Some("my-cloid-1"));
    assert_eq!(fill.timestamp_ms, 1_717_200_001_000);
    assert!(fill.is_taker);
}

#[test]
fn parse_user_fills_push_defaults_missing_wire_ids() {
    let text = serde_json::json!({
        "topic": "userFills.0x1111222233334444555566667777888899990000",
        "address": "0x1111222233334444555566667777888899990000",
        "height": 1,
        "timestamp_ms": 1,
        "fills": [{
            "block_height": 1,
            "event_seq": 1,
            "market_id": 1,
            "price": "1",
            "qty": "1",
            "notional": "1",
            "fee": "0",
            "is_taker": false,
            "aggressor_side": "Ask"
        }]
    })
    .to_string();
    let msg = parse_ws_message(&text).expect("userFills frame");
    let WsMessage::UserFills(push) = msg else {
        panic!("expected UserFills");
    };
    let fill = &push.fills[0];
    assert!(fill.symbol.is_empty());
    assert_eq!(fill.order_id, 0);
    assert!(fill.client_order_id.is_none());
    assert_eq!(fill.timestamp_ms, 0);
}
