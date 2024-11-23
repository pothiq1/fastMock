// src/main.rs

// Author: Md Hasan Basri
// Email: pothiq@gmail.com

mod models;
mod routes;
mod state;
mod utils;

use crate::routes::{
    delete_mock, delete_mock_internal, get_mock, handle_mock, list_mocks, save_mock,
    save_mock_internal, update_mock, update_mock_internal,
};
use crate::state::AppState;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use dashmap::DashMap;
use env_logger::Env;
use handlebars::Handlebars;
use log::{error, info};
use rust_embed::RustEmbed;
use std::io::Write;
use std::panic;
use std::sync::{Arc, Mutex};
use utils::get_other_pod_ips;
use uuid::Uuid;

use once_cell::sync::Lazy;

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
    match StaticFiles::get(filename) {
        Some(content) => {
            let content_type = match filename.split('.').last() {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                _ => "text/plain",
            };
            HttpResponse::Ok()
                .content_type(content_type)
                .body(content.data)
        }
        None => HttpResponse::NotFound().body("404 - Not Found"),
    }
}

// Initialize a global static instance of Handlebars wrapped in a Mutex
static HANDLEBARS: Lazy<Mutex<Handlebars<'static>>> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();
    utils::register_helpers(&mut handlebars);
    Mutex::new(handlebars)
});

async fn sync_data_from_peers(app_data: Arc<AppState>) -> Result<(), String> {
    let peer_pod_ips = get_other_pod_ips()
        .await
        .map_err(|e| format!("Failed to fetch pod IPs: {}", e))?;
    if peer_pod_ips.is_empty() {
        println!("No peers found. Starting as a fresh instance.");
        return Ok(());
    }

    println!("Discovered peer pods: {:?}", peer_pod_ips);
    app_data
        .sync_data_from_peers(peer_pod_ips)
        .await
        .map_err(|e| e.to_string())
}

async fn run_server(app_data: Arc<AppState>) -> std::io::Result<()> {
    HttpServer::new({
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
                .service(delete_mock_internal)
                .service(update_mock_internal)
                .route("/mock/{api_name}", web::route().to(handle_mock))
                .route("/", web::get().to(index))
                .route("/static/{filename:.*}", web::get().to(static_files))
        }
    })
    .workers(num_cpus::get())
    .max_connections(20_000)
    .backlog(1024)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {:?}", panic_info);
    }));

    info!("Application is starting...");
    std::io::stdout().flush().unwrap();

    // Initialize AppState
    let app_data = Arc::new(AppState {
        mocks: DashMap::new(),
        api_name_to_id: DashMap::new(),
        peer_pods: DashMap::new(),
    });

    // Sync data from peer pods before starting the server
    if let Err(e) = sync_data_from_peers(app_data.clone()).await {
        error!("Error syncing data from peers: {}", e);
    }

    // Start the server
    if let Err(e) = run_server(app_data).await {
        eprintln!("Application error: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
