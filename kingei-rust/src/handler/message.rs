use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::message::{
    ListMessageQuery, MessageResponse, ReadCursorResponse, SendMessageRequest,
    UpdateReadCursorRequest,
};
use crate::state::AppState;

pub async fn send(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let result = state.message_service.send(user_id, id, req).await?;
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
