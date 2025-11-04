// MCP client interface definition
use anyhow::Error;
use log::{debug, info, warn};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::mcp::JSONRPCRequest;
use crate::mcp::JSONRPCResponse;

// MCP tool structure
#[derive(Debug,Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    // Other tool-related fields
}

// Simple MCP client implementation, modify SimpleMcpClient structure, add tool handler field
#[derive(Clone)]
pub struct SimpleMcpClient {
    pub url: String,
    pub available_tools: Vec<McpTool>,
    // Use Arc to wrap tool handlers to support cloning
    pub tool_handlers: HashMap<String, Arc<dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<Value, Error>> + Send>> + Send + Sync>>,
    // Connection status flag, indicates whether successfully connected to MCP server
    pub is_mcp_server_connected: Arc<Mutex<bool>>,
}

// Implement methods for SimpleMcpClient structure
impl SimpleMcpClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            available_tools: Vec::new(),
            tool_handlers: HashMap::new(),
            is_mcp_server_connected: Arc::new(Mutex::new(false)), // Initial state is disconnected
        }
    }
    
    // Add custom tool method
    pub fn add_tool(&mut self, tool: McpTool) {
        self.available_tools.push(tool);
    }
    
    // Register tool handler method
    pub fn register_tool_handler<F, Fut>(&mut self, tool_name: String, handler: F)
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, Error>> + Send + 'static,
    {
        self.tool_handlers.insert(tool_name, Arc::new(move |params| {
            let params_clone = params.clone();
            Box::pin(handler(params_clone))
        }));
    }
    
    // Batch add tools method
    pub fn add_tools(&mut self, tools: Vec<McpTool>) {
        self.available_tools.extend(tools);
    }
    
    // Clear tool list method
    pub fn clear_tools(&mut self) {
        self.available_tools.clear();
    }
    
    // Set server connection status
    pub fn set_server_connected(&self, connected: bool) {
        if let Ok(mut conn_status) = self.is_mcp_server_connected.lock() {
            *conn_status = connected;
            if connected {
                info!("MCP server connection status set to connected");
            } else {
                info!("MCP server connection status set to disconnected");
            }
        }
    }
    
    // Get server connection status
    pub fn is_server_connected(&self) -> bool {
        *self.is_mcp_server_connected.lock().unwrap_or_else(|e| e.into_inner())
    }
}

// Implement McpClient trait for SimpleMcpClient
impl McpClient for SimpleMcpClient {
    // Connect to MCP server
    fn connect(&mut self, url: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_>> {
        let url = url.to_string();
        Box::pin(async move {
            self.url = url;
            Ok(())
        })
    }
    
    // Get available tool list
    fn get_tools(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<McpTool>, Error>> + Send + '_>> {
        let url = self.url.clone();
        let local_tools = self.available_tools.clone();
        let is_connected = self.is_mcp_server_connected.clone();
        Box::pin(async move {
            // First check connection status flag, return local tool list directly if not connected
            let connected = if let Ok(conn) = is_connected.lock() {
                *conn
            } else {
                false
            };

            if !connected {
                warn!("MCP server is not connected, returning local tools only");
                return Ok(local_tools);
            }
            
            if !url.is_empty() {
                // Construct JSON-RPC request
                let request = JSONRPCRequest {
                    jsonrpc: "2.0".to_string(),
                    id: Some(Value::String(Uuid::new_v4().to_string())),
                    method: "tools/list".to_string(),
                    params: None,
                };

                // Send HTTP POST request
                let client = reqwest::Client::new();
                let response = client
                    .post(&format!("{}/rpc", url))
                    .json(&request)
                    .send()
                    .await;

                // Check if request was sent successfully
                match response {
                    Ok(response) => {
                        // Check HTTP status code
                        if !response.status().is_success() {
                            let status = response.status();
                            let body = response.text().await.unwrap_or_else(|_| "Unable to read response body".to_string());
                            warn!("MCP server returned HTTP error {}: {}. Response body: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown error"), body);
                            // Return local tool list when server returns error
                            return Ok(local_tools);
                        }

                        // Get response text for debugging
                        let response_text = response.text().await
                            .map_err(|e| Error::msg(format!("Failed to read response body: {}", e)))?;
                        
                        // Check if response is empty
                        if response_text.trim().is_empty() {
                            warn!("MCP server returned empty response");
                            // Return local tool list when server returns empty response
                            return Ok(local_tools);
                        }

                        // Try to parse JSON
                        let rpc_response: JSONRPCResponse = serde_json::from_str(&response_text)
                            .map_err(|e| {
                                warn!("Failed to parse response as JSON: {}. Response content: {}", e, response_text);
                                // Return local tool list when JSON parsing fails
                                Error::msg(format!("Failed to parse response as JSON: {}. Response content: {}", e, response_text))
                            })?;
                        
                        // Check for errors
                        if let Some(error) = rpc_response.error {
                            warn!("JSON-RPC error: {} (code: {})", error.message, error.code);
                            // Return local tool list when JSON-RPC returns error
                            return Ok(local_tools);
                        }
                        
                        // Parse tool list
                        if let Some(result) = rpc_response.result {
                            debug!("Server response result: {:?}", result);
                            if let Some(tools_value) = result.get("tools") {
                                debug!("Tools value: {:?}", tools_value);
                                if let Ok(tools_array) = serde_json::from_value::<Vec<serde_json::Value>>(tools_value.clone()) {
                                    let mut tools = Vec::new();
                                    // First add local tools to tools
                                    tools.extend(local_tools);
                                    for tool_value in tools_array {
                                        debug!("Processing tool value: {:?}", tool_value);
                                        if let (Ok(name), Ok(description)) = (
                                            serde_json::from_value::<String>(tool_value["name"].clone()),
                                            serde_json::from_value::<String>(tool_value["description"].clone())
                                        ) {
                                            tools.push(McpTool {
                                                name,
                                                description,
                                            });
                                        } else {
                                            warn!("Failed to parse tool from server response: {:?}", tool_value);
                                        }
                                    }
                                    return Ok(tools);
                                } else {
                                    warn!("Failed to parse tools array from server response: {:?}", tools_value);
                                }
                            } else {
                                warn!("No 'tools' field in server response result: {:?}", result);
                            }
                        } else {
                            warn!("No result in JSON-RPC response");
                        }
                        
                        // Return local tool list if parsing fails
                        warn!("Failed to parse tools from server response");
                        Ok(local_tools)
                    }
                    Err(e) => {
                        // Return local tool list when unable to connect to server
                        warn!("Failed to send request to MCP server: {}", e);
                        Ok(local_tools)
                    }
                }
            } else {
                // Return local tool list if no URL is set
                Ok(local_tools)
            }
        })
    }
    
