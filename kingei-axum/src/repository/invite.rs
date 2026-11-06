use crate::database::DbPool;
use crate::model::invite::Invite;

pub struct InviteRepository {
    pool: DbPool,
}

impl InviteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<Option<Invite>, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            r#"
            SELECT id, code
            FROM invite
            WHERE id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<Invite>, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            r#"
            SELECT id, code
            FROM invite
            WHERE id = 1 AND code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert(&self, code: &str) -> Result<Invite, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            r#"
            INSERT INTO invite (id, code) VALUES (1, $1)
            ON CONFLICT (id) DO UPDATE SET code = excluded.code
            RETURNING id, code
            "#,
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::database::{DbPool, run_migrations};

    #[tokio::test]
    async fn migrate_has_no_invite_row() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = InviteRepository::new(pool);

        assert!(repo.get().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn upsert_inserts_then_replaces() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = InviteRepository::new(pool);

        let first = repo.upsert("ABCDEFGH").await.unwrap();
        assert_eq!(first.id, 1);
        assert_eq!(first.code, "ABCDEFGH");
        assert_eq!(repo.get().await.unwrap().unwrap().code, "ABCDEFGH");
        assert!(repo.find_by_code("ABCDEFGH").await.unwrap().is_some());

        let second = repo.upsert("JKLMNPQR").await.unwrap();
        assert_eq!(second.code, "JKLMNPQR");
        assert_eq!(repo.get().await.unwrap().unwrap().code, "JKLMNPQR");
        assert!(repo.find_by_code("ABCDEFGH").await.unwrap().is_none());
        assert!(repo.find_by_code("JKLMNPQR").await.unwrap().is_some());
    }

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
}
