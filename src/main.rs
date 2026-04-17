mod cache;
mod qr;
mod sessions;
mod ui;
mod config;
mod db;
mod error;
mod net;
mod scanner;
mod serve;
mod vlc;

use std::sync::Arc;

use clap::Parser;
use tracing::info;

use cache::MediaCache;
use config::Config;
use serve::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Config::parse();
    config.resolve_tls();

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("Reelcast v{}", env!("CARGO_PKG_VERSION"));

    let pool = db::open(&config.db).await?;

    // Populate cache immediately from DB
    let media_cache = MediaCache::new();
    let existing = db::get_all(&pool).await?;
    let preloaded = existing.len();
    media_cache.set(existing).await;
    info!("Loaded {} files from DB (serving immediately)", preloaded);

    // Background scan
    {
        let pool = pool.clone();
        let cache = media_cache.clone();
        let library = config.library.clone();
        tokio::spawn(async move {
            info!("Background scan started");
            match scanner::scan_library(&pool, &library).await {
                Ok(count) => match db::get_all(&pool).await {
                    Ok(files) => {
                        cache.set(files).await;
                        info!("Background scan complete — {} files indexed", count);
                    }
                    Err(e) => tracing::error!("Cache refresh failed: {}", e),
                },
                Err(e) => tracing::error!("Background scan failed: {}", e),
            }
        });
    }

    let state = AppState {
        pool,
        media_cache,
        config: Arc::new(config.clone()),
        sessions: sessions::SessionRegistry::new(),
    };

    log_ui_info();
    print_urls(&config);

    if config.tls_enabled() {
        // HTTPS on main port, plain HTTP on media_port (for VLC)
        let https_addr: std::net::SocketAddr =
            format!("{}:{}", config.host, config.port).parse()?;
        let http_addr: std::net::SocketAddr =
            format!("{}:{}", config.host, config.media_port).parse()?;

        // HTTP listener for VLC file access — files + playlists only
        let http_state = state.clone();
        tokio::spawn(async move {
            info!("HTTP media port on http://{}", http_addr);
            let app = serve::file_router(http_state);
            let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        let app = serve::router(state).into_make_service_with_connect_info::<std::net::SocketAddr>();
        tls::serve_svc(app, https_addr, config).await?;
    } else {
        let addr: std::net::SocketAddr =
            format!("{}:{}", config.host, config.port).parse()?;
        info!("Listening on http://{}", addr);
        axum::serve(
            tokio::net::TcpListener::bind(addr).await?,
            serve::router(state).into_make_service_with_connect_info::<std::net::SocketAddr>()
        ).await?;
    }

    Ok(())
}

fn log_ui_info() {


    let feature = if cfg!(feature = "svelte") { "svelte" } else { "vanilla" };
    let files: Vec<_> = crate::ui::Assets::iter().collect();
    info!("UI: {} ({} embedded files)", feature, files.len());
    for f in &files {
        tracing::debug!("  embedded: {}", f);
    }
    if files.is_empty() {
        tracing::warn!("⚠️  No UI files embedded — dist/ may be empty. Run: cargo build --features svelte");
    }
}

fn print_urls(config: &Config) {
    let ip = crate::net::local_ip().map(|ip| ip.to_string()).unwrap_or_else(|| config.host.clone());
    let control_url = format!("{}://{}:{}/control", config.scheme(), ip, config.port);
    info!("  Remote control: {}", control_url);
    // Print compact QR to terminal
    let qr_str = print_qr_terminal(&control_url);
    for line in qr_str.lines() {
        info!("{}", line);
    }
}

fn print_qr_terminal(url: &str) -> String {
    use qrcode::{QrCode, EcLevel};
    use qrcode::render::unicode;
    QrCode::with_error_correction_level(url, EcLevel::M)
        .or_else(|_| QrCode::new(url))
        .map(|code| code.render::<unicode::Dense1x2>()
            .quiet_zone(true)
            .build())
        .unwrap_or_default()
}

fn _old_print_urls(config: &Config) {
    let scheme = config.scheme();
    let ip = net::local_ip().map(|ip| ip.to_string()).unwrap_or_else(|| config.host.clone());
    info!("  → {}://{}:{}", scheme, ip, config.port);
    info!("  → {}://localhost:{}", scheme, config.port);
    if config.tls_enabled() {
        info!("  (VLC media port: http://{}:{})", ip, config.media_port);
    } else {
        info!("  (run scripts/gencert to enable HTTPS)");
    }
}

mod tls {
    use std::fs::File;
    use std::io::BufReader;
    use std::net::SocketAddr;
    use std::sync::Arc;

    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
    use tracing::info;

    use crate::config::Config;

    pub async fn serve_svc(
        app: axum::routing::IntoMakeServiceWithConnectInfo<axum::Router, std::net::SocketAddr>,
        addr: SocketAddr,
        config: Config,
    ) -> anyhow::Result<()> {
        let cert_path = config.cert.as_ref().unwrap();
        let key_path  = config.key.as_ref().unwrap();

        let certs: Vec<Certificate> = rustls_pemfile::certs(
            &mut BufReader::new(File::open(cert_path)?)
        )?.into_iter().map(Certificate).collect();

        let key: PrivateKey = rustls_pemfile::pkcs8_private_keys(
            &mut BufReader::new(File::open(key_path)?)
        )?.into_iter().next()
            .map(PrivateKey)
            .ok_or_else(|| anyhow::anyhow!("no private key in {}", key_path.display()))?;

        let tls_cfg = Arc::new(
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)?
        );
        let acceptor = TlsAcceptor::from(tls_cfg);
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on https://{}", addr);

        loop {
            let (tcp, _peer) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let app = app.clone();
            let mut make_svc = app.clone();
            tokio::spawn(async move {
                let Ok(tls) = acceptor.accept(tcp).await else { return };
                use tower::Service;
                let router = make_svc.call(peer).await.unwrap();
                let io = hyper_util::rt::TokioIo::new(tls);
                let svc = hyper::service::service_fn(move |req| {
                    let mut r = router.clone();
                    async move { tower::Service::call(&mut r, req).await }
                });
                let _ = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new()
                )
                .serve_connection(io, svc)
                .await;
            });
        }
    }
}
