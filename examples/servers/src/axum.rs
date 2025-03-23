use rmcp::{ServiceExt, transport::sse_server::SseServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tracing_subscriber::{self};
mod common;
use common::counter::Counter;

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ct = SseServer::serve(BIND_ADDRESS.parse()?)
        .await?
        .with_mcp(Counter::new);

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}
