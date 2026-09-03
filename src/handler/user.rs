use crate::error::AppError;
use crate::serializer::user::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest,
    RefreshTokenResponse, UpdateUserRequest, UserResponse,
};
use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use std::sync::Arc;
use uuid::Uuid;
pub async fn register(
    State(service): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = service.user_service.register(req).await?;
    Ok(Json(user))
}

pub async fn login(
    State(service): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let result = service.user_service.login(req).await?;
    Ok(Json(result))
}

pub async fn change_password(
    State(service): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let result = service.user_service.change_password(user_id, req).await?;
    Ok(Json(result))
}

pub async fn update(
    State(service): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let result = service.user_service.update(user_id, req).await?;
    Ok(Json(result))
}

pub async fn find(
    State(service): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<Json<UserResponse>, AppError> {
    let result = service.user_service.find(&email).await?;
    Ok(Json(result))
}

pub async fn refresh(
    State(service): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    let result = service.user_service.refresh(req).await?;
    Ok(Json(result))
}
