// src/state.rs

use crate::models::MockAPI;
use dashmap::DashMap;
use handlebars::Handlebars;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub mocks: DashMap<Uuid, MockAPI>,
    pub api_name_to_id: DashMap<String, Uuid>,
    pub own_ip: String,
    pub handlebars: Arc<Handlebars<'static>>,
    #[allow(dead_code)]
    pub peer_pods: DashMap<String, ()>, // Store IP addresses of peer pods
}

impl AppState {
    /// Sync data from another pod
    #[allow(dead_code)]
    pub async fn sync_data_from_peer(&self, peer_ip: &str) {
        let client = Client::new();
        let url = format!("http://{}/list-mocks", peer_ip);
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Deserialize list of mocks from peer
                    if let Ok(mocks) = response.json::<Vec<MockAPI>>().await {
                        for mock in mocks {
                            if let Some(id) = mock.id {
                                // Insert the mock data into local storage
                                self.mocks.insert(id, mock.clone());
                                self.api_name_to_id.insert(mock.api_name.clone(), id);
                            }
                        }
                        println!("Successfully synchronized mocks from {}", peer_ip);
                    } else {
                        eprintln!("Failed to parse mocks from peer {}", peer_ip);
                    }
                } else {
                    eprintln!("Failed to fetch mocks from {}: Status {}", peer_ip, response.status());
                }
            }
            Err(e) => eprintln!("Error connecting to peer {}: {}", peer_ip, e),
        }
    }
}