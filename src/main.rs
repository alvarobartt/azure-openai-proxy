//! # OpenAI Azure Proxy
//!
//! `openai-azure-proxy` is a binary that runs a proxy to make any OpenAI-compatible API (text
//! generation only at the moment) compatible with Azure AI Model Inference API schemas.
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
//! openai-azure-proxy --host 0.0.0.0 --port 80 --upstream-host 0.0.0.0 --upstream-port 8080
//! ```

use clap::Parser;

mod errors;
mod handlers;
mod proxy;
mod utils;

use proxy::start_server;

#[derive(Parser)]
#[command(name = "openai-azure-proxy", version, about)]
struct Cli {
    #[arg(short, long, env, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, env, default_value = "80")]
    port: u16,

    #[arg(short, long, env, default_value = "0.0.0.0")]
    upstream_host: String,

    #[arg(short, long, env, default_value = "8080")]
    upstream_port: u16,
}

/// Entrypoint for the binary, that runs the Axum proxy
#[tokio::main]
async fn main() {
    let args = Cli::parse();
    start_server(
        Some(&args.host),
        Some(&args.port),
        &args.upstream_host,
        Some(&args.upstream_port),
    )
    .await;
}
