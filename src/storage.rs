use std::env;
use std::time::Duration;

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{
    Credentials, Region, RequestChecksumCalculation, ResponseChecksumValidation,
};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::presigning::PresigningConfig;

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
            .response_content_disposition(&content_disposition(filename))
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
}

pub struct ObjectMeta {
    pub content_length: i64,
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

fn content_disposition(filename: &str) -> String {
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
    format!(
        "attachment; filename=\"{fallback}\"; filename*=UTF-8''{}",
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
