use std::sync::Arc;

use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::serializer::instance::{InstanceResponse, UpdateInstanceRequest};
use crate::state::AppState;

pub async fn get(State(state): State<Arc<AppState>>) -> Result<Json<InstanceResponse>, AppError> {
    let result = state.instance_service.get().await?;
    Ok(Json(result))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateInstanceRequest>,
) -> Result<Json<InstanceResponse>, AppError> {
    let result = state.instance_service.update(req).await?;
    Ok(Json(result))
}
