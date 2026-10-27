use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize)]
#[cfg_attr(feature = "sqlite", sqlx(rename_all = "PascalCase"))]
#[cfg_attr(
    feature = "postgres",
    sqlx(type_name = "text", rename_all = "PascalCase")
)]
pub enum FileStatus {
    Pending,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize)]
#[cfg_attr(feature = "sqlite", sqlx(rename_all = "PascalCase"))]
#[cfg_attr(
    feature = "postgres",
    sqlx(type_name = "text", rename_all = "PascalCase")
)]
#[serde(rename_all = "PascalCase")]
pub enum MediaKind {
    Image,
    Video,
}

#[derive(Debug, FromRow)]
pub struct File {
    pub id: i64,
    pub owner_id: Uuid,
    pub object_key: String,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub status: FileStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct Media {
    pub file_id: i64,
    pub kind: MediaKind,
    pub width: i64,
    pub height: i64,
    pub duration_ms: Option<i64>,
    pub thumbnail_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}
