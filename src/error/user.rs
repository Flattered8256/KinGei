use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("UserAlreadyExists")]
    UserAlreadyExists,
    #[error("NotFound")]
    NotFound,
    #[error("InvalidCredentials")]
    InvalidCredentials,
    #[error("PasswordMismatch")]
    PasswordMismatch,
    #[error("InvalidToken")]
    InvalidToken,
}

#[derive(Debug, Serialize)]
struct UserErrorResponse {
    code: String,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
                UserError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
                UserError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
                UserError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
                UserError::PasswordMismatch => (StatusCode::UNAUTHORIZED, self.to_string()),
                UserError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            };

            (status, Json(UserErrorResponse { code: error_response })).into_response()
        }
}