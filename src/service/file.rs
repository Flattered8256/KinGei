use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::error::file::FileError;
use crate::model::file::{File, FileStatus};
use crate::repository::file::{FileRepository, NewThumbnail, ReadyMedia};
use crate::serializer::file::{
    CreateFileRequest, FileResponse, PresignDownloadResponse, PresignUploadResponse,
};
use crate::storage::{PRESIGN_TTL, Storage};
use crate::utils::media::{media_kind_from_content_type, probe_path, write_thumbnail};

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
            return self.to_response(file).await;
        }

        let meta = match self.storage.head_object(&file.object_key).await {
            Ok(Some(meta)) => meta,
            Ok(None) => return Err(AppError::File(FileError::UploadIncomplete)),
            Err(e) => return Err(AppError::Storage(e)),
        };

        if meta.content_length != file.size {
            return Err(AppError::File(FileError::SizeMismatch));
        }

        let (media, thumbnail) = self.probe_and_thumb(&file).await;
        let (updated, saved_media) = self
            .repo
            .mark_ready(file.id, media, thumbnail)
            .await?
            .ok_or(AppError::File(FileError::FileNotFound))?;
        Ok(FileResponse::from(updated, saved_media))
    }

    async fn to_response(&self, file: File) -> Result<FileResponse, AppError> {
        let media = self.repo.find_media(file.id).await?;
        Ok(FileResponse::from(file, media))
    }

    async fn probe_and_thumb(&self, file: &File) -> (Option<ReadyMedia>, Option<NewThumbnail>) {
        let Some(kind) = media_kind_from_content_type(&file.content_type) else {
            return (None, None);
        };

        let dir = match tempfile::TempDir::new() {
            Ok(dir) => dir,
            Err(e) => {
                tracing::error!(?e, "media tempdir failed");
                return (None, None);
            }
        };
        let source = dir.path().join("source");
        if let Err(e) = self
            .storage
            .get_object_to_path(&file.object_key, &source)
            .await
        {
            tracing::error!(?e, file_id = file.id, "download for probe failed");
            return (None, None);
        }

        let source_for_probe = source.clone();
        let probed =
            match tokio::task::spawn_blocking(move || probe_path(&source_for_probe, kind)).await {
                Ok(value) => value,
                Err(e) => {
                    tracing::error!(?e, "ffprobe join failed");
                    return (None, None);
                }
            };
        let Some(probe) = probed else {
            return (None, None);
        };
        let media = ReadyMedia {
            kind: probe.kind,
            width: probe.width,
            height: probe.height,
            duration_ms: probe.duration_ms,
        };

        let thumb_path = dir.path().join("thumbnail.jpg");
        let source_for_thumb = source.clone();
        let dest_for_thumb = thumb_path.clone();
        let width = probe.width;
        let height = probe.height;
        let thumb_ok = match tokio::task::spawn_blocking(move || {
            write_thumbnail(&source_for_thumb, &dest_for_thumb, width, height)
        })
        .await
        {
            Ok(Ok(())) => true,
            Ok(Err(e)) => {
                tracing::error!(error = %e, file_id = file.id, "ffmpeg thumbnail failed");
                false
            }
            Err(e) => {
                tracing::error!(?e, "ffmpeg join failed");
                false
            }
        };

        if !thumb_ok {
            return (Some(media), None);
        }

        let object_key = format!("{}/{}", file.owner_id, Uuid::new_v4());
        match self
            .storage
            .put_object(&object_key, "image/jpeg", &thumb_path)
            .await
        {
            Ok(size) => (
                Some(media),
                Some(NewThumbnail {
                    owner_id: file.owner_id,
                    object_key,
                    size,
                }),
            ),
            Err(e) => {
                tracing::error!(?e, file_id = file.id, "thumbnail upload failed");
                (Some(media), None)
            }
        }
    }

    pub async fn download(&self, id: i64) -> Result<PresignDownloadResponse, AppError> {
        let file = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::File(FileError::FileNotFound))?;

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

const IMAGE_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

pub async fn require_ready_file(
    repo: &FileRepository,
    user_id: Uuid,
    file_id: i64,
) -> Result<File, AppError> {
    let file = repo
        .find_by_id(file_id)
        .await?
        .ok_or(AppError::File(FileError::FileNotFound))?;
    if file.owner_id != user_id {
        return Err(AppError::File(FileError::NotOwner));
    }
    if file.status != FileStatus::Ready {
        return Err(AppError::File(FileError::FileNotReady));
    }
    Ok(file)
}

pub async fn require_image(
    repo: &FileRepository,
    user_id: Uuid,
    file_id: i64,
) -> Result<(), AppError> {
    let file = require_ready_file(repo, user_id, file_id).await?;
    if !IMAGE_CONTENT_TYPES
        .iter()
        .any(|allowed| file.content_type.eq_ignore_ascii_case(allowed))
    {
        return Err(AppError::File(FileError::InvalidImage));
    }
    Ok(())
}
