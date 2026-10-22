use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::model::file::{File, FileStatus};

pub struct FileRepository {
    pool: SqlitePool,
}

impl FileRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        owner_id: Uuid,
        object_key: &str,
        name: &str,
        content_type: &str,
        size: i64,
        status: FileStatus,
    ) -> Result<File, sqlx::Error> {
        sqlx::query_as::<_, File>(
            r#"
            INSERT INTO file(owner_id, object_key, name, content_type, size, status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id, owner_id, object_key, name, content_type, size, status, created_at
            "#,
        )
        .bind(owner_id)
        .bind(object_key)
        .bind(name)
        .bind(content_type)
        .bind(size)
        .bind(status)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<File>, sqlx::Error> {
        sqlx::query_as::<_, File>(
            r#"
            SELECT id, owner_id, object_key, name, content_type, size, status, created_at
            FROM file
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_status(
        &self,
        id: i64,
        status: FileStatus,
    ) -> Result<Option<File>, sqlx::Error> {
        sqlx::query_as::<_, File>(
            r#"
            UPDATE file
            SET status = ?
            WHERE id = ?
            RETURNING id, owner_id, object_key, name, content_type, size, status, created_at
            "#,
        )
        .bind(status)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM file
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
