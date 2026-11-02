use std::sync::Arc;

use axum::{Router, routing::get};

use crate::handler::instance;
use crate::state::AppState;

pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(instance::get))
}
