use std::sync::Arc;

use axum::{Router, middleware, routing::get};

use crate::handler::presence;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(presence::snapshot))
        .layer(middleware::from_fn(auth::auth_middleware))
}
