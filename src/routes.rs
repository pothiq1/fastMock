// src/routes.rs

// Author: Md Hasan Basri
// Email: pothiq@gmail.com

use crate::models::MockAPI;
use crate::state::AppState;
use crate::utils::get_other_pod_ips;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use actix_web_codegen::route; // Import the route attribute macro from actix_web_codegen
use log::{error, info};
use reqwest::Client;
use serde_json::Value;
use std::sync::atomic::Ordering;
use tokio::spawn; // <-- Add this line
use uuid::Uuid;

/// Endpoint to update an existing mock
#[put("/update-mock/{id}")]
pub async fn update_mock(
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
        mock_entry.timestamp = updated_mock.timestamp; // Update timestamp

        // Register the template
        let template_name = mock_id.to_string();
        let mut handlebars = state.handlebars.lock().unwrap();
        match handlebars.register_template_string(&template_name, &updated_mock.response) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error compiling template: {}", e);
                return HttpResponse::InternalServerError().json("Error compiling template");
            }
        }
    } else {
        return HttpResponse::NotFound().json("Mock not found");
    }

    // Synchronize with other pods
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
pub async fn get_mock(path: web::Path<Uuid>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();

    if let Some(mock_entry) = state.mocks.get(&id) {
        HttpResponse::Ok().json(mock_entry.clone())
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

/// Endpoint to save a new mock
#[post("/save-mock")]
pub async fn save_mock(data: web::Json<MockAPI>, state: web::Data<AppState>) -> impl Responder {
    let mut mock = data.into_inner();
    let mock_id = Uuid::new_v4();
    mock.id = Some(mock_id);

    // Register the template
    let template_name = mock_id.to_string();
    let mut handlebars = state.handlebars.lock().unwrap();
    match handlebars.register_template_string(&template_name, &mock.response) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error compiling template: {}", e);
            return HttpResponse::InternalServerError().json("Error compiling template");
        }
    }

    // Insert into mocks
    state.mocks.insert(mock_id, mock.clone());

    // Map api_name to ID
    state.api_name_to_id.insert(mock.api_name.clone(), mock_id);

    // Synchronize with other pods
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
                .header("X-Internal-Token", "S8d6xG1dA3fN7K9mA2jH4R6kB8vL0T5w")
                .json(&mock_clone)
                .send()
                .await;
        });
    }

    HttpResponse::Ok().json(mock)
}

/// Endpoint to list all mocks
#[get("/list-mocks")]
pub async fn list_mocks(state: web::Data<AppState>) -> impl Responder {
    let mocks: Vec<MockAPI> = state
        .mocks
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    HttpResponse::Ok().json(mocks)
}

/// Endpoint to delete a mock by ID
#[delete("/delete-mock/{id}")]
pub async fn delete_mock(path: web::Path<Uuid>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();

    // Perform local mutation
    if let Some((_, mock)) = state.mocks.remove(&id) {
        // Remove from api_name_to_id mapping
        state.api_name_to_id.remove(&mock.api_name);

        // Unregister the template
        let template_name = id.to_string();
        let mut handlebars = state.handlebars.lock().unwrap();
        handlebars.unregister_template(&template_name);
    } else {
        return HttpResponse::NotFound().json("Mock not found");
    }

    // Synchronize with other pods
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

/// Internal endpoint to delete all mocks (used for synchronization)
#[delete("/delete-all-mocks-internal")]
pub async fn delete_all_mocks_internal(
    req: HttpRequest,
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

    info!("Received internal request to delete all mocks");

    // Perform local mutation
    state.mocks.clear();
    state.api_name_to_id.clear();

    // Clear all registered templates
    let mut handlebars = state.handlebars.lock().unwrap();
    handlebars.clear_templates();

    info!("Internal mocks, API mappings, and templates cleared");

    HttpResponse::Ok().json("All mocks deleted internally")
}

/// Endpoint to delete all mocks with synchronization
#[delete("/delete-all-mocks")]
pub async fn delete_all_mocks(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to delete all mocks");

    // Perform local mutation
    state.mocks.clear();
    state.api_name_to_id.clear();

    // Clear all registered templates
    let mut handlebars = state.handlebars.lock().unwrap();
    handlebars.clear_templates();

    info!("Local mocks, API mappings, and templates cleared");

    // Synchronize with other pods
    let other_pod_ips = match get_other_pod_ips().await {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Failed to get other pod IPs: {}", e);
            Vec::new()
        }
    };

    let client = Client::new();

    for ip in other_pod_ips {
        let url = format!("http://{}:8080/delete-all-mocks-internal", ip);
        let client_clone = client.clone();

        spawn(async move {
            match client_clone
                .delete(&url)
                .header("X-Internal-Token", "S8d6xG1dA3fN7K9mA2jH4R6kB8vL0T5w")
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Successfully deleted mocks on peer {}", ip);
                    } else {
                        error!(
                            "Failed to delete mocks on peer {}: Status {}",
                            ip,
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    error!("Error deleting mocks on peer {}: {}", ip, e);
                }
            }
        });
    }

    HttpResponse::Ok().json("All mocks deleted successfully")
}

/// Health check endpoint
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json("OK")
}

