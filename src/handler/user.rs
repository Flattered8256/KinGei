use crate::error::AppError;
use crate::middleware::auth;
use crate::serializer::user::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, RefreshTokenRequest, UpdateUserRequest,
    UserResponse,
};
use crate::state::AppState;
use axum::response::IntoResponse;
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.user_service.login(req).await?;
    let token = result.access_token.clone();
    Ok(auth::with_access_cookie(&token, Json(result)))
}

pub async fn logout() -> impl IntoResponse {
    auth::cleared_access_cookie()
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.user_service.refresh(req).await?;
    let token = result.access_token.clone();
    Ok(auth::with_access_cookie(&token, Json(result)))
}
