mod config;
mod db;
mod error;
mod scanner;
mod serve;
mod vlc;

use std::sync::Arc;

use clap::Parser;
use tracing::info;

use config::Config;
use serve::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("Reelcast v{}", env!("CARGO_PKG_VERSION"));

    let pool = db::open(&config.db).await?;
    info!("Database: {}", config.db.display());

    if config.scan_on_start {
        let count = scanner::scan_library(&pool, &config.library).await?;
        info!("Indexed {} media files", count);
    }

    let addr: std::net::SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let state = AppState {
        pool,
        config: Arc::new(config.clone()),
    };

    let app = serve::router(state);

    if config.tls_enabled() {
        let cert = config.cert.as_ref().unwrap();
        let key  = config.key.as_ref().unwrap();
        let tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key).await?;
        info!("Listening on https://{}", addr);
        axum_server::bind_rustls(addr, tls)
            .serve(app.into_make_service())
            .await?;
    } else {
        info!("Listening on http://{} (no TLS — run scripts/plz tls to enable)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
