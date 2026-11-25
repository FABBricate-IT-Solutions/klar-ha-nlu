//! JSON error bodies for the operator UI (`{ "error", "path" }`).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// An HTTP failure the operator UI can show by name and path.
#[derive(Debug, Clone)]
pub struct HttpError {
    pub status: StatusCode,
    pub error: String,
    pub path: String,
}

impl HttpError {
    pub fn new(status: StatusCode, error: impl Into<String>) -> Self {
        Self { status, error: error.into(), path: String::new() }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Overlay write failed after validation — the running state was not changed.
    pub fn save_failed(err: std::io::Error) -> Self {
        tracing::error!("klar_nlu.json nicht gespeichert: {err}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, format!("could not save: {err}")).with_path("klar_nlu.json")
    }
}

impl From<StatusCode> for HttpError {
    fn from(status: StatusCode) -> Self {
        Self::new(status, status.canonical_reason().unwrap_or("error"))
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.error, "path": self.path }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn body_is_error_and_path() {
        let response = HttpError::new(StatusCode::BAD_REQUEST, "invalid policy id").with_path("policies").into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        let row: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(row["error"], "invalid policy id");
        assert_eq!(row["path"], "policies");
    }
}
