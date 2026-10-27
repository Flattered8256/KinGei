use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::database::{Db, DbPool};
use crate::model::user::User;

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

    pub async fn create(&self, user: User) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO "user"(id, username, email, created_at, password_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, created_at, password_hash, avatar
            "#,
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.created_at)
        .bind(&user.password_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            DELETE FROM "user"
            WHERE id = $1
            RETURNING id, username, email, created_at, password_hash, avatar
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }
    pub async fn list(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, created_at, password_hash, avatar
            FROM "user"
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, created_at, password_hash, avatar
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
            SELECT id, username, email, created_at, password_hash, avatar
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
            SELECT id, username, email, created_at, password_hash, avatar
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
        builder.push(" RETURNING id, username, email, created_at, password_hash, avatar");

        builder
            .build_query_as::<User>()
            .fetch_optional(&self.pool)
            .await
    }
}
