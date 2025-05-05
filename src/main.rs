mod errors;
mod handlers;
mod proxy;

use proxy::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
}
