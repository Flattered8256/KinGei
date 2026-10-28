use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::header::{self, HeaderName};
use axum::http::{HeaderMap, StatusCode, Uri, Version};
use axum::response::{IntoResponse, Response};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

type ProxyClient = Client<hyper_util::client::legacy::connect::HttpConnector, Body>;

struct Site {
    bucket: String,
    endpoint: String,
    client: ProxyClient,
    files: ServeDir<ServeFile>,
}

pub fn with_frontend(api: Router, static_dir: String) -> Router {
    let root = PathBuf::from(static_dir.trim());
    let index = root.join("index.html");
    if !index.is_file() {
        panic!(
            "STATIC_DIR缺少index.html: {},请检查环境变量",
            index.display()
        );
    }

    let site = Arc::new(Site {
        bucket: require_bucket(),
        endpoint: require_endpoint(),
        client: Client::builder(TokioExecutor::new()).build_http(),
        // history 路由要和原先 nginx try_files 一样返回 200。not_found_service 会改成 404。
        files: ServeDir::new(&root).fallback(ServeFile::new(index)),
    });

    // 前端请求 /api，而路由本身没有这个前缀。先 nest，再和静态页合并，body 限制也只对页面和对象存储反代关掉。
    mount(api, site)
}

fn mount(api: Router, site: Arc<Site>) -> Router {
    let api = Router::new().nest("/api", api);
    let pages = Router::new()
        .fallback(dispatch)
        .layer(DefaultBodyLimit::disable())
        .with_state(site);
    api.merge(pages)
}

fn require_bucket() -> String {
    let bucket = std::env::var("S3_BUCKET")
        .unwrap_or_else(|_| panic!("缺少S3_BUCKET,请检查项目根目录下的.env文件"));
    if bucket.is_empty() || bucket.contains('/') || bucket == "." || bucket == ".." {
        panic!("S3_BUCKET无效,请检查项目根目录下的.env文件");
    }
    bucket
}

fn require_endpoint() -> String {
    let raw = std::env::var("S3_ENDPOINT")
        .unwrap_or_else(|_| panic!("缺少S3_ENDPOINT,请检查项目根目录下的.env文件"));
    let endpoint = raw.trim().trim_end_matches('/').to_string();
    let Ok(uri) = endpoint.parse::<Uri>() else {
        panic!("S3_ENDPOINT必须是http://主机:端口,请检查项目根目录下的.env文件");
    };
    let path = uri.path();
    if !matches!(uri.scheme_str(), Some("http") | Some("https"))
        || uri.authority().is_none()
        || !(path.is_empty() || path == "/")
        || uri.query().is_some()
    {
        panic!("S3_ENDPOINT必须是http://主机:端口,请检查项目根目录下的.env文件");
    }
    endpoint
}

async fn dispatch(State(site): State<Arc<Site>>, req: Request) -> Response {
    if is_bucket_path(req.uri().path(), &site.bucket) {
        proxy(&site, req).await
    } else {
        serve_files(&site, req).await
    }
}

async fn serve_files(site: &Site, req: Request) -> Response {
    match site.files.clone().oneshot(req).await {
        Ok(response) => response.into_response(),
        Err(err) => match err {},
    }
}

async fn proxy(site: &Site, req: Request) -> Response {
    let path = req
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    let target = format!("{}{path}", site.endpoint);
    let Ok(uri) = target.parse::<Uri>() else {
        tracing::error!(target, "s3 proxy uri");
        return StatusCode::BAD_GATEWAY.into_response();
    };

    let (mut parts, body) = req.into_parts();
    // 预签名校验浏览器发出的 Host，不能改成 S3_ENDPOINT 的主机名。
    parts.uri = uri;
    parts.version = Version::HTTP_11;
    strip_hop_by_hop(&mut parts.headers);

    match site.client.request(Request::from_parts(parts, body)).await {
        Ok(response) => {
            let (mut parts, body) = response.into_parts();
            strip_hop_by_hop(&mut parts.headers);
            Response::from_parts(parts, Body::new(body))
        }
        Err(error) => {
            tracing::error!(%error, "s3 proxy");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

fn is_bucket_path(path: &str, bucket: &str) -> bool {
    let Some(rest) = path.strip_prefix('/') else {
        return false;
    };
    if bucket.is_empty() {
        return false;
    }
    match rest.split_once('/') {
        Some((name, _)) => name == bucket,
        None => rest == bucket,
    }
}

fn strip_hop_by_hop(headers: &mut HeaderMap) {
    let listed: Vec<HeaderName> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| HeaderName::try_from(value).ok())
        .collect();

    for name in [
        header::CONNECTION,
        header::PROXY_AUTHENTICATE,
        header::PROXY_AUTHORIZATION,
        header::TE,
        header::TRAILER,
        header::TRANSFER_ENCODING,
        header::UPGRADE,
        HeaderName::from_static("keep-alive"),
        HeaderName::from_static("proxy-connection"),
    ] {
        headers.remove(name);
    }
    for name in listed {
        headers.remove(name);
    }
}

#[cfg(test)]
mod tests {
    use super::is_bucket_path;
    use super::{Client, ServeDir, ServeFile, Site, TokioExecutor};
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::routing::{get, post};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[test]
    fn bucket_root_and_key() {
        assert!(is_bucket_path("/kingei", "kingei"));
        assert!(is_bucket_path("/kingei/", "kingei"));
        assert!(is_bucket_path("/kingei/a/b", "kingei"));
    }

    #[test]
    fn other_paths() {
        assert!(!is_bucket_path("/", "kingei"));
        assert!(!is_bucket_path("/api/health", "kingei"));
        assert!(!is_bucket_path("/kingei-extra", "kingei"));
        assert!(!is_bucket_path("/Kingei/a", "kingei"));
        assert!(!is_bucket_path("/other/kingei", "kingei"));
        assert!(!is_bucket_path("kingei", "kingei"));
        assert!(!is_bucket_path("/kingei", ""));
    }

    #[tokio::test]
    async fn api_post_is_not_static_method_not_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let index = dir.path().join("index.html");
        std::fs::write(&index, b"home").unwrap();
        let site = Arc::new(Site {
            bucket: "kingei".to_string(),
            endpoint: "http://127.0.0.1:9".to_string(),
            client: Client::builder(TokioExecutor::new()).build_http(),
            files: ServeDir::new(dir.path()).fallback(ServeFile::new(index)),
        });
        let api = Router::new()
            .route("/health", get(|| async { StatusCode::NO_CONTENT }))
            .route("/user/login", post(|| async { StatusCode::UNAUTHORIZED }));
        let app = super::mount(api, site);

        let health = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::NO_CONTENT);

        let login = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/user/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(login.status(), StatusCode::UNAUTHORIZED);

        let page = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::OK);
        assert_eq!(
            page.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html"
        );
    }
}
