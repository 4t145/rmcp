use std::{
    borrow::Cow,
    sync::{Arc, OnceLock},
};

use rmcp::{
    Error as McpError, RoleServer, ServerHandler, const_string,
    handler::server::{
        tool_v1::{ToolSet, ToolTrait},
        tool_v2::{Callee, Parameter, ToolCallContext},
    },
    model::*,
    service::RequestContext,
    tool,
};

use serde_json::json;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
}

impl Counter {
    pub fn new() -> Self {
        let counter = Arc::new(Mutex::new(0));

        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Decrement the counter by 1")]
    async fn decrement(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get_value(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Say hello to the client")]
    fn say_hello(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("hello")]))
    }

    #[tool(description = "Repeat what you say")]
    fn echo(&self, #[tool(param)] saying: String) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(saying)]))
    }

    // pub fn tool_box() -> &'static ToolBox<Counter> {
    //     static TOOL_BOX: OnceLock<ToolBox<Counter>> = OnceLock::new();
    //     TOOL_BOX.get_or_init(|| {
    //         let mut tool_box = ToolBox::new();
    //         tool_box.add(Self::increment_tool_attr(), Self::increment);
    //         // tool_box.add(Self::decrement_tool_attr(), Self::decrement);
    //         // tool_box.add(Self::get_value_tool_attr(), Self::get_value);
    //         // tool_box.add(Self::say_hello_tool_attr(), Self::say_hello);
    //         // tool_box.add(Self::echo_tool_attr(), Self::echo_call);
    //         tool_box
    //     })
    // }
}
const_string!(Echo = "echo");
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
        let tools = vec![
            Self::increment_tool_attr(),
            Self::decrement_tool_attr(),
            Self::get_value_tool_attr(),
            Self::say_hello_tool_attr(),
            Self::echo_tool_attr(),
        ];
        Ok(ListToolsResult {
            next_cursor: None,
            tools
        })
    }

    async fn call_tool(
        &self,
        call_tool_request_param: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        static TOOL_STATIC_ROOTER: std::sync::OnceLock<
            std::collections::HashMap<
                String,
                Box<
                    dyn for<'a> Fn(
                            ToolCallContext<'a, Counter>,
                        ) -> std::pin::Pin<
                            std::boxed::Box<
                                dyn Future<Output = Result<CallToolResult, McpError>> + Send + 'a,
                            >,
                        > + Send
                        + Sync,
                >,
            >,
        > = std::sync::OnceLock::new();
        let map = TOOL_STATIC_ROOTER.get_or_init(|| {
            let mut map = <std::collections::HashMap<
                String,
                Box<
                    dyn for<'a> Fn(
                            ToolCallContext<'a, Counter>,
                        ) -> std::pin::Pin<
                            std::boxed::Box<
                                dyn Future<
                                        Output = Result<rmcp::model::CallToolResult, rmcp::Error>,
                                    > + Send
                                    + 'a,
                            >,
                        > + Send
                        + Sync,
                >,
            >>::new();
            map.insert(
                "increment".to_string(),
                Box::new(|context: ToolCallContext<'_, Counter>| {
                    Box::pin(context.invoke(Self::increment))
                }),
            );
            map.insert(
                "decrement".to_string(),
                Box::new(|context: ToolCallContext<'_, Counter>| {
                    Box::pin(context.invoke(Self::decrement))
                }),
            );
            map
        });
        let context = ToolCallContext::new(self, call_tool_request_param, context);
        if let Some(caller) = map.get(context.name()) {
            (caller)(context).await
        } else {
            Err(McpError::invalid_params("method not found", None))
        }
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
