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
            .create_if_missing(true)
            .foreign_keys(true);
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
    #[tokio::test]
    async fn cascade_rebuild_with_existing_chats() {
        use sqlx::migrate::Migrate;

        let pool = test_pool().await;
        let migrator = sqlx::migrate!("./migrations/sqlite");
        let mut conn = pool.acquire().await.unwrap();
        conn.ensure_migrations_table("_sqlx_migrations")
            .await
            .unwrap();

        for migration in migrator.iter() {
            if migration.version == 20261030120000 {
                let id = Uuid::new_v4();
                let now = Utc::now();
                sqlx::query(
                    r#"
                    INSERT INTO "user"(id, username, email, created_at, password_hash)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(id)
                .bind("alice")
                .bind("alice@example.com")
                .bind(now)
                .bind("hash")
                .execute(&mut *conn)
                .await
                .unwrap();

                sqlx::query(
                    r#"
                    INSERT INTO chat(chat_type, name, created_at)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(crate::model::chat::ChatType::Group)
                .bind("group")
                .bind(now)
                .execute(&mut *conn)
                .await
                .unwrap();

                let chat_id: i64 = sqlx::query_scalar("SELECT id FROM chat")
                    .fetch_one(&mut *conn)
                    .await
                    .unwrap();

                sqlx::query(
                    r#"
                    INSERT INTO chat_member(chat_id, user_id, created_at)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(chat_id)
                .bind(id)
                .bind(now)
                .execute(&mut *conn)
                .await
                .unwrap();
            }
            conn.apply("_sqlx_migrations", migration).await.unwrap();
        }

        let members: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_member")
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        assert_eq!(members, 1);
    }

    #[cfg(feature = "sqlite")]
    async fn test_pool() -> DbPool {
        use std::str::FromStr;

        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);
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
