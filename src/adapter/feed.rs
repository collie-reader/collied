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

use crate::config::SharedAppState;

pub async fn create(
    State(SharedAppState { conn, config, .. }): State<SharedAppState>,
    Json(arg): Json<FeedToCreate>,
) -> (StatusCode, Json<bool>) {
    if arg.link.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(false));
    }

    let proxy = config.producer.proxy;
    let html_content = fetch_content(&arg.link, proxy.as_deref()).unwrap();
    let is_feed = html_content.parse::<SyndicationFeed>().is_ok();

    let link = if is_feed {
        arg.link.clone()
    } else if let Some(rss_link) = find_feed_link(&html_content).unwrap() {
        rss_link
    } else {
        return (StatusCode::BAD_REQUEST, Json(false));
    };

    let title = match fetch_feed_title(&link, proxy.as_deref()) {
        Ok(title) => title,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    };

    let arg = FeedToCreate {
        title,
        link,
        fetch_old_items: arg.fetch_old_items,
    };

    match feed::create(&conn, &arg) {
        Ok(_) => {
            let _ = create_new_items(&conn, proxy.as_deref());
            (StatusCode::OK, Json(true))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn read_all(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
) -> (StatusCode, Json<Vec<Feed>>) {
    match feed::read_all(&conn) {
        Ok(feeds) => (StatusCode::OK, Json(feeds)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn read(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Option<Feed>>) {
    match feed::read(&conn, id) {
        Ok(feed) => (StatusCode::OK, Json(feed)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}

pub async fn update(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Json(arg): Json<FeedToUpdate>,
) -> (StatusCode, Json<bool>) {
    match feed::update(&conn, &arg) {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}

pub async fn delete(
    State(SharedAppState { conn, .. }): State<SharedAppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<bool>) {
    match feed::delete(&conn, id) {
        Ok(_) => (StatusCode::OK, Json(true)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(false)),
    }
}
