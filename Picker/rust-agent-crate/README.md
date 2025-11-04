# Rust Agent: Next Generation AI Agent Framework

Rust Agent is a powerful and flexible AI Agent framework. It provides a comprehensive set of tools and components for building complex AI agents that can interact with various systems and perform complex tasks.

## Features

- **Modular Architecture**: Clear separation of concerns with well-defined modules for agents, tools, memory, models, and more
- **MCP Integration**: Built-in support for Model Context Protocol (MCP) client and server implementations
- **Flexible Tool System**: Extensible tool interface supporting custom implementations and MCP tool adapters
- **Multi-Model Support**: Integration with various AI models, including OpenAI-compatible APIs
- **Memory Management**: Built-in memory components for maintaining context between interactions
- **Asynchronous Design**: Fully asynchronous architecture leveraging Tokio for high-performance operations
- **Error Handling**: Comprehensive error handling using the anyhow crate
- **Hybrid Mode**: Support for mixed use of local tools and remote MCP server tools

## Architecture Overview

The framework consists of several key modules:

### 1. Core Layer
Defines the fundamental `Runnable` trait and related components, forming the foundation for all executable components in the framework.

### 2. Models Layer
Provides interfaces and implementations for various AI models:
- `ChatModel`: Chat-based model interface
- `OpenAIChatModel`: OpenAI-compatible API implementation

### 3. Agents Layer
Implements core agent logic with `Agent` and `AgentRunner` interfaces:
- `McpAgent`: Main agent implementation with MCP service integration
- `SimpleAgent`: Basic agent implementation for simple use cases

### 4. Tools Layer
Defines tool interfaces and implementation mechanisms:
- `Tool`: Core tool interface
- `Toolkit`: Interface for managing related tool groups
- `McpToolAdapter`: Adapter for integrating MCP tools with the framework's tool system

### 5. MCP Integration Layer
Provides components for interacting with MCP services:
- `McpClient`: Interface for MCP client implementations
- `SimpleMcpClient`: Basic MCP client implementation
- `McpServer`: Interface for MCP server implementations
- `SimpleMcpServer`: Basic MCP server implementation

### 6. Memory Layer
Provides memory management components:
- `BaseMemory`: Base memory interface
- `SimpleMemory`: Simple memory implementation
- `MessageHistoryMemory`: Message history memory implementation
- `SummaryMemory`: Summary memory implementation
- `CompositeMemory`: Composite memory implementation combining multiple memory strategies

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rust-agent = "0.0.5"
```

## Quick Start

Here's a simple example of creating an AI agent using the framework:

```rust
use rust_agent::{McpAgent, SimpleMcpClient, McpTool, ChatMessage, ChatMessageContent, AgentOutput};
use std::sync::Arc;
use std::collections::HashMap;

// Create MCP client
let mut mcp_client = SimpleMcpClient::new("http://localhost:6000".to_string());

// Add some MCP tools
mcp_client.add_tools(vec![
    McpTool {
        name: "get_weather".to_string(),
        description: "Get weather information for a specified city".to_string(),
    }
]);

// Wrap MCP client in Arc
let mcp_client_arc = Arc::new(mcp_client);

// Create McpAgent instance
let mut agent = McpAgent::new(
    mcp_client_arc.clone(),
    "You are a helpful assistant".to_string()
);

// Automatically add tools from MCP client
if let Err(e) = agent.auto_add_tools().await {
    println!("Failed to automatically add tools to McpAgent: {}", e);
}

// Build user input
let mut input = HashMap::new();
input.insert("input".to_string(), "What's the weather like in Beijing?".to_string());

// Call agent to process input
let result = agent.invoke(input).await;

