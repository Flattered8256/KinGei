use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::event::PresenceChangedEvent;
use crate::state::AppState;

pub async fn snapshot(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<PresenceChangedEvent>>, AppError> {
    let result = state.presence_service.snapshot(user_id).await?;
    Ok(Json(result))
}
