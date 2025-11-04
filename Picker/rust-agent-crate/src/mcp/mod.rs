// MCP adapter implementation module definition
mod client;
mod adapter;
mod server;

// Re-export module content
pub use client::{McpClient, SimpleMcpClient, McpTool};
pub use adapter::McpToolAdapter;
pub use server::{McpServer, SimpleMcpServer};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct JSONRPCRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JSONRPCResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<JSONRPCError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JSONRPCError {
    code: i32,
    message: String,
}

// MCP Ping interface test
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_mcp_ping() {
        // create MCP server
        let server = SimpleMcpServer::new();
        
        // start MCP server
        let server_address = "127.0.0.1:6000";
        if let Err(e) = server.start(server_address).await {
            panic!("Failed to start MCP server: {}", e);
        }
        
        // wait for server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // create MCP client
        let mut client = SimpleMcpClient::new(format!("http://{}", server_address));
        
        // connect to MCP server
        if let Err(e) = client.connect(&format!("http://{}", server_address)).await {
            panic!("Failed to connect to MCP server: {}", e);
        }
        
        // send ping request
        match timeout(Duration::from_secs(5), client.ping()).await {
            Ok(Ok(_)) => {
                println!("Ping request succeeded!");
            }
            Ok(Err(e)) => {
                panic!("Ping request failed: {}", e);
            }
            Err(_) => {
                panic!("Ping request timed out");
            }
        }
        
        // stop MCP server
        if let Err(e) = server.stop().await {
            panic!("Failed to stop MCP server: {}", e);
        }
    }
}
