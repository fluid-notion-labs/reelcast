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

    /// Scan libraries on startup
    #[arg(long, default_value_t = true)]
    pub scan_on_start: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,

    /// HTTP port for VLC media access (used when TLS is enabled)
    #[arg(long, default_value = "3001", env = "REELCAST_MEDIA_PORT")]
    pub media_port: u16,

    /// TLS certificate (PEM) — enables HTTPS when provided
    #[arg(long, env = "REELCAST_CERT")]
    pub cert: Option<PathBuf>,

    /// TLS private key (PEM) — required when --cert is set
    #[arg(long, env = "REELCAST_KEY")]
    pub key: Option<PathBuf>,
}

impl Config {
    /// Resolve cert/key — explicit flags win, then auto-detect ~/.config/reelcast/
    pub fn resolve_tls(&mut self) {
        if self.cert.is_none() && self.key.is_none() {
            let base = dirs_next::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
                .join("reelcast");
            let cert = base.join("cert.pem");
            let key  = base.join("key.pem");
            if cert.exists() && key.exists() {
                self.cert = Some(cert);
                self.key  = Some(key);
            }
        }
    }

    pub fn tls_enabled(&self) -> bool {
        self.cert.is_some() && self.key.is_some()
    }

    pub fn scheme(&self) -> &'static str {
        if self.tls_enabled() { "https" } else { "http" }
    }
}
