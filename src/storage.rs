use std::env;
use std::path::Path;
use std::time::Duration;

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{
    Credentials, Region, RequestChecksumCalculation, ResponseChecksumValidation,
};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use tokio::io::AsyncWriteExt;

pub const PRESIGN_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug)]
pub struct StorageError(pub String);

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StorageError {}

pub struct Storage {
    client: Client,
    bucket: String,
}

impl Storage {
    pub async fn connect() -> Self {
        let endpoint =
            env::var("GARAGE_ENDPOINT").expect("缺少GARAGE_ENDPOINT,请检查项目根目录下的.env文件");
        let access_key = env::var("GARAGE_ACCESS_KEY")
            .expect("缺少GARAGE_ACCESS_KEY,请检查项目根目录下的.env文件");
        let secret_key = env::var("GARAGE_SECRET_KEY")
            .expect("缺少GARAGE_SECRET_KEY,请检查项目根目录下的.env文件");
        let bucket =
            env::var("GARAGE_BUCKET").expect("缺少GARAGE_BUCKET,请检查项目根目录下的.env文件");
        let region = env::var("GARAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let credentials = Credentials::new(access_key, secret_key, None, None, "garage");
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(&endpoint)
            .region(Region::new(region))
            .credentials_provider(credentials)
            .load()
            .await;

        let config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired)
            .build();
        let client = Client::from_conf(config);

        let listed = client.list_buckets().send().await.expect("连接Garage失败");
        let exists = listed
            .buckets()
            .iter()
            .any(|item| item.name() == Some(bucket.as_str()));
        if !exists {
            client
                .create_bucket()
                .bucket(&bucket)
                .send()
                .await
                .expect("创建Garage bucket失败");
        }

        tracing::info!(bucket = %bucket, "garage connected");
        Self { client, bucket }
    }

    pub async fn presign_put(
        &self,
        key: &str,
        content_type: &str,
        size: i64,
    ) -> Result<String, StorageError> {
        let presigning =
            PresigningConfig::expires_in(PRESIGN_TTL).map_err(|e| StorageError(e.to_string()))?;
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .content_length(size)
            .presigned(presigning)
            .await
            .map_err(|e| StorageError(format!("{e:?}")))?;
        Ok(presigned.uri().to_string())
    }

    pub async fn presign_get(
        &self,
        key: &str,
        content_type: &str,
        filename: &str,
    ) -> Result<String, StorageError> {
        let presigning =
            PresigningConfig::expires_in(PRESIGN_TTL).map_err(|e| StorageError(e.to_string()))?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .response_content_type(content_type)
            .response_content_disposition(&content_disposition(filename, false))
            .presigned(presigning)
            .await
            .map_err(|e| StorageError(format!("{e:?}")))?;
        Ok(presigned.uri().to_string())
    }

    pub async fn head_object(&self, key: &str) -> Result<Option<ObjectMeta>, StorageError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(output) => Ok(Some(ObjectMeta {
                content_length: output.content_length().unwrap_or(0),
            })),
            Err(e) => {
                if is_not_found(&e) {
                    Ok(None)
                } else {
                    Err(StorageError(format!("{e:?}")))
                }
            }
        }
    }

    pub async fn get_object(
        &self,
        key: &str,
        range: Option<&str>,
    ) -> Result<ObjectStream, StorageError> {
        let mut request = self.client.get_object().bucket(&self.bucket).key(key);
        if let Some(range) = range {
            request = request.range(range);
        }
        let output = request
            .send()
            .await
            .map_err(|e| StorageError(format!("{e:?}")))?;
        Ok(ObjectStream {
            content_length: output.content_length().unwrap_or(0),
            content_range: output.content_range().map(str::to_string),
            body: output.body,
        })
    }

    pub async fn get_object_to_path(&self, key: &str, path: &Path) -> Result<(), StorageError> {
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError(format!("{e:?}")))?;
        let mut reader = object.body.into_async_read();
        let mut writer = tokio::fs::File::create(path)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        tokio::io::copy(&mut reader, &mut writer)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    pub async fn put_object(
        &self,
        key: &str,
        content_type: &str,
        path: &Path,
    ) -> Result<i64, StorageError> {
        let size = tokio::fs::metadata(path)
            .await
            .map_err(|e| StorageError(e.to_string()))?
            .len() as i64;
        let body = ByteStream::from_path(path)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .content_length(size)
            .body(body)
            .send()
            .await
            .map_err(|e| StorageError(format!("{e:?}")))?;
        Ok(size)
    }
}

pub struct ObjectMeta {
    pub content_length: i64,
}

pub struct ObjectStream {
    pub content_length: i64,
    pub content_range: Option<String>,
    pub body: ByteStream,
}

pub fn parse_bytes_range(header: &str) -> Option<String> {
    let spec = header.trim().strip_prefix("bytes=")?.trim();
    if spec.is_empty() || spec.contains(',') {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    if start.is_empty() && end.is_empty() {
        return None;
    }
    if !start.chars().all(|c| c.is_ascii_digit()) || !end.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("bytes={spec}"))
}

fn is_not_found(error: &SdkError<HeadObjectError>) -> bool {
    if error
        .as_service_error()
        .is_some_and(HeadObjectError::is_not_found)
    {
        return true;
    }
    error
        .raw_response()
        .is_some_and(|response| response.status().as_u16() == 404)
}

pub fn content_disposition(filename: &str, inline: bool) -> String {
    let sanitized: String = filename
        .chars()
        .filter(|c| *c != '\r' && *c != '\n')
        .collect();
    let fallback: String = sanitized
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let fallback = if fallback.is_empty() {
        "download".to_string()
    } else {
        fallback
    };
    let kind = if inline { "inline" } else { "attachment" };
    format!(
        "{kind}; filename=\"{fallback}\"; filename*=UTF-8''{}",
        percent_encode(sanitized.as_bytes())
    )
}

fn percent_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if is_unreserved(b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn is_unreserved(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_ascii_filename() {
        assert_eq!(
            content_disposition("hello.txt", false),
            "attachment; filename=\"hello.txt\"; filename*=UTF-8''hello.txt"
        );
    }

    #[test]
    fn inline_ascii_filename() {
        assert_eq!(
            content_disposition("hello.txt", true),
            "inline; filename=\"hello.txt\"; filename*=UTF-8''hello.txt"
        );
    }

    #[test]
    fn chinese_filename() {
        assert_eq!(
            content_disposition("照片.png", false),
            "attachment; filename=\"__.png\"; filename*=UTF-8''%E7%85%A7%E7%89%87.png"
        );
    }

    #[test]
    fn parse_open_and_closed_range() {
        assert_eq!(
            parse_bytes_range("bytes=0-1023"),
            Some("bytes=0-1023".to_string())
        );
        assert_eq!(
            parse_bytes_range("bytes=500-"),
            Some("bytes=500-".to_string())
        );
        assert_eq!(
            parse_bytes_range("bytes=-500"),
            Some("bytes=-500".to_string())
        );
        assert_eq!(parse_bytes_range("bytes=0-1,2-3"), None);
        assert_eq!(parse_bytes_range("bytes="), None);
        assert_eq!(parse_bytes_range("items=0-1"), None);
    }
}
