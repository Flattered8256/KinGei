use std::env;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use futures::StreamExt;
use object_store::aws::AmazonS3Builder;
use object_store::buffered::BufWriter;
use object_store::local::LocalFileSystem;
use object_store::path::Path as ObjectPath;
use object_store::{GetOptions, GetRange, GetResultPayload, ObjectStore, ObjectStoreExt};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio_util::io::StreamReader;

const DEFAULT_LOCAL_DIR: &str = "./KinGei-data/files";
const UPLOAD_BUFFER_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("object store: {0}")]
    Object(#[from] object_store::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type ObjectBody = Pin<Box<dyn AsyncRead + Send>>;

pub struct ObjectStream {
    pub content_length: i64,
    pub content_range: Option<String>,
    pub body: ObjectBody,
}

#[derive(Clone)]
pub struct Storage {
    store: Arc<dyn ObjectStore>,
}

impl Storage {
    pub async fn connect() -> Self {
        let driver = env::var("STORAGE_DRIVER").unwrap_or_else(|_| "local".to_string());
        match driver.trim().to_ascii_lowercase().as_str() {
            "local" => Self::connect_local().await,
            "s3" => Self::connect_s3().await,
            other => {
                panic!("STORAGE_DRIVER必须是local或s3,当前为{other},请检查项目根目录下的.env文件")
            }
        }
    }

    async fn connect_local() -> Self {
        let dir = env::var("STORAGE_LOCAL_DIR").unwrap_or_else(|_| DEFAULT_LOCAL_DIR.to_string());
        let dir = PathBuf::from(dir.trim());
        if dir.as_os_str().is_empty() {
            panic!("STORAGE_LOCAL_DIR不能为空,请检查项目根目录下的.env文件");
        }
        tokio::fs::create_dir_all(&dir)
            .await
            .unwrap_or_else(|e| panic!("创建本地存储目录失败 {}: {e}", dir.display()));
        let store = LocalFileSystem::new_with_prefix(&dir)
            .unwrap_or_else(|e| panic!("打开本地存储目录失败 {}: {e}", dir.display()))
            .with_automatic_cleanup(true);
        tracing::info!(dir = %dir.display(), "local storage ready");
        Self {
            store: Arc::new(store),
        }
    }

    async fn connect_s3() -> Self {
        let endpoint = require_env("S3_ENDPOINT");
        let bucket = require_env("S3_BUCKET");
        let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let path_style = env_flag("S3_PATH_STYLE", true);
        let allow_http = endpoint.starts_with("http://");

        let store = AmazonS3Builder::new()
            .with_endpoint(endpoint.as_str())
            .with_access_key_id(require_env("S3_ACCESS_KEY"))
            .with_secret_access_key(require_env("S3_SECRET_KEY"))
            .with_bucket_name(bucket.as_str())
            .with_region(region.as_str())
            .with_allow_http(allow_http)
            .with_virtual_hosted_style_request(!path_style)
            .build()
            .unwrap_or_else(|e| panic!("初始化S3失败: {e}"));

        // object_store 没有建桶 API，启动时校验桶存在；Garage 由 compose 自动建桶。
        store
            .list_with_delimiter(None)
            .await
            .unwrap_or_else(|e| panic!("S3 bucket不存在或无权访问({bucket}): {e}"));
        tracing::info!(
            bucket = %bucket,
            endpoint = %endpoint,
            path_style,
            "s3 connected"
        );
        Self {
            store: Arc::new(store),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_store(store: Arc<dyn ObjectStore>) -> Self {
        Self { store }
    }

    pub async fn put_object(&self, key: &str, source: &Path) -> Result<i64, StorageError> {
        let location = object_path(key)?;
        let mut source = tokio::fs::File::open(source).await?;
        let mut writer =
            BufWriter::with_capacity(self.store.clone(), location, UPLOAD_BUFFER_BYTES);

        match tokio::io::copy(&mut source, &mut writer).await {
            Ok(size) => {
                writer.shutdown().await?;
                Ok(size as i64)
            }
            Err(e) => {
                if let Err(abort) = writer.abort().await {
                    tracing::error!(?abort, "abort upload failed");
                }
                Err(StorageError::Io(e))
            }
        }
    }

    pub async fn get_object(
        &self,
        key: &str,
        range: Option<&ByteRange>,
    ) -> Result<ObjectStream, StorageError> {
        let location = object_path(key)?;
        let options = match range {
            Some(range) => GetOptions::new().with_range(Some(range.to_get_range()?)),
            None => GetOptions::new(),
        };
        let result = self.store.get_opts(&location, options).await?;

        // object_store 返回的 range 是本次实际返回的字节区间，end 为开区间。
        let span = result.range.clone();
        let content_length = span.end.saturating_sub(span.start);
        let content_range = if range.is_some() && content_length > 0 {
            Some(format!(
                "bytes {}-{}/{}",
                span.start,
                span.end - 1,
                result.meta.size
            ))
        } else {
            None
        };

        let body: ObjectBody = match result.payload {
            GetResultPayload::File(file, _) => {
                let mut file = tokio::fs::File::from_std(file);
                file.seek(std::io::SeekFrom::Start(span.start)).await?;
                Box::pin(file.take(content_length))
            }
            GetResultPayload::Stream(stream) => {
                let stream = stream.map(|item| item.map_err(std::io::Error::other));
                Box::pin(StreamReader::new(stream))
            }
        };

        Ok(ObjectStream {
            content_length: content_length as i64,
            content_range,
            body,
        })
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        let location = object_path(key)?;
        match self.store.delete(&location).await {
            Ok(()) => Ok(()),
            // local 删除不存在的文件会报 NotFound，S3 是幂等的；这里统一成幂等。
            Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(e) => Err(StorageError::Object(e)),
        }
    }
}

fn object_path(key: &str) -> Result<ObjectPath, StorageError> {
    if key.trim().is_empty() {
        return Err(StorageError::Other("object key不能为空".to_string()));
    }
    ObjectPath::parse(key).map_err(|e| StorageError::Other(format!("非法object key {key}: {e}")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    Bounded { start: u64, end: u64 },
    Open { start: u64 },
    Suffix { len: u64 },
}

impl ByteRange {
    pub fn to_get_range(self) -> Result<GetRange, StorageError> {
        match self {
            Self::Bounded { start, end } => {
                // HTTP 的 end 是闭区间，GetRange 的 end 是开区间。
                let end = end
                    .checked_add(1)
                    .ok_or_else(|| StorageError::Other("range end溢出".to_string()))?;
                Ok(GetRange::Bounded(start..end))
            }
            Self::Open { start } => Ok(GetRange::Offset(start)),
            Self::Suffix { len } => Ok(GetRange::Suffix(len)),
        }
    }
}

pub fn parse_byte_range(header: &str) -> Option<ByteRange> {
    let spec = header.trim().strip_prefix("bytes=")?.trim();
    if spec.is_empty() || spec.contains(',') {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    if start.is_empty() && end.is_empty() {
        return None;
    }
    if start.is_empty() {
        let len = parse_u64(end)?;
        return (len > 0).then_some(ByteRange::Suffix { len });
    }
    let start = parse_u64(start)?;
    if end.is_empty() {
        return Some(ByteRange::Open { start });
    }
    let end = parse_u64(end)?;
    (end >= start).then_some(ByteRange::Bounded { start, end })
}

fn parse_u64(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn require_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("缺少{name},请检查项目根目录下的.env文件"))
}

fn env_flag(name: &str, default: bool) -> bool {
    match env::var(name) {
        Ok(value) => parse_bool(&value)
            .unwrap_or_else(|| panic!("{name}必须是true或false,请检查项目根目录下的.env文件")),
        Err(_) => default,
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
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
    fn parse_bounded_range() {
        assert_eq!(
            parse_byte_range("bytes=0-1023"),
            Some(ByteRange::Bounded {
                start: 0,
                end: 1023
            })
        );
    }

    #[test]
    fn parse_open_range() {
        assert_eq!(
            parse_byte_range("bytes=500-"),
            Some(ByteRange::Open { start: 500 })
        );
    }

    #[test]
    fn parse_suffix_range() {
        assert_eq!(
            parse_byte_range("bytes=-500"),
            Some(ByteRange::Suffix { len: 500 })
        );
    }

    #[test]
    fn reject_invalid_ranges() {
        assert_eq!(parse_byte_range("bytes=0-1,2-3"), None);
        assert_eq!(parse_byte_range("bytes="), None);
        assert_eq!(parse_byte_range("bytes=-"), None);
        assert_eq!(parse_byte_range("items=0-1"), None);
        assert_eq!(parse_byte_range("bytes=10-5"), None);
        assert_eq!(parse_byte_range("bytes=-0"), None);
        assert_eq!(parse_byte_range("bytes=abc-1"), None);
    }

    #[test]
    fn map_ranges_to_object_store() {
        assert_eq!(
            ByteRange::Bounded { start: 0, end: 4 }
                .to_get_range()
                .unwrap(),
            GetRange::Bounded(0..5)
        );
        assert_eq!(
            ByteRange::Open { start: 6 }.to_get_range().unwrap(),
            GetRange::Offset(6)
        );
        assert_eq!(
            ByteRange::Suffix { len: 5 }.to_get_range().unwrap(),
            GetRange::Suffix(5)
        );
        assert!(
            ByteRange::Bounded {
                start: 0,
                end: u64::MAX
            }
            .to_get_range()
            .is_err()
        );
    }

    #[test]
    fn parse_bool_flags() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    async fn read_body(mut body: ObjectBody) -> Vec<u8> {
        let mut bytes = Vec::new();
        body.read_to_end(&mut bytes).await.unwrap();
        bytes
    }

    async fn local_storage() -> (tempfile::TempDir, Storage, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalFileSystem::new_with_prefix(dir.path()).unwrap();
        let storage = Storage::from_store(Arc::new(store));
        let source = dir.path().join("source.bin");
        tokio::fs::write(&source, b"hello world").await.unwrap();
        (dir, storage, source)
    }

    #[tokio::test]
    async fn local_put_get_delete_roundtrip() {
        let (dir, storage, source) = local_storage().await;

        let size = storage.put_object("owner/key", &source).await.unwrap();
        assert_eq!(size, 11);
        assert!(dir.path().join("owner/key").is_file());

        let object = storage.get_object("owner/key", None).await.unwrap();
        assert_eq!(object.content_length, 11);
        assert_eq!(object.content_range, None);
        assert_eq!(read_body(object.body).await, b"hello world");

        storage.delete_object("owner/key").await.unwrap();
        assert!(!dir.path().join("owner/key").exists());
        storage.delete_object("owner/key").await.unwrap();
    }

    #[tokio::test]
    async fn local_get_bounded_range() {
        let (_dir, storage, source) = local_storage().await;
        storage.put_object("owner/key", &source).await.unwrap();

        let range = parse_byte_range("bytes=2-6").unwrap();
        let object = storage.get_object("owner/key", Some(&range)).await.unwrap();
        assert_eq!(object.content_length, 5);
        assert_eq!(object.content_range.as_deref(), Some("bytes 2-6/11"));
        assert_eq!(read_body(object.body).await, b"llo w");
    }

    #[tokio::test]
    async fn local_get_open_range() {
        let (_dir, storage, source) = local_storage().await;
        storage.put_object("owner/key", &source).await.unwrap();

        let range = parse_byte_range("bytes=6-").unwrap();
        let object = storage.get_object("owner/key", Some(&range)).await.unwrap();
        assert_eq!(object.content_length, 5);
        assert_eq!(object.content_range.as_deref(), Some("bytes 6-10/11"));
        assert_eq!(read_body(object.body).await, b"world");
    }

    #[tokio::test]
    async fn local_get_suffix_range() {
        let (_dir, storage, source) = local_storage().await;
        storage.put_object("owner/key", &source).await.unwrap();

        let range = parse_byte_range("bytes=-5").unwrap();
        let object = storage.get_object("owner/key", Some(&range)).await.unwrap();
        assert_eq!(object.content_length, 5);
        assert_eq!(object.content_range.as_deref(), Some("bytes 6-10/11"));
        assert_eq!(read_body(object.body).await, b"world");
    }

    #[tokio::test]
    async fn local_range_is_clamped_to_file() {
        let (_dir, storage, source) = local_storage().await;
        storage.put_object("owner/key", &source).await.unwrap();

        let range = parse_byte_range("bytes=0-999").unwrap();
        let object = storage.get_object("owner/key", Some(&range)).await.unwrap();
        assert_eq!(object.content_length, 11);
        assert_eq!(object.content_range.as_deref(), Some("bytes 0-10/11"));
        assert_eq!(read_body(object.body).await, b"hello world");
    }

    #[tokio::test]
    async fn local_range_out_of_bounds_fails() {
        let (_dir, storage, source) = local_storage().await;
        storage.put_object("owner/key", &source).await.unwrap();

        let range = parse_byte_range("bytes=11-20").unwrap();
        assert!(storage.get_object("owner/key", Some(&range)).await.is_err());
    }

    #[tokio::test]
    async fn local_missing_object_fails() {
        let (_dir, storage, _source) = local_storage().await;
        assert!(storage.get_object("owner/missing", None).await.is_err());
    }

    #[tokio::test]
    async fn local_rejects_invalid_keys() {
        let (_dir, storage, source) = local_storage().await;
        assert!(storage.put_object("../escape", &source).await.is_err());
        assert!(storage.put_object("", &source).await.is_err());
        assert!(storage.delete_object("../escape").await.is_err());
    }
}
