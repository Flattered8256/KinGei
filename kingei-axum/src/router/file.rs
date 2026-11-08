use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};

use crate::handler::file;
use crate::middleware::auth;
use crate::state::AppState;

pub fn protected_routes() -> Router<Arc<AppState>> {
    // Multipart 默认受全局 2MB body 限制，上传路由必须放开，体积由 handler 边收边计数。
    let upload = Router::new()
        .route("/", post(file::upload))
        .layer(DefaultBodyLimit::disable());

    let bearer = upload.layer(middleware::from_fn(auth::auth_middleware));

    let content = Router::new()
        .route("/{id}/content", get(file::content))
        .route("/{id}/download", get(file::download))
        .layer(middleware::from_fn(auth::auth_bearer_or_cookie));

    bearer.merge(content)
}
