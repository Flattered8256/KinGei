use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handler::file;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(file::create))
        .route("/{id}/complete", post(file::complete))
        .route("/{id}/download", get(file::download))
        .layer(middleware::from_fn(auth::auth_middleware))
}
