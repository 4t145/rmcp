use crate::error::Error as McpError;
use crate::model::{CancelledNotification, JsonRpcMessage, Message, RequestId};
use crate::transport::IntoTransport;
use thiserror::Error;
#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::*;
use tokio_util::sync::CancellationToken;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ServiceError {
    #[error("Mcp error: {0}")]
    McpError(McpError),
    #[error("Transport error: {0}")]
    Transport(std::io::Error),
    #[error("Unexpected response type")]
    UnexpectedResponse,
    #[error("task cancelled for reasion {}", reason.as_deref().unwrap_or("<unknown>"))]
    Cancelled { reason: Option<String> },
}

impl ServiceError {}
trait TransferObject:
    std::fmt::Debug + Clone + serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static
{
}

impl<T> TransferObject for T where
    T: std::fmt::Debug
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Send
        + Sync
        + 'static
        + Clone
{
}

#[allow(private_bounds, reason = "there's no the third implementation")]
pub trait ServiceRole: std::fmt::Debug + Send + Sync + 'static + Copy {
    type Req: TransferObject;
    type Resp: TransferObject;
    type Not: TryInto<CancelledNotification, Error = Self::Not>
        + From<CancelledNotification>
        + TransferObject;
    type PeerReq: TransferObject;
    type PeerResp: TransferObject;
    type PeerNot: TryInto<CancelledNotification, Error = Self::PeerNot>
        + From<CancelledNotification>
        + TransferObject;
    const IS_CLIENT: bool;
    type Info: TransferObject;
    type PeerInfo: TransferObject;
}

pub(crate) type TxJsonRpcMessage<R> =
    JsonRpcMessage<<R as ServiceRole>::Req, <R as ServiceRole>::Resp, <R as ServiceRole>::Not>;
pub(crate) type RxJsonRpcMessage<R> = JsonRpcMessage<
    <R as ServiceRole>::PeerReq,
    <R as ServiceRole>::PeerResp,
    <R as ServiceRole>::PeerNot,
>;
pub trait Service: Send + Sync + 'static {
    type Role: ServiceRole;
    fn handle_request(
        &self,
        request: <Self::Role as ServiceRole>::PeerReq,
        context: RequestContext<Self::Role>,
    ) -> impl Future<Output = Result<<Self::Role as ServiceRole>::Resp, McpError>> + Send + '_;
    fn handle_notification(
        &self,
        notification: <Self::Role as ServiceRole>::PeerNot,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_;
    fn get_peer(&self) -> Option<Peer<Self::Role>>;
    fn set_peer(&mut self, peer: Peer<Self::Role>);
    fn set_peer_info(&mut self, peer: <Self::Role as ServiceRole>::PeerInfo);
    fn get_peer_info(&self) -> Option<<Self::Role as ServiceRole>::PeerInfo>;
    fn get_info(&self) -> <Self::Role as ServiceRole>::Info;
}

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use tokio::sync::mpsc;

pub trait RequestIdProvider: Send + Sync + 'static {
    fn next_request_id(&self) -> RequestId;
}

#[derive(Debug, Default)]
pub struct AtomicU32RequestIdProvider {
    id: AtomicU32,
}

