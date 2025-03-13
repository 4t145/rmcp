use crate::error::Error as McpError;
use crate::model::*;
use crate::service::{Peer, RequestContext, RoleClient, Service, ServiceRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ClientHandlerService<H> {
    pub handler: H,
}

impl<H: ClientHandler> ClientHandlerService<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }
}

impl<H: ClientHandler> Service for ClientHandlerService<H> {
    type Role = RoleClient;

    async fn handle_request(
        &self,
        request: <Self::Role as ServiceRole>::PeerReq,
        context: RequestContext<Self::Role>,
    ) -> Result<<Self::Role as ServiceRole>::Resp, McpError> {
        match request {
            ServerRequest::PingRequest(_) => {
                self.handler.ping(context).await.map(ClientResult::empty)
            }
            ServerRequest::CreateMessageRequest(request) => self
                .handler
                .create_message(request.params, context)
                .await
                .map(ClientResult::CreateMessageResult),
            ServerRequest::ListRootsRequest(_) => self
                .handler
                .list_roots(context)
                .await
                .map(ClientResult::ListRootsResult),
        }
    }

    async fn handle_notification(
        &self,
        notification: <Self::Role as ServiceRole>::PeerNot,
    ) -> Result<(), McpError> {
        match notification {
            ServerNotification::CancelledNotification(notification) => {
                self.handler.on_cancelled(notification.params).await
            }
            ServerNotification::ProgressNotification(notification) => {
                self.handler.on_progress(notification.params).await
            }
            ServerNotification::LoggingMessageNotification(notification) => {
                self.handler.on_logging_message(notification.params).await
            }
            ServerNotification::ResourceUpdatedNotification(notification) => {
                self.handler.on_resource_updated(notification.params).await
            }
            ServerNotification::ResourceListChangedNotification(_notification_no_param) => {
                self.handler.on_resource_list_changed().await
            }
            ServerNotification::ToolListChangedNotification(_notification_no_param) => {
                self.handler.on_tool_list_changed().await
            }
            ServerNotification::PromptListChangedNotification(_notification_no_param) => {
                self.handler.on_prompt_list_changed().await
            }
        };
        Ok(())
    }

    fn get_peer(&self) -> Option<Peer<Self::Role>> {
        self.handler.get_peer()
    }

    fn set_peer(&mut self, peer: Peer<Self::Role>) {
        self.handler.set_peer(peer);
    }

    fn set_peer_info(&mut self, peer: <Self::Role as ServiceRole>::PeerInfo) {
        self.handler.set_peer_info(peer);
    }

    fn get_peer_info(&self) -> Option<<Self::Role as ServiceRole>::PeerInfo> {
        self.handler.get_peer_info()
    }

    fn get_info(&self) -> <Self::Role as ServiceRole>::Info {
        self.handler.get_info()
    }
}

#[allow(unused_variables)]
pub trait ClientHandler: Sized + Send + Sync + 'static {
    fn ping(
        &self,
        context: RequestContext<RoleClient>,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        std::future::ready(Ok(()))
    }

    fn create_message(
        &self,
        params: CreateMessageRequestParam,
        context: RequestContext<RoleClient>,
    ) -> impl Future<Output = Result<CreateMessageResult, McpError>> + Send + '_ {
        std::future::ready(Err(
            McpError::method_not_found::<CreateMessageRequestMethod>(),
        ))
    }
    fn list_roots(
        &self,
        context: RequestContext<RoleClient>,
    ) -> impl Future<Output = Result<ListRootsResult, McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<ListRootsRequestMethod>()))
    }

    fn on_cancelled(
        &self,
        params: CancelledNotificationParam,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_progress(
        &self,
        params: ProgressNotificationParam,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_logging_message(
        &self,
        params: LoggingMessageNotificationParam,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_resource_updated(
        &self,
        params: ResourceUpdatedNotificationParam,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_resource_list_changed(&self) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_tool_list_changed(&self) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_prompt_list_changed(&self) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }

    fn get_peer(&self) -> Option<Peer<RoleClient>>;

    fn set_peer(&mut self, peer: Peer<RoleClient>);

    fn set_peer_info(&mut self, peer: ServerInfo) {
        drop(peer);
    }

    fn get_peer_info(&self) -> Option<ServerInfo> {
        None
    }

    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}
