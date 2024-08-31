use axum::{extract::State, Extension, Json};
use collie::auth::{
    error::Error,
    token::{self, Login},
};
use http::StatusCode;
use std::sync::Arc;

use crate::config::AppState;

pub async fn authorize(
    State(app_state): State<Arc<AppState>>,
    Extension(arg): Extension<Login>,
) -> (StatusCode, Json<String>) {
    let AppState {
        conn,
        server_secret,
        ..
    } = &*app_state;
    match token::issue(conn, &arg.access, &arg.secret, server_secret) {
        Ok(token) => (StatusCode::OK, Json(token)),
        Err(kind) => match kind {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, Json("".to_string())),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json("".to_string())),
        },
    }
}
