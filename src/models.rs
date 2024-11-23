// src/models.rs

// Author: Md Hasan Basri
// Email: pothiq@gmail.com

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Struct representing a mock API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAPI {
    pub id: Option<Uuid>,
    pub api_name: String,
    pub response: String, // Can contain Handlebars placeholders
    pub status: u16,
    pub delay: u64,     // Delay in milliseconds
    pub method: String, // HTTP method (e.g., GET, POST)
}
