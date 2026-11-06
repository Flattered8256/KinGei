use std::sync::Arc;

use axum::body::Body;
use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::Response;
use axum::{Extension, Json};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::file::FileError;
use crate::serializer::file::FileResponse;
use crate::service::file::{FileContent, ReceivedUpload};
use crate::state::AppState;
use crate::storage::{StorageError, content_disposition};
use crate::utils::config::max_upload_bytes;

pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Json<FileResponse>, AppError> {
    let multipart = multipart.map_err(|_| AppError::File(FileError::InvalidMultipart))?;
    let upload = receive(multipart, max_upload_bytes() as u64).await?;
    let result = state.file_service.store(user_id, upload).await?;
    Ok(Json(result))
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = state
        .file_service
        .content(id, range_header(&headers))
        .await?;
    stream(result, false)
}

pub async fn content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = state
        .file_service
        .content(id, range_header(&headers))
        .await?;
    stream(result, true)
}

fn range_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
}

fn stream(result: FileContent, inline: bool) -> Result<Response, AppError> {
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

/// 把 multipart 的 `file` 字段流式写到临时文件，不把整个上传读进内存。
async fn receive(mut multipart: Multipart, max: u64) -> Result<ReceivedUpload, AppError> {
    let dir = tempfile::TempDir::new().map_err(io_error)?;

    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        if field.name() != Some("file") {
            continue;
        }

        let name = field.file_name().unwrap_or_default().trim().to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let source = dir.path().join("source");
        let mut file = tokio::fs::File::create(&source).await.map_err(io_error)?;
        let mut size: u64 = 0;
        let mut field = field;
        while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
            size += chunk.len() as u64;
            if size > max {
                return Err(AppError::File(FileError::FileTooLarge));
            }
            file.write_all(&chunk).await.map_err(io_error)?;
        }
        file.flush().await.map_err(io_error)?;
        drop(file);
        drop(field);

        return Ok(ReceivedUpload {
            dir,
            source,
            name,
            content_type,
            size: i64::try_from(size).unwrap_or(i64::MAX),
        });
    }

    Err(AppError::File(FileError::MissingFile))
}

fn multipart_error(_: MultipartError) -> AppError {
    AppError::File(FileError::InvalidMultipart)
}

fn io_error(error: std::io::Error) -> AppError {
    AppError::Storage(StorageError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequest;
    use axum::http::{Request, header};

    /// (字段名, 文件名, Content-Type, 内容)
    type Part<'a> = (&'a str, Option<&'a str>, Option<&'a str>, &'a [u8]);

    fn multipart_request(parts: &[Part<'_>]) -> Request<Body> {
        let boundary = "X-KINGEI-BOUNDARY";
        let mut body = Vec::new();
        for (name, filename, content_type, data) in parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            match filename {
                Some(filename) => body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
                    )
                    .as_bytes(),
                ),
                None => body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n").as_bytes(),
                ),
            }
            if let Some(content_type) = content_type {
                body.extend_from_slice(format!("Content-Type: {content_type}\r\n").as_bytes());
            }
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        Request::builder()
            .method("POST")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    async fn receive_parts(parts: &[Part<'_>], max: u64) -> Result<ReceivedUpload, AppError> {
        let multipart = Multipart::from_request(multipart_request(parts), &())
            .await
            .unwrap();
        receive(multipart, max).await
    }

    const DEFAULT_MAX: u64 = 100 * 1024 * 1024;

    #[tokio::test]
    async fn receives_file_field() {
        let upload = receive_parts(
            &[("file", Some("照片.png"), Some("image/png"), b"hello")],
            DEFAULT_MAX,
        )
        .await
        .unwrap();

        assert_eq!(upload.name, "照片.png");
        assert_eq!(upload.content_type, "image/png");
        assert_eq!(upload.size, 5);
        assert_eq!(tokio::fs::read(&upload.source).await.unwrap(), b"hello");
    }

    #[tokio::test]
    async fn skips_other_fields() {
        let upload = receive_parts(
            &[
                ("note", None, None, b"ignored"),
                ("file", Some("a.txt"), None, b"data"),
            ],
            DEFAULT_MAX,
        )
        .await
        .unwrap();

        assert_eq!(upload.name, "a.txt");
        assert_eq!(upload.content_type, "application/octet-stream");
        assert_eq!(upload.size, 4);
    }

    #[tokio::test]
    async fn rejects_oversized_file() {
        let error = receive_parts(&[("file", Some("a.bin"), None, b"0123456789")], 4)
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::File(FileError::FileTooLarge)));
    }

    #[tokio::test]
    async fn missing_file_field() {
        let error = receive_parts(&[("note", None, None, b"no file here")], DEFAULT_MAX)
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::File(FileError::MissingFile)));
    }

    #[tokio::test]
    async fn malformed_body() {
        let request = Request::builder()
            .method("POST")
            .header(header::CONTENT_TYPE, "multipart/form-data; boundary=broken")
            .body(Body::from("not a multipart body"))
            .unwrap();
        let multipart = Multipart::from_request(request, &()).await.unwrap();
        let error = receive(multipart, DEFAULT_MAX).await.unwrap_err();

        assert!(matches!(error, AppError::File(FileError::InvalidMultipart)));
    }
}
