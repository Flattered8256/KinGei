use axum::{
    Router, middleware,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;

use crate::handler::friend;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(friend::list_friends))
        .route("/{friend_id}", delete(friend::delete_friend))
        .route("/request", post(friend::send_request))
        .route("/request/received", get(friend::list_received))
        .route("/request/sent", get(friend::list_sent))
        .route("/request/{id}/accept", patch(friend::accept))
        .route("/request/{id}/reject", patch(friend::reject))
        .route("/request/{id}", delete(friend::cancel_request))
        .layer(middleware::from_fn(auth::auth_middleware))
}
