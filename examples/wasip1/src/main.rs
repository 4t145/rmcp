use anyhow::Result;
use rmcp::{ServerHandlerService, serve_server, transport::io::async_rw};

use tracing_subscriber::{self, EnvFilter};
mod common;
use rmcp::transport::io::wasip1::{stdin, stdout};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our counter router
    let service = ServerHandlerService::new(common::counter::Counter::new());
    let transport = async_rw(stdin(), stdout());
    let service = serve_server(service, transport).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}