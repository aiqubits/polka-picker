// MCP server abstract definition
use anyhow::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::tools::Tool;
use serde::{Deserialize, Serialize};
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use serde_json::Value;
use log::{info, error};

use crate::mcp::JSONRPCRequest;
use crate::mcp::JSONRPCResponse;
use crate::mcp::JSONRPCError;

#[derive(Debug, Deserialize, Serialize)]
struct CallToolParams {
    name: String,
    arguments: Option<std::collections::HashMap<String, serde_json::Value>>,
}

// MCP server implementation
pub struct SimpleMcpServer {
    address: String,
    tools: Arc<Mutex<HashMap<String, Arc<dyn Tool>>>>,
    is_running: Arc<Mutex<bool>>,
    server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl SimpleMcpServer {
    pub fn new() -> Self {
        Self {
            address: "127.0.0.1:6000".to_string(),
            tools: Arc::new(Mutex::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(false)),
            server_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn with_address(mut self, address: String) -> Self {
        self.address = address;
        self
    }
}

// Simple test handler
#[axum::debug_handler]
async fn test_handler() -> &'static str {
    "Hello, Rust-Agent!"
}

// Handle JSON-RPC request
#[axum::debug_handler]
async fn handle_jsonrpc_request(
    State(state): State<Arc<SimpleMcpServerState>>,
    Json(payload): Json<JSONRPCRequest>,
) -> Json<JSONRPCResponse> {
    let response = match payload.method.as_str() {
        "tools/call" => {
            // Handle tool call request
            match handle_tool_call(state, payload.params).await {
                Ok(result) => {
                    JSONRPCResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Some(payload.id.unwrap_or(Value::Null)),
                        result: Some(result),
                        error: None,
                    }
                }
                Err(e) => {
                    JSONRPCResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Some(payload.id.unwrap_or(Value::Null)),
                        result: None,
                        error: Some(JSONRPCError {
                            code: -32603,
                            message: e.to_string(),
                        }),
                    }
                }
            }
        }
        "ping" => {
            // Handle ping request
            JSONRPCResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(payload.id.unwrap_or(Value::Null)),
                result: Some(Value::Object(serde_json::Map::new())),
                error: None,
            }
        }
        "tools/list" => {
            // Handle tool list request
            match handle_list_tools(state).await {
                Ok(result) => {
                    JSONRPCResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Some(payload.id.unwrap_or(Value::Null)),
                        result: Some(result),
                        error: None,
                    }
                }
                Err(e) => {
                    JSONRPCResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Some(payload.id.unwrap_or(Value::Null)),
                        result: None,
                        error: Some(JSONRPCError {
                            code: -32603,
                            message: e.to_string(),
                        }),
                    }
                }
            }
        }
        _ => {
            // Unsupported method
            JSONRPCResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(payload.id.unwrap_or(Value::Null)),
                result: None,
                error: Some(JSONRPCError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
            }
        }
    };
    
    Json(response)
}

async fn handle_list_tools(
    state: Arc<SimpleMcpServerState>,
) -> Result<serde_json::Value, Error> {
    // Get all registered tools
    let tools_map = state.tools.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
    
    // Convert to tool format required by MCP protocol
    let mut tools_list = Vec::new();
    for (_, tool) in tools_map.iter() {
        let mcp_tool = serde_json::json!({
            "name": tool.name(),
            "description": tool.description(),
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        });
        tools_list.push(mcp_tool);
    }
    
    // Construct response
    let result = serde_json::json!({
        "tools": tools_list
    });
    
    Ok(result)
}

async fn handle_tool_call(
    state: Arc<SimpleMcpServerState>,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    // Parse parameters
    let call_params: CallToolParams = serde_json::from_value(params.unwrap_or(serde_json::Value::Null))
        .map_err(|e| Error::msg(format!("Invalid parameters: {}", e)))?;
    
    // Find tool and get its Arc reference
    let tool = {
        let tools = state.tools.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
        tools.get(&call_params.name)
            .ok_or_else(|| Error::msg(format!("Tool '{}' not found", call_params.name)))?
            .clone()
    };
    
    // Prepare tool input parameters
    let input_str = if let Some(args) = call_params.arguments {
        serde_json::to_string(&args)?
    } else {
        "{}".to_string()
    };
    
    // Call tool (now can be called without holding the lock)
    let result = tool.invoke(&input_str).await?;
    Ok(serde_json::Value::String(result))
}

// Server state structure
#[derive(Clone)]
struct SimpleMcpServerState {
    tools: Arc<Mutex<HashMap<String, Arc<dyn Tool>>>>,
}

// MCP server abstraction
#[async_trait::async_trait]
pub trait McpServer: Send + Sync {
    // Start MCP server
    async fn start(&self, address: &str) -> Result<(), Error>;
    
    // Register tool to MCP server
    fn register_tool(&self, tool: Arc<dyn Tool>) -> Result<(), Error>;
    
    // Stop MCP server
    async fn stop(&self) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl McpServer for SimpleMcpServer {
    // Start MCP server
    async fn start(&self, address: &str) -> Result<(), Error> {
        info!("Starting MCP server on {}", address);
        
        // Create server state
        let state = Arc::new(SimpleMcpServerState {
            tools: self.tools.clone(),
        });
        
        // Create routes
        let app = Router::new()
            .route("/rpc", post(handle_jsonrpc_request))
            .route("/test", get(test_handler))
            .with_state(state)
            .layer(CorsLayer::permissive()); // Allow all CORS requests
        
        // Start server
        let listener = TcpListener::bind(address).await
            .map_err(|e| Error::msg(format!("Failed to bind to address {}: {}", address, e)))?;
        
        info!("MCP server listening on http://{}", address);
        
        // Run server in background task
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app.into_make_service()).await {
                error!("Server error: {}", e);
            }
        });
        
        // Update server status
        {
            let mut is_running = self.is_running.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
            *is_running = true;
        }
        
        // Save server handle
        {
            let mut server_handle = self.server_handle.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
            *server_handle = Some(handle);
        }
        
        Ok(())
    }
    
    // Register tool to MCP server
    fn register_tool(&self, tool: Arc<dyn Tool>) -> Result<(), Error> {
        let name = tool.name().to_string();
        let mut tools = self.tools.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
        tools.insert(name, tool);
        Ok(())
    }
    
    // Stop MCP server
    async fn stop(&self) -> Result<(), Error> {
        info!("Stopping MCP server");
        
        // Update server status
        {
            let mut is_running = self.is_running.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
            *is_running = false;
        }
        
        // Cancel server task
        {
            let mut server_handle = self.server_handle.lock().map_err(|e| Error::msg(format!("Failed to acquire lock: {}", e)))?;
            if let Some(handle) = server_handle.take() {
                handle.abort();
            }
        }
        
        Ok(())
    }
}