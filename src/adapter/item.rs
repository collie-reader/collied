use axum::{extract::State, http::StatusCode, Json};
use collie::model::item::{
    self, Item, ItemReadOption, ItemToCreate, ItemToUpdate, ItemToUpdateAll,
};

use crate::config::SharedAppState;

pub async fn create(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Json(arg): Json<ItemToCreate>,
) -> (StatusCode, Json<bool>) {
    match item::create(&conn, &arg) {
        Ok(count) => {
            if count > 0 {
                (StatusCode::OK, Json(true))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(false))
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn read_all(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Json(arg): Json<ItemReadOption>,
) -> (StatusCode, Json<Vec<Item>>) {
    match item::read_all(&conn, &arg) {
        Ok(items) => (StatusCode::OK, Json(items)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn count_all(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Json(arg): Json<ItemReadOption>,
) -> (StatusCode, Json<i64>) {
    match item::count_all(&conn, &arg) {
        Ok(count) => {
            if count > 0 {
                (StatusCode::OK, Json(count))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(0))
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(0)),
    }
}

pub async fn update(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Json(arg): Json<ItemToUpdate>,
) -> (StatusCode, Json<bool>) {
    match item::update(&conn, &arg) {
        Ok(count) => {
            if count > 0 {
                (StatusCode::OK, Json(true))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(false))
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn update_all(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Json(arg): Json<ItemToUpdateAll>,
) -> (StatusCode, Json<bool>) {
    match item::update_all(&conn, &arg) {
        Ok(count) => {
            if count > 0 {
                (StatusCode::OK, Json(true))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(false))
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}
