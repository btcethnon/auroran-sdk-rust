use serde::{Deserialize, Serialize};

// ── Meta ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionMetaItem {
    pub name: String,
    pub auth: String,
    pub role: Option<String>,
    pub master_only: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionsMetaResponse {
    pub action_version: u32,
    pub actions: Vec<ActionMetaItem>,
}
