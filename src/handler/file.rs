use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::file::{
    CreateFileRequest, FileResponse, PresignDownloadResponse, PresignUploadResponse,
};
use crate::state::AppState;

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
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<PresignDownloadResponse>, AppError> {
    let result = state.file_service.download(user_id, id).await?;
    Ok(Json(result))
}