impl RequestIdProvider for AtomicU32RequestIdProvider {
    fn next_request_id(&self) -> RequestId {
        RequestId::Number(self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

type Responder<T> = tokio::sync::oneshot::Sender<T>;

/// A handle to a remote request
///
/// You can cancel it by call [`Peer::send_notification`] with [`CancelledNotification`]
///
/// or wait for response by call [`RequestHandle::await_response`]
#[derive(Debug)]
pub struct RequestHandle<R: ServiceRole> {
    rx: tokio::sync::oneshot::Receiver<Result<R::PeerResp, ServiceError>>,
    pub id: RequestId,
}

impl<R: ServiceRole> RequestHandle<R> {
    pub async fn await_response(self) -> Result<R::PeerResp, ServiceError> {
        self.rx
            .await
            .map_err(|_e| ServiceError::Transport(std::io::Error::other("disconnected")))?
    }
}

#[derive(Debug)]
pub enum PeerSinkMessage<R: ServiceRole> {
    Request(
        R::Req,
        RequestId,
        Responder<Result<R::PeerResp, ServiceError>>,
    ),
    Notification(R::Not, Responder<Result<(), ServiceError>>),
}

/// An interface to fetch the remote client or server
///
/// For general perpose, call [`Peer::send_request`] or [`Peer::send_notification`] to send message to remote peer.
///
/// To create a cancellable request, call [`Peer::send_cancellable_request`].
#[derive(Clone)]
pub struct Peer<R: ServiceRole> {
    tx: mpsc::Sender<PeerSinkMessage<R>>,
    request_id_provider: Arc<dyn RequestIdProvider>,
    info: R::PeerInfo,
}

impl<R: ServiceRole> std::fmt::Debug for Peer<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeerSink")
            .field("tx", &self.tx)
            .field("is_client", &R::IS_CLIENT)
            .finish()
    }
}

type ProxyOutbound<R> = mpsc::Receiver<PeerSinkMessage<R>>;
impl<R: ServiceRole> Peer<R> {
    const CLIENT_CHANNEL_BUFFER_SIZE: usize = 1024;
    pub fn new(
        request_id_provider: Arc<dyn RequestIdProvider>,
        peer_info: R::PeerInfo,
    ) -> (Peer<R>, ProxyOutbound<R>) {
        let (tx, rx) = mpsc::channel(Self::CLIENT_CHANNEL_BUFFER_SIZE);
        (
            Self {
                tx,
                request_id_provider,
                info: peer_info,
            },
            rx,
        )
    }
    pub async fn send_notification(&self, notification: R::Not) -> Result<(), ServiceError> {
        let (responder, receiver) = tokio::sync::oneshot::channel();
        self.tx
            .send(PeerSinkMessage::Notification(notification, responder))
            .await
            .map_err(|_m| ServiceError::Transport(std::io::Error::other("disconnected")))?;
        receiver
            .await
            .map_err(|_e| ServiceError::Transport(std::io::Error::other("disconnected")))?
    }
    pub async fn send_request(&self, request: R::Req) -> Result<R::PeerResp, ServiceError> {
        self.send_cancellable_request(request)
            .await?
            .await_response()
            .await
    }
    pub async fn send_cancellable_request(
        &self,
        request: R::Req,
    ) -> Result<RequestHandle<R>, ServiceError> {
        let id = self.request_id_provider.next_request_id();
        let (responder, receiver) = tokio::sync::oneshot::channel();
        self.tx
            .send(PeerSinkMessage::Request(request, id.clone(), responder))
            .await
            .map_err(|_m| ServiceError::Transport(std::io::Error::other("disconnected")))?;
        Ok(RequestHandle { id, rx: receiver })
    }
    pub fn peer_info(&self) -> &R::PeerInfo {
        &self.info
    }
}

#[derive(Debug)]
pub struct RunningService<S: Service> {
    service: Arc<S>,
    peer: Peer<S::Role>,
    handle: tokio::task::JoinHandle<QuitReason>,
    /// cancellation token
    ct: CancellationToken,
}
impl<S: Service> Deref for RunningService<S> {
    type Target = Peer<S::Role>;

