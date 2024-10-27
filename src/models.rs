// src/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct MockAPI {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>, // Make `id` optional
    pub api_name: String,
    pub response: String,
    pub status: u16,
    pub delay: u64,
    pub method: String,
}
