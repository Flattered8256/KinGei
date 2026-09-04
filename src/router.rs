pub mod chat;
pub mod event;
pub mod friend;
pub mod message;
pub mod user;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

/// 创建并组装所有应用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest(
            "/user",
            user::public_routes().merge(user::protected_routes()),
        )
        .nest("/friend", friend::protected_routes())
        .nest(
            "/chat",
            chat::protected_routes().merge(message::protected_routes()),
        )
        .nest("/event", event::protected_routes())
        .with_state(state)
}
