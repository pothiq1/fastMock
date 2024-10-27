// src/state.rs
use crate::models::MockAPI;
use std::sync::Mutex;

pub struct AppState {
    pub mocks: Mutex<Vec<MockAPI>>,
}
