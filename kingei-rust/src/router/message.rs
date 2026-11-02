use axum::{
    Router, middleware,
    routing::{get, patch},
};
use std::sync::Arc;

use crate::handler::message;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}/message", get(message::list).post(message::send))
        .route("/{id}/read", patch(message::update_read_cursor))
        .layer(middleware::from_fn(auth::auth_middleware))
}
