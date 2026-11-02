use std::sync::Arc;

use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub async fn admin_middleware(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    state.user_service.require_admin(user_id).await?;
    Ok(next.run(req).await)
}
