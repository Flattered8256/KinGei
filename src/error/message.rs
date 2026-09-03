use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("EmptyContent")]
    EmptyContent,
    #[error("ChatNotFound")]
    ChatNotFound,
    #[error("NotFriends")]
    NotFriends,
    #[error("MessageNotFound")]
    MessageNotFound,
    #[error("NotMember")]
    NotMember,
}

#[derive(Debug, Serialize)]
struct MessageErrorResponse {
    code: String,
}

impl IntoResponse for MessageError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            MessageError::EmptyContent => (StatusCode::BAD_REQUEST, self.to_string()),
            MessageError::ChatNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            MessageError::NotFriends => (StatusCode::NOT_FOUND, self.to_string()),
            MessageError::MessageNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            MessageError::NotMember => (StatusCode::FORBIDDEN, self.to_string()),
        };

        (
            status,
            Json(MessageErrorResponse {
                code: error_response,
            }),
        )
            .into_response()
    }
}
