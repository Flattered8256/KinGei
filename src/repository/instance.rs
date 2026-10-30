use sqlx::QueryBuilder;

use crate::database::{Db, DbPool};
use crate::model::instance::Instance;

#[derive(Debug)]
pub struct UpdateInstance {
    pub allow_add_friend: Option<bool>,
    pub allow_create_group: Option<bool>,
}

pub struct InstanceRepository {
    pool: DbPool,
}

impl InstanceRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<Instance, sqlx::Error> {
        sqlx::query_as::<_, Instance>(
            r#"
            SELECT id, allow_add_friend, allow_create_group, initializing
            FROM instance
            WHERE id = 1
            "#,
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, instance: UpdateInstance) -> Result<Instance, sqlx::Error> {
        let mut builder = QueryBuilder::<Db>::new("UPDATE instance SET ");
        let mut has_set = false;

        if let Some(allow_add_friend) = instance.allow_add_friend {
            builder.push("allow_add_friend = ");
            builder.push_bind(allow_add_friend);
            has_set = true;
        }

        if let Some(allow_create_group) = instance.allow_create_group {
            if has_set {
                builder.push(",");
            }
            builder.push("allow_create_group = ");
            builder.push_bind(allow_create_group);
            has_set = true;
        }

        if !has_set {
            return self.get().await;
        }

        builder
            .push(" WHERE id = 1 RETURNING id, allow_add_friend, allow_create_group, initializing");
        builder
            .build_query_as::<Instance>()
            .fetch_one(&self.pool)
            .await
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::database::{DbPool, run_migrations};

    #[tokio::test]
    async fn migrate_seeds_both_flags_true() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = InstanceRepository::new(pool);

        let instance = repo.get().await.unwrap();
        assert_eq!(instance.id, 1);
        assert!(instance.allow_add_friend);
        assert!(instance.allow_create_group);
        assert!(instance.initializing);
    }

    #[tokio::test]
    async fn update_partial_leaves_other_flag() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = InstanceRepository::new(pool);

        let updated = repo
            .update(UpdateInstance {
                allow_add_friend: Some(false),
                allow_create_group: None,
            })
            .await
            .unwrap();
        assert!(!updated.allow_add_friend);
        assert!(updated.allow_create_group);
        assert!(updated.initializing);

        let current = repo.get().await.unwrap();
        assert!(!current.allow_add_friend);
        assert!(current.allow_create_group);
        assert!(current.initializing);
    }

    #[tokio::test]
    async fn update_both_flags() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = InstanceRepository::new(pool);

        let updated = repo
            .update(UpdateInstance {
                allow_add_friend: Some(false),
                allow_create_group: Some(false),
            })
            .await
            .unwrap();
        assert!(!updated.allow_add_friend);
        assert!(!updated.allow_create_group);
        assert!(updated.initializing);
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
