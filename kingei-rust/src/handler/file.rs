use std::sync::Arc;

use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::HeaderMap,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::file::{
    CreateFileRequest, FileResponse, PresignDownloadResponse, PresignUploadResponse,
};
use crate::state::AppState;
use crate::storage::content_disposition;

pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateFileRequest>,
) -> Result<Json<PresignUploadResponse>, AppError> {
    let result = state.file_service.create(user_id, req).await?;
    Ok(Json(result))
}

pub async fn complete(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<FileResponse>, AppError> {
    let result = state.file_service.complete(user_id, id).await?;
    Ok(Json(result))
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<PresignDownloadResponse>, AppError> {
    let result = state.file_service.download(id).await?;
    Ok(Json(result))
}

pub async fn content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok());
    let result = state.file_service.content(id, range).await?;
    let status = if result.ranged && result.object.content_range.is_some() {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };

    let mut response = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, result.file.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            content_disposition(&result.file.name, true),
        )
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "private, max-age=86400");

    if result.object.content_length > 0 {
        response = response.header(
            header::CONTENT_LENGTH,
            result.object.content_length.to_string(),
        );
    }
    if let Some(content_range) = result.object.content_range {
        response = response.header(header::CONTENT_RANGE, content_range);
    }

    let body = Body::from_stream(ReaderStream::new(result.object.body.into_async_read()));
    response
        .body(body)
        .map_err(|e| AppError::Storage(crate::storage::StorageError(e.to_string())))
}
