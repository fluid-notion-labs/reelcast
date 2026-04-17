use std::path::Path;

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::error::Result;

/// A scanned media file record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaFile {
    pub id: String,
    pub path: String,
    pub title: String,
    pub year: Option<i64>,
    pub duration_secs: Option<i64>,
    pub size_bytes: i64,
    pub container: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlayHistory {
    pub media_id: String,
    pub title: String,
    pub played_at: i64, // unix timestamp
}

pub async fn open(db_path: &Path) -> Result<SqlitePool> {
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await?;
    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS media_files (
            id            TEXT PRIMARY KEY,
            path          TEXT NOT NULL UNIQUE,
            title         TEXT NOT NULL,
            year          INTEGER,
            duration_secs INTEGER,
            size_bytes    INTEGER NOT NULL,
            container     TEXT,
            video_codec   TEXT,
            audio_codec   TEXT,
            width         INTEGER,
            height        INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_media_title ON media_files(title);

        CREATE TABLE IF NOT EXISTS play_history (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            media_id  TEXT NOT NULL,
            title     TEXT NOT NULL,
            played_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_play_history_time ON play_history(played_at DESC);
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert(pool: &SqlitePool, m: &MediaFile) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO media_files
            (id, path, title, year, duration_secs, size_bytes, container, video_codec, audio_codec, width, height)
        VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
            title         = excluded.title,
            year          = excluded.year,
            duration_secs = excluded.duration_secs,
            size_bytes    = excluded.size_bytes,
            container     = excluded.container,
            video_codec   = excluded.video_codec,
            audio_codec   = excluded.audio_codec,
            width         = excluded.width,
            height        = excluded.height
        "#,
    )
    .bind(&m.id)
    .bind(&m.path)
    .bind(&m.title)
    .bind(m.year)
    .bind(m.duration_secs)
    .bind(m.size_bytes)
    .bind(&m.container)
    .bind(&m.video_codec)
    .bind(&m.audio_codec)
    .bind(m.width)
    .bind(m.height)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<MediaFile>> {
    Ok(sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<MediaFile>> {
    Ok(sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY title")
        .fetch_all(pool)
        .await?)
}

pub async fn search_by_title(pool: &SqlitePool, q: &str) -> Result<Vec<MediaFile>> {
    let pattern = format!("%{}%", q);
    Ok(sqlx::query_as::<_, MediaFile>(
        "SELECT * FROM media_files WHERE title LIKE ? ORDER BY title LIMIT 50",
    )
    .bind(pattern)
    .fetch_all(pool)
    .await?)
}

pub async fn record_play(pool: &SqlitePool, media_id: &str, title: &str) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    sqlx::query(
        "INSERT INTO play_history (media_id, title, played_at) VALUES (?, ?, ?)",
    )
    .bind(media_id)
    .bind(title)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn recent_plays(pool: &SqlitePool, limit: i64) -> Result<Vec<PlayHistory>> {
    // Distinct by media_id, most recent play per title
    Ok(sqlx::query_as::<_, PlayHistory>(
        r#"
        SELECT media_id, title, MAX(played_at) as played_at
        FROM play_history
        GROUP BY media_id
        ORDER BY played_at DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn delete_missing(pool: &SqlitePool) -> Result<u64> {
    let all = get_all(pool).await?;
    let mut removed = 0u64;
    for m in all {
        if !std::path::Path::new(&m.path).exists() {
            sqlx::query("DELETE FROM media_files WHERE id = ?")
                .bind(&m.id)
                .execute(pool)
                .await?;
            removed += 1;
        }
    }
    Ok(removed)
}
