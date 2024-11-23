// src/main.rs

// Author: Md Hasan Basri
// Email: pothiq@gmail.com

mod models;
mod routes;
mod state;
mod utils;

use crate::routes::{
    delete_mock, delete_mock_internal, get_mock, handle_mock, list_mocks, save_mock,
    save_mock_internal, update_mock,
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
use std::sync::Arc;
use tokio::time::{sleep, Duration};
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

async fn discover_and_sync_peers(app_data: Arc<AppState>) {
    sleep(Duration::from_secs(2)).await;

    let other_pod_ips = match get_other_pod_ips().await {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Failed to get other pod IPs: {}", e);
            Vec::new()
        }
    };

    for ip in other_pod_ips {
        println!("Discovered peer pod at IP: {}", ip);
        app_data.peer_pods.insert(ip.clone(), ());

        // Call sync_data_from_peer here
        let app_data_clone = app_data.clone();
        let ip_clone = ip.clone();
        tokio::spawn(async move {
            if let Err(e) = app_data_clone.sync_data_from_peer(&ip_clone).await {
                error!("Error syncing data from peer {}: {}", ip_clone, e);
            }
        });
    }
}

async fn run_server() -> std::io::Result<()> {
    // Remove the duplicate logger initialization
    // env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Initialize Handlebars and register helpers
    let mut handlebars = Handlebars::new();
    utils::register_helpers(&mut handlebars);

    let app_data = Arc::new(AppState {
        mocks: DashMap::new(),
        api_name_to_id: DashMap::new(),
        handlebars: Arc::new(handlebars),
        peer_pods: DashMap::new(),
    });

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
                .service(delete_mock_internal)
                .route("/mock/{api_name}", web::route().to(handle_mock))
                .route("/", web::get().to(index))
                .route("/static/{filename:.*}", web::get().to(static_files))
            // Remove the following line if using RustEmbed
            // .service(fs::Files::new("/static", "./static").show_files_listing())
        }
    })
    .workers(num_cpus::get())
    .max_connections(20_000)
    .backlog(1024)
    .bind("0.0.0.0:8080")?
    .run();

    tokio::spawn({
        let app_data = app_data.clone();
        async move {
            discover_and_sync_peers(app_data).await;
        }
    });

    server.await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {:?}", panic_info);
    }));

    info!("Application is starting...");
    std::io::stdout().flush().unwrap();

    if let Err(e) = run_server().await {
        eprintln!("Application error: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
