use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, patch},
    Router,
};
use base64::prelude::*;
use collie::{
    auth::token::{self, Login},
    producer::worker::create_new_items,
};
use std::sync::Arc;

use crate::{adapter, config::AppState};

#[tokio::main]
pub async fn serve(app_state: Arc<AppState>, addr: &str) {
    let gateway = Router::new()
        .route("/auth", get(adapter::auth::authorize))
        .route_layer(middleware::from_fn(authorize));

    let protected = Router::new()
        .route("/", get(echo))
        .route(
            "/feeds/:id",
            get(adapter::feed::read)
                .patch(adapter::feed::update)
                .delete(adapter::feed::delete),
        )
        .route(
            "/feeds",
            get(adapter::feed::read_all).post(adapter::feed::create),
        )
        .route(
            "/items",
            get(adapter::item::read_all)
                .post(adapter::item::create)
                .patch(adapter::item::update_all),
        )
        .route("/items/:id", patch(adapter::item::update))
        .route("/items/count", get(adapter::item::count_all))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            authenticate,
        ));

    let app = Router::new()
        .nest("/", gateway)
        .nest("/", protected)
        .with_state(app_state.clone());

    tokio::spawn(async move {
        let AppState { conn, config, .. } = &*app_state;

        loop {
            let _ = create_new_items(conn, &config.producer.proxy).await;
            tokio::time::sleep(std::time::Duration::from_secs(
                config.producer.polling_frequency,
            ))
            .await;
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn authenticate(
    State(app_state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let AppState { server_secret, .. } = &*app_state;

    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let access = auth_header
        .split_whitespace()
        .last()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if token::verify(access, server_secret).is_ok() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn authorize(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let base64 = auth_header
        .split_whitespace()
        .last()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_header = String::from_utf8(
        BASE64_STANDARD
            .decode(base64)
            .map_err(|_| StatusCode::UNAUTHORIZED)?,
    )
    .unwrap();

    let (access, secret) = auth_header
        .split_once(':')
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let login = Login {
        access: access.to_string(),
        secret: secret.to_string(),
    };

    req.extensions_mut().insert(login);
    Ok(next.run(req).await)
}

async fn echo() -> &'static str {
    "hello-world"
}
