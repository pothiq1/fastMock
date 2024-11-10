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
use actix_files as fs;
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder};
use dashmap::DashMap;
use env_logger::Env;
use handlebars::Handlebars;
use models::MockAPI;
use num_cpus;
use reqwest::Client;
use utils::{get_other_pod_ips, register_helpers};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"] // Updated to reference the folder directly within /app
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

    if other_pod_ips.is_empty() {
        println!("No peer pods found for synchronization.");
        return;
    }

    let client = Client::new();
    for ip in other_pod_ips {
        let url = format!("http://{}:8080/list-mocks", ip);
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<MockAPI>>().await {
                        Ok(mocks) => {
                            for mock in mocks {
                                if let Some(id) = mock.id {
                                    app_data.mocks.insert(id, mock.clone());
                                    app_data.api_name_to_id.insert(mock.api_name.clone(), id);
                                }
                            }
                            println!("Successfully synchronized mocks from {}", ip);
                            break;
                        }
                        Err(e) => eprintln!("Failed to parse mocks from {}: {}", ip, e),
                    }
                } else {
                    eprintln!("Failed to fetch mocks from {}: Status {}", ip, response.status());
                }
            }
            Err(e) => eprintln!("Error connecting to {}: {}", ip, e),
        }
    }
}

async fn run_server() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let own_ip = std::env::var("POD_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mut handlebars = Handlebars::new();
    register_helpers(&mut handlebars);

    let app_data = Arc::new(AppState {
        mocks: DashMap::new(),
        api_name_to_id: DashMap::new(),
        own_ip,
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
                .service(handle_mock)
                .route("/", web::get().to(index))
                .route("/static/{filename:.*}", web::get().to(static_files))
                .service(fs::Files::new("/static", "./static").show_files_listing())
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
            sleep(Duration::from_secs(2)).await;
            discover_and_sync_peers(app_data).await;
        }
    });

    server.await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = run_server().await {
        eprintln!("Application error: {}", e);
    }
    Ok(())
}