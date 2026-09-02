use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::friend::{SendFriendRequest, FriendRequestResponse, FriendResponse};
use crate::state::AppState;

pub async fn send_request(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<SendFriendRequest>,
) -> Result<Json<FriendRequestResponse>, AppError> {
    let result = state.friend_service.send_request(user_id, req).await?;
    Ok(Json(result))
}

pub async fn list_received(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<FriendRequestResponse>>, AppError> {
    let result = state.friend_service.list_received(user_id).await?;
    Ok(Json(result))
}

pub async fn list_sent(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<FriendRequestResponse>>, AppError> {
    let result = state.friend_service.list_sent(user_id).await?;
    Ok(Json(result))
}

pub async fn accept(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<FriendRequestResponse>, AppError> {
    let result = state.friend_service.accept(user_id, id).await?;
    Ok(Json(result))
}

pub async fn reject(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<FriendRequestResponse>, AppError> {
    let result = state.friend_service.reject(user_id, id).await?;
    Ok(Json(result))
}

pub async fn cancel_request(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<FriendRequestResponse>, AppError> {
    let result = state.friend_service.cancel_request(user_id, id).await?;
    Ok(Json(result))
}

pub async fn list_friends(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<FriendResponse>>, AppError> {
    let result = state.friend_service.list_friends(user_id).await?;
    Ok(Json(result))
}

pub async fn delete_friend(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(friend_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.friend_service.delete_friend(user_id, friend_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
