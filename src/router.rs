pub mod chat;
pub mod event;
pub mod file;
pub mod friend;
pub mod message;
pub mod user;

use std::sync::Arc;

use axum::{Router, http::StatusCode, routing::get};

use crate::state::AppState;

/// 创建并组装所有应用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
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
        .nest("/file", file::protected_routes())
        .with_state(state)
}
