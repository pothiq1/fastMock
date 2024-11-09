// src/routes.rs

use crate::models::MockAPI;
use crate::state::AppState;
use crate::utils::get_other_pod_ips;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use reqwest::Client;
use serde_json::Value;
use tokio::spawn;
use uuid::Uuid;

/// Endpoint to update an existing mock
#[put("/update-mock/{id}")]
async fn update_mock(
    path: web::Path<Uuid>,
    data: web::Json<MockAPI>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mock_id = path.into_inner();
    let updated_mock = data.into_inner();

    // Perform local mutation
    if let Some(mut mock_entry) = state.mocks.get_mut(&mock_id) {
        // If api_name has changed, update the mapping
        if mock_entry.api_name != updated_mock.api_name {
            // Remove the old mapping
            state.api_name_to_id.remove(&mock_entry.api_name);
            // Insert the new mapping
            state
                .api_name_to_id
                .insert(updated_mock.api_name.clone(), mock_id);
        }

        // Update the MockAPI fields
        mock_entry.api_name = updated_mock.api_name.clone();
        mock_entry.response = updated_mock.response.clone();
        mock_entry.status = updated_mock.status;
        mock_entry.delay = updated_mock.delay;
        mock_entry.method = updated_mock.method.clone();
    } else {
        return HttpResponse::NotFound().json("Mock not found");
    }

    // Call get_other_pod_ips without arguments
    let other_pod_ips = match get_other_pod_ips().await {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Failed to get other pod IPs: {}", e);
            Vec::new()
        }
    };

    let client = Client::new();

    for ip in other_pod_ips {
        let url = format!("http://{}:8080/update-mock-internal/{}", ip, mock_id);
        let client_clone = client.clone();
        let updated_mock_clone = updated_mock.clone();

        spawn(async move {
            let _ = client_clone
                .put(&url)
                .json(&updated_mock_clone)
                .send()
                .await;
        });
    }

    HttpResponse::Ok().json(updated_mock)
}

/// Endpoint to retrieve a single mock by ID
#[get("/get-mock/{id}")]
async fn get_mock(
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();

    if let Some(mock_entry) = state.mocks.get(&id) {
        HttpResponse::Ok().json(mock_entry.clone())
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

/// Endpoint to save a new mock
#[post("/save-mock")]
async fn save_mock(
    data: web::Json<MockAPI>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut mock = data.into_inner();
    let mock_id = Uuid::new_v4();
    mock.id = Some(mock_id);

    // Insert into mocks
    state.mocks.insert(mock_id, mock.clone());

    // Map api_name to ID
    state
        .api_name_to_id
        .insert(mock.api_name.clone(), mock_id);

    // Call get_other_pod_ips without arguments
    let other_pod_ips = match get_other_pod_ips().await {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Failed to get other pod IPs: {}", e);
            Vec::new()
        }
    };

    let client = Client::new();

    for ip in other_pod_ips {
        let url = format!("http://{}:8080/save-mock-internal", ip);
        let client_clone = client.clone();
        let mock_clone = mock.clone();

        spawn(async move {
            let _ = client_clone
                    .post(&url)
                    .header("X-Internal-Token", "S8d6xG1dA3fN7K9mA2jH4R6kB8vL0T5w") // Add the token header
                    .json(&mock_clone)
                    .send()
                    .await;
        });
    }

    HttpResponse::Ok().json(mock)
}

/// Endpoint to list all mocks
#[get("/list-mocks")]
async fn list_mocks(state: web::Data<AppState>) -> impl Responder {
    let mocks: Vec<MockAPI> = state
        .mocks
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    HttpResponse::Ok().json(mocks)
}

/// Endpoint to delete a mock by ID
#[delete("/delete-mock/{id}")]
async fn delete_mock(
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();

    // Perform local mutation
    if let Some((_, mock)) = state.mocks.remove(&id) {
        // Remove from api_name_to_id mapping
        state.api_name_to_id.remove(&mock.api_name);
    } else {
        return HttpResponse::NotFound().json("Mock not found");
    }

    // Call get_other_pod_ips without arguments
    let other_pod_ips = match get_other_pod_ips().await {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Failed to get other pod IPs: {}", e);
            Vec::new()
        }
    };

    let client = Client::new();

    for ip in other_pod_ips {
        let url = format!("http://{}:8080/delete-mock-internal/{}", ip, id);
        let client_clone = client.clone();

        spawn(async move {
            let _ = client_clone
            .delete(&url)
            .header("X-Internal-Token", "S8d6xG1dA3fN7K9mA2jH4R6kB8vL0T5w")
            .send()
            .await;
        });
    }

    HttpResponse::Ok().json("Mock deleted successfully")
}

