use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::TempDir;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::file::FileError;
use crate::model::file::{File, FileStatus};
use crate::repository::file::{FileRepository, NewFile, NewThumbnail, ReadyMedia};
use crate::serializer::file::FileResponse;
use crate::storage::{ObjectStream, Storage, parse_bytes_range};
use crate::utils::media::{media_kind_from_content_type, probe_path, write_thumbnail};

const MAX_NAME_CHARS: usize = 255;
const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

pub struct FileService {
    repo: Arc<FileRepository>,
    storage: Storage,
}

/// 上传请求已落盘的部分，传输层（multipart）与业务层之间的边界。
#[derive(Debug)]
pub struct ReceivedUpload {
    pub dir: TempDir,
    pub source: PathBuf,
    pub name: String,
    pub content_type: String,
    pub size: i64,
}

impl FileService {
    pub fn new(repo: Arc<FileRepository>, storage: Storage) -> Self {
        Self { repo, storage }
    }

    pub async fn store(
        &self,
        owner_id: Uuid,
        upload: ReceivedUpload,
    ) -> Result<FileResponse, AppError> {
        let name = upload.name.trim();
        if name.is_empty() {
            return Err(AppError::File(FileError::EmptyName));
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(AppError::File(FileError::FileNameTooLong));
        }
        if upload.size <= 0 {
            return Err(AppError::File(FileError::InvalidSize));
        }

        let content_type = normalize_content_type(&upload.content_type);
        let (media, thumbnail_path) = self
            .probe_media(&upload.source, &content_type, &upload.dir)
            .await;

        let object_key = format!("{}/{}", owner_id, Uuid::new_v4());
        let size = self
            .storage
            .put_object(&object_key, &content_type, &upload.source)
            .await?;

        let thumbnail = self.upload_thumbnail(owner_id, thumbnail_path).await;
        let thumbnail_key = thumbnail.as_ref().map(|item| item.object_key.clone());

        let new_file = NewFile {
            owner_id,
            object_key: object_key.clone(),
            name: name.to_string(),
            content_type: content_type.clone(),
            size,
        };

        match self.repo.create_ready(new_file, media, thumbnail).await {
            Ok((file, saved_media)) => Ok(FileResponse::from(file, saved_media)),
            Err(e) => {
                self.cleanup_object(&object_key).await;
                if let Some(key) = thumbnail_key {
                    self.cleanup_object(&key).await;
                }
                Err(AppError::Database(e))
            }
        }
    }

    async fn upload_thumbnail(
        &self,
        owner_id: Uuid,
        path: Option<PathBuf>,
    ) -> Option<NewThumbnail> {
        let path = path?;
        let object_key = format!("{}/{}", owner_id, Uuid::new_v4());
        match self
            .storage
            .put_object(&object_key, "image/jpeg", &path)
            .await
        {
            Ok(size) => Some(NewThumbnail {
                owner_id,
                object_key,
                size,
            }),
            Err(e) => {
                tracing::error!(?e, "thumbnail upload failed");
                None
            }
        }
    }

    async fn cleanup_object(&self, key: &str) {
        if let Err(e) = self.storage.delete_object(key).await {
            tracing::error!(?e, key, "compensate delete failed");
        }
    }

    async fn probe_media(
        &self,
        source: &Path,
        content_type: &str,
        dir: &TempDir,
    ) -> (Option<ReadyMedia>, Option<PathBuf>) {
        let Some(kind) = media_kind_from_content_type(content_type) else {
            return (None, None);
        };

        let source_for_probe = source.to_path_buf();
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
        let source_for_thumb = source.to_path_buf();
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
                tracing::error!(error = %e, "ffmpeg thumbnail failed");
                false
            }
            Err(e) => {
                tracing::error!(?e, "ffmpeg join failed");
                false
            }
        };

        if thumb_ok {
            (Some(media), Some(thumb_path))
        } else {
            (Some(media), None)
        }
    }

    pub async fn content(
        &self,
        id: i64,
        range_header: Option<&str>,
    ) -> Result<FileContent, AppError> {
        let file = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::File(FileError::FileNotFound))?;

        if file.status != FileStatus::Ready {
            return Err(AppError::File(FileError::FileNotReady));
        }

        let range = range_header.and_then(parse_bytes_range);
        let object = self
            .storage
            .get_object(&file.object_key, range.as_deref())
            .await?;

        Ok(FileContent {
            file,
            object,
            ranged: range.is_some(),
        })
    }
}

pub struct FileContent {
    pub file: File,
    pub object: ObjectStream,
    pub ranged: bool,
}

fn normalize_content_type(raw: &str) -> String {
    let value: String = raw
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(255)
        .collect();
    if value.is_empty() {
        DEFAULT_CONTENT_TYPE.to_string()
    } else {
        value
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
