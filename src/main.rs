mod database;
mod error;
mod event;
mod handler;
mod middleware;
mod model;
mod repository;
mod router;
mod serializer;
mod service;
mod storage;
mod utils;
mod web;
use crate::state::AppState;
use crate::storage::Storage;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
mod state;
#[tokio::main]
async fn main() {
    dotenv().ok();
    crate::utils::media::require_tools();

    let database_url =
        env::var("DATABASE_URL").expect("缺少DATABASE_URL,请检查项目根目录下的.env文件");

    let pool = database::create_pool(&database_url).await;
    tracing_subscriber::fmt::init();
    database::run_migrations(&pool).await;

    let storage = Storage::connect().await;
    let app_state = Arc::new(AppState::new(pool, storage));

    let api = router::create_router(app_state);
    let app = match env::var("STATIC_DIR") {
        Ok(dir) if !dir.trim().is_empty() => web::with_frontend(api, dir),
        _ => api,
    };
    let app = app.into_make_service();

    let port = env::var("PORT").expect("缺少PORT,请检查项目根目录下的.env文件");
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("服务器正在监听{}", addr);

    axum::serve(listener, app).await.unwrap();
}
