mod model;
mod serializer;
mod repository;
mod database;
use dotenvy::dotenv;
use std::env;
#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("缺少DATABASE_URL,请检查项目根目录下的.env文件");

    let pool = database::create_pool(&database_url).await;

}