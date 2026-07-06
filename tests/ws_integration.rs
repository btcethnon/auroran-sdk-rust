//! Unit tests for WS ↔ events integration helpers (no network).

use auroran_sdk_rust::{
    parse_block_events_response, BlockEventsResponse, DoneReason, MarketId, OrderUpdateItem,
    OrderUpdateKind, TriggerCancelReason, TriggerUpdateItem, TriggerUpdateKind,
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
fn trigger_update_cancel_reason_parses() {
    let item = TriggerUpdateItem {
        block_height: 1,
        event_seq: 1,
        kind: TriggerUpdateKind::Cancelled,
        market_id: MarketId(1),
        symbol: None,
        trigger_id: Some(7),
        pair_id: None,
        reason: Some("ByOwner".into()),
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
fn trigger_fire_failed_reason_parses_debug_tag() {
    let item = TriggerUpdateItem {
        block_height: 1,
        event_seq: 1,
        kind: TriggerUpdateKind::FireFailed,
        market_id: MarketId(1),
        symbol: None,
        trigger_id: Some(7),
        pair_id: None,
        reason: Some("NonceMismatch".into()),
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
