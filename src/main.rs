//! # OpenAI Azure Proxy
//!
//! `openai-azure-proxy` is a binary that runs a proxy to make any OpenAI-compatible API (text
//! generation only at the moment) compatible with Azure AI I/O schemas.
//!
//! ## Installation
//!
//! ```
//! cargo install --path .
//! ```
//!
//! ## Usage
//!
//! First you need to have access to a running API with an OpenAI-compatible interface i.e. with
//! the `v1/chat/completions` endpoint available. If deployed locally or within the same instance
//! as the proxy, it should be deployed on a port different than the port 80, which is reserved for
//! the proxy.
//!
//! ```
//! openai-azure-proxy --upstream-host ... --upstream-port ...
//! ```

mod errors;
mod handlers;
mod proxy;
mod utils;

use proxy::start_server;

// Entrypoint for the binary, that runs the Axum proxy
#[tokio::main]
async fn main() {
    start_server().await;
}
