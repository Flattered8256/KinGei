use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

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

    pub async fn find_media(&self, file_id: i64) -> Result<Option<Media>, sqlx::Error> {
        sqlx::query_as::<_, Media>(
            r#"
            SELECT file_id, kind, width, height, duration_ms, thumbnail_id, created_at
            FROM media
            WHERE file_id = ?
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
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
                VALUES (?, ?, ?, ?, ?, ?, ?)
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
                VALUES (?, ?, ?, ?, ?, ?, ?)
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
            SET status = ?
            WHERE id = ?
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