/// Readiness check endpoint
#[get("/ready")]
pub async fn readiness_check(state: web::Data<AppState>) -> impl Responder {
    // Check if synchronization with peers has occurred
    if state.synced_peers.load(Ordering::SeqCst) == 0 {
        return HttpResponse::ServiceUnavailable().json("Not synchronized with peers yet");
    }

    // All readiness checks passed
    HttpResponse::Ok().json("Ready")
}

/// Internal endpoint to save a mock (used for synchronization)
#[post("/save-mock-internal")]
pub async fn save_mock_internal(
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

    // Register the template
    let template_name = mock_id.to_string();
    let mut handlebars = state.handlebars.lock().unwrap();
    match handlebars.register_template_string(&template_name, &mock.response) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error compiling template: {}", e);
            return HttpResponse::InternalServerError().json("Error compiling template");
        }
    }

    // Insert into mocks
    state.mocks.insert(mock_id, mock.clone());

    // Map api_name to ID
    state.api_name_to_id.insert(mock.api_name.clone(), mock_id);

    HttpResponse::Ok().json("Mock saved internally")
}

/// Internal endpoint to update a mock (used for synchronization)
#[put("/update-mock-internal/{id}")]
pub async fn update_mock_internal(
    req: HttpRequest,
    path: web::Path<Uuid>,
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

    let mock_id = path.into_inner();
    let updated_mock = data.into_inner();

    // Perform local mutation
    if let Some(mut mock_entry) = state.mocks.get_mut(&mock_id) {
        if updated_mock.timestamp > mock_entry.timestamp {
            // Update the mock only if the incoming timestamp is newer
            *mock_entry = updated_mock.clone();
            state
                .api_name_to_id
                .insert(updated_mock.api_name.clone(), mock_id);

            // Register the template
            let template_name = mock_id.to_string();
            let mut handlebars = state.handlebars.lock().unwrap();
            match handlebars.register_template_string(&template_name, &updated_mock.response) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error compiling template: {}", e);
                    return HttpResponse::InternalServerError().json("Error compiling template");
                }
            }

            HttpResponse::Ok().json("Mock updated internally")
        } else {
            HttpResponse::Ok().json("Local mock is newer or equal; no update performed")
        }
    } else {
        // Insert new mock
        state.mocks.insert(mock_id, updated_mock.clone());
        state
            .api_name_to_id
            .insert(updated_mock.api_name.clone(), mock_id);

        // Register the template
        let template_name = mock_id.to_string();
        let mut handlebars = state.handlebars.lock().unwrap();
        match handlebars.register_template_string(&template_name, &updated_mock.response) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error compiling template: {}", e);
                return HttpResponse::InternalServerError().json("Error compiling template");
            }
        }

        HttpResponse::Ok().json("Mock inserted internally")
    }
}

/// Internal endpoint to delete a mock by ID (used for synchronization)
#[delete("/delete-mock-internal/{id}")]
pub async fn delete_mock_internal(
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

        // Unregister the template
        let template_name = id.to_string();
        let mut handlebars = state.handlebars.lock().unwrap();
        handlebars.unregister_template(&template_name);

        HttpResponse::Ok().json("Mock deleted internally")
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

/// Handle mock requests based on api_name with dynamic methods and placeholders
#[route(
    "/mock/{api_name:.*}",
    method = "GET",
    method = "POST",
    method = "PUT",
    method = "DELETE"
)]
pub async fn handle_mock(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> impl Responder {
    let api_name = path.into_inner();

    // Retrieve the mock ID from api_name
    if let Some(mock_id) = state.api_name_to_id.get(&api_name) {
        if let Some(mock) = state.mocks.get(&mock_id.value()) {
            let mock = mock.clone();

            if req.method().as_str().eq_ignore_ascii_case(&mock.method) {
                let mut data = serde_json::Map::new();

                // Add api_name to data
                data.insert("api_name".to_string(), Value::String(api_name.clone()));

                // Extract headers
                for (key, value) in req.headers().iter() {
                    if let Ok(val) = value.to_str() {
                        data.insert(key.to_string(), Value::String(val.to_string()));
                    }
                }

                // Extract query parameters
                for (key, value) in query.into_inner() {
                    data.insert(key, Value::String(value));
                }

                // Determine if the template uses variables from the body
                let uses_body = mock.response.contains("{{") && mock.response.contains("}}");

                // Parse request body only if necessary
                if uses_body && !body.is_empty() {
                    if let Some(content_type) = req.headers().get("Content-Type") {
                        if content_type
                            .to_str()
                            .unwrap_or("")
                            .contains("application/json")
                        {
                            // Parse the JSON body
                            let json_body: Value = match serde_json::from_slice(&body) {
                                Ok(json) => json,
                                Err(e) => {
                                    eprintln!("Failed to parse JSON body: {}", e);
                                    return HttpResponse::BadRequest()
                                        .json("Failed to parse JSON body");
                                }
                            };
                            // Merge JSON body into data
                            merge_json(&mut data, &json_body);
                        }
                    }
                }

                // Render the response template using registered template
                let template_name = mock_id.value().to_string();
                let handlebars = state.handlebars.lock().unwrap();
                let rendered = match handlebars.render(&template_name, &data) {
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

fn merge_json(data: &mut serde_json::Map<String, Value>, value: &Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                data.insert(k.clone(), v.clone());
            }
        }
        _ => {}
    }
}
