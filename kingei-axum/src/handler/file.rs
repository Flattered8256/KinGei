use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::multipart::{Field, Multipart, MultipartRejection};
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::HeaderMap,
};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::file::FileError;
use crate::serializer::file::FileResponse;
use crate::state::AppState;
use crate::storage::{StorageError, content_disposition};

const MAX_NAME_FIELD_BYTES: usize = 4 * 1024;

#[derive(Debug)]
struct ReceivedUpload {
    name: String,
    content_type: String,
    size: i64,
    path: PathBuf,
    _dir: tempfile::TempDir,
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Json<FileResponse>, AppError> {
    let multipart = multipart.map_err(|_| AppError::File(FileError::InvalidUpload))?;
    let received = receive(multipart, state.file_service.max_upload_bytes()).await?;
    let result = state
        .file_service
        .upload(
            user_id,
            &received.name,
            &received.content_type,
            received.size,
            &received.path,
        )
        .await?;
    Ok(Json(result))
}

pub async fn content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_file(state, id, &headers, true).await
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_file(state, id, &headers, false).await
}

async fn stream_file(
    state: Arc<AppState>,
    id: i64,
    headers: &HeaderMap,
    inline: bool,
) -> Result<Response, AppError> {
    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok());
    let result = state.file_service.content(id, range).await?;
    let status = if result.ranged && result.object.content_range.is_some() {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };

    let mut response = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, result.file.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            content_disposition(&result.file.name, inline),
        )
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "private, max-age=86400");

    if result.object.content_length > 0 {
        response = response.header(
            header::CONTENT_LENGTH,
            result.object.content_length.to_string(),
        );
    }
    if let Some(content_range) = result.object.content_range {
        response = response.header(header::CONTENT_RANGE, content_range);
    }

    let body = Body::from_stream(ReaderStream::new(result.object.body.into_async_read()));
    response
        .body(body)
        .map_err(|e| AppError::Storage(StorageError(e.to_string())))
}

async fn receive(mut multipart: Multipart, max_bytes: u64) -> Result<ReceivedUpload, AppError> {
    let dir = tempfile::TempDir::new().map_err(internal_io)?;
    let mut name_field: Option<String> = None;
    let mut part_name: Option<String> = None;
    let mut content_type = String::new();
    let mut size: i64 = 0;
    let mut path: Option<PathBuf> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::File(FileError::InvalidUpload))?
    {
        match field.name() {
            Some("name") => {
                if name_field.is_none() {
                    name_field = Some(read_text(&mut field).await?);
                }
            }
            Some("file") => {
                if path.is_some() {
                    return Err(AppError::File(FileError::InvalidUpload));
                }
                content_type = field.content_type().unwrap_or_default().to_string();
                part_name = field.file_name().map(str::to_string);
                let target = dir.path().join("upload");
                let mut writer = tokio::fs::File::create(&target)
                    .await
                    .map_err(internal_io)?;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|_| AppError::File(FileError::InvalidUpload))?
                {
                    size += chunk.len() as i64;
                    if size as u64 > max_bytes {
                        return Err(AppError::File(FileError::FileTooLarge));
                    }
                    writer.write_all(&chunk).await.map_err(internal_io)?;
                }
                writer.flush().await.map_err(internal_io)?;
                path = Some(target);
            }
            _ => {}
        }
    }

    let path = path.ok_or(AppError::File(FileError::InvalidUpload))?;
    Ok(ReceivedUpload {
        name: name_field.or(part_name).unwrap_or_default(),
        content_type,
        size,
        path,
        _dir: dir,
    })
}

async fn read_text(field: &mut Field<'_>) -> Result<String, AppError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| AppError::File(FileError::InvalidUpload))?
    {
        if bytes.len() + chunk.len() > MAX_NAME_FIELD_BYTES {
            return Err(AppError::File(FileError::FileNameTooLong));
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| AppError::File(FileError::InvalidUpload))
}

fn internal_io(error: std::io::Error) -> AppError {
    AppError::Storage(StorageError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::FromRequest;
    use axum::http::Request;

    const BOUNDARY: &str = "KinGeiBoundary";

    fn part(headers: &str, content: &str) -> String {
        format!("--{BOUNDARY}\r\n{headers}\r\n\r\n{content}\r\n")
    }

    fn request(parts: Vec<String>) -> Request<Body> {
        let mut body = String::new();
        for item in parts {
            body.push_str(&item);
        }
        body.push_str(&format!("--{BOUNDARY}--\r\n"));
        Request::builder()
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={BOUNDARY}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    async fn receive_body(parts: Vec<String>, max_bytes: u64) -> Result<ReceivedUpload, AppError> {
        let multipart = Multipart::from_request(request(parts), &()).await.unwrap();
        receive(multipart, max_bytes).await
    }

    fn file_part(filename: &str, content_type: &str, content: &str) -> String {
        part(
            &format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}"
            ),
            content,
        )
    }

    fn name_part(name: &str) -> String {
        part("Content-Disposition: form-data; name=\"name\"", name)
    }

    #[tokio::test]
    async fn receives_name_and_file() {
        let received = receive_body(
            vec![
                name_part("照片.png"),
                file_part("raw.png", "image/png", "hello"),
            ],
            1024,
        )
        .await
        .unwrap();

        assert_eq!(received.name, "照片.png");
        assert_eq!(received.content_type, "image/png");
        assert_eq!(received.size, 5);
        assert_eq!(std::fs::read(&received.path).unwrap(), b"hello");
    }

    #[tokio::test]
    async fn name_field_wins_over_part_filename() {
        let received = receive_body(
            vec![
                file_part("raw.png", "image/png", "hello"),
                name_part("照片.png"),
            ],
            1024,
        )
        .await
        .unwrap();

        assert_eq!(received.name, "照片.png");
    }

    #[tokio::test]
    async fn rejects_too_large() {
        let error = receive_body(vec![file_part("big.bin", "application/zip", "hello")], 4)
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::File(FileError::FileTooLarge)));
    }

    #[tokio::test]
    async fn rejects_missing_file_part() {
        let error = receive_body(vec![name_part("照片.png")], 1024)
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::File(FileError::InvalidUpload)));
    }
}
