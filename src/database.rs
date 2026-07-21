use sqlx::{SqlitePool,sqlite::SqlitePoolOptions};

pub async fn create_pool(database_url: &str) -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("连接数据库失败")
}