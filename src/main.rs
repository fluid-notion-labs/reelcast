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

    if config.scan_on_start {
        let count = scanner::scan_library(&pool, &config.library).await?;
        info!("Indexed {} media files", count);
    }

    let addr: std::net::SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let state = AppState { pool, config: Arc::new(config.clone()) };
    let app = serve::router(state);

    if config.tls_enabled() {
        tls::serve(app, addr, config).await?;
    } else {
        info!("Listening on http://{} (run scripts/gencert to enable HTTPS)", addr);
        axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    }

    Ok(())
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

    pub async fn serve(app: axum::Router, addr: SocketAddr, config: Config) -> anyhow::Result<()> {
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
            tokio::spawn(async move {
                let Ok(tls) = acceptor.accept(tcp).await else { return };
                let io = hyper_util::rt::TokioIo::new(tls);
                let svc = hyper::service::service_fn(move |req| {
                    let mut app = app.clone();
                    async move {
                        use tower::Service;
                        app.call(req).await
                    }
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
