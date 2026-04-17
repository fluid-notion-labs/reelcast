use std::path::PathBuf;

use clap::Parser;

/// Reelcast — local media server for VLC
#[derive(Debug, Clone, Parser)]
#[command(name = "reelcast", version, about)]
pub struct Config {
    /// One or more directories to scan for media files
    #[arg(short, long, required = true, value_name = "DIR", env = "REELCAST_LIBRARY")]
    pub library: Vec<PathBuf>,

    /// Port to listen on
    #[arg(short, long, default_value = "3000", env = "REELCAST_PORT")]
    pub port: u16,

    /// Host/bind address
    #[arg(long, default_value = "0.0.0.0", env = "REELCAST_HOST")]
    pub host: String,

    /// SQLite database path
    #[arg(long, default_value = "reelcast.db", env = "REELCAST_DB")]
    pub db: PathBuf,

    /// Tantivy search index directory
    #[arg(long, default_value = "reelcast.idx", env = "REELCAST_INDEX")]
    pub index: PathBuf,

    /// Scan libraries on startup
    #[arg(long, default_value_t = true)]
    pub scan_on_start: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,
}
