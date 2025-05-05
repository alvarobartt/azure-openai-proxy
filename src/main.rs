use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::post,
    Router,
};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type HttpClient = hyper_util::client::legacy::Client<HttpConnector, Body>;

mod errors;
use errors::AzureError;

const API_VERSIONS: &[&str] = &["2024-05-01-preview", "2025-04-01"];

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let client: HttpClient =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    let app = Router::new()
        .route("/chat/completions", post(chat_completions_handler))
        .with_state(client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    tracing::info!(
        target: "openai-azure-proxy",
        "Listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn chat_completions_handler(
    State(client): State<HttpClient>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, AzureError> {
    let version = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("api-version="))
                .and_then(|p| p.split('=').nth(1))
        })
        .ok_or(AzureError::MissingApiVersionParameter)?;

    if !API_VERSIONS.contains(&version) {
        return Err(AzureError::UnsupportedApiVersionValue(
            version.to_string(),
            API_VERSIONS.join(", ").into(),
        ));
    }

    *req.uri_mut() = Uri::try_from(format!(
        "http://{}:{}/v1/chat/completions",
        std::env::var("UPSTREAM_HOST").unwrap_or_else(|_| "localhost".into()),
        std::env::var("UPSTREAM_PORT").unwrap_or_else(|_| "8080".into()),
    ))
    .unwrap();

    tracing::info!(
        target: "openai-azure-proxy",
        "Proxying {} request to {}",
        req.method(),
        req.uri()
    );

    client
        .request(req)
        .await
        .map_err(|e| AzureError::Upstream(StatusCode::BAD_GATEWAY, e.to_string()))
        .map(|res| res.into_response())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
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
