use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum InviteError {
    #[error("InviteNotFound")]
    InviteNotFound,
}

#[derive(Debug, Serialize)]
struct InviteErrorResponse {
    code: String,
}

impl IntoResponse for InviteError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            InviteError::InviteNotFound => (StatusCode::NOT_FOUND, self.to_string()),
        };

        (
            status,
            Json(InviteErrorResponse {
                code: error_response,
            }),
        )
            .into_response()
    }
}
