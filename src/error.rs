// error.rs
pub mod chat;
pub mod friend;
pub mod message;
pub mod user;

use crate::error::chat::ChatError;
use crate::error::friend::FriendError;
use crate::error::message::MessageError;
use crate::error::user::UserError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("{0}")]
    User(#[from] UserError),

    #[error("{0}")]
    Friend(#[from] FriendError),

    #[error("{0}")]
    Chat(#[from] ChatError),

    #[error("{0}")]
    Message(#[from] MessageError),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
}

fn internal_error() -> Response {
    let code = "InternalServerError".to_string();
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { code }),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::User(user_err) => user_err.into_response(),
            AppError::Friend(friend_err) => friend_err.into_response(),
            AppError::Chat(chat_err) => chat_err.into_response(),
            AppError::Message(message_err) => message_err.into_response(),
            AppError::Database(e) => {
                tracing::error!(?e, "database error");
                internal_error()
            }
            AppError::Bcrypt(e) => {
                tracing::error!(?e, "bcrypt error");
                internal_error()
            }
            AppError::JWT(e) => {
                tracing::error!(?e, "jwt error");
                internal_error()
            }
        }
    }
}
