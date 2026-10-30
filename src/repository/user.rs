use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::database::{Db, DbPool};
use crate::model::user::{User, UserRole};

#[derive(Debug)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub avatar: Option<i64>,
}

pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, mut user: User) -> Result<User, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        #[cfg(feature = "sqlite")]
        sqlx::query(r#"UPDATE "user" SET username = username WHERE 0"#)
            .execute(&mut *tx)
            .await?;

        #[cfg(feature = "postgres")]
        sqlx::query(r#"LOCK TABLE "user" IN EXCLUSIVE MODE"#)
            .execute(&mut *tx)
            .await?;

        let count: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM "user""#)
            .fetch_one(&mut *tx)
            .await?;
        user.role = if count == 0 {
            UserRole::Admin
        } else {
            UserRole::Member
        };

        let created = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO "user"(id, username, email, created_at, password_hash, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, email, created_at, password_hash, avatar, role
            "#,
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.created_at)
        .bind(&user.password_hash)
        .bind(user.role)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(created)
    }

    pub async fn delete(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            DELETE FROM "user"
            WHERE id = $1
            RETURNING id, username, email, created_at, password_hash, avatar, role
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }
    pub async fn list(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, created_at, password_hash, avatar, role
            FROM "user"
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, created_at, password_hash, avatar, role
            FROM "user"
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, created_at, password_hash, avatar, role
            FROM "user"
            WHERE id = $1
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder = QueryBuilder::<Db>::new(
            r#"
            SELECT id, username, email, created_at, password_hash, avatar, role
            FROM "user"
            WHERE id IN (
            "#,
        );
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        builder.push(")");

        builder.build_query_as::<User>().fetch_all(&self.pool).await
    }

    pub async fn update(&self, id: Uuid, user: UpdateUser) -> Result<Option<User>, sqlx::Error> {
        let mut builder = QueryBuilder::<Db>::new(r#"UPDATE "user" SET "#);

        let mut has_set = false;

        if let Some(username) = &user.username {
            builder.push("username = ");
            builder.push_bind(username);
            has_set = true;
        }

        if let Some(email) = &user.email {
            if has_set {
                builder.push(",");
            }
            builder.push("email = ");
            builder.push_bind(email);
            has_set = true;
        }

        if let Some(password_hash) = &user.password_hash {
            if has_set {
                builder.push(",");
            }
            builder.push("password_hash = ");
            builder.push_bind(password_hash);
            has_set = true;
        }

        if let Some(avatar) = user.avatar {
            if has_set {
                builder.push(",");
            }
            builder.push("avatar = ");
            builder.push_bind(avatar);
            has_set = true;
        }

        if !has_set {
            return Ok(None);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" RETURNING id, username, email, created_at, password_hash, avatar, role");

        builder
            .build_query_as::<User>()
            .fetch_optional(&self.pool)
            .await
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::database::{DbPool, run_migrations};
    use crate::model::user::UserRole;

    #[tokio::test]
    async fn first_user_is_admin_second_is_member() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = UserRepository::new(pool);

        let first = repo
            .create(sample("alice", "alice@example.com"))
            .await
            .unwrap();
        assert_eq!(first.role, UserRole::Admin);

        let second = repo.create(sample("bob", "bob@example.com")).await.unwrap();
        assert_eq!(second.role, UserRole::Member);
    }

    #[tokio::test]
    async fn list_orders_by_created_at() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = UserRepository::new(pool);

        repo.create(sample("alice", "alice@example.com"))
            .await
            .unwrap();
        repo.create(sample("bob", "bob@example.com")).await.unwrap();

        let users = repo.list().await.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "alice");
        assert_eq!(users[0].role, UserRole::Admin);
        assert_eq!(users[1].username, "bob");
        assert_eq!(users[1].role, UserRole::Member);
    }

    #[tokio::test]
    async fn delete_member_removes_from_list() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = UserRepository::new(pool);

        repo.create(sample("alice", "alice@example.com"))
            .await
            .unwrap();
        let bob = repo.create(sample("bob", "bob@example.com")).await.unwrap();

        let deleted = repo.delete(bob.id).await.unwrap();
        assert!(deleted.is_some());

        let users = repo.list().await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "alice");
    }

    #[tokio::test]
    async fn delete_member_cascades_friends() {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let repo = UserRepository::new(pool.clone());

        let alice = repo
            .create(sample("alice", "alice@example.com"))
            .await
            .unwrap();
        let bob = repo.create(sample("bob", "bob@example.com")).await.unwrap();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO friend(user_id, friend_id, created_at)
            VALUES ($1, $2, $3), ($2, $1, $3)
            "#,
        )
        .bind(alice.id)
        .bind(bob.id)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        repo.delete(bob.id).await.unwrap();

        let remaining: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM friend WHERE user_id = $1 OR friend_id = $1"#,
        )
        .bind(alice.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(remaining, 0);
        assert_eq!(repo.list().await.unwrap().len(), 1);
    }

    fn sample(username: &str, email: &str) -> User {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
            password_hash: "hash".to_string(),
            avatar: None,
            role: UserRole::Member,
        }
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
