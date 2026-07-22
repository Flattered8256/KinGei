use axum::{Router, middleware, routing::{get, patch, post}, {extract::{State, Path}, Json}};
use crate::{handler::user, service::user::UserService};
use std::sync::Arc;
use crate::middleware::auth;
pub fn public_routes() -> Router<Arc<UserService>> {
    Router::new()
        .route("/register", post(user::register))
        .route("/login", post(user::login))
}

pub fn protected_routes() -> Router<Arc<UserService>> {
    Router::new()
        .route("/change_password", patch(user::change_password))
        .route("/update", patch(user::update))
        .route("/user/{email}", get(user::find))
        .layer(middleware::from_fn(auth::auth_middleware))
}