    // Call specified tool
    fn call_tool(&self, tool_name: &str, params: HashMap<String, Value>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Error>> + Send + '_>> {
        let url = self.url.clone();
        let tool_name = tool_name.to_string();
        let params = params.clone();
        let handler_opt = self.tool_handlers.get(&tool_name).cloned();
        Box::pin(async move {
            // Check if there is a custom tool handler
            if let Some(handler) = handler_opt {
                // If there is a custom handler, call it
                info!("Calling tool {} with params {:?}", tool_name, params);
                handler(params.clone()).await
            } else {
                // Otherwise send JSON-RPC request via HTTP
                if !url.is_empty() {
                    // Construct JSON-RPC request
                    let request = JSONRPCRequest {
                        jsonrpc: "2.0".to_string(),
                        id: Some(Value::String(Uuid::new_v4().to_string())),
                        method: "tools/call".to_string(),
                        params: Some(json!({
                            "name": tool_name,
                            "arguments": params
                        })),
                    };

                    // Send HTTP POST request
                    let client = reqwest::Client::new();
                    let response = client
                        .post(&format!("{}/rpc", url))
                        .json(&request)
                        .send()
                        .await?;

                    // Parse response
                    let rpc_response: JSONRPCResponse = response.json().await?;
                    
                    // Check for errors
                    if let Some(error) = rpc_response.error {
                        return Err(Error::msg(format!("JSON-RPC error: {} (code: {})", error.message, error.code)));
                    }
                    
                    // Return result
                    Ok(rpc_response.result.unwrap_or(Value::Null))
                } else {
                    // If no URL is set and no custom handler, use default processing logic
                    match tool_name.as_str() {
                        "get_weather" => {
                            // Bind default values to variables to extend lifetime
                            let default_city = Value::String("Beijing".to_string());
                            let city_value = params.get("city").unwrap_or(&default_city);
                            let city = city_value.as_str().unwrap_or("Beijing");
                            Ok(json!({
                                "city": city,
                                "temperature": "25°C",
                                "weather": "cloudy",
                                "humidity": "60%"
                            }))
                        },
                        _ => Err(Error::msg(format!("Unknown tool: {}", tool_name)))
                    }
                }
            }
        })
    }
    
