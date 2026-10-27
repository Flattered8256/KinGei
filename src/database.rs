use sqlx::Pool;

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("启用 sqlite 或 postgres 其中一个");

#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!("需要 sqlite 或 postgres feature");

#[cfg(feature = "sqlite")]
pub type Db = sqlx::Sqlite;
#[cfg(feature = "postgres")]
pub type Db = sqlx::Postgres;

pub type DbPool = Pool<Db>;

pub async fn create_pool(database_url: &str) -> DbPool {
    #[cfg(feature = "sqlite")]
    {
        use std::str::FromStr;

        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let options = SqliteConnectOptions::from_str(database_url)
            .expect("DATABASE_URL无效")
            .create_if_missing(true);
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("连接数据库失败")
    }

    #[cfg(feature = "postgres")]
    {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .expect("连接数据库失败")
    }
}

pub async fn run_migrations(pool: &DbPool) {
    #[cfg(feature = "sqlite")]
    sqlx::migrate!("./migrations/sqlite")
        .run(pool)
        .await
        .expect("数据库迁移失败");

    #[cfg(feature = "postgres")]
    sqlx::migrate!("./migrations/postgres")
        .run(pool)
        .await
        .expect("数据库迁移失败");
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn migrate_and_write_user() {
        let pool = test_pool().await;
        run_migrations(&pool).await;

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO "user"(id, username, email, created_at, password_hash)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind("alice")
        .bind("alice@example.com")
        .bind(Utc::now())
        .bind("hash")
        .execute(&pool)
        .await
        .unwrap();

        let username: String = sqlx::query_scalar(r#"SELECT username FROM "user" WHERE id = $1"#)
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(username, "alice");

        let friend_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO "user"(id, username, email, created_at, password_hash)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(friend_id)
        .bind("bob")
        .bind("bob@example.com")
        .bind(Utc::now())
        .bind("hash")
        .execute(&pool)
        .await
        .unwrap();

        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO friend_request(sender_id, receiver_id, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(friend_id)
        .bind(crate::model::friend::Status::Pending)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    }

    #[cfg(feature = "sqlite")]
    async fn test_pool() -> DbPool {
        use std::str::FromStr;

        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap()
    }

    #[cfg(feature = "postgres")]
    async fn test_pool() -> DbPool {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://kingei:kingei@127.0.0.1:5432/kingei".to_string());
        create_pool(&url).await
    }
}
