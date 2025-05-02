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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=info,tower_http=debug,axum=trace",
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
        .route("/chat/completions", post(handler))
        .with_state(client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    tracing::info!(
        target: "oaiaz-proxy",
        "Listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handler(
    State(client): State<HttpClient>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, StatusCode> {
    let requires_rewrite = req.uri().path() == "/chat/completions"
        && req
            .uri()
            .query()
            .map(|q| q.contains("api-version"))
            .unwrap_or(false);

    if requires_rewrite {
        *req.uri_mut() =
            Uri::try_from("http:://0.0.0.0:8080/v1/chat/completions").map_err(|_| {
                tracing::info!(target: "oaiaz-proxy", "URI rewrite failed for: {}", req.uri());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    tracing::info!(
        target: "oaiaz-proxy",
        "Proxying {} request to {}",
        req.method(),
        req.uri()
    );

    client
        .request(req)
        .await
        .map_err(|e| {
            tracing::info!(target: "oaiaz-proxy", "Proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
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
