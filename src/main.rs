mod model;
mod handler;
mod serializer;
mod repository;
mod database;
mod error;
mod service;
mod utils;
mod middleware;
mod router;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use crate::state::AppState;
mod state;
#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("缺少DATABASE_URL,请检查项目根目录下的.env文件");

    let pool = database::create_pool(&database_url).await;
    tracing_subscriber::fmt::init();
    sqlx::migrate!("./migrations").run(&pool).await.expect("数据库迁移失败");
    
    let app_state = Arc::new(AppState::new(pool));

    let app = router::user::public_routes()
        .merge(router::user::protected_routes())
        .with_state(app_state)
        .into_make_service();

    let port = env::var("PORT").expect("缺少PORT,请检查项目根目录下的.env文件");
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("服务器正在监听{}", addr);

    axum::serve(listener, app).await.unwrap();
}