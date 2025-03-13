use anyhow::Result;
use rmcp::{
    model::CallToolRequestParam, serve_client, transport::child_process::child_process,
    ClientHandlerService,
};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
mod common;
use common::simple_client::SimpleClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let (mut process, transport) = child_process(
        tokio::process::Command::new("uvx")
            .arg("mcp-server-git")
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?,
    )?;

    let service = serve_client(
        ClientHandlerService::new(SimpleClient::default()),
        transport,
    )
    .await
    .inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;

    // Initialize
    let server_info = service.peer().info();
    tracing::info!("Connected to server: {server_info:#?}");

    // List tools
    let tools = service.peer().list_tools(Default::default()).await?;
    tracing::info!("Available tools: {tools:#?}");

    // Call tool 'git_status' with arguments = {"repo_path": "."}
    let tool_result = service
        .peer()
        .call_tool(CallToolRequestParam {
            name: "git_status".into(),
            arguments: serde_json::json!({ "repo_path": "." }).as_object().cloned(),
        })
        .await?;
    tracing::info!("Tool result: {tool_result:#?}");
    service.cancel().await?;
    process.kill().await?;
    Ok(())
}
