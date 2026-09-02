use axum::{
    extract::{Path, State},
    Extension, Json,
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::chat::{
    ChatResponse, CreateDirectRequest, CreateGroupRequest, InviteMemberRequest, UpdateChatRequest,
};
use crate::state::AppState;

pub async fn create_direct(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateDirectRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let result = state.chat_service.create_direct(user_id, req).await?;
    Ok(Json(result))
}

pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let result = state.chat_service.create_group(user_id, req).await?;
    Ok(Json(result))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<ChatResponse>>, AppError> {
    let result = state.chat_service.list(user_id).await?;
    Ok(Json(result))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<ChatResponse>, AppError> {
    let result = state.chat_service.get(user_id, id).await?;
    Ok(Json(result))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let result = state.chat_service.update(user_id, id, req).await?;
    Ok(Json(result))
}

pub async fn invite(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
    Json(req): Json<InviteMemberRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let result = state.chat_service.invite(user_id, id, req).await?;
    Ok(Json(result))
}

pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path((id, user_id_to_remove)): Path<(i64, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .chat_service
        .remove_member(user_id, id, user_id_to_remove)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
