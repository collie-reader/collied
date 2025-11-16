use axum::{
    extract::{Request, State},
    http::{header::HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, patch},
    Router,
};
use base64::prelude::*;
use collie::{auth::model::token::Login, auth::service::token, worker::Worker};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor, GovernorLayer,
};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

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

    let cors = CorsLayer::new()
        .allow_origin([
            "tauri://localhost".parse::<HeaderValue>().unwrap(),
            "https://tauri.localhost".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([http::header::AUTHORIZATION, http::header::CONTENT_TYPE]);

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .key_extractor(PeerIpKeyExtractor)
        .finish()
        .unwrap();

    let app = Router::new()
        .merge(gateway)
        .merge(protected)
        .layer(middleware::from_fn(log_request))
        .layer(cors)
        .layer(GovernorLayer {
            config: Arc::new(governor_conf),
        })
        .layer(middleware::from_fn(add_security_headers))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
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

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();
    if let Err(e) = axum::serve(listener, make_service).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
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
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let (access, secret) = auth_header
        .split_once(':')
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if access.is_empty() || secret.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let is_valid_key_char = |c: char| c.is_ascii_alphanumeric() || "!@#$%^&*_-".contains(c);

    if !access.chars().all(is_valid_key_char) || !secret.chars().all(is_valid_key_char) {
        return Err(StatusCode::UNAUTHORIZED);
    }

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

async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    println!("<-- [{}] {}", method, path);

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed().as_millis();
    let status = response.status().as_u16();

    println!("--> [{}] {} {} {}ms", method, path, status, elapsed);

    response
}

async fn add_security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'"),
    );
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );

    response
}
