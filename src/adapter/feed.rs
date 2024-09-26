use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use collie::{
    model::feed::{Feed, FeedToCreate, FeedToUpdate},
    service::feed,
};
use std::sync::Arc;

use crate::config::Context;

pub async fn create(
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<FeedToCreate>,
) -> (StatusCode, Json<bool>) {
    let Context { conn, config, .. } = &*ctx;
    match feed::create(conn, &arg, config.producer.proxy.as_deref()).await {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn read_all(State(ctx): State<Arc<Context>>) -> (StatusCode, Json<Vec<Feed>>) {
    let Context { conn, .. } = &*ctx;
    match feed::read_all(conn) {
        Ok(feeds) => (StatusCode::OK, Json(feeds)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn read(
    State(ctx): State<Arc<Context>>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Option<Feed>>) {
    let Context { conn, .. } = &*ctx;
    match feed::read(conn, id) {
        Ok(feed) => (StatusCode::OK, Json(feed)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}

pub async fn update(
    State(ctx): State<Arc<Context>>,
    Json(arg): Json<FeedToUpdate>,
) -> (StatusCode, Json<bool>) {
    let Context { conn, .. } = &*ctx;
    match feed::update(conn, &arg) {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn delete(
    State(ctx): State<Arc<Context>>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<bool>) {
    let Context { conn, .. } = &*ctx;
    match feed::delete(conn, id) {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}
