use auroran_sdk_rust::{
    events_in_domain, find_rejected, parse_event, parse_receipt_events, EventDomain, EventEnvelope,
    MarketId, RejectReason, Side, TxReceiptResponse,
};

#[test]
fn event_envelope_parses_leverage_updated() {
    let json = serde_json::json!({
        "seq": 1,
        "block_height": 100,
        "envelope_idx": 0,
        "kind": {
            "Exec": {
                "LeverageUpdated": {
                    "owner": "0x1111222233334444555566667777888899990000",
                    "market_id": 1,
                    "leverage": 25
                }
            }
        }
    });
    let ev: EventEnvelope = parse_event(&json).unwrap();
    assert_eq!(ev.path(), Some(("Exec", "LeverageUpdated")));
    assert_eq!(ev.domain_enum(), Some(EventDomain::Exec));
    let body = ev.as_leverage_updated().unwrap();
    assert_eq!(body.leverage, 25);
    assert_eq!(body.market_id, MarketId(1));
}

#[test]
fn event_envelope_parses_filled() {
    let json = serde_json::json!({
        "seq": 2,
        "block_height": 101,
        "envelope_idx": 1,
        "kind": {
            "Exec": {
                "Filled": {
                    "taker_order_id": 1,
                    "maker_order_id": 2,
                    "market_id": 1,
                    "taker_owner": "0x1111222233334444555566667777888899990000",
                    "maker_owner": "0x2222333344445555666677778888999900001111",
                    "price": "100.0",
                    "qty": "0.00100",
                    "notional": "0.100000",
                    "taker_fee": "0.000050",
                    "maker_fee": "-0.000010",
                    "aggressor_side": "Bid"
                }
            }
        }
    });
    let ev: EventEnvelope = parse_event(&json).unwrap();
    let fill = ev.as_filled().unwrap();
    assert_eq!(fill.aggressor_side, Side::Bid);
    assert_eq!(fill.qty, "0.00100");
}

#[test]
fn rejected_event_parses_typed_reason() {
    use auroran_sdk_rust::RejectReason;

    let json = serde_json::json!({
        "seq": 5,
        "block_height": 103,
        "envelope_idx": 0,
        "kind": {
            "Core": {
                "Rejected": {
                    "action_kind": "PlaceOrder",
                    "reason": {
                        "InsufficientBalance": {
                            "required": "10.000000",
                            "have": "1.000000"
                        }
                    }
                }
            }
        }
    });
    let ev: EventEnvelope = parse_event(&json).unwrap();
    let rejected = ev.as_rejected().unwrap();
    match rejected.reason {
        RejectReason::InsufficientBalance { required, have } => {
            assert_eq!(required, "10.000000");
            assert_eq!(have, "1.000000");
        }
        other => panic!("unexpected reason: {other:?}"),
    }
    assert!(ev.kind().is_some());
}

#[test]
fn rejected_event_parses_dust_and_fok_reasons() {
    let dust = serde_json::json!({
        "seq": 6,
        "block_height": 104,
        "envelope_idx": 0,
        "kind": {
            "Core": {
                "Rejected": {
                    "action_kind": "PlaceOrder",
                    "reason": { "DustNotionalFill": null }
                }
            }
        }
    });
    let ev = parse_event(&dust).unwrap();
    assert!(matches!(ev.as_rejected().unwrap().reason, RejectReason::DustNotionalFill));

    let fok = serde_json::json!({
        "seq": 7,
        "block_height": 105,
        "envelope_idx": 0,
        "kind": {
            "Core": {
                "Rejected": {
                    "action_kind": "PlaceOrder",
                    "reason": { "FokRejected": null }
                }
            }
        }
    });
    let ev = parse_event(&fok).unwrap();
    assert!(matches!(ev.as_rejected().unwrap().reason, RejectReason::FokRejected));
    assert_eq!(RejectReason::FokRejected.tag(), "FokRejected");
    assert_eq!(RejectReason::DustNotionalFill.tag(), "DustNotionalFill");
}

#[test]
fn event_envelope_parses_bridge_and_core_variants() {
    let deposit = serde_json::json!({
        "seq": 3,
        "block_height": 102,
        "envelope_idx": 0,
        "kind": {
            "Bridge": {
                "DepositCredited": {
                    "seq": 7,
                    "owner": "0x1111222233334444555566667777888899990000",
                    "amount": "100.000000",
                    "new_balance": "500.000000"
                }
            }
        }
    });
    let ev: EventEnvelope = parse_event(&deposit).unwrap();
    let credited = ev.as_deposit_credited().unwrap();
    assert_eq!(credited.amount, "100.000000");

    let balance = serde_json::json!({
        "seq": 4,
        "block_height": 102,
        "envelope_idx": 1,
        "kind": {
            "Core": {
                "BalanceChanged": {
                    "owner": "0x1111222233334444555566667777888899990000",
                    "delta": "100.000000",
                    "new_balance": "500.000000",
                    "reason": "BridgeDeposit"
                }
            }
        }
    });
    let ev: EventEnvelope = parse_event(&balance).unwrap();
    let changed = ev.as_balance_changed().unwrap();
    assert_eq!(changed.reason, "BridgeDeposit");
}

#[test]
fn events_in_domain_filters_receipt_events() {
    let receipt: TxReceiptResponse = serde_json::from_value(serde_json::json!({
        "tx_hash": "0xabc",
        "height": 10,
        "envelope_idx": 0,
        "signer": "0x1111222233334444555566667777888899990000",
        "nonce": 1,
        "action": {},
        "status": "accepted",
        "events": [
            {
                "seq": 1,
                "block_height": 10,
                "envelope_idx": 0,
                "kind": { "Exec": { "OrderAccepted": {
                    "order_id": 1,
                    "owner": "0x1111222233334444555566667777888899990000",
                    "market_id": 1,
                    "side": "Bid",
                    "limit_price": "1.0",
                    "qty": "0.1"
                }}}
            },
            {
                "seq": 2,
                "block_height": 10,
                "envelope_idx": 0,
                "kind": { "Core": { "Rejected": {
                    "action_kind": "PlaceOrder",
                    "reason": { "InsufficientBalance": { "required": "1.0", "have": "0.5" } }
                }}}
            }
        ]
    }))
    .unwrap();
    let events = parse_receipt_events(&receipt);
    assert_eq!(events.len(), 2);
    assert_eq!(events_in_domain(&events, EventDomain::Exec).len(), 1);
    assert!(find_rejected(&receipt).is_some());
}
