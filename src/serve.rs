use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use collie::model::database::Connection;
use std::sync::Arc;

use crate::{adapter, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<Connection>,
    pub config: Config,
}

pub async fn serve(conn: Connection, addr: &str, config: &Config) {
    let app_state = AppState {
        conn: Arc::new(conn),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/", get(echo))
        .route("/feed", get(adapter::feed::read_all))
        .route("/feed/:id", get(adapter::feed::read))
        .route("/feed", post(adapter::feed::create))
        .route("/feed/:id", patch(adapter::feed::update))
        .route("/feed/:id", delete(adapter::feed::delete))
        .route("/item", get(adapter::item::read_all))
        .route("/item/count", get(adapter::item::count_all))
        .route("/item", post(adapter::item::create))
        .route("/item", patch(adapter::item::update))
        .route("/item", patch(adapter::item::update_all))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn echo() -> &'static str {
    "hello-world"
}
