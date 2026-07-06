//! In-memory submitted-tx recorder (`feature = "test-support"` only).

use std::sync::{Mutex, OnceLock};

/// One accepted submission record (ordered by `submit_action` call sequence).
#[derive(Clone, Debug)]
pub struct SubmittedTx {
    pub seq: usize,
    pub method: String,
    pub tx_hash: String,
    pub status: String,
}

static LOG: OnceLock<Mutex<Vec<SubmittedTx>>> = OnceLock::new();

fn log() -> &'static Mutex<Vec<SubmittedTx>> {
    LOG.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn record(method: &str, tx_hash: &str, status: &str) {
    let mut g = log().lock().expect("tx_recorder poisoned");
    let seq = g.len();
    g.push(SubmittedTx {
        seq,
        method: method.to_string(),
        tx_hash: tx_hash.to_string(),
        status: status.to_string(),
    });
}

pub fn snapshot() -> Vec<SubmittedTx> {
    log().lock().expect("tx_recorder poisoned").clone()
}

pub fn drain() -> Vec<SubmittedTx> {
    std::mem::take(&mut *log().lock().expect("tx_recorder poisoned"))
}

pub fn len() -> usize {
    log().lock().expect("tx_recorder poisoned").len()
}

pub fn clear() {
    log().lock().expect("tx_recorder poisoned").clear();
}
