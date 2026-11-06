use std::sync::Arc;

use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::serializer::invite::InviteResponse;
use crate::state::AppState;

pub async fn get(State(state): State<Arc<AppState>>) -> Result<Json<InviteResponse>, AppError> {
    let result = state.invite_service.get().await?;
    Ok(Json(result))
}

pub async fn post(State(state): State<Arc<AppState>>) -> Result<Json<InviteResponse>, AppError> {
    let result = state.invite_service.regenerate().await?;
    Ok(Json(result))
}
