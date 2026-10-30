use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{delete, get},
};

use crate::handler::admin as handler;
use crate::middleware::admin;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/user", get(handler::list_users))
        .route("/user/{id}", delete(handler::delete_user))
        .layer(middleware::from_fn_with_state(
            state,
            admin::admin_middleware,
        ))
        .layer(middleware::from_fn(auth::auth_middleware))
}