// Handle result
match result {
    Ok(AgentOutput::Finish(finish)) => {
        if let Some(answer) = finish.return_values.get("answer") {
            println!("AI Response: {}", answer);
        }
    },
    Ok(AgentOutput::Action(action)) => {
        println!("Need to call tool: {}", action.tool);
        // Execute tool call...
        if let Some(thought) = &action.thought {
            println!("Thought process: {}", thought);
        }
    },
    Err(e) => {
        println!("Error occurred: {}", e);
    }
}
```

## MCP Server Implementation

The framework now includes built-in MCP server implementation:

```rust
use rust_agent::{SimpleMcpServer, McpServer, ExampleTool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP server instance
    let server = SimpleMcpServer::new().with_address("127.0.0.1:6000".to_string());
    
    // Create example tools
    let weather_tool = ExampleTool::new(
        "get_weather".to_string(),
        "Get weather information for a specified city".to_string()
    );
    
    let calculator_tool = ExampleTool::new(
        "calculate".to_string(),
        "Perform simple mathematical calculations".to_string()
    );
    
    // Register tools with server
    server.register_tool(Box::new(weather_tool))?;
    server.register_tool(Box::new(calculator_tool))?;
    
    // Start server
    server.start("127.0.0.1:6000").await?;
    
    println!("MCP server started at 127.0.0.1:6000");
    println!("Registered tools: get_weather, calculate");
    
    // Simulate server running for some time
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Stop server
    server.stop().await?;
    println!("MCP server stopped");
    
    Ok(())
}
```

### Hybrid Mode Example

The framework supports hybrid mode, allowing simultaneous use of local tools and remote MCP server tools:

```rust
use rust_agent::{run_agent, OpenAIChatModel, McpClient, SimpleMcpClient, McpTool, McpAgent, CompositeMemory};
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    // Create OpenAI model instance
    let model = OpenAIChatModel::new(api_key, base_url)
        .with_model("gpt-3.5-turbo")
        .with_temperature(0.7);
    
    // Initialize MCP client
    let mut mcp_client = SimpleMcpClient::new("http://127.0.0.1:6000".to_string());
    
    // Add local tools
    mcp_client.add_tools(vec![
        McpTool {
            name: "get_local_time".to_string(),
            description: "Get current local time and date".to_string(),
        },
    ]);
    
    // Register local tool handler
    mcp_client.register_tool_handler("get_local_time".to_string(), |_params: HashMap<String, Value>| async move {
        let now = chrono::Local::now();
        Ok(json!({
            "current_time": now.format("%Y-%m-%d %H:%M:%S").to_string(),
            "timezone": "Local"
        }))
    });
    
    // Connect to MCP server
    if let Ok(_) = mcp_client.connect("http://127.0.0.1:6000").await {
        mcp_client.set_server_connected(true);
    }
    
    // Create memory module
    let memory = CompositeMemory::with_basic_params("data".into(), 200, 10).await.unwrap();
    
    // Create Agent instance
    let client_arc: Arc<dyn McpClient> = Arc::new(mcp_client);
    let mut agent = McpAgent::with_openai_model_and_memory(
        client_arc.clone(),
        "You are an AI assistant that can use both local tools and remote MCP server tools.".to_string(),
        model,
        Box::new(memory)
    );
    
    // Automatically get tools from MCP client and add to Agent
    if let Err(e) = agent.auto_add_tools().await {
        eprintln!("Warning: Failed to automatically add tools to McpAgent: {}", e);
    }
    
    // Use Agent to process user input
    match run_agent(&agent, "What time is it now?".to_string()).await {
        Ok(response) => println!("Assistant: {}", response),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Creating Custom Tools

To create custom tools for MCP servers, you need to implement the `Tool` trait:

```rust
use rust_agent::Tool;
use anyhow::Error;
use std::pin::Pin;

pub struct CustomTool {
    name: String,
    description: String,
}

impl CustomTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl Tool for CustomTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn invoke(&self, input: &str) -> Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send + '_>> {
        let input_str = input.to_string();
        let name = self.name.clone();
        
        Box::pin(async move {
            // Your custom tool logic
            Ok(format!("Custom tool {} processed: {}", name, input_str))
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

## Examples

The project provides several examples demonstrating how to use the framework to build different types of AI agents. Examples are located in the `examples/` directory.

- `agent_example.rs`: Basic agent usage example
- `mcp_agent_client_chatbot.rs`: MCP client chatbot example (server-side tools only)
- `mcp_agent_hybrid_chatbot.rs`: Hybrid mode MCP agent example (local get_local_time tool + server-side tools)
- `mcp_agent_local_chatbot.rs`: Local MCP agent chatbot example (local tools only)
- `mcp_server_complete_example.rs`: Complete MCP server example with real tool implementations (providing get_weather and simple_calculate tools)

### 1. Basic Agent Example (`agent_example.rs`)

Shows how to create a simple agent with custom tools:

```bash
# Run example
cargo run --example agent_example
```

### 2. MCP Client Chatbot (`mcp_agent_client_chatbot.rs`)

Demonstrates how to use `McpAgent` to build a simple chatbot that connects to an MCP server and uses only server-side tools. This example shows a pure client implementation, completely relying on remote tools:

- No local tools implemented
- All tools are provided by the MCP server (e.g., `get_weather`, `simple_calculate`)

```bash
# Run example
cargo run --example mcp_agent_client_chatbot
```

### 3. Hybrid Mode MCP Agent Chatbot (`mcp_agent_hybrid_chatbot.rs`)

Demonstrates how to use `McpAgent` in hybrid mode, combining local tools (like get_local_time) with server-side tools. This example shows how an agent can use both local and remote tools based on task requirements:

- Local tool: `get_local_time` - Get current local time and date
- Remote tools: All tools provided by the MCP server (e.g., `get_weather`, `simple_calculate`)

```bash
# Run example
cargo run --example mcp_agent_hybrid_chatbot
```

### 4. Local MCP Agent Chatbot (`mcp_agent_local_chatbot.rs`)

Demonstrates how to use `McpAgent` with only local tools. This example shows how an agent can run without connecting to any remote MCP server, using only locally implemented tools:

- Local tools: 
  - `get_weather` - Get weather information for a specified city
  - `simple_calculate` - Perform simple mathematical calculations

```bash
# Run example
cargo run --example mcp_agent_local_chatbot
```

### 5. Complete MCP Server (`mcp_server_complete_example.rs`)

A more complete example showing how to implement custom tools with actual functionality, such as get_weather and simple_calculate:

```bash
# Run example
cargo run --example mcp_server_complete_example
```

## Running Tests

The project includes unit tests to verify the framework's functionality:

```bash
# Run all tests
cargo test
```

## Building the Project

To build the project, simply run:

```bash
# Build project
cargo build
```

The project builds successfully with only some warnings about unused fields in structs, which do not affect functionality.

## Configuration and Environment Variables

When using the framework, you may need to configure the following environment variables:

- `OPENAI_API_KEY`: OpenAI compatible API key
- `OPENAI_API_URL`: OpenAI compatible API base URL (optional, defaults to official OpenAI API)
- `OPENAI_API_MODEL`: OpenAI compatible API model name (optional, defaults to gpt-3.5-turbo)
- `MCP_URL`: MCP server URL (optional, defaults to http://127.0.0.1:6000)

## Notes

- The framework uses an asynchronous programming model and requires the Tokio runtime
- Tool calls need to implement the `Tool` interface or use `McpToolAdapter`
- The current version may have some unimplemented features or simplified implementations, please be aware when using

## Development and Contributing

If you'd like to contribute to the project, please follow these steps:

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

[GPL-3.0](LICENSE)