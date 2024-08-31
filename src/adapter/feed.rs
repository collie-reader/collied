use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use collie::{
    model::feed::{self, Feed, FeedToCreate, FeedToUpdate},
    producer::{
        syndication::{fetch_content, fetch_feed_title, find_feed_link, Feed as SyndicationFeed},
        worker::create_new_items,
    },
};
use std::sync::Arc;

use crate::config::AppState;

pub async fn create(
    State(app_state): State<Arc<AppState>>,
    Json(arg): Json<FeedToCreate>,
) -> (StatusCode, Json<bool>) {
    let AppState { conn, config, .. } = &*app_state;

    if arg.link.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(false));
    }

    let proxy = &config.producer.proxy;
    let html_content = fetch_content(&arg.link, proxy).await.unwrap();
    let is_feed = html_content.parse::<SyndicationFeed>().is_ok();

    let link = if is_feed {
        arg.link.clone()
    } else if let Some(rss_link) = find_feed_link(&html_content).unwrap() {
        rss_link
    } else {
        return (StatusCode::BAD_REQUEST, Json(false));
    };

    let title = match fetch_feed_title(&link, proxy).await {
        Ok(title) => title,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    };

    let arg = FeedToCreate {
        title,
        link,
        fetch_old_items: arg.fetch_old_items,
    };

    match feed::create(conn, &arg) {
        Ok(_) => {
            let _ = create_new_items(conn, proxy).await;
            (StatusCode::OK, Json(true))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn read_all(State(app_state): State<Arc<AppState>>) -> (StatusCode, Json<Vec<Feed>>) {
    let AppState { conn, .. } = &*app_state;
    match feed::read_all(conn) {
        Ok(feeds) => (StatusCode::OK, Json(feeds)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn read(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Option<Feed>>) {
    let AppState { conn, .. } = &*app_state;
    match feed::read(conn, id) {
        Ok(feed) => (StatusCode::OK, Json(feed)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}

pub async fn update(
    State(app_state): State<Arc<AppState>>,
    Json(arg): Json<FeedToUpdate>,
) -> (StatusCode, Json<bool>) {
    let AppState { conn, .. } = &*app_state;
    match feed::update(conn, &arg) {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn delete(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<bool>) {
    let AppState { conn, .. } = &*app_state;
    match feed::delete(conn, id) {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}