/// Internal endpoint to save a mock (used for synchronization)
#[post("/save-mock-internal")]
async fn save_mock_internal(
    req: HttpRequest,
    data: web::Json<MockAPI>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Validate a custom header for authentication
    if req
        .headers()
        .get("X-Internal-Token")
        .and_then(|h| h.to_str().ok())
        != Some("S8d6xG1dA3fN7K9mA2jH4R6kB8vL0T5w")
    {
        return HttpResponse::Unauthorized().json("Unauthorized");
    }

    let mut mock = data.into_inner();

    // Assign a new UUID if not present
    let mock_id = mock.id.unwrap_or_else(Uuid::new_v4);
    mock.id = Some(mock_id);

    // Insert into mocks
    state.mocks.insert(mock_id, mock.clone());

    // Map api_name to ID
    state.api_name_to_id.insert(mock.api_name.clone(), mock_id);

    HttpResponse::Ok().json("Mock saved internally")
}

/// Internal endpoint to delete a mock by ID (used for synchronization)
#[delete("/delete-mock-internal/{id}")]
async fn delete_mock_internal(
    req: HttpRequest,
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Validate a custom header for authentication
    if req
        .headers()
        .get("X-Internal-Token")
        .and_then(|h| h.to_str().ok())
        != Some("S8d6xG1dA3fN7K9mA2jH4R6kB8vL0T5w")
    {
        return HttpResponse::Unauthorized().json("Unauthorized");
    }

    let id = path.into_inner();

    if let Some((_, mock)) = state.mocks.remove(&id) {
        // Remove from api_name_to_id mapping
        state.api_name_to_id.remove(&mock.api_name);

        HttpResponse::Ok().json("Mock deleted internally")
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

/// Handle mock requests based on api_name with dynamic placeholders
#[get("/mock/{api_name}")]
async fn handle_mock(
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Bytes, // Accept body as `web::Bytes`
    state: web::Data<AppState>,
) -> impl Responder {
    let api_name = path.into_inner();

    // Retrieve the mock ID from api_name
    if let Some(mock_id) = state.api_name_to_id.get(&api_name) {
        if let Some(mock) = state.mocks.get(&mock_id.value()) {
            let mock = mock.clone();

            if req.method() == mock.method.as_str() {
                // Initialize data map
                let mut data = serde_json::Map::new();

                // Extract headers
                for (key, value) in req.headers().iter() {
                    if let Ok(val) = value.to_str() {
                        data.insert(key.to_string(), Value::String(val.to_string()));
                    }
                }

                // Extract query parameters
                for (key, value) in req.query_string().split('&').filter_map(|s| {
                    let mut split = s.splitn(2, '=');
                    let key = split.next()?.to_string();
                    let value = split.next().unwrap_or("").to_string();
                    Some((key, value))
                }) {
                    data.insert(key, Value::String(value));
                }

                // Extract request body
                if let Some(content_type) = req.headers().get("Content-Type") {
                    if content_type.to_str().unwrap_or("").contains("application/json") {
                        // Read and parse the request body
                        let json_body: Value = match serde_json::from_slice(&body) {
                            Ok(json) => json,
                            Err(e) => {
                                eprintln!("Failed to parse JSON body: {}", e);
                                return HttpResponse::BadRequest().json("Failed to parse JSON body");
                            }
                        };
                        if let Some(obj) = json_body.as_object() {
                            data.extend(obj.clone());
                        }
                    }
                }

                // Render the response template using Handlebars
                let handlebars = &state.handlebars;
                let rendered = match handlebars.render_template(&mock.response, &data) {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("Template rendering error: {}", e);
                        return HttpResponse::InternalServerError()
                            .json("Template rendering error");
                    }
                };

                // Introduce delay if specified
                if mock.delay > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(mock.delay)).await;
                }

                // Return the rendered response with the specified status code
                HttpResponse::build(
                    actix_web::http::StatusCode::from_u16(mock.status)
                        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
                )
                .append_header(("Content-Type", "application/json"))
                .body(rendered)
            } else {
                HttpResponse::MethodNotAllowed().json("Method not allowed for this mock")
            }
        } else {
            HttpResponse::NotFound().json("Mock not found")
        }
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}