use std::sync::Arc;

use axum::{Router, middleware};

use crate::middleware::admin;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .layer(middleware::from_fn_with_state(
            state,
            admin::admin_middleware,
        ))
        .layer(middleware::from_fn(auth::auth_middleware))
}
