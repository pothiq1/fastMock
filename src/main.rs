mod models;
mod routes;
mod state;

use crate::routes::{delete_mock, list_mocks, save_mock, update_mock, get_mock};
use crate::state::AppState;
use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result, http::Method};
use env_logger::Env;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serve the main index.html page or a 404 page if the file is not found.
async fn index() -> Result<impl Responder> {
    let path = PathBuf::from("src/static/index.html");
    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(_) => Ok(fs::NamedFile::open("src/static/404.html")?.use_last_modified(true)),
    }
}

/// Handle requests to mock endpoints dynamically based on the saved `api_name`.
async fn handle_mock(
    api_name: web::Path<String>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let mocks = state.mocks.lock().unwrap();

    // Search for a mock with the given API name
    if let Some(mock) = mocks.iter().find(|m| m.api_name == *api_name) {
        // Check if the request method matches the mock's specified method
        if req.method() == Method::from_bytes(mock.method.as_bytes()).unwrap() {
            HttpResponse::build(actix_web::http::StatusCode::from_u16(mock.status).unwrap())
                .header("Content-Type", "application/json")
                .body(mock.response.clone())
        } else {
            HttpResponse::MethodNotAllowed().json("Method not allowed for this mock")
        }
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment logger for debugging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Shared application state with in-memory storage for mocks
    let app_data = web::Data::new(AppState {
        mocks: Mutex::new(Vec::new()),
    });

    // Set up and run the server
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone()) // Share app state
            .service(save_mock)         // Register routes
            .service(list_mocks)
            .service(delete_mock)
            .service(update_mock)
            .service(get_mock)
            .route("/", web::get().to(index)) // Serve main index
            .route("/mock/{api_name}", web::get().to(handle_mock)) // Dynamic mock route
            .service(fs::Files::new("/static", "./src/static").show_files_listing()) // Serve static files
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
