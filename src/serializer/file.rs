use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::file::{File, FileStatus};

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub name: String,
    pub content_type: Option<String>,
    pub size: i64,
}

#[derive(Debug, Serialize)]
pub struct PresignUploadResponse {
    pub id: i64,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub upload_url: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: i64,
    pub owner_id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub status: FileStatus,
    pub created_at: DateTime<Utc>,
}

impl From<File> for FileResponse {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            owner_id: file.owner_id,
            name: file.name,
            content_type: file.content_type,
            size: file.size,
            status: file.status,
            created_at: file.created_at,
        }
    }
}
