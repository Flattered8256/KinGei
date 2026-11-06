use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use std::sync::Arc;

use crate::handler::chat;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/direct", post(chat::create_direct))
        .route("/group", post(chat::create_group))
        .route("/", get(chat::list))
        .route("/{id}", get(chat::get).patch(chat::update))
        .route("/{id}/member", post(chat::invite))
        .route("/{id}/member/{user_id}", delete(chat::remove_member))
        .layer(middleware::from_fn(auth::auth_middleware))
}
