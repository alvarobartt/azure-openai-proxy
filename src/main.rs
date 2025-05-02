use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::post,
    Router,
};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

type HttpClient = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    let client: HttpClient =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    let app = Router::new()
        .route("/chat/completions", post(handler))
        .with_state(client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
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

        *req.uri_mut() = Uri::try_from(new_uri).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)
        .map(|res| res.into_response())
}
