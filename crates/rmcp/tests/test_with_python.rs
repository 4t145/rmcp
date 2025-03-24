// use rmcp::transport::sse_server::SseServer;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// mod common;
// use common::calculator::Calculator;

// const BIND_ADDRESS: &str = "127.0.0.1:8000";

// #[tokio::test]
// async fn test_with_python() -> anyhow::Result<()> {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env()
//                 .unwrap_or_else(|_| "debug".to_string().into()),
//         )
//         .with(tracing_subscriber::fmt::layer())
//         .init();
//     tokio::process::Command::new("uv")
//         .arg("run")
//         .current_dir("tests/test_with_python")
//         .spawn()?
//         .wait()
//         .await?;

//     let ct = SseServer::serve(BIND_ADDRESS.parse()?)
//         .await?
//         .with_mcp(Calculator::default);

//     tokio::process::Command::new("node")
//         .arg("tests/test_with_js/index.js")
//         .spawn()?
//         .wait()
//         .await?;
//     ct.cancel();
//     Ok(())
// }
