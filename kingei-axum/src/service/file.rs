use std::path::Path;
use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::file::FileError;
use crate::model::file::{File, FileStatus};
use crate::repository::file::{FileRepository, NewThumbnail, ReadyMedia};
use crate::serializer::file::FileResponse;
use crate::storage::{ObjectStream, Storage, parse_bytes_range};
use crate::utils::media::{media_kind_from_content_type, probe_path, write_thumbnail};

const MAX_NAME_CHARS: usize = 255;

pub struct FileService {
    repo: Arc<FileRepository>,
    storage: Storage,
    max_upload_bytes: u64,
}

impl FileService {
    pub fn new(repo: Arc<FileRepository>, storage: Storage, max_upload_bytes: u64) -> Self {
        Self {
            repo,
            storage,
            max_upload_bytes,
        }
    }

    /// handler 收流时要提前掐断，所以上限由传输层读取；判定只在 handler 里做一次。
    pub fn max_upload_bytes(&self) -> u64 {
        self.max_upload_bytes
    }

    /// 中转上传：调用方已把请求体落到 `source`，这里只做校验、入库和探测。
    pub async fn upload(
        &self,
        owner_id: Uuid,
        name: &str,
        content_type: &str,
        size: i64,
        source: &Path,
    ) -> Result<FileResponse, AppError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::File(FileError::EmptyName));
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(AppError::File(FileError::FileNameTooLong));
        }
        if size <= 0 {
            return Err(AppError::File(FileError::InvalidSize));
        }

        let content_type = content_type
            .split(';')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("application/octet-stream")
            .to_string();

        let object_key = format!("{}/{}", owner_id, Uuid::new_v4());
        let stored_size = self
            .storage
            .put_object(&object_key, &content_type, source)
            .await?;

        let file = match self
            .repo
            .create(
                owner_id,
                &object_key,
                name,
                &content_type,
                stored_size,
                FileStatus::Pending,
            )
            .await
        {
            Ok(file) => file,
            Err(e) => {
                self.discard_object(&object_key).await;
                return Err(e.into());
            }
        };

        let (media, thumbnail) = self.probe_and_thumb(&file, source).await;
        let (updated, saved_media) = match self.repo.mark_ready(file.id, media, thumbnail).await {
            Ok(Some(result)) => result,
            Ok(None) => {
                self.compensate(&file).await;
                return Err(AppError::File(FileError::FileNotFound));
            }
            Err(e) => {
                self.compensate(&file).await;
                return Err(e.into());
            }
        };

        Ok(FileResponse::from(updated, saved_media))
    }

    async fn compensate(&self, file: &File) {
        self.discard_object(&file.object_key).await;
        if let Err(e) = self.repo.delete(file.id).await {
            tracing::error!(?e, file_id = file.id, "compensate delete failed");
        }
    }

    async fn discard_object(&self, key: &str) {
        if let Err(e) = self.storage.delete_object(key).await {
            tracing::error!(?e, object_key = key, "compensate delete failed");
        }
    }

    async fn probe_and_thumb(
        &self,
        file: &File,
        source: &Path,
    ) -> (Option<ReadyMedia>, Option<NewThumbnail>) {
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
