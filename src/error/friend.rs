use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum FriendError {
    #[error("CannotAddSelf")]
    CannotAddSelf,
    #[error("UserNotFound")]
    UserNotFound,
    #[error("AlreadyFriends")]
    AlreadyFriends,
    #[error("RequestAlreadyPending")]
    RequestAlreadyPending,
    #[error("RequestNotFound")]
    RequestNotFound,
    #[error("NotRequestReceiver")]
    NotRequestReceiver,
    #[error("NotRequestSender")]
    NotRequestSender,
    #[error("RequestNotPending")]
    RequestNotPending,
    #[error("NotFriends")]
    NotFriends,
}

#[derive(Debug, Serialize)]
struct FriendErrorResponse {
    code: String,
}

impl IntoResponse for FriendError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            FriendError::CannotAddSelf => (StatusCode::BAD_REQUEST, self.to_string()),
            FriendError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            FriendError::AlreadyFriends => (StatusCode::CONFLICT, self.to_string()),
            FriendError::RequestAlreadyPending => (StatusCode::CONFLICT, self.to_string()),
            FriendError::RequestNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            FriendError::NotRequestReceiver => (StatusCode::FORBIDDEN, self.to_string()),
            FriendError::NotRequestSender => (StatusCode::FORBIDDEN, self.to_string()),
            FriendError::RequestNotPending => (StatusCode::CONFLICT, self.to_string()),
            FriendError::NotFriends => (StatusCode::NOT_FOUND, self.to_string()),
        };

        (
            status,
            Json(FriendErrorResponse {
                code: error_response,
            }),
        )
            .into_response()
    }
}
