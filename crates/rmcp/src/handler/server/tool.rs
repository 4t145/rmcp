use std::borrow::Borrow;
use std::collections::HashMap;
use std::{borrow::Cow, future::Future, ops::Deref, pin::Pin};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::Error as McpError;
use crate::model::{CallToolResult, JsonObject, Tool};

/// Trait for implementing MCP tools
pub trait DynTool: Send + Sync {
    /// The name of the tool
    fn name(&self) -> Cow<'static, str>;

    /// A description of what the tool does
    fn description(&self) -> Cow<'static, str>;

    /// JSON schema describing the tool's parameters
    fn schema(&self) -> JsonObject;

    /// Execute the tool with the given parameters
    fn call(
        &self,
        params: JsonObject,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + '_>>;
}

/// Trait for implementing MCP tools with specified types
pub trait ToolTrait: Send + Sync {
    #[cfg(feature = "default-json-schema")]
    type Params: DeserializeOwned + schemars::JsonSchema;

    #[cfg(not(feature = "default-json-schema"))]
    type Params: DeserializeOwned;
    /// The name of the tool
    fn name(&self) -> Cow<'static, str>;

    /// A description of what the tool does
    fn description(&self) -> Cow<'static, str>;

    /// JSON schema describing the tool's parameters
    #[cfg(feature = "default-json-schema")]
    fn schema(&self) -> JsonObject {
        let value = serde_json::to_value(schemars::schema_for!(Self::Params))
            .expect("json schema should always be a valid json value");
        match value {
            Value::Object(map) => map,
            _ => unreachable!("json schema should always be a valid json value"),
        }
    }

    #[cfg(not(feature = "default-json-schema"))]
    fn schema(&self) -> JsonObject;

    /// Execute the tool with the given parameters
    fn call(
        &self,
        params: Self::Params,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send;
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub struct Dynamic<H>(pub H);

impl<H: ToolTrait> DynTool for Dynamic<H> {
    fn name(&self) -> Cow<'static, str> {
        ToolTrait::name(&self.0)
    }

    fn description(&self) -> Cow<'static, str> {
        ToolTrait::description(&self.0)
    }

    fn schema(&self) -> JsonObject {
        ToolTrait::schema(&self.0)
    }

    fn call(
        &self,
        params: JsonObject,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + '_>> {
        Box::pin(async {
            let input = serde_json::from_value(serde_json::Value::Object(params))
                .map_err(|e| McpError::invalid_params(format!("parse argument error {e}"), None))?;
            let result = ToolTrait::call(&self.0, input).await?;
            Ok(result)
        })
    }
}

impl dyn DynTool {
    pub fn tool_data(&self) -> Tool {
        Tool {
            name: self.name().clone(),
            description: self.description().clone(),
            input_schema: self.schema(),
        }
    }
}

pub struct BoxedDynTool(Box<dyn DynTool>);

impl Deref for BoxedDynTool {
    type Target = dyn DynTool;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl BoxedDynTool {
    /// Convert from a [`TypedToolHandler`]
    pub fn new<H: DynTool + 'static>(handler: H) -> Self {
        Self(Box::new(handler))
    }
    pub fn new_boxed(handler: Box<dyn DynTool>) -> Self {
        Self(handler)
    }
}

#[derive(Default)]
pub struct ToolSet {
    tools: HashMap<Cow<'static, str>, BoxedDynTool>,
}

impl std::fmt::Debug for ToolSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.tools.keys()).finish()
    }
}

impl ToolSet {
    pub fn add_tool<H: ToolTrait + 'static>(&mut self, tool: H) -> Option<BoxedDynTool> {
        self.tools
            .insert(tool.name().clone(), BoxedDynTool::new(Dynamic(tool)))
    }
    pub fn add_dyn_tool<H: DynTool + 'static>(&mut self, tool: H) -> Option<BoxedDynTool> {
        self.tools
            .insert(tool.name().clone(), BoxedDynTool::new(tool))
    }
    pub fn add_boxed_tool(&mut self, tool: Box<dyn DynTool>) -> Option<BoxedDynTool> {
        self.tools
            .insert(tool.name().clone(), BoxedDynTool::new_boxed(tool))
    }
    pub fn remove_tool<S>(&mut self, name: &S) -> std::option::Option<BoxedDynTool>
    where
        Cow<'static, str>: Borrow<S>,
        S: std::hash::Hash + Eq + ?Sized,
    {
        self.tools.remove(name)
    }
    pub fn get_tool<S>(&self, name: &S) -> Option<&BoxedDynTool>
    where
        Cow<'static, str>: Borrow<S>,
        S: std::hash::Hash + Eq + ?Sized,
    {
        self.tools.get(name)
    }

    pub fn extend(&mut self, tool_set: ToolSet) {
        self.tools.extend(tool_set.tools);
    }

    pub async fn call(
        &self,
        name: &str,
        params: Option<JsonObject>,
    ) -> Result<CallToolResult, McpError> {
        let handler = self.get_tool(name).ok_or(McpError::invalid_params(
            format!("Unknown tool: {name}"),
            None,
        ))?;
        let result = handler.call(params.unwrap_or_default()).await?;
        Ok(result)
    }

    pub fn list_all(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|handler| handler.deref().tool_data())
            .collect()
    }
}
