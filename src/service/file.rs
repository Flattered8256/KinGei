use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::error::file::FileError;
use crate::model::file::FileStatus;
use crate::repository::file::FileRepository;
use crate::serializer::file::{
    CreateFileRequest, FileResponse, PresignDownloadResponse, PresignUploadResponse,
};
use crate::storage::{PRESIGN_TTL, Storage};

const MAX_NAME_CHARS: usize = 255;

pub struct FileService {
    repo: Arc<FileRepository>,
    storage: Storage,
}

impl FileService {
    pub fn new(repo: Arc<FileRepository>, storage: Storage) -> Self {
        Self { repo, storage }
    }

    pub async fn create(
        &self,
        owner_id: Uuid,
        request: CreateFileRequest,
    ) -> Result<PresignUploadResponse, AppError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(AppError::File(FileError::EmptyName));
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(AppError::File(FileError::FileNameTooLong));
        }
        if request.size <= 0 {
            return Err(AppError::File(FileError::InvalidSize));
        }

        let content_type = request
            .content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("application/octet-stream")
            .to_string();

        let object_key = format!("{}/{}", owner_id, Uuid::new_v4());
        let file = self
            .repo
            .create(
                owner_id,
                &object_key,
                name,
                &content_type,
                request.size,
                FileStatus::Pending,
            )
            .await?;

        let upload_url = match self
            .storage
            .presign_put(&file.object_key, &file.content_type, file.size)
            .await
        {
            Ok(url) => url,
            Err(e) => {
                if let Err(delete_err) = self.repo.delete(file.id).await {
                    tracing::error!(?delete_err, "compensate delete failed");
                }
                return Err(AppError::Storage(e));
            }
        };

        Ok(PresignUploadResponse {
            id: file.id,
            name: file.name,
            content_type: file.content_type,
            size: file.size,
            upload_url,
            expires_at: Utc::now() + Duration::seconds(PRESIGN_TTL.as_secs() as i64),
        })
    }

    pub async fn complete(&self, user_id: Uuid, id: i64) -> Result<FileResponse, AppError> {
        let file = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::File(FileError::FileNotFound))?;

        if file.owner_id != user_id {
            return Err(AppError::File(FileError::NotOwner));
        }

        if file.status == FileStatus::Ready {
            return Ok(FileResponse::from(file));
        }

        let meta = match self.storage.head_object(&file.object_key).await {
            Ok(Some(meta)) => meta,
            Ok(None) => return Err(AppError::File(FileError::UploadIncomplete)),
            Err(e) => return Err(AppError::Storage(e)),
        };

        if meta.content_length != file.size {
            return Err(AppError::File(FileError::SizeMismatch));
        }

        let updated = self
            .repo
            .update_status(file.id, FileStatus::Ready)
            .await?
            .ok_or(AppError::File(FileError::FileNotFound))?;
        Ok(FileResponse::from(updated))
    }

    pub async fn download(
        &self,
        user_id: Uuid,
        id: i64,
    ) -> Result<PresignDownloadResponse, AppError> {
        let file = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::File(FileError::FileNotFound))?;

        if file.owner_id != user_id {
            return Err(AppError::File(FileError::NotOwner));
        }

        if file.status != FileStatus::Ready {
            return Err(AppError::File(FileError::FileNotReady));
        }

        let download_url = self
            .storage
            .presign_get(&file.object_key, &file.content_type, &file.name)
            .await?;

        Ok(PresignDownloadResponse {
            id: file.id,
            name: file.name,
            content_type: file.content_type,
            size: file.size,
            download_url,
            expires_at: Utc::now() + Duration::seconds(PRESIGN_TTL.as_secs() as i64),
        })
    }
}
