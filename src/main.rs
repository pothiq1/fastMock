// src/main.rs

mod models;
mod routes;
mod state;
mod utils;

use crate::routes::{
    delete_mock, delete_mock_internal, get_mock, handle_mock, list_mocks, save_mock,
    save_mock_internal, update_mock,
};
use crate::state::AppState;
use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder, Result as ActixResult};
use dashmap::DashMap;
use env_logger::Env;
use handlebars::Handlebars;
use models::MockAPI;
use num_cpus;
use reqwest::Client;
use utils::{get_other_pod_ips, register_helpers};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

async fn index() -> ActixResult<impl Responder> {
    let path = std::path::PathBuf::from("src/static/index.html");
    match fs::NamedFile::open_async(path).await {
        Ok(file) => Ok(file),
        Err(_) => Ok(
            fs::NamedFile::open_async("src/static/404.html")
                .await?
                .use_last_modified(true),
        ),
    }
}

// Function to handle discovery and synchronization from peers
/// Discover peer pods and synchronize mocks from the first available peer.
async fn discover_and_sync_peers(app_data: Arc<AppState>) {
    // Wait a few seconds to allow the server to start
    sleep(Duration::from_secs(2)).await;

    // Call get_other_pod_ips without arguments
    let other_pod_ips = match get_other_pod_ips().await {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Failed to get other pod IPs: {}", e);
            Vec::new()
        }
    };

    if other_pod_ips.is_empty() {
        println!("No peer pods found for synchronization.");
        return;
    }

    // Initialize HTTP client for synchronization requests
    let client = Client::new();

    // Iterate over each peer pod IP and attempt to fetch mock data
    for ip in other_pod_ips {
        let url = format!("http://{}:8080/list-mocks", ip);
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<MockAPI>>().await {
                        Ok(mocks) => {
                            // Populate the local state with mocks from the peer pod
                            for mock in mocks {
                                if let Some(id) = mock.id {
                                    app_data.mocks.insert(id, mock.clone());
                                    app_data.api_name_to_id.insert(mock.api_name.clone(), id);
                                }
                            }
                            println!("Successfully synchronized mocks from {}", ip);
                            break; // Stop after syncing with the first available peer
                        }
                        Err(e) => eprintln!("Failed to parse mocks from {}: {}", ip, e),
                    }
                } else {
                    eprintln!(
                        "Failed to fetch mocks from {}: Status {}",
                        ip,
                        response.status()
                    );
                }
            }
            Err(e) => eprintln!("Error connecting to {}: {}", ip, e),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment logger for debugging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Get own IP address
    let own_ip = std::env::var("POD_IP").unwrap_or_else(|_| "127.0.0.1".to_string());

    // Initialize Handlebars and declare it as mutable
    let mut handlebars = Handlebars::new();
    register_helpers(&mut handlebars); // Now handlebars can be used as mutable

    // Shared application state with in-memory storage for mocks and Handlebars
    let app_data = Arc::new(AppState {
        mocks: DashMap::new(),
        api_name_to_id: DashMap::new(),
        own_ip,
        handlebars: Arc::new(handlebars), // No need to make handlebars mutable here
        peer_pods: DashMap::new(),
    });

    // Set up and run the server
    let server = HttpServer::new({
        let app_data = app_data.clone();
        move || {
            App::new()
                .app_data(web::Data::from(app_data.clone())) // Share app state
                .service(save_mock) // Register routes
                .service(list_mocks)
                .service(delete_mock)
                .service(update_mock)
                .service(get_mock)
                .service(save_mock_internal)
                .service(delete_mock_internal)
                .service(handle_mock) // Register handle_mock route
                .route("/", web::get().to(index)) // Serve main index
                .service(fs::Files::new("/static", "./src/static").show_files_listing()) // Serve static files
        }
    })
    .workers(num_cpus::get()) // Number of worker threads
    .max_connections(20_000)  // Increase maximum connections
    .backlog(1024)            // TCP backlog
    .bind("0.0.0.0:8080")?
    .run();

    // Spawn a task to synchronize with other pods after startup
    tokio::spawn({
        let app_data = app_data.clone();
        async move {
            // Wait for the server to start
            sleep(Duration::from_secs(2)).await;
            discover_and_sync_peers(app_data).await;
        }
    });

    // Await the server to run
    server.await
}