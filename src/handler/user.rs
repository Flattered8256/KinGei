use axum::{extract::State, Json, Extension};
use std::sync::Arc;
use crate::error::AppError;
use crate::serializer::user::{CreateUserRequest, UserResponse, LoginRequest, LoginResponse, ChangePasswordRequest, UpdateUserRequest};
use crate::service::user::UserService;
use uuid::Uuid;

pub async fn register(
    State(service): State<Arc<UserService>>, 
    Json(req): Json<CreateUserRequest>
) ->Result<Json<UserResponse>, AppError> {
    let user = service.register(req).await?;
    Ok(Json(user))
}

pub async fn login(
    State(service): State<Arc<UserService>>, 
    Json(req): Json<LoginRequest>
) -> Result<Json<LoginResponse>, AppError> {
    let result = service.login(req).await?;
    Ok(Json(result))
}

pub async fn change_password(
    State(service): State<Arc<UserService>>, 
    Json(req): Json<ChangePasswordRequest>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let result = service.change_password(user_id, req).await?;
    Ok(Json(result))
}

pub async fn update(
    State(service): State<Arc<UserService>>, 
    Json(req): Json<UpdateUserRequest>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let result = service.update(user_id, req).await?;
    Ok(Json(result))
}