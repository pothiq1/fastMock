// src/routes.rs

use crate::models::{MockAPI, ResponseVariant};
use crate::state::AppState;
use crate::utils::get_other_pod_ips;
use actix_web::{
    delete, get, post, put, web, HttpRequest, HttpResponse, Responder, Result as ActixResult,
};
use actix_web_codegen::route;
use log::{error, info};
use meval::eval_str;
use rand::Rng;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tokio::spawn;
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
        *mock_entry = updated_mock.clone();

        // Register templates for response variants
        state.register_mock_templates(&updated_mock);
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

    // Register templates for response variants
    state.register_mock_templates(&mock);

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

        // Unregister the templates
        let mut handlebars = state.handlebars.lock().unwrap();
        for (index, _) in mock.response_variants.iter().enumerate() {
            let template_name = format!("{}_{}", id, index);
            handlebars.unregister_template(&template_name);
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

    // Register templates for response variants
    state.register_mock_templates(&mock);

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

            // Register templates for response variants
            state.register_mock_templates(&updated_mock);

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

        // Register templates for response variants
        state.register_mock_templates(&updated_mock);

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

        // Unregister the templates
        let mut handlebars = state.handlebars.lock().unwrap();
        for (index, _) in mock.response_variants.iter().enumerate() {
            let template_name = format!("{}_{}", id, index);
            handlebars.unregister_template(&template_name);
        }

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
    query: web::Query<HashMap<String, String>>,
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> impl Responder {
    state.request_count.fetch_add(1, Ordering::SeqCst);

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

                // Parse request body if JSON
                if let Some(content_type) = req.headers().get("Content-Type") {
                    if content_type
                        .to_str()
                        .unwrap_or("")
                        .contains("application/json")
                    {
                        if !body.is_empty() {
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

                // Select response variant
                let variant = match select_response_variant(&mock.response_variants, &data) {
                    Some(v) => v,
                    None => {
                        return HttpResponse::InternalServerError()
                            .json("No matching response variant found")
                    }
                };

                // Render the response template
                let template_name = format!("{}_{}", mock_id.value(), variant.0);
                let handlebars = state.handlebars.lock().unwrap();
                let rendered = match handlebars.render(&template_name, &data) {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("Template rendering error: {}", e);
                        return HttpResponse::InternalServerError()
                            .json(format!("Template rendering error: {}", e));
                    }
                };

                // Prepare the response builder
                let mut response_builder = HttpResponse::build(
                    actix_web::http::StatusCode::from_u16(variant.1.status)
                        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
                );

                // Set Content-Type header
                response_builder.append_header(("Content-Type", "application/json"));

                // Process response headers
                if let Some(headers) = &variant.1.response_headers {
                    for (key, value_template) in headers {
                        // Render header value using Handlebars
                        let header_template_name =
                            format!("{}_{}_header_{}", mock_id.value(), variant.0, key);
                        let rendered_value = match handlebars.render(&header_template_name, &data) {
                            Ok(val) => val,
                            Err(e) => {
                                eprintln!("Header template rendering error: {}", e);
                                return HttpResponse::InternalServerError()
                                    .json(format!("Header template rendering error: {}", e));
                            }
                        };
                        // Add the header to the response
                        response_builder.append_header((key.as_str(), rendered_value));
                    }
                }

                // Introduce delay if specified
                if let Some(delay) = variant.1.delay {
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                // Return the response
                response_builder.body(rendered)
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

fn select_response_variant<'a>(
    variants: &'a [ResponseVariant],
    data: &serde_json::Map<String, Value>,
) -> Option<(usize, &'a ResponseVariant)> {
    // Filter variants based on conditions
    let filtered_variants: Vec<(usize, &'a ResponseVariant)> = variants
        .iter()
        .enumerate()
        .filter(|(_, variant)| {
            if let Some(condition) = &variant.condition {
                match evaluate_condition(condition, data) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("Condition evaluation error: {}", e);
                        false
                    }
                }
            } else {
                true
            }
        })
        .collect();

    if filtered_variants.is_empty() {
        return None;
    }

    // Calculate total weight
    let total_weight: u32 = filtered_variants.iter().map(|(_, v)| v.weight).sum();
    if total_weight == 0 {
        return None;
    }

    // Select variant based on weight
    let mut rng = rand::thread_rng();
    let mut cumulative_weight = 0;
    let selected_weight = rng.gen_range(0..total_weight);
    for (index, variant) in filtered_variants {
        cumulative_weight += variant.weight;
        if selected_weight < cumulative_weight {
            return Some((index, variant));
        }
    }
    None
}

fn evaluate_condition(
    condition: &str,
    data: &serde_json::Map<String, Value>,
) -> Result<bool, String> {
    let mut expr = condition.to_string();
    for (key, value) in data {
        if let Some(str_value) = value.as_str() {
            expr = expr.replace(&format!("{{{{{}}}}}", key), str_value);
        } else if let Some(num_value) = value.as_f64() {
            expr = expr.replace(&format!("{{{{{}}}}}", key), &num_value.to_string());
        }
    }
    eval_str(&expr).map(|v| v != 0.0).map_err(|e| e.to_string())
}
