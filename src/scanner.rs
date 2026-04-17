use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::{info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::db::{self, MediaFile};
use crate::error::{ReelcastError, Result};

/// File extensions we consider media
const MEDIA_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "m4v", "wmv", "flv", "webm", "ts", "m2ts",
];

pub async fn scan_library(pool: &SqlitePool, roots: &[impl AsRef<Path>]) -> Result<usize> {
    let mut count = 0usize;

    for root in roots {
        let root = root.as_ref();
        info!("Scanning library root: {}", root.display());

        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            let Some(ext) = ext else { continue };
            if !MEDIA_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }

            match probe_and_upsert(pool, path).await {
                Ok(()) => count += 1,
                Err(e) => warn!("Skipping {}: {}", path.display(), e),
            }
        }
    }

    let removed = db::delete_missing(pool).await?;
    if removed > 0 {
        info!("Pruned {} missing files from DB", removed);
    }

    info!("Scan complete: {} files indexed", count);
    Ok(count)
}

async fn probe_and_upsert(pool: &SqlitePool, path: &Path) -> Result<()> {
    let meta = std::fs::metadata(path)?;
    let size_bytes = meta.len() as i64;

    let probe = ffprobe(path)?;

    let title = title_from_path(path);
    let year = year_from_title(&title);

    let video = probe
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"));
    let audio = probe
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"));

    let duration_secs = probe
        .format
        .duration
        .as_deref()
        .and_then(|d| d.parse::<f64>().ok())
        .map(|d| d as i64);

    let container = probe
        .format
        .format_name
        .clone()
        .map(|s| s.split(',').next().unwrap_or(&s).to_string());

    let m = MediaFile {
        id: stable_id(path),
        path: path.to_string_lossy().into_owned(),
        title,
        year,
        duration_secs,
        size_bytes,
        container,
        video_codec: video.and_then(|s| s.codec_name.clone()),
        audio_codec: audio.and_then(|s| s.codec_name.clone()),
        width: video.and_then(|s| s.width),
        height: video.and_then(|s| s.height),
    };

    db::upsert(pool, &m).await
}

/// Deterministic ID from path — stable across rescans
fn stable_id(path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    // Format as a UUID-shaped string for aesthetics
    let n = h.finish();
    Uuid::from_u64_pair(n, n.wrapping_mul(0x9e3779b97f4a7c15)).to_string()
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .replace(['.', '_', '-'], " ")
        .trim()
        .to_string()
}

fn year_from_title(title: &str) -> Option<i64> {
    // Look for a 4-digit year like (2001) or .2001. in the title
    let re = regex_lite::Regex::new(r"\b(19\d{2}|20\d{2})\b").ok()?;
    re.find(title)?.as_str().parse().ok()
}

// ── ffprobe types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    format_name: Option<String>,
}

fn ffprobe(path: &Path) -> Result<FfprobeOutput> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            "-show_format",
        ])
        .arg(path)
        .output()
        .map_err(|e| ReelcastError::Ffprobe {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReelcastError::Ffprobe {
            path: path.display().to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|e| ReelcastError::Ffprobe {
        path: path.display().to_string(),
        reason: e.to_string(),
    })
}
