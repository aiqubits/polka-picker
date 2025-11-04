// Tool adapter definition
use anyhow::Error;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::tools::Tool;
use super::client::{McpClient, McpTool};
use log::info;
// MCP tool adapter
pub struct McpToolAdapter {
    mcp_client: Arc<dyn McpClient>,
    mcp_tool: McpTool,
}

impl McpToolAdapter {
    pub fn new(mcp_client: Arc<dyn McpClient>, mcp_tool: McpTool) -> Self {
        Self {
            mcp_client,
            mcp_tool,
        }
    }
    
    // Constructor method to convert from Box to Arc
    pub fn new_from_box(mcp_client: Box<dyn McpClient>, mcp_tool: McpTool) -> Self {
        Self {
            mcp_client: Arc::from(mcp_client),
            mcp_tool,
        }
    }
    
    // Get reference to the client
    pub fn get_client(&self) -> Arc<dyn McpClient> {
        self.mcp_client.clone()
    }
    
    // Get clone of the MCP tool
    pub fn get_mcp_tool(&self) -> McpTool {
        self.mcp_tool.clone()
    }
}

impl Tool for McpToolAdapter {
    fn name(&self) -> &str {
        &self.mcp_tool.name
    }
    
    fn description(&self) -> &str {
        &self.mcp_tool.description
    }
    
    fn invoke(&self, input: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send + '_>> {
        let client = self.mcp_client.clone();
        let tool_name = self.mcp_tool.name.clone();
        let input_str = input.to_string();
        info!("Invoking MCP tool {} with input: {}", tool_name, input_str);
        Box::pin(async move {
            // Try to parse input as JSON parameters, add fault tolerance
            let parameters: HashMap<String, Value> = match serde_json::from_str(&input_str) {
                Ok(params) => params,
                Err(_) => {
                    // Simple handling, use input as default parameter
                    let mut map = HashMap::new();
                    map.insert("query".to_string(), Value::String(input_str.clone()));
                    map
                },
            };
            
            // Call the tool on the MCP server
            let result_future = client.call_tool(&tool_name, parameters);
            let result = result_future.await?;
            
            // Convert result to string
            Ok(serde_json::to_string_pretty(&result)?)
        })
    }
    
    // Implement as_any method to support runtime type checking
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}