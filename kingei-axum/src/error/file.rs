use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("EmptyName")]
    EmptyName,
    #[error("FileNameTooLong")]
    FileNameTooLong,
    #[error("InvalidSize")]
    InvalidSize,
    #[error("FileNotFound")]
    FileNotFound,
    #[error("NotOwner")]
    NotOwner,
    #[error("InvalidUpload")]
    InvalidUpload,
    #[error("FileTooLarge")]
    FileTooLarge,
    #[error("FileNotReady")]
    FileNotReady,
    #[error("InvalidImage")]
    InvalidImage,
}

#[derive(Debug, Serialize)]
struct FileErrorResponse {
    code: String,
}

impl IntoResponse for FileError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            FileError::EmptyName => (StatusCode::BAD_REQUEST, self.to_string()),
            FileError::FileNameTooLong => (StatusCode::BAD_REQUEST, self.to_string()),
            FileError::InvalidSize => (StatusCode::BAD_REQUEST, self.to_string()),
            FileError::FileNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            FileError::NotOwner => (StatusCode::FORBIDDEN, self.to_string()),
            FileError::InvalidUpload => (StatusCode::BAD_REQUEST, self.to_string()),
            FileError::FileTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            FileError::FileNotReady => (StatusCode::BAD_REQUEST, self.to_string()),
            FileError::InvalidImage => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        (
            status,
            Json(FileErrorResponse {
                code: error_response,
            }),
        )
            .into_response()
    }
}
