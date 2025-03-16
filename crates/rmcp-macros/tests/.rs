fn sum_tool_attr() -> rmcp::model::Tool {
    rmcp::model::Tool {
        name: "test_tool".into(),
        description: "test tool".into(),
        input_schema: rmcp::handler::server::tool::cached_schema_for_type::<req: StructRequest>()
            .into(),
    }
}
async fn sum_tool_call(
    context: rmcp::handler::server::tool::ToolCallContext<'_, Self>,
) -> std::result::Result<rmcp::model::CallToolResult, rmcp::Error> {
    use rmcp::handler::server::tool::*;
    let (__rmcp_tool_receiver, context) = <&Self>::from_tool_call_context_part(context)?;
    let (Parameters(req), context) =
        <Parameters<StructRequest>>::from_tool_call_context_part(context)?;
    Self::sum(__rmcp_tool_receiver, req).into_call_tool_result()
}
fn sum(&self, req: StructRequest) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(saying)]))
}
