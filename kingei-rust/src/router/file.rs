use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handler::file;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    let bearer = Router::new()
        .route("/", post(file::create))
        .route("/{id}/complete", post(file::complete))
        .route("/{id}/download", get(file::download))
        .layer(middleware::from_fn(auth::auth_middleware));

    let content = Router::new()
        .route("/{id}/content", get(file::content))
        .layer(middleware::from_fn(auth::auth_bearer_or_cookie));

    bearer.merge(content)
}