    fn deref(&self) -> &Self::Target {
        self.peer()
    }
}

impl<S: Service> RunningService<S> {
    #[inline]
    pub fn peer(&self) -> &Peer<S::Role> {
        &self.peer
    }
    #[inline]
    pub fn service(&self) -> &S {
        self.service.as_ref()
    }
    pub async fn waiting(self) -> Result<QuitReason, tokio::task::JoinError> {
        self.handle.await
    }
    pub async fn cancel(self) -> Result<QuitReason, tokio::task::JoinError> {
        self.ct.cancel();
        self.waiting().await
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum QuitReason {
    Cancelled,
    Closed,
}

/// Request execution context
#[derive(Debug, Clone)]
pub struct RequestContext<R: ServiceRole> {
    /// this token will be cancelled when the [`CancelledNotification`] is received.
    pub ct: CancellationToken,
    pub id: RequestId,
    /// An interface to fetch the remote client or server
    pub peer: Peer<R>,
}

/// Use this function to skip initialization process
pub async fn serve_directly<S, T, E, A>(
    service: S,
    transport: T,
    peer_info: <S::Role as ServiceRole>::PeerInfo,
) -> Result<RunningService<S>, E>
where
    S: Service,
    T: IntoTransport<S::Role, E, A>,
    E: std::error::Error + Send + Sync + 'static,
{
    serve_inner(service, transport, peer_info, Default::default()).await
}

async fn serve_inner<S, T, E, A>(
    mut service: S,
    transport: T,
    peer_info: <S::Role as ServiceRole>::PeerInfo,
    id_provider: Arc<AtomicU32RequestIdProvider>,
) -> Result<RunningService<S>, E>
where
    S: Service,
    T: IntoTransport<S::Role, E, A>,
    E: std::error::Error + Send + Sync + 'static,
{
    use futures::{SinkExt, StreamExt};
    const SINK_PROXY_BUFFER_SIZE: usize = 64;
    tracing::info!("Server started");
    let (sink_proxy_tx, mut sink_proxy_rx) = tokio::sync::mpsc::channel::<
        Message<
            <S::Role as ServiceRole>::Req,
            <S::Role as ServiceRole>::Resp,
            <S::Role as ServiceRole>::Not,
        >,
    >(SINK_PROXY_BUFFER_SIZE);

    if S::Role::IS_CLIENT {
        tracing::info!(?peer_info, "Server initialized as client");
    } else {
        tracing::info!(?peer_info, "Server initialized as server");
    }

    let (peer, mut peer_proxy) = <Peer<S::Role>>::new(id_provider, peer_info);
    service.set_peer(peer.clone());
    let mut local_responder_pool = HashMap::new();
    let mut local_ct_pool = HashMap::<RequestId, CancellationToken>::new();
    let shared_service = Arc::new(service);
    // for return
    let service = shared_service.clone();

    // let message_sink = tokio::sync::
    // let mut stream = std::pin::pin!(stream);
    let ct = CancellationToken::new();
    let serve_loop_ct = ct.child_token();
    let peer_return: Peer<<S as Service>::Role> = peer.clone();
    let handle = tokio::spawn(async move {
        let (mut sink, mut stream) = transport.into_transport();
        let mut sink = std::pin::pin!(sink);
        let mut stream = std::pin::pin!(stream);
        #[derive(Debug)]
        enum Event<P, R, T> {
            ProxyMessage(P),
            PeerMessage(R),
            ToSink(T),
        }
        let quit_reason = loop {
            let evt = tokio::select! {
                m = sink_proxy_rx.recv() => {
                    if let Some(m) = m {
                        Event::ToSink(m)
                    } else {
                        continue
                    }
                }
                m = stream.next() => {
                    if let Some(m) = m {
                        Event::PeerMessage(m.into_message())
                    } else {
                        // input stream closed
                        tracing::info!("input stream tarminated");
                        break QuitReason::Closed
                    }
                }
                m = peer_proxy.recv() => {
                    if let Some(m) = m {
                        Event::ProxyMessage(m)
                    } else {
                        continue
                    }
                }
                _ = serve_loop_ct.cancelled() => {
                    tracing::info!("task cancelled");
                    break QuitReason::Cancelled
                }
            };
            tracing::debug!(?evt, "new event");
            match evt {
                // response and error
                Event::ToSink(e) => {
                    if let Some(id) = match &e {
                        Message::Response(_, id) => Some(id),
                        Message::Error(_, id) => Some(id),
                        _ => None,
                    } {
                        if let Some(ct) = local_ct_pool.remove(id) {
                            ct.cancel();
                        }
                        let send_result = sink.send(e.into_json_rpc_message()).await;
                        if let Err(error) = send_result {
                            tracing::error!(%error, "fail to response message");
                        }
                    }
                }
                Event::ProxyMessage(PeerSinkMessage::Request(request, id, responder)) => {
                    local_responder_pool.insert(id.clone(), responder);
                    let send_result = sink
                        .send(Message::Request(request, id.clone()).into_json_rpc_message())
                        .await;
                    if let Err(e) = send_result {
                        if let Some(responder) = local_responder_pool.remove(&id) {
                            let _ = responder
                                .send(Err(ServiceError::Transport(std::io::Error::other(e))));
                        }
                    }
                }
                Event::ProxyMessage(PeerSinkMessage::Notification(notification, responder)) => {
                    // catch cancellation notification
                    let mut cancellation_param = None;
                    let notification = match notification.try_into() {
                        Ok::<CancelledNotification, _>(cancelled) => {
                            cancellation_param.replace(cancelled.params.clone());
                            cancelled.into()
                        }
                        Err(notification) => notification,
                    };
                    let send_result = sink
                        .send(Message::Notification(notification).into_json_rpc_message())
                        .await;
                    if let Err(e) = send_result {
                        let _ =
                            responder.send(Err(ServiceError::Transport(std::io::Error::other(e))));
                    }
                    if let Some(param) = cancellation_param {
                        if let Some(responder) = local_responder_pool.remove(&param.request_id) {
                            tracing::info!(id = %param.request_id, reason = param.reason, "cancelled");
                            let _response_result = responder.send(Err(ServiceError::Cancelled {
                                reason: param.reason.clone(),
                            }));
                        }
                    }
                }
                Event::PeerMessage(Message::Request(request, id)) => {
                    tracing::info!(%id, ?request, "received request");
                    {
                        let service = shared_service.clone();
                        let sink = sink_proxy_tx.clone();
                        let request_ct = serve_loop_ct.child_token();
                        let context_ct = request_ct.child_token();
                        local_ct_pool.insert(id.clone(), request_ct);
                        let context = RequestContext {
                            ct: context_ct,
                            id: id.clone(),
                            peer: peer.clone(),
                        };
                        tokio::spawn(async move {
                            let result = service.handle_request(request, context).await;
                            let response = match result {
                                Ok(result) => {
                                    tracing::info!(%id, ?result, "response message");
                                    Message::Response(result, id)
                                }
                                Err(error) => {
                                    tracing::warn!(%id, ?error, "response error");
                                    Message::Error(error, id)
                                }
                            };
                            let _send_result = sink.send(response).await;
                        });
                    }
                }
                Event::PeerMessage(Message::Notification(notification)) => {
                    tracing::info!(?notification, "received notification");
                    // catch cancelled notification
                    let notification = match notification.try_into() {
                        Ok::<CancelledNotification, _>(cancelled) => {
                            if let Some(ct) = local_ct_pool.remove(&cancelled.params.request_id) {
                                tracing::info!(id = %cancelled.params.request_id, reason = cancelled.params.reason, "cancelled");
                                ct.cancel();
                            }
                            cancelled.into()
                        }
                        Err(notification) => notification,
                    };
                    {
                        let service = shared_service.clone();
                        tokio::spawn(async move {
                            let result = service.handle_notification(notification).await;
                            if let Err(error) = result {
                                tracing::warn!(%error, "Error sending notification");
                            }
                        });
                    }
                }
                Event::PeerMessage(Message::Response(result, id)) => {
                    if let Some(responder) = local_responder_pool.remove(&id) {
                        let response_result = responder.send(Ok(result));
                        if let Err(_error) = response_result {
                            tracing::warn!(%id, "Error sending response");
                        }
                    }
                }
                Event::PeerMessage(Message::Error(error, id)) => {
                    if let Some(responder) = local_responder_pool.remove(&id) {
                        let _response_result = responder.send(Err(ServiceError::McpError(error)));
                        if let Err(_error) = _response_result {
                            tracing::warn!(%id, "Error sending response");
                        }
                    }
                }
            }
        };
        tracing::info!(?quit_reason, "serve finished");
        quit_reason
    });
    Ok(RunningService {
        service,
        peer: peer_return,
        handle,
        ct,
    })
}
