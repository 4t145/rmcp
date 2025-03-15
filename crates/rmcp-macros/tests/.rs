fn echo_tool_attr() -> rmcp::model::Tool {
    rmcp::model::Tool {
        name: "test_tool".into(),
        description: "test tool".into(),
        input_schema: {
            #[derive(serde :: Serialize, serde :: Deserialize, schemars :: JsonSchema)]
            pub struct __ECHOToolCallParam {
                pub echo: String,
            }
            rmcp::handler::server::tool::cached_schema_for_type::<__ECHOToolCallParam>()
        }
        .into(),
    }
}
fn echo_inner(&self, echo: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(echo)]))
}
fn echo(
    &self,
    __rmcp_tool_req: rmcp::model::JsonObject,
) -> std::result::Result<rmcp::model::CallToolResult, rmcp::Error> {
    use rmcp::handler::server::tool::*;
    #[derive(serde :: Serialize, serde :: Deserialize, schemars :: JsonSchema)]
    pub struct __ECHOToolCallParam {
        pub echo: String,
    }
    let __ECHOToolCallParam { echo } = parse_json_object(__rmcp_tool_req)?;
    echo_inner(self, echo).into_call_tool_result()
}
