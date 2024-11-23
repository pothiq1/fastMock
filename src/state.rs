// Author: Md Hasan Basri
// Email: pothiq@gmail.com

use crate::models::MockAPI;
use anyhow::Error;
use dashmap::DashMap;
use handlebars::Handlebars;
use log::{error, info, warn};
use reqwest::Client;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub mocks: DashMap<Uuid, MockAPI>,
    pub api_name_to_id: DashMap<String, Uuid>,
    // Remove this line:
    // pub own_ip: String,
    pub handlebars: Arc<Handlebars<'static>>,
    pub peer_pods: DashMap<String, ()>, // Add this field
}

impl AppState {
    /// Sync data from another pod with retries
    pub async fn sync_data_from_peer(&self, peer_ip: &str) -> Result<(), Error> {
        let client = Client::new();
        let url = format!("http://{}/list-mocks", peer_ip);

        for attempt in 1..=3 {
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(mocks) = response.json::<Vec<MockAPI>>().await {
                            for mock in mocks {
                                if let Some(id) = mock.id {
                                    self.mocks.insert(id, mock.clone());
                                    self.api_name_to_id.insert(mock.api_name.clone(), id);
                                }
                            }
                            info!("Successfully synchronized mocks from {}", peer_ip);
                            return Ok(());
                        } else {
                            error!("Failed to parse mocks from peer {}", peer_ip);
                        }
                    } else {
                        warn!(
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
            sleep(Duration::from_secs(2)).await;
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
}
