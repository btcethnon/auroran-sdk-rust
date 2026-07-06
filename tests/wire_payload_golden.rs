//! L1 msgpack `connection_id` golden — matches chain fixture.

use auroran_sdk_rust::{
    l1_connection_id, Action, Address20, AmendTriggerOrderAction, CancelOrderAction,
};

fn fixed_signer() -> Address20 {
    Address20::from_bytes([0x11; 20])
}

#[test]
fn golden_l1_connection_id_is_deterministic() {
    let action = Action::CancelOrder(CancelOrderAction {
        owner: fixed_signer(),
        symbol: None,
        order_id: Some(42),
        client_order_id: None,
    });

    let cid1 = l1_connection_id(&action, 7);
    let cid2 = l1_connection_id(&action, 7);
    assert_eq!(cid1, cid2);
    assert_ne!(l1_connection_id(&action, 8), cid1);
}

#[test]
fn amend_trigger_order_connection_id_is_deterministic() {
    let action = Action::AmendTriggerOrder(AmendTriggerOrderAction {
        owner: fixed_signer(),
        trigger_id: 7001,
        new_trigger_price: None,
        new_qty: None,
        new_limit_price: None,
        new_tif: None,
        new_reduce_only: None,
        new_expires_at_ms: None,
    });
    let cid1 = l1_connection_id(&action, 43);
    let cid2 = l1_connection_id(&action, 43);
    assert_eq!(cid1, cid2);
    assert_ne!(l1_connection_id(&action, 44), cid1);
}
