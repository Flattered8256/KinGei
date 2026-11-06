use std::path::PathBuf;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

pub fn with_frontend(api: Router, static_dir: String) -> Router {
    let root = PathBuf::from(static_dir.trim());
    let index = root.join("index.html");
    if !index.is_file() {
        panic!(
            "STATIC_DIR缺少index.html: {},请检查环境变量",
            index.display()
        );
    }

    // history 路由要和原先 nginx try_files 一样返回 200。not_found_service 会改成 404。
    let files = ServeDir::new(&root).fallback(ServeFile::new(index));
    Router::new().nest("/api", api).fallback_service(files)
}

#[cfg(test)]
mod tests {
    use super::with_frontend;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::routing::get;
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_api_and_index_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let index = dir.path().join("index.html");
        std::fs::write(&index, b"home").unwrap();

        let api = Router::new().route("/health", get(|| async { StatusCode::NO_CONTENT }));
        let app = with_frontend(api, dir.path().to_string_lossy().into_owned());

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

        let page = app
            .oneshot(
                Request::builder()
                    .uri("/chat/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::OK);
        assert_eq!(
            page.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html"
        );
    }
}
