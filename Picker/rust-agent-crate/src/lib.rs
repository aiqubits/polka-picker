// Rust Agent: AI Agent framework aligned with LangChain-Core

mod core;
mod models;
pub mod tools;
pub mod memory;
mod agents;
mod callbacks;
mod mcp;

// Re-export main components for external use
pub use core::{Runnable, RunnableExt, RunnableSequence};
pub use models::{ChatModel, ChatMessage as ModelChatMessage, ChatMessageContent, ChatCompletion, TokenUsage, OpenAIChatModel};
pub use tools::{Tool, Toolkit, ExampleTool, ExampleToolkit, find_matching_tool_index, parse_model_output};
pub use memory::{BaseMemory, SimpleMemory, MessageHistoryMemory, SummaryMemory, CompositeMemory, CompositeMemoryConfig, ChatMessageRecord, ChatMessage};
pub use agents::{Agent, McpAgent, AgentAction, AgentFinish, AgentOutput, AgentRunner, SimpleAgent, SimpleAgentRunner};
pub use callbacks::CallbackHandler;
pub use mcp::{McpClient, SimpleMcpClient, McpTool, McpToolAdapter, McpServer, SimpleMcpServer};
use anyhow::Error;
use std::collections::HashMap;

// Utility functions and error handling
pub use core::pipe;
// Export anyhow error handling library to ensure consistent error handling for third-party users
pub use anyhow;

// Main function to run Agent
pub async fn run_agent(agent: &McpAgent, input: String) -> Result<String, Error> {
    let mut inputs = HashMap::new();
    inputs.insert("input".to_string(), input);
    let output = agent.invoke(inputs).await?;
    
    match output {
        AgentOutput::Action(action) => {
            // Find the corresponding tool using fuzzy matching mechanism
            let tools = agent.tools();
            match find_matching_tool_index(&tools, &action.tool) {
                Some(matched_name) => {
                    // After finding a matching tool name, search for the specific tool again
                    if let Some(tool) = tools.iter().find(|t| t.name() == matched_name) {
                        // Invoke the tool
                        let tool_result = tool.invoke(&action.tool_input).await?;
                        
                        // Feed the tool execution result back to Agent for further processing
                        let mut new_inputs = HashMap::new();
                        new_inputs.insert("input".to_string(), format!("[CUSTOMIZE_TOOL_RESULT] {{\"tool\": \"{}\", \"result\": {}}}", matched_name, tool_result));
                        let new_output = agent.invoke(new_inputs).await?;
                        
                        match new_output {
                            AgentOutput::Finish(finish) => {
                                Ok(finish.return_values.get("answer").map(|s| s.clone()).unwrap_or_else(|| "".to_string()))
                            },
                            _ => {
                                // If still Action, simply return the tool result for now
                                Ok(format!("Tool {} executed successfully, result: {}", matched_name, tool_result))
                            }
                        }
                    } else {
                        Err(Error::msg(format!("Tool {} does not exist", matched_name)))
                    }
                },
                None => {
                    Err(Error::msg(format!("Tool {} does not exist", action.tool)))
                }
            }
        },
        AgentOutput::Finish(finish) => {
            Ok(finish.return_values.get("answer").map(|s| s.clone()).unwrap_or_else(|| "".to_string()))
        },
    }
}
