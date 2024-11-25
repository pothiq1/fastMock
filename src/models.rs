// src/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Struct representing a response variant with weight and condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseVariant {
    pub response: String, // Can contain Handlebars placeholders
    pub weight: u32,      // Weight for weighted responses
    pub status: u16,
    pub response_headers: Option<HashMap<String, String>>, // Custom response headers
    pub condition: Option<String>,                         // Condition to evaluate
    pub delay: Option<u64>,                                // Delay in milliseconds
}

/// Struct representing a mock API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAPI {
    pub id: Option<Uuid>,
    pub api_name: String,
    pub method: String,                          // HTTP method (e.g., GET, POST)
    pub timestamp: DateTime<Utc>,                // Timestamp field
    pub response_variants: Vec<ResponseVariant>, // Multiple response variants
}
