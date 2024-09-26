use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, patch},
    Router,
};
use base64::prelude::*;
use collie::{auth::model::token::Login, auth::service::token, worker::Worker};
use std::sync::Arc;

use crate::{adapter, config::Context};

#[tokio::main]
pub async fn serve(ctx: Arc<Context>, addr: &str) {
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
        .route_layer(middleware::from_fn_with_state(ctx.clone(), authenticate));

    let app = Router::new()
        .nest("/", gateway)
        .nest("/", protected)
        .with_state(ctx.clone());

    tokio::spawn(async move {
        let Context { conn, config, .. } = &*ctx;
        let worker = Worker::new(conn.clone(), config.producer.proxy.clone());

        loop {
            let _ = worker.execute().await;
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
    State(ctx): State<Arc<Context>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let Context { server_secret, .. } = &*ctx;

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
