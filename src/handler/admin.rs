use std::sync::Arc;

use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::serializer::user::UserResponse;
use crate::state::AppState;

pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let result = state.user_service.list().await?;
    Ok(Json(result))
}
