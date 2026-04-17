use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;
use tower_http::{cors::CorsLayer, services::ServeFile, trace::TraceLayer};

use crate::{
    config::Config,
    db::{self, MediaFile},
    error::{ReelcastError, Result},
    vlc,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Arc<Config>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/media", get(list_media))
        .route("/search", get(search))
        .route("/recent", get(recent_played))
        .route("/play/:id", get(play_xspf))
        .route("/playlist/:id", get(play_m3u))
        .route("/file/:id", get(serve_file))
        .route("/health", get(health))
        .route("/setup", get(setup_page))
        .route("/cert", get(serve_cert))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn index() -> impl IntoResponse {
    axum::response::Html(include_str!("index.html"))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

#[derive(Serialize)]
struct MediaItem {
    id: String,
    title: String,
    year: Option<i64>,
    duration_secs: Option<i64>,
    size_bytes: i64,
    container: Option<String>,
    resolution: Option<String>,
    file_url: String,
    play_url: String,
    playlist_url: String,
}

impl MediaItem {
    fn from_media(m: MediaFile, base_url: &str) -> Self {
        let resolution = match (m.width, m.height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        };
        Self {
            file_url: format!("{}/file/{}", base_url, m.id),
            play_url: format!("{}/play/{}", base_url, m.id),
            playlist_url: format!("{}/playlist/{}", base_url, m.id),
            id: m.id,
            title: m.title,
            year: m.year,
            duration_secs: m.duration_secs,
            size_bytes: m.size_bytes,
            container: m.container,
            resolution,
        }
    }
}

#[derive(Serialize)]
struct RecentItem {
    media_id: String,
    title: String,
    played_at: i64,
    file_url: String,
    play_url: String,
}

async fn list_media(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let base_url = base_url(&state.config);
    let items: Vec<_> = db::get_all(&state.pool).await?
        .into_iter().map(|m| MediaItem::from_media(m, &base_url)).collect();
    Ok(Json(items))
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse> {
    let base_url = base_url(&state.config);
    let media = match params.q.as_deref().filter(|q| !q.is_empty()) {
        Some(q) => db::search_by_title(&state.pool, q).await?,
        None    => db::get_all(&state.pool).await?,
    };
    let items: Vec<_> = media.into_iter().map(|m| MediaItem::from_media(m, &base_url)).collect();
    Ok(Json(items))
}

async fn recent_played(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let base_url = base_url(&state.config);
    let rows = db::recent_plays(&state.pool, 20).await?;
    let items: Vec<_> = rows.into_iter().map(|r| RecentItem {
        file_url: format!("{}/file/{}", base_url, r.media_id),
        play_url: format!("{}/play/{}", base_url, r.media_id),
        media_id: r.media_id,
        title: r.title,
        played_at: r.played_at,
    }).collect();
    Ok(Json(items))
}

async fn play_xspf(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response> {
    let m = get_media_or_404(&state.pool, &id).await?;
    // Record play
    let _ = db::record_play(&state.pool, &m.id, &m.title).await;
    let base_url = base_url(&state.config);
    let disposition = format!("attachment; filename=\"{}.xspf\"", m.title);
    let playlist = vlc::xspf(&[m], &base_url);
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "application/xspf+xml"),
            (axum::http::header::CONTENT_DISPOSITION, disposition.as_str()),
        ],
        playlist,
    ).into_response())
}

async fn play_m3u(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response> {
    let m = get_media_or_404(&state.pool, &id).await?;
    let _ = db::record_play(&state.pool, &m.id, &m.title).await;
    let base_url = base_url(&state.config);
    let disposition = format!("attachment; filename=\"{}.m3u\"", m.title);
    let playlist = vlc::m3u_single(&m, &base_url);
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "audio/x-mpegurl"),
            (axum::http::header::CONTENT_DISPOSITION, disposition.as_str()),
        ],
        playlist,
    ).into_response())
}

async fn serve_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response> {
    let m = get_media_or_404(&state.pool, &id).await?;
    let path = std::path::PathBuf::from(&m.path);
    if !path.exists() {
        return Err(ReelcastError::MediaNotFound { id });
    }
    let mut req = axum::http::Request::builder()
        .body(axum::body::Body::empty())
        .unwrap();
    *req.headers_mut() = headers;
    ServeFile::new(&path)
        .oneshot(req)
        .await
        .map(|r| r.map(axum::body::Body::new).into_response())
        .map_err(|e| ReelcastError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
}

async fn setup_page() -> impl IntoResponse {
    axum::response::Html(include_str!("setup.html"))
}

async fn serve_cert(State(state): State<AppState>) -> Result<Response> {
    let cert_path = state.config.cert.as_ref()
        .ok_or_else(|| ReelcastError::Scanner("No cert configured".into()))?;
    let bytes = tokio::fs::read(cert_path).await?;
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "application/x-pem-file"),
            (axum::http::header::CONTENT_DISPOSITION, "attachment; filename="reelcast.pem""),
        ],
        bytes,
    ).into_response())
}

async fn get_media_or_404(pool: &SqlitePool, id: &str) -> Result<MediaFile> {
    db::get_by_id(pool, id)
        .await?
        .ok_or_else(|| ReelcastError::MediaNotFound { id: id.to_string() })
}

fn base_url(config: &Config) -> String {
    let host = crate::net::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| config.host.clone());
    format!("{}://{}:{}", config.scheme(), host, config.port)
}
