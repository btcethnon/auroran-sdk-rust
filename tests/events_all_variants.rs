//! Smoke tests for the full typed event catalog (all domains / variants).

use auroran_sdk_rust::{
    parse_event, EventDomain, EventEnvelope, Side, TimeInForce, TriggerDirection,
};

const ADDR: &str = "0x1111222233334444555566667777888899990000";

fn assert_path(ev: &EventEnvelope, domain: &str, variant: &str) {
    assert_eq!(ev.path(), Some((domain, variant)));
    assert_eq!(ev.domain_enum(), EventDomain::parse(domain));
}

#[test]
fn core_variants_parse() {
    let cases = [
        ("MarkUpdated", serde_json::json!({"Core":{"MarkUpdated":{"market_id":1,"mark_price":"1.0","previous_mark_price":"0.9"}}})),
        ("MarketStatsSnapshot", serde_json::json!({"Core":{"MarketStatsSnapshot":{"market_id":1,"long_size":"1.0","short_size":"0.5","net_size":"0.5","oracle_counter_pnl":"0.0","fills_in_block":2}}})),
        ("BatchItemRejected", serde_json::json!({"Core":{"BatchItemRejected":{"index":0,"action_kind":"PlaceOrder","reason":{"InsufficientBalance":{"required":"1.0","have":"0.0"}}}}})),
        ("RuntimeReject", serde_json::json!({"Core":{"RuntimeReject":{"order_id":1,"engine_reason":"PostOnlyWouldCross"}}})),
        ("Bankruptcy", serde_json::json!({"Core":{"Bankruptcy":{"target_owner":ADDR,"market_id":1,"shortfall_amount":"10.0","source":"Liquidation"}}})),
        ("UserFeeRateChanged", serde_json::json!({"Core":{"UserFeeRateChanged":{"owner":ADDR,"maker_fee_rate":"0.0001","taker_fee_rate":"0.0005"}}})),
        ("ReferrerRegistered", serde_json::json!({"Core":{"ReferrerRegistered":{"owner":ADDR,"code":"ABC"}}})),
        ("ReferrerBound", serde_json::json!({"Core":{"ReferrerBound":{"owner":ADDR,"code":"ABC"}}})),
        ("InviterKeepRatioChanged", serde_json::json!({"Core":{"InviterKeepRatioChanged":{"owner":ADDR,"old_ratio_bps":5000,"new_ratio_bps":6000}}})),
        ("DeadMansSwitchScheduled", serde_json::json!({"Core":{"DeadMansSwitchScheduled":{"owner":ADDR,"trigger_time_ms":1234567890}}})),
        ("DeadMansSwitchTriggered", serde_json::json!({"Core":{"DeadMansSwitchTriggered":{"owner":ADDR,"cancelled":3}}})),
    ];
    for (variant, kind) in cases {
        let json = envelope(kind);
        let ev = parse_event(&json).unwrap();
        assert_path(&ev, "Core", variant);
    }
}

#[test]
fn exec_ops_bridge_trigger_oco_variants_parse() {
    let mark_updated = parse_event(&envelope(serde_json::json!({
        "Exec": {"PositionFlattened": {"owner": ADDR, "market_id": 1, "old_size": "0.5"}}
    })))
    .unwrap();
    assert!(mark_updated.as_position_flattened().is_some());

    let ops = parse_event(&envelope(serde_json::json!({
        "Ops": {"AgentRegistered": {
            "owner": ADDR,
            "agent_address": ADDR,
            "role_mask": 1,
            "expires_at_ms": 999
        }}
    })))
    .unwrap();
    assert!(ops.as_agent_registered().is_some());

    let bridge = parse_event(&envelope(serde_json::json!({
        "Bridge": {"WithdrawRequested": {
            "request_id": 1,
            "owner": ADDR,
            "amount": "10.0",
            "chain": "bsc",
            "new_balance": "90.0"
        }}
    })))
    .unwrap();
    assert!(bridge.as_withdraw_requested().is_some());

    let trigger = parse_event(&envelope(serde_json::json!({
        "Trigger": {"TriggerOrderPlaced": {
            "trigger_id": 7,
            "market_id": 1,
            "owner": ADDR,
            "trigger_price": "100.0",
            "trigger_direction": "Above",
            "payload": {"Market": {"side": "Bid", "qty": "0.1", "reduce_only": false}},
            "expires_at_ms": null
        }}
    })))
    .unwrap();
    let placed = trigger.as_trigger_order_placed().unwrap();
    assert_eq!(placed.trigger_direction, TriggerDirection::Above);
    assert_eq!(placed.payload.side(), Side::Bid);

    let oco = parse_event(&envelope(serde_json::json!({
        "Oco": {"OcoPairResolved": {
            "pair_id": 1,
            "owner": ADDR,
            "market_id": 1,
            "winner_leg": {"Order": 42},
            "reason": "OrderResolved"
        }}
    })))
    .unwrap();
    assert!(oco.as_oco_pair_resolved().is_some());

    let trigger_oco = parse_event(&envelope(serde_json::json!({
        "Trigger": {"OcoPairResolved": {
            "pair_id": 2,
            "owner": ADDR,
            "market_id": 1,
            "winner_leg": null,
            "reason": "ManualCancel"
        }}
    })))
    .unwrap();
    assert!(trigger_oco.as_trigger_oco_pair_resolved().is_some());
    assert!(trigger_oco.as_oco_pair_resolved().is_none());

    let order_accepted = parse_event(&envelope(serde_json::json!({
        "Exec": {"OrderAccepted": {
            "order_id": 1,
            "owner": ADDR,
            "market_id": 1,
            "side": "Bid",
            "limit_price": "1.0",
            "qty": "0.1",
            "tif": "Gtc"
        }}
    })))
    .unwrap();
    assert_eq!(order_accepted.as_order_accepted().unwrap().tif, TimeInForce::Gtc);
}

fn envelope(kind: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "seq": 1,
        "block_height": 100,
        "envelope_idx": 0,
        "kind": kind
    })
}
