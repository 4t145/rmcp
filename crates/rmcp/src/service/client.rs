use crate::model::{
    CallToolRequest, CallToolRequestParam, CallToolResult, CancelledNotification,
    CancelledNotificationParam, ClientInfo, ClientMessage, ClientNotification, ClientRequest,
    ClientResult, CompleteRequest, CompleteRequestParam, CompleteResult, GetPromptRequest,
    GetPromptRequestParam, GetPromptResult, InitializeRequest, InitializedNotification,
    ListPromptsRequest, ListPromptsResult, ListResourceTemplatesRequest,
    ListResourceTemplatesResult, ListResourcesRequest, ListResourcesResult, ListToolsRequest,
    ListToolsResult, PaginatedRequestParam, ProgressNotification, ProgressNotificationParam,
    ReadResourceRequest, ReadResourceRequestParam, ReadResourceResult,
    RootsListChangedNotification, ServerInfo, ServerNotification, ServerRequest, ServerResult,
    SetLevelRequest, SetLevelRequestParam, SubscribeRequest, SubscribeRequestParam,
    UnsubscribeRequest, UnsubscribeRequestParam,
};

use super::*;
use futures::{SinkExt, StreamExt};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RoleClient;

impl ServiceRole for RoleClient {
    type Req = ClientRequest;
    type Resp = ClientResult;
    type Not = ClientNotification;
    type PeerReq = ServerRequest;
    type PeerResp = ServerResult;
    type PeerNot = ServerNotification;
    type Info = ClientInfo;
    type PeerInfo = ServerInfo;

    const IS_CLIENT: bool = true;
}

pub type ServerSink = Peer<RoleClient>;

impl<S: Service<RoleClient>> ServiceExt<RoleClient> for S {
    fn serve<T, E, A>(
        self,
        transport: T,
    ) -> impl Future<Output = Result<RunningService<RoleClient, Self>, E>> + Send
    where
        T: IntoTransport<RoleClient, E, A>,
        E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
        Self: Sized,
    {
        serve_client(self, transport)
    }
}

pub async fn serve_client<S, T, E, A>(
    service: S,
    transport: T,
) -> Result<RunningService<RoleClient, S>, E>
where
    S: Service<RoleClient>,
    T: IntoTransport<RoleClient, E, A>,
    E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
{
    let (sink, stream) = transport.into_transport();
    let mut sink = Box::pin(sink);
    let mut stream = Box::pin(stream);
    let id_provider = <Arc<AtomicU32RequestIdProvider>>::default();
    // service
    let id = id_provider.next_request_id();
    let init_request = InitializeRequest {
        method: Default::default(),
        params: service.get_info(),
    };
    sink.send(
        ClientMessage::Request(ClientRequest::InitializeRequest(init_request), id.clone())
            .into_json_rpc_message(),
    )
    .await?;
    let (response, response_id) = stream
        .next()
        .await
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "expect initialize response",
        ))?
        .into_message()
        .into_result()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expect initialize result",
        ))?;
    if id != response_id {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "conflict initialize response id",
        )
        .into());
    }
    let response = response.map_err(std::io::Error::other)?;
    let ServerResult::InitializeResult(initialize_result) = response else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expect initialize result",
        )
        .into());
    };
    // send notification
    let notification = ClientMessage::Notification(ClientNotification::InitializedNotification(
        InitializedNotification {
            method: Default::default(),
        },
    ));
    sink.send(notification.into_json_rpc_message()).await?;
    serve_inner(service, (sink, stream), initialize_result, id_provider).await
}

macro_rules! method {
    (peer_req $method:ident $Req:ident() => $Resp: ident ) => {
        pub async fn $method(&self) -> Result<$Resp, ServiceError> {
            let result = self
                .send_request(ClientRequest::$Req($Req {
                    method: Default::default(),
                }))
                .await?;
            match result {
                ServerResult::$Resp(result) => Ok(result),
                _ => Err(ServiceError::UnexpectedResponse),
            }
        }
    };
    (peer_req $method:ident $Req:ident($Param: ident) => $Resp: ident ) => {
        pub async fn $method(&self, params: $Param) -> Result<$Resp, ServiceError> {
            let result = self
                .send_request(ClientRequest::$Req($Req {
                    method: Default::default(),
                    params,
                }))
                .await?;
            match result {
                ServerResult::$Resp(result) => Ok(result),
                _ => Err(ServiceError::UnexpectedResponse),
            }
        }
    };
    (peer_req $method:ident $Req:ident($Param: ident)) => {
        pub async fn $method(&self, params: $Param) -> Result<(), ServiceError> {
            let result = self
                .send_request(ClientRequest::$Req($Req {
                    method: Default::default(),
                    params,
                }))
                .await?;
            match result {
                ServerResult::EmptyResult(_) => Ok(()),
                _ => Err(ServiceError::UnexpectedResponse),
            }
        }
    };

    (peer_not $method:ident $Not:ident($Param: ident)) => {
        pub async fn $method(&self, params: $Param) -> Result<(), ServiceError> {
            self.send_notification(ClientNotification::$Not($Not {
                method: Default::default(),
                params,
            }))
            .await?;
            Ok(())
        }
    };
    (peer_not $method:ident $Not:ident) => {
        pub async fn $method(&self) -> Result<(), ServiceError> {
            self.send_notification(ClientNotification::$Not($Not {
                method: Default::default(),
            }))
            .await?;
            Ok(())
        }
    };
}

impl Peer<RoleClient> {
    method!(peer_req complete CompleteRequest(CompleteRequestParam) => CompleteResult);
    method!(peer_req set_level SetLevelRequest(SetLevelRequestParam));
    method!(peer_req get_prompt GetPromptRequest(GetPromptRequestParam) => GetPromptResult);
    method!(peer_req list_prompts ListPromptsRequest(PaginatedRequestParam) => ListPromptsResult);
    method!(peer_req list_resources ListResourcesRequest(PaginatedRequestParam) => ListResourcesResult);
    method!(peer_req list_resource_templates ListResourceTemplatesRequest(PaginatedRequestParam) => ListResourceTemplatesResult);
    method!(peer_req read_resource ReadResourceRequest(ReadResourceRequestParam) => ReadResourceResult);
    method!(peer_req subscribe SubscribeRequest(SubscribeRequestParam) );
    method!(peer_req unsubscribe UnsubscribeRequest(UnsubscribeRequestParam));
    method!(peer_req call_tool CallToolRequest(CallToolRequestParam) => CallToolResult);
    method!(peer_req list_tools ListToolsRequest(PaginatedRequestParam) => ListToolsResult);

    method!(peer_not notify_cancelled CancelledNotification(CancelledNotificationParam));
    method!(peer_not notify_progress ProgressNotification(ProgressNotificationParam));
    method!(peer_not notify_initialized InitializedNotification);
    method!(peer_not notify_roots_list_changed RootsListChangedNotification);
}
