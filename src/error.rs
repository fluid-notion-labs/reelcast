use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReelcastError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Media file not found: {id}")]
    MediaNotFound { id: String },

    #[error("FFprobe failed for {path}: {reason}")]
    Ffprobe { path: String, reason: String },

    #[error("Scanner error: {0}")]
    Scanner(String),
}

impl axum::response::IntoResponse for ReelcastError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::Json;

        let (status, message) = match &self {
            ReelcastError::MediaNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        tracing::error!("{}", message);
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub type Result<T, E = ReelcastError> = std::result::Result<T, E>;
