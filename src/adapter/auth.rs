use axum::{extract::State, Extension, Json};
use collie::auth::{
    error::Error,
    token::{self, Login},
};
use http::StatusCode;

use crate::config::SharedAppState;

pub async fn authorize(
    State(SharedAppState {
        conn,
        server_secret,
        ..
    }): State<SharedAppState>,
    Extension(arg): Extension<Login>,
) -> (StatusCode, Json<String>) {
    match token::issue(&conn, &arg.access, &arg.secret, &server_secret) {
        Ok(token) => (StatusCode::OK, Json(token)),
        Err(kind) => match kind {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, Json("".to_string())),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json("".to_string())),
        },
    }
}
