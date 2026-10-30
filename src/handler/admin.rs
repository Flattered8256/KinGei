use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::user::UserResponse;
use crate::state::AppState;

pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let result = state.user_service.list().await?;
    Ok(Json(result))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.admin_service.delete(user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
