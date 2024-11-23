// src/main.rs

// Author: Md Hasan Basri
// Email: pothiq@gmail.com

mod models;
mod routes;
mod state;
mod utils;

use crate::routes::{
    delete_all_mocks, delete_all_mocks_internal, delete_mock, get_mock, handle_mock, health_check,
    list_mocks, readiness_check, save_mock, save_mock_internal, update_mock, update_mock_internal,
};
use crate::state::AppState;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use dashmap::DashMap;
use env_logger::Env;
use handlebars::Handlebars;
use log::{error, info};
use routes::delete_mock_internal;
use rust_embed::RustEmbed;
use std::io::Write;
use std::panic;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::time::{interval, sleep, Duration};
use utils::get_other_pod_ips;

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticFiles;

async fn index() -> HttpResponse {
    match StaticFiles::get("index.html") {
        Some(content) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content.data),
        None => HttpResponse::NotFound().body("404 - Not Found"),
    }
}

async fn static_files(req: HttpRequest) -> HttpResponse {
    let filename: &str = req.match_info().query("filename");
    let file_path = format!("static/{}", filename); // Ensure this matches your directory structure
    match StaticFiles::get(&file_path) {
        Some(content) => {
            let content_type = match filename.split('.').last() {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("html") => "text/html",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("ico") => "image/x-icon",
                _ => "application/octet-stream",
            };
            HttpResponse::Ok()
                .content_type(content_type)
                .body(content.data)
        }
        None => HttpResponse::NotFound().body("404 - Not Found"),
    }
}

/// Function to discover peer pods and synchronize mocks
async fn discover_and_sync_peers(app_data: Arc<AppState>) {
    // Wait a bit to ensure the pod is fully initialized
    sleep(Duration::from_secs(2)).await;

    // Periodically sync with peers
    let mut sync_interval = interval(Duration::from_secs(60)); // Every 60 seconds

    loop {
        sync_interval.tick().await;

        let peer_pod_ips = match get_other_pod_ips().await {
            Ok(ips) => ips,
            Err(e) => {
                eprintln!("Failed to get other pod IPs: {}", e);
                continue;
            }
        };

        if peer_pod_ips.is_empty() {
            println!("No peers found. Skipping synchronization.");
            continue;
        }

        println!("Discovered peer pods: {:?}", &peer_pod_ips);

        for ip in &peer_pod_ips {
            println!("Attempting to synchronize with peer at IP: {}", ip);
            let app_data_clone = app_data.clone();
            let ip_clone = ip.clone();
            tokio::spawn(async move {
                if let Err(e) = app_data_clone.sync_data_from_peer(&ip_clone).await {
                    error!("Error syncing data from peer {}: {}", ip_clone, e);
                }
            });
        }
    }
}

async fn run_server() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {:?}", panic_info);
    }));

    info!("Application is starting...");
    std::io::stdout().flush().unwrap();

    // Initialize Handlebars and register helpers
    let mut handlebars = Handlebars::new();
    utils::register_helpers(&mut handlebars);

    let app_data = Arc::new(AppState {
        mocks: DashMap::new(),
        api_name_to_id: DashMap::new(),
        handlebars: Arc::new(handlebars),
        peer_pods: DashMap::new(),
        synced_peers: AtomicUsize::new(0), // Initialize synced_peers
    });

    // Start the peer discovery and synchronization in the background
    {
        let app_data_clone = app_data.clone();
        tokio::spawn(async move {
            discover_and_sync_peers(app_data_clone).await;
        });
    }

    let server = HttpServer::new({
        let app_data = app_data.clone();
        move || {
            App::new()
                .app_data(web::Data::from(app_data.clone()))
                .service(save_mock)
                .service(list_mocks)
                .service(delete_mock)
                .service(update_mock)
                .service(get_mock)
                .service(save_mock_internal)
                .service(update_mock_internal)
                .service(delete_mock_internal)
                .service(delete_all_mocks)
                .service(delete_all_mocks_internal)
                .service(health_check) // Register /health
                .service(readiness_check) // Register /ready
                .service(handle_mock) // Register dynamic mock handler
                .route("/", web::get().to(index))
                .route("/static/{filename:.*}", web::get().to(static_files))
        }
    })
    .workers(num_cpus::get())
    .max_connections(20_000)
    .backlog(1024)
    .bind("0.0.0.0:8080")?
    .run();

    server.await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = run_server().await {
        eprintln!("Application error: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
