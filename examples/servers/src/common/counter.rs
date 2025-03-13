use std::{borrow::Cow, sync::Arc};

use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    handler::server::tool::{ToolSet, ToolTrait},
    model::*,
    service::RequestContext,
};

use serde_json::json;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Counter {
    _counter: Arc<Mutex<i32>>,
    tool_set: Arc<ToolSet>,
}

pub struct IncrementTool(Arc<Mutex<i32>>);

impl ToolTrait for IncrementTool {
    type Params = EmptyObject;

    fn name(&self) -> Cow<'static, str> {
        "increment".into()
    }

    fn description(&self) -> Cow<'static, str> {
        "Increment the counter by 1".into()
    }

    async fn call(&self, _params: Self::Params) -> Result<CallToolResult, McpError> {
        let mut counter = self.0.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![RawContent::text(
            counter.to_string(),
        )]))
    }
}

pub struct DecrementTool(Arc<Mutex<i32>>);

impl ToolTrait for DecrementTool {
    type Params = EmptyObject;

    fn name(&self) -> Cow<'static, str> {
        "decrement".into()
    }

    fn description(&self) -> Cow<'static, str> {
        "Decrement the counter by 1".into()
    }

    async fn call(&self, _params: Self::Params) -> Result<CallToolResult, McpError> {
        let mut counter = self.0.lock().await;
        *counter -= 1;
        Ok(CallToolResult::success(vec![RawContent::text(
            counter.to_string(),
        )]))
    }
}

pub struct GetValueTool(Arc<Mutex<i32>>);

impl ToolTrait for GetValueTool {
    type Params = EmptyObject;

    fn name(&self) -> Cow<'static, str> {
        "get_value".into()
    }

    fn description(&self) -> Cow<'static, str> {
        "Get the current counter value".into()
    }

    async fn call(&self, _params: Self::Params) -> Result<CallToolResult, McpError> {
        let counter = self.0.lock().await;
        Ok(CallToolResult::success(vec![RawContent::text(
            counter.to_string(),
        )]))
    }
}

impl Counter {
    pub fn new() -> Self {
        let mut tool_set = ToolSet::default();
        let counter = Arc::new(Mutex::new(0));

        tool_set.add_tool(IncrementTool(counter.clone()));
        tool_set.add_tool(DecrementTool(counter.clone()));
        tool_set.add_tool(GetValueTool(counter.clone()));
        Self {
            _counter: Arc::new(Mutex::new(0)),
            tool_set: Arc::new(tool_set),
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }
}

impl ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                experimental: None,
                logging: None,
                prompts: Some(PromptsCapability::default()),
                resources: Some(ResourcesCapability::default()),
                tools: Some(ToolsCapability {
                    list_changed: None,
                }),
            },
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.".to_string()),
        }
    }
    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = self.tool_set.list_all();
        Ok(ListToolsResult {
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        CallToolRequestParam { name, arguments }: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.tool_set.call(&name, arguments).await
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                self._create_resource_text("str:////Users/to/some/path/", "cwd"),
                self._create_resource_text("memo://insights", "memo-name"),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(cwd, uri)],
                })
            }
            "memo://insights" => {
                let memo = "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(memo, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "example_prompt",
                Some("This is an example prompt that takes one required agrument, message"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments: _ }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "example_prompt" => {
                let prompt = "This is an example prompt with your message here: '{message}'";
                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => Err(McpError::invalid_params("prompt not found", None)),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}
