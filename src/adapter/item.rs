use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use collie::model::item::{
    self, Item, ItemReadOption, ItemToCreate, ItemToUpdate, ItemToUpdateAll,
};

use crate::config::Context;

pub async fn create(
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<ItemToCreate>,
) -> (StatusCode, Json<bool>) {
    let Context { conn, .. } = &*ctx;
    match item::create(conn, &arg) {
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
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<ItemReadOption>,
) -> (StatusCode, Json<Vec<Item>>) {
    let Context { conn, .. } = &*ctx;
    match item::read_all(conn, &arg) {
        Ok(items) => (StatusCode::OK, Json(items)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn count_all(
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<ItemReadOption>,
) -> (StatusCode, Json<i64>) {
    let Context { conn, .. } = &*ctx;
    match item::count_all(conn, &arg) {
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
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<ItemToUpdate>,
) -> (StatusCode, Json<bool>) {
    let Context { conn, .. } = &*ctx;
    match item::update(conn, &arg) {
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
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<ItemToUpdateAll>,
) -> (StatusCode, Json<bool>) {
    let Context { conn, .. } = &*ctx;
    match item::update_all(conn, &arg) {
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
