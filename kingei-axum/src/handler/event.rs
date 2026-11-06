use std::sync::Arc;
use std::time::Duration;

use axum::{
    Extension,
    extract::State,
    http::header::{CACHE_CONTROL, HeaderName, HeaderValue},
    response::{
        IntoResponse,
        sse::{KeepAlive, Sse},
    },
};
use uuid::Uuid;

use crate::state::AppState;

pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> impl IntoResponse {
    let stream = state.event_hub.subscribe(user_id).into_sse_stream();
    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    );

    (
        [
            (CACHE_CONTROL, HeaderValue::from_static("no-cache")),
            (
                HeaderName::from_static("x-accel-buffering"),
                HeaderValue::from_static("no"),
            ),
        ],
        sse,
    )
}
