use anyhow::Result;
use common::counter::Counter;
use rmcp::{ServerHandlerService, serve_server};

use tracing_subscriber::{self, EnvFilter};
mod common;
/// npx @modelcontextprotocol/inspector cargo run -p mcp-server-examples --example tokio_std_io
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our counter router
    let service = serve_server(
        ServerHandlerService::new(Counter::new()),
        (tokio::io::stdin(), tokio::io::stdout()),
    )
    .await
    .inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
