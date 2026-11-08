use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::model::file::{File, FileStatus, Media, MediaKind};

#[derive(Debug, Clone, Serialize)]
pub struct MediaResponse {
    pub kind: MediaKind,
    pub width: i64,
    pub height: i64,
    pub duration_ms: Option<i64>,
    pub thumbnail_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileResponse {
    pub id: i64,
    pub owner_id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub status: FileStatus,
    pub media: Option<MediaResponse>,
    pub created_at: DateTime<Utc>,
}

impl FileResponse {
    pub fn from(file: File, media: Option<Media>) -> Self {
        Self {
            id: file.id,
            owner_id: file.owner_id,
            name: file.name,
            content_type: file.content_type,
            size: file.size,
            status: file.status,
            media: media.map(MediaResponse::from),
            created_at: file.created_at,
        }
    }
}

impl From<Media> for MediaResponse {
    fn from(media: Media) -> Self {
        Self {
            kind: media.kind,
            width: media.width,
            height: media.height,
            duration_ms: media.duration_ms,
            thumbnail_id: media.thumbnail_id,
        }
    }
}
