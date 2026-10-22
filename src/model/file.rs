use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize)]
#[sqlx(rename_all = "PascalCase")]
pub enum FileStatus {
    Pending,
    Ready,
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
