use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("CannotAddSelf")]
    CannotAddSelf,
    #[error("UserNotFound")]
    UserNotFound,
    #[error("NotFriends")]
    NotFriends,
    #[error("ChatNotFound")]
    ChatNotFound,
    #[error("NotMember")]
    NotMember,
    #[error("NotOwner")]
    NotOwner,
    #[error("CannotModifyDirect")]
    CannotModifyDirect,
    #[error("AlreadyMember")]
    AlreadyMember,
    #[error("EmptyName")]
    EmptyName,
}

#[derive(Debug, Serialize)]
struct ChatErrorResponse {
    code: String,
}

impl IntoResponse for ChatError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            ChatError::CannotAddSelf => (StatusCode::BAD_REQUEST, self.to_string()),
            ChatError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ChatError::NotFriends => (StatusCode::NOT_FOUND, self.to_string()),
            ChatError::ChatNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ChatError::NotMember => (StatusCode::FORBIDDEN, self.to_string()),
            ChatError::NotOwner => (StatusCode::FORBIDDEN, self.to_string()),
            ChatError::CannotModifyDirect => (StatusCode::BAD_REQUEST, self.to_string()),
            ChatError::AlreadyMember => (StatusCode::CONFLICT, self.to_string()),
            ChatError::EmptyName => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        (
            status,
            Json(ChatErrorResponse {
                code: error_response,
            }),
        )
            .into_response()
    }
}
