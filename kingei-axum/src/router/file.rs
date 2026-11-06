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
use crate::utils::config::max_upload_bytes;

/// multipart 边界、字段头等额外开销，给 body 限制留出余量，精确上限由流内计数判定。
const MULTIPART_OVERHEAD: usize = 64 * 1024;

pub fn protected_routes() -> Router<Arc<AppState>> {
    let upload = Router::new()
        .route(
            "/",
            post(file::create).layer(DefaultBodyLimit::max(
                max_upload_bytes() + MULTIPART_OVERHEAD,
            )),
        )
        .layer(middleware::from_fn(auth::auth_middleware));

    // 浏览器的 <img> 和下载链接带不了 Authorization，这两条认 cookie。
    let streamed = Router::new()
        .route("/{id}/download", get(file::download))
        .route("/{id}/content", get(file::content))
        .layer(middleware::from_fn(auth::auth_bearer_or_cookie));

    upload.merge(streamed)
}
