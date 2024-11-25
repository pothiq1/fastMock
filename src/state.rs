// src/state.rs

use anyhow::Result;
use dashmap::DashMap;
use handlebars::Handlebars;
use log::{error, info};
use reqwest::Client;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::models::MockAPI;

/// Application state
pub struct AppState {
    pub mocks: DashMap<Uuid, MockAPI>,
    pub api_name_to_id: DashMap<String, Uuid>,
    pub handlebars: Arc<Mutex<Handlebars<'static>>>, // Changed to Arc<Mutex<Handlebars>>
    pub synced_peers: AtomicUsize,                   // Counter for synchronized peers
    pub request_count: AtomicUsize,                  // Request counter
}

impl AppState {
    /// Sync data from another pod with retries and timestamp comparison
    pub async fn sync_data_from_peer(&self, peer_ip: &str) -> Result<()> {
        let client = Client::new();
        let url = format!("http://{}:8080/list-mocks", peer_ip);

        for attempt in 1..=3 {
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(peer_mocks) = response.json::<Vec<MockAPI>>().await {
                            for peer_mock in peer_mocks {
                                if let Some(id) = peer_mock.id {
                                    if let Some(mut local_mock) = self.mocks.get_mut(&id) {
                                        // Compare timestamps
                                        if peer_mock.timestamp > local_mock.timestamp {
                                            // Update local mock with peer's mock
                                            *local_mock = peer_mock.clone();
                                            self.api_name_to_id
                                                .insert(peer_mock.api_name.clone(), id);
                                            info!("Updated mock {} from peer {}", id, peer_ip);

                                            // Register the templates
                                            self.register_mock_templates(&peer_mock);
                                        }
                                    } else {
                                        // Insert new mock
                                        self.mocks.insert(id, peer_mock.clone());
                                        self.api_name_to_id.insert(peer_mock.api_name.clone(), id);
                                        info!("Added new mock {} from peer {}", id, peer_ip);

                                        // Register the templates
                                        self.register_mock_templates(&peer_mock);
                                    }
                                }
                            }
                            info!("Successfully synchronized mocks from {}", peer_ip);
                            self.synced_peers.fetch_add(1, Ordering::SeqCst); // Increment synchronized peers
                            return Ok(());
                        } else {
                            error!("Failed to parse mocks from peer {}", peer_ip);
                        }
                    } else {
                        error!(
                            "Attempt {}: Failed to fetch mocks from {}: Status {}",
                            attempt,
                            peer_ip,
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Attempt {}: Error connecting to peer {}: {}",
                        attempt, peer_ip, e
                    );
                }
            }
            let backoff = Duration::from_secs(2_u64.pow(attempt));
            sleep(backoff).await;
        }
        error!(
            "Failed to synchronize mocks from {} after multiple attempts",
            peer_ip
        );
        Err(anyhow::anyhow!(
            "Failed to synchronize mocks from {}",
            peer_ip
        ))
    }

    /// Register templates for a mock's response variants
    pub fn register_mock_templates(&self, mock: &MockAPI) {
        let mut handlebars = self.handlebars.lock().unwrap();
        for (index, variant) in mock.response_variants.iter().enumerate() {
            let template_name = format!("{}_{}", mock.id.unwrap(), index);
            if let Err(e) = handlebars.register_template_string(&template_name, &variant.response) {
                error!("Error compiling template {}: {}", template_name, e);
            }

            // Register response headers templates
            if let Some(headers) = &variant.response_headers {
                for (header_name, header_value) in headers {
                    let header_template_name =
                        format!("{}_{}_header_{}", mock.id.unwrap(), index, header_name);
                    if let Err(e) =
                        handlebars.register_template_string(&header_template_name, header_value)
                    {
                        error!(
                            "Error compiling header template {}: {}",
                            header_template_name, e
                        );
                    }
                }
            }
        }
    }
}
