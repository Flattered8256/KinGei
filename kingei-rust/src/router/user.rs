use crate::handler::user;
use crate::middleware::auth;
use crate::state::AppState;
use axum::{
    Router, middleware,
    routing::{get, patch, post},
};
use std::sync::Arc;
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(user::register))
        .route("/login", post(user::login))
        .route("/logout", post(user::logout))
        .route("/refresh", post(user::refresh))
}

pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/change_password", patch(user::change_password))
        .route("/update", patch(user::update))
        .route("/{email}", get(user::find))
        .layer(middleware::from_fn(auth::auth_middleware))
}
