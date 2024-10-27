// src/routes.rs
use crate::models::MockAPI;
use crate::state::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// Endpoint to update an existing mock
#[put("/update-mock/{id}")]
async fn update_mock(path: web::Path<Uuid>, data: web::Json<MockAPI>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mut mocks = state.mocks.lock().unwrap();

    if let Some(mock) = mocks.iter_mut().find(|m| m.id == Some(id)) {
        mock.api_name = data.api_name.clone();
        mock.response = data.response.clone();
        mock.status = data.status;
        mock.delay = data.delay;
        mock.method = data.method.clone();
        HttpResponse::Ok().json(mock.clone())
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

// Retrieve a single mock by ID
#[get("/get-mock/{id}")]
async fn get_mock(path: web::Path<Uuid>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mocks = state.mocks.lock().unwrap();

    if let Some(mock) = mocks.iter().find(|m| m.id == Some(id)) {
        HttpResponse::Ok().json(mock)
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

#[post("/save-mock")]
async fn save_mock(data: web::Json<MockAPI>, state: web::Data<AppState>) -> impl Responder {
    let mut mocks = state.mocks.lock().unwrap();
    let mut mock = data.into_inner();
    mock.id = Some(Uuid::new_v4());
    mocks.push(mock.clone());
    HttpResponse::Ok().json(mock)
}

#[get("/list-mocks")]
async fn list_mocks(state: web::Data<AppState>) -> impl Responder {
    let mocks = state.mocks.lock().unwrap();
    HttpResponse::Ok().json(&*mocks)
}

#[delete("/delete-mock/{id}")]
async fn delete_mock(path: web::Path<Uuid>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mut mocks = state.mocks.lock().unwrap();
    if let Some(index) = mocks.iter().position(|mock| mock.id == Some(id)) {
        mocks.remove(index);
        HttpResponse::Ok().json("Mock deleted successfully")
    } else {
        HttpResponse::NotFound().json("Mock not found")
    }
}

