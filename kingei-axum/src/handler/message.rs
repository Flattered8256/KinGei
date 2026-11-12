use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::message::{
    ListMessageQuery, MessageResponse, ReadCursorResponse, SendMessageRequest, SetTypingRequest,
    UpdateReadCursorRequest,
};
use crate::state::AppState;

pub async fn send(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let result = state.message_service.send(user_id, id, req, None).await?;
    Ok(Json(result))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Query(query): Query<ListMessageQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let result = state.message_service.list(user_id, id, query).await?;
    Ok(Json(result))
}

pub async fn reply(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path((id, message_id)): Path<(i64, i64)>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let result = state
        .message_service
        .send(user_id, id, req, Some(message_id))
        .await?;
    Ok(Json(result))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path((id, message_id)): Path<(i64, i64)>,
) -> Result<Json<MessageResponse>, AppError> {
    let result = state
        .message_service
        .delete(user_id, id, message_id)
        .await?;
    Ok(Json(result))
}

pub async fn update_read_cursor(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateReadCursorRequest>,
) -> Result<Json<ReadCursorResponse>, AppError> {
    let result = state
        .message_service
        .update_read_cursor(user_id, id, req)
        .await?;
    Ok(Json(result))
}

pub async fn set_typing(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Json(req): Json<SetTypingRequest>,
) -> Result<StatusCode, AppError> {
    state.typing_service.set(user_id, id, req.typing).await?;
    Ok(StatusCode::NO_CONTENT)
}
