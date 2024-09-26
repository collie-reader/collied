use axum::{extract::State, Extension, Json};
use collie::auth::{error::Error, model::token::Login, service::token};
use http::StatusCode;
use std::sync::Arc;

use crate::config::Context;

pub async fn authorize(
    State(ctx): State<Arc<Context>>,
    Extension(arg): Extension<Login>,
) -> (StatusCode, Json<String>) {
    let Context {
        conn,
        server_secret,
        ..
    } = &*ctx;
    match token::issue(conn, &arg.access, &arg.secret, server_secret) {
        Ok(token) => (StatusCode::OK, Json(token)),
        Err(kind) => match kind {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, Json("".to_string())),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json("".to_string())),
        },
    }
}
