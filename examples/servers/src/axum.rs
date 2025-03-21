use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::get,
};
use futures::{SinkExt, StreamExt, stream::Stream};
use rmcp::{ServiceExt, model::ClientJsonRpcMessage};
use std::collections::HashMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use anyhow::Result;
use std::sync::Arc;
use tokio::io::{self};
use tracing_subscriber::{self};
mod common;

type SessionId = Arc<str>;

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[derive(Clone, Default)]
pub struct App {
    txs: Arc<
        tokio::sync::RwLock<HashMap<SessionId, tokio::sync::mpsc::Sender<ClientJsonRpcMessage>>>,
    >,
}

impl App {
    pub fn new() -> Self {
        Self {
            txs: Default::default(),
        }
    }
    pub fn router(&self) -> Router {
        Router::new()
            .route("/sse", get(sse_handler).post(post_event_handler))
            .with_state(self.clone())
    }
}

fn session_id() -> SessionId {
    let id = format!("{:016x}", rand::random::<u128>());
    Arc::from(id)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEventQuery {
    pub session_id: String,
}

async fn post_event_handler(
    State(app): State<App>,
    Query(PostEventQuery { session_id }): Query<PostEventQuery>,
    Json(message): Json<ClientJsonRpcMessage>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!(session_id, ?message, "new client message");
    let tx = {
        let rg = app.txs.read().await;
        rg.get(session_id.as_str())
            .ok_or(StatusCode::NOT_FOUND)?
            .clone()
    };
    if tx.send(message).await.is_err() {
        tracing::error!("send message error");
        return Err(StatusCode::GONE);
    }
    Ok(StatusCode::ACCEPTED)
}

async fn sse_handler(State(app): State<App>) -> Sse<impl Stream<Item = Result<Event, io::Error>>> {
    // it's 4KB
    let session = session_id();
    tracing::info!(%session, "sse connection");
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::sync::PollSender;
    let (from_client_tx, from_client_rx) = tokio::sync::mpsc::channel(64);
    let (to_client_tx, to_client_rx) = tokio::sync::mpsc::channel(64);
    app.txs
        .write()
        .await
        .insert(session.clone(), from_client_tx);
    {
        let session = session.clone();
        tokio::spawn(async move {
            let service = common::counter::Counter::new();
            let stream = ReceiverStream::new(from_client_rx);
            let sink = PollSender::new(to_client_tx).sink_map_err(std::io::Error::other);
            let result = service.serve((sink, stream)).await.inspect_err(|e| {
                tracing::error!("serving error: {:?}", e);
            });

            if let Err(e) = result {
                tracing::error!(error = ?e, "initialize error");
                app.txs.write().await.remove(&session);
                return;
            }
            let _running_result = result.unwrap().waiting().await.inspect_err(|e| {
                tracing::error!(error = ?e, "running error");
            });
            app.txs.write().await.remove(&session);
        });
    }

    let stream = futures::stream::once(futures::future::ok(
        Event::default()
            .event("endpoint")
            .data(format!("?sessionId={session}")),
    ))
    .chain(ReceiverStream::new(to_client_rx).map(|message| {
        match serde_json::to_string(&message) {
            Ok(bytes) => Ok(Event::default().event("message").data(&bytes)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }));
    Sse::new(stream)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, App::new().router()).await
}
