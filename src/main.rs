use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::post,
    Router,
};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

type HttpClient = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_filter(EnvFilter::from_default_env()),
        )
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
    axum::serve(listener, app).await.unwrap();
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
        let query = req.uri().query().unwrap_or_default();
        let new_uri = format!("http://0.0.0.0:8080/v1/chat/completions?{}", query);

        *req.uri_mut() = Uri::try_from(new_uri).map_err(|_| {
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
