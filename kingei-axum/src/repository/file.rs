use chrono::Utc;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::database::{Db, DbPool};
use crate::model::file::{File, FileStatus, Media, MediaKind};

pub struct ReadyMedia {
    pub kind: MediaKind,
    pub width: i64,
    pub height: i64,
    pub duration_ms: Option<i64>,
}

pub struct NewThumbnail {
    pub owner_id: Uuid,
    pub object_key: String,
    pub size: i64,
}

pub struct FileRepository {
    pool: DbPool,
}

impl FileRepository {
    pub fn new(pool: DbPool) -> Self {
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
            VALUES ($1, $2, $3, $4, $5, $6, $7)
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
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_media(&self, file_id: i64) -> Result<Option<Media>, sqlx::Error> {
        sqlx::query_as::<_, Media>(
            r#"
            SELECT file_id, kind, width, height, duration_ms, thumbnail_id, created_at
            FROM media
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_ids(&self, ids: &[i64]) -> Result<Vec<File>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder = QueryBuilder::<Db>::new(
            r#"
            SELECT id, owner_id, object_key, name, content_type, size, status, created_at
            FROM file
            WHERE id IN (
            "#,
        );
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        builder.push(")");

        builder.build_query_as::<File>().fetch_all(&self.pool).await
    }

    pub async fn find_media_by_file_ids(
        &self,
        file_ids: &[i64],
    ) -> Result<Vec<Media>, sqlx::Error> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder = QueryBuilder::<Db>::new(
            r#"
            SELECT file_id, kind, width, height, duration_ms, thumbnail_id, created_at
            FROM media
            WHERE file_id IN (
            "#,
        );
        let mut separated = builder.separated(", ");
        for id in file_ids {
            separated.push_bind(id);
        }
        builder.push(")");

        builder
            .build_query_as::<Media>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn mark_ready(
        &self,
        id: i64,
        media: Option<ReadyMedia>,
        thumbnail: Option<NewThumbnail>,
    ) -> Result<Option<(File, Option<Media>)>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let thumbnail_id = if media.is_some()
            && let Some(thumb) = thumbnail
        {
            let created = sqlx::query_as::<_, File>(
                r#"
                INSERT INTO file(owner_id, object_key, name, content_type, size, status, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, owner_id, object_key, name, content_type, size, status, created_at
                "#,
            )
            .bind(thumb.owner_id)
            .bind(&thumb.object_key)
            .bind("thumbnail.jpg")
            .bind("image/jpeg")
            .bind(thumb.size)
            .bind(FileStatus::Ready)
            .bind(Utc::now())
            .fetch_one(&mut *tx)
            .await?;
            Some(created.id)
        } else {
            None
        };

        let saved_media = if let Some(media) = media {
            let saved = sqlx::query_as::<_, Media>(
                r#"
                INSERT INTO media(file_id, kind, width, height, duration_ms, thumbnail_id, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING file_id, kind, width, height, duration_ms, thumbnail_id, created_at
                "#,
            )
            .bind(id)
            .bind(media.kind)
            .bind(media.width)
            .bind(media.height)
            .bind(media.duration_ms)
            .bind(thumbnail_id)
            .bind(Utc::now())
            .fetch_one(&mut *tx)
            .await?;
            Some(saved)
        } else {
            None
        };

        let updated = sqlx::query_as::<_, File>(
            r#"
            UPDATE file
            SET status = $1
            WHERE id = $2
            RETURNING id, owner_id, object_key, name, content_type, size, status, created_at
            "#,
        )
        .bind(FileStatus::Ready)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(updated.map(|file| (file, saved_media)))
    }

    pub async fn list_object_keys_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT object_key FROM file
            WHERE owner_id = $1
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM file
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