    // Disconnect
    fn disconnect(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_>> {
        let url = self.url.clone();
        let is_connected = self.is_mcp_server_connected.clone();
        Box::pin(async move {
            // Simple implementation: simulate successful disconnection
            if let Ok(mut conn) = is_connected.lock() {
                *conn = false;
            }
            info!("Disconnected from MCP server at {}", url);
            Ok(())
        })
    }
    
    // Get tool response
    fn get_response(&self, tool_call_id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Error>> + Send + '_>> {
        let tool_call_id = tool_call_id.to_string();
        Box::pin(async move {
            // Simple implementation: return simulated tool response
            Ok(serde_json::json!({
                "tool_call_id": tool_call_id,
                "status": "completed",
                "response": {
                    "data": "Sample tool response data"
                }
            }))
        })
    }
    
    // Clone method
    fn clone(&self) -> Box<dyn McpClient> {
        // Manually create deep copy of available_tools
        let tools = self.available_tools.iter().map(|t| McpTool {
            name: t.name.clone(),
            description: t.description.clone()
        }).collect();
        
        // Copy tool handlers
        let tool_handlers = self.tool_handlers.clone();

        // Clone connection status
        let is_connected = if let Ok(conn) = self.is_mcp_server_connected.lock() {
            Arc::new(Mutex::new(*conn))
        } else {
            Arc::new(Mutex::new(false))
        };
        
        Box::new(SimpleMcpClient {
            url: self.url.clone(),
            available_tools: tools,
            tool_handlers,
            is_mcp_server_connected: is_connected,
        })
    }
    
    // Ping服务器
    fn ping(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_>> {
        let url = self.url.clone();
        Box::pin(async move {
            if !url.is_empty() {
                // 创建 ping 请求
                let request = JSONRPCRequest {
                    jsonrpc: "2.0".to_string(),
                    id: Some(Value::Number(serde_json::Number::from(1))),
                    method: "ping".to_string(),
                    params: None,
                };
            
                // 发送请求到服务器 - 使用正确的路径 /rpc
                let url = format!("{}/rpc", url);
                let client = reqwest::Client::new();
                let response = client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| Error::msg(format!("Failed to send ping request: {}", e)))?;
            
                // 检查响应状态
                if !response.status().is_success() {
                    return Err(Error::msg(format!("Ping request failed with status: {}", response.status())));
                }
            
                // 解析响应
                let response_text = response.text().await
                    .map_err(|e| Error::msg(format!("Failed to read response: {}", e)))?;
                let response_value: Value = serde_json::from_str(&response_text)
                    .map_err(|e| Error::msg(format!("Failed to parse response: {}", e)))?;
            
                // 检查响应中是否有错误
                if let Some(error) = response_value.get("error") {
                    if !error.is_null() {
                        return Err(Error::msg(format!("Ping request returned error: {}", error)));
                    }
                }
                
                // 检查是否有结果字段
                if let Some(_result) = response_value.get("result") {
                    // Ping 成功，返回空结果
                    Ok(())
                } else {
                    Err(Error::msg("No result in ping response"))
                }
            } else {
                Err(Error::msg("No URL set for MCP client"))
            }
        })
    }
}

// MCP client interface
pub trait McpClient: Send + Sync {
    // Connect to MCP server
    fn connect(&mut self, _url: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_>> {
        Box::pin(async move {
            // Simple implementation: simulate successful connection
            Ok(())
        })
    }
    
    // Get available tool list
    fn get_tools(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<McpTool>, Error>> + Send + '_>> {
        Box::pin(async move {
            // Simple implementation: return simulated tool list
            Ok(vec![McpTool {
                name: "example_tool".to_string(),
                description: "Example tool description".to_string()
            }])
        })
    }
    
    // Call specified tool
    fn call_tool(&self, tool_name: &str, params: HashMap<String, Value>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Error>> + Send + '_>> {
        let _tool_name = tool_name.to_string();
        let _params = params.clone();
        Box::pin(async move {
            // Default implementation returns error because trait doesn't know how to send HTTP requests
            Err(Error::msg("HTTP client not implemented in trait"))
        })
    }
    
    // Disconnect
    fn disconnect(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_>> {
        Box::pin(async move {
            // Simple implementation: simulate successful disconnection
            Ok(())
        })
    }
    
    // Get tool response
    fn get_response(&self, tool_call_id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Error>> + Send + '_>> {
        let tool_call_id = tool_call_id.to_string();
        Box::pin(async move {
            // Simple implementation: return simulated tool response
            Ok(serde_json::json!({
                "tool_call_id": tool_call_id,
                "status": "completed",
                "response": {
                    "data": "Sample tool response data"
                }
            }))
        })
    }
    
    // Clone method
    fn clone(&self) -> Box<dyn McpClient>;
    
    // Ping server
    fn ping(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + '_>> {
        Box::pin(async move {
            // Default implementation returns error because trait doesn't know how to send HTTP requests
            Err(Error::msg("HTTP client not implemented in trait"))
        })
    }
}
