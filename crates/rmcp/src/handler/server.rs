use crate::error::Error as McpError;
use crate::model::*;
use crate::service::{Peer, RequestContext, RoleServer, Service, ServiceRole};

mod resource;
pub mod tool;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ServerHandlerService<H> {
    pub handler: H,
}

impl<H: ServerHandler> ServerHandlerService<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }
}

impl<H: ServerHandler> Service for ServerHandlerService<H> {
    type Role = RoleServer;

    async fn handle_request(
        &self,
        request: <Self::Role as ServiceRole>::PeerReq,
        context: RequestContext<Self::Role>,
    ) -> Result<<Self::Role as ServiceRole>::Resp, McpError> {
        match request {
            ClientRequest::InitializeRequest(request) => self
                .handler
                .initialize(request.params, context)
                .await
                .map(ServerResult::InitializeResult),
            ClientRequest::PingRequest(_request) => {
                self.handler.ping(context).await.map(ServerResult::empty)
            }
            ClientRequest::CompleteRequest(request) => self
                .handler
                .complete(request.params, context)
                .await
                .map(ServerResult::CompleteResult),
            ClientRequest::SetLevelRequest(request) => self
                .handler
                .set_level(request.params, context)
                .await
                .map(ServerResult::empty),
            ClientRequest::GetPromptRequest(request) => self
                .handler
                .get_prompt(request.params, context)
                .await
                .map(ServerResult::GetPromptResult),
            ClientRequest::ListPromptsRequest(request) => self
                .handler
                .list_prompts(request.params, context)
                .await
                .map(ServerResult::ListPromptsResult),
            ClientRequest::ListResourcesRequest(request) => self
                .handler
                .list_resources(request.params, context)
                .await
                .map(ServerResult::ListResourcesResult),
            ClientRequest::ListResourceTemplatesRequest(request) => self
                .handler
                .list_resource_templates(request.params, context)
                .await
                .map(ServerResult::ListResourceTemplatesResult),
            ClientRequest::ReadResourceRequest(request) => self
                .handler
                .read_resource(request.params, context)
                .await
                .map(ServerResult::ReadResourceResult),
            ClientRequest::SubscribeRequest(request) => self
                .handler
                .subscribe(request.params, context)
                .await
                .map(ServerResult::empty),
            ClientRequest::UnsubscribeRequest(request) => self
                .handler
                .unsubscribe(request.params, context)
                .await
                .map(ServerResult::empty),
            ClientRequest::CallToolRequest(request) => self
                .handler
                .call_tool(request.params, context)
                .await
                .map(ServerResult::CallToolResult),
            ClientRequest::ListToolsRequest(request) => self
                .handler
                .list_tools(request.params, context)
                .await
                .map(ServerResult::ListToolsResult),
        }
    }

    async fn handle_notification(
        &self,
        notification: <Self::Role as ServiceRole>::PeerNot,
    ) -> Result<(), McpError> {
        match notification {
            ClientNotification::CancelledNotification(notification) => {
                self.handler.on_cancelled(notification.params).await
            }
            ClientNotification::ProgressNotification(notification) => {
                self.handler.on_progress(notification.params).await
            }
            ClientNotification::InitializedNotification(_notification) => {
                self.handler.on_initialized().await
            }
            ClientNotification::RootsListChangedNotification(_notification) => {
                self.handler.on_roots_list_changed().await
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

    fn get_info(&self) -> <Self::Role as ServiceRole>::Info {
        self.handler.get_info()
    }
}

#[allow(unused_variables)]
pub trait ServerHandler: Sized + Clone + Send + Sync + 'static {
    fn ping(
        &self,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        std::future::ready(Ok(()))
    }
    // handle requests
    fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + Send + '_ {
        std::future::ready(Ok(self.get_info()))
    }
    fn complete(
        &self,
        request: CompleteRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CompleteResult, McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<CompleteRequestMethod>()))
    }
    fn set_level(
        &self,
        request: SetLevelRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<SetLevelRequestMethod>()))
    }
    fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<GetPromptRequestMethod>()))
    }
    fn list_prompts(
        &self,
        request: PaginatedRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListPromptsResult::default()))
    }
    fn list_resources(
        &self,
        request: PaginatedRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult::default()))
    }
    fn list_resource_templates(
        &self,
        request: PaginatedRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourceTemplatesResult::default()))
    }
    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        std::future::ready(Err(
            McpError::method_not_found::<ReadResourceRequestMethod>(),
        ))
    }
    fn subscribe(
        &self,
        request: SubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<SubscribeRequestMethod>()))
    }
    fn unsubscribe(
        &self,
        request: UnsubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<UnsubscribeRequestMethod>()))
    }
    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        std::future::ready(Err(McpError::method_not_found::<CallToolRequestMethod>()))
    }
    fn list_tools(
        &self,
        request: PaginatedRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult::default()))
    }

    fn on_cancelled(
        &self,
        notification: CancelledNotificationParam,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_progress(
        &self,
        notification: ProgressNotificationParam,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
    fn on_initialized(&self) -> impl Future<Output = ()> + Send + '_ {
        tracing::info!("client initialized");
        std::future::ready(())
    }
    fn on_roots_list_changed(&self) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }

    fn get_peer(&self) -> Option<Peer<RoleServer>> {
        None
    }

    fn set_peer(&mut self, peer: Peer<RoleServer>) {
        drop(peer);
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }
}
