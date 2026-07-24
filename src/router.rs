pub mod user;

use axum::Router;
use std::sync::Arc;
use crate::state::AppState;

/// 创建并组装所有应用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/user", user::public_routes().merge(user::protected_routes()))
        .with_state(state)
}