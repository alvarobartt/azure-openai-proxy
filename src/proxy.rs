use crate::handlers::{
    chat_completions::chat_completions_handler, health::health_handler, info::info_handler,
};
use axum::{
    body::Body,
    http::Uri,
    routing::{get, post},
    Router,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use tokio::signal;

/// Custom type for the Hyper HTTP Client that will be used / shared as the application state
pub type HttpClient = Client<HttpConnector, Body>;

/// Custom API state to be shared across all the proxy endpoints
#[derive(Debug, Clone)]
pub struct ProxyState {
    pub client: HttpClient,
    pub uri: Uri,
}

/// Starts the Axum server i.e. the proxy
pub async fn start_server(
    host: Option<&str>,
    port: Option<&u16>,
    upstream_host: &str,
    upstream_port: Option<&u16>,
) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .compact()
        .init();

    let client: HttpClient = Client::builder(TokioExecutor::new()).build(HttpConnector::new());

    let uri: Uri = {
        let port_str = upstream_port.map(|p| format!(":{p}")).unwrap_or_default();
        let full_uri = format!("{}{}", upstream_host, port_str);
        Uri::try_from(full_uri).unwrap()
    };

    let state = ProxyState { client, uri };

    // TODO(env): use env to control which routes should be exposed
    // TODO: add periodic health checks to the underlying service to terminate the proxy if the
    // underlying service is down
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/chat/completions", post(chat_completions_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        host.unwrap_or("0.0.0.0"),
        port.unwrap_or(&80u16),
    ))
    .await
    .unwrap();

    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Handles the shutdown signal for the Axum application for a graceful shutdown
///
/// Reference: https://github.com/tokio-rs/axum/tree/main/examples/graceful-shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
