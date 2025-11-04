use anyhow::anyhow;
use std::pin::Pin;
use std::sync::Arc;
use log::info;

use crate::{
    Agent, AgentAction, AgentFinish, AgentOutput, BaseMemory, ModelChatMessage, ChatMessageContent, ChatModel,
    McpClient, McpToolAdapter, OpenAIChatModel, Runnable, Tool, parse_model_output
};
use serde_json::Value;

/// McpAgent is an intelligent agent implementation based on MCP services
/// It can connect to MCP servers, process user inputs, call tools, and generate responses
pub struct McpAgent {
    client: Arc<dyn McpClient>,
    tools: Vec<Box<dyn Tool + Send + Sync>>,
    system_prompt: String,
    openai_model: Option<OpenAIChatModel>,
    memory: Option<Box<dyn BaseMemory>>,
}

impl McpAgent {
    /// Create a new McpAgent instance
    pub fn new(client: Arc<dyn McpClient>, system_prompt: String) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt,
            openai_model: None, // Default to not setting OpenAI model
            memory: None, // Default to not setting memory module
        }
    }
    
    /// Create a new McpAgent instance with specified OpenAIChatModel
    pub fn with_openai_model(client: Arc<dyn McpClient>, system_prompt: String, openai_model: OpenAIChatModel) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt,
            openai_model: Some(openai_model),
            memory: None, // Default to not setting memory module
        }
    }
    
    /// Create a new McpAgent instance with specified memory module
    pub fn with_memory(client: Arc<dyn McpClient>, system_prompt: String, memory: Box<dyn BaseMemory>) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt,
            openai_model: None,
            memory: Some(memory),
        }
    }

    /// Create a new McpAgent instance with specified OpenAIChatModel and memory module
    pub fn with_openai_model_and_memory(client: Arc<dyn McpClient>, system_prompt: String, openai_model: OpenAIChatModel, memory: Box<dyn BaseMemory>) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt,
            openai_model: Some(openai_model),
            memory: Some(memory),
        }
    }
    
    /// Get a reference to the memory module
    pub fn get_memory(&self) -> Option<&Box<dyn BaseMemory>> {
        self.memory.as_ref()
    }

    /// Add a tool to the Agent
    pub fn add_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.push(tool);
    }
    
    /// Automatically get tools from MCP client and add them to the Agent
    /// This method gets all available tools from the MCP client and wraps them as McpToolAdapter before adding to the Agent
    /// Local tool registration and addition are handled by the caller
    pub async fn auto_add_tools(&mut self) -> Result<(), anyhow::Error> {
        use crate::McpToolAdapter;
        
        // Get tool list from MCP client
        let tools = self.client.get_tools().await?;

        // Print information about the obtained tools
        for tool in &tools {
            info!("MCP Client Get Tool: {} - {}", tool.name, tool.description);
        }
        
        // Wrap each tool as McpToolAdapter and add to the Agent
        for tool in tools {
            let tool_adapter = McpToolAdapter::new(
                self.client.clone(),
                tool
            );
            self.add_tool(Box::new(tool_adapter));
        }
        
        Ok(())
    }
}

impl Agent for McpAgent {
    fn tools(&self) -> Vec<Box<dyn Tool + Send + Sync>> {
        // Return a cloned version of the tool list
        // To solve the problem that Box<dyn Tool> cannot be directly cloned, we create new tool adapter instances
        let mut cloned_tools: Vec<Box<dyn Tool + Send + Sync>> = Vec::new();

        // Since McpToolAdapter can be recreated through client and McpTool,
        // we iterate through existing tools and create new adapter instances for each tool
        for tool in &self.tools {
            // Check if the tool is of type McpToolAdapter
            if let Some(mcp_tool_adapter) = tool.as_any().downcast_ref::<McpToolAdapter>() {
                // Recreate McpToolAdapter instance
                let cloned_adapter = McpToolAdapter::new(
                    mcp_tool_adapter.get_client(),
                    mcp_tool_adapter.get_mcp_tool(),
                );
                cloned_tools.push(Box::new(cloned_adapter));
            } else {
                // For other types of tools, we skip or need to implement other cloning mechanisms
                // Here we can add logs or error handling
                info!(
                    "Warning: Unable to clone non-McpToolAdapter type tool: {}",
                    tool.name()
                );
            }
        }

        cloned_tools
    }

    fn execute(
        &self,
        _action: &AgentAction,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + Send + '_>,
    > {
        Box::pin(async move {
            // In practical applications, there should be a mechanism here to find and call tools
            // Since we cannot clone the tool list, we simplify the implementation here
            Err(anyhow!("Tool execution functionality is not implemented yet"))
        })
    }

    fn clone_agent(&self) -> Box<dyn Agent> {
        // Create a new McpAgent instance, copy basic fields, but do not copy tools (simplified implementation)
        let new_agent = McpAgent::new(
            self.client.clone(),
            self.system_prompt.clone(),
        );

        // Note: We do not copy tools here because Box<dyn Tool> cannot be directly cloned
        Box::new(new_agent)
    }
}

impl Clone for McpAgent {
    fn clone(&self) -> Self {
        // Create a new McpAgent instance, but do not copy the tool list (simplified implementation)
        Self {
            client: Arc::clone(&self.client),
            tools: Vec::new(), // Do not copy tools because Box<dyn Tool> cannot be directly cloned
            system_prompt: self.system_prompt.clone(),
            openai_model: self.openai_model.clone(), // Clone OpenAI model instance
            memory: self.memory.clone(), // Clone memory module
        }
    }
}

impl Runnable<std::collections::HashMap<String, String>, AgentOutput> for McpAgent {
    fn invoke(
        &self,
        input: std::collections::HashMap<String, String>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<AgentOutput, anyhow::Error>> + Send>> {
        // Capture system prompt in advance
        let system_prompt = self.system_prompt.clone();
        let input_text = input
            .get("input")
            .cloned()
            .unwrap_or_default()
            .to_string()
            .trim()
            .to_string();

        // Capture tool descriptions in advance to avoid using self in async move
        let tool_descriptions: String = if !self.tools.is_empty() {
            let mut descriptions = String::new();
            for tool in &self.tools {
                descriptions.push_str(&format!("- {}: {}\n", tool.name(), tool.description()));
            }
            descriptions
        } else {
            String::new()
        };

        // Capture memory module in advance to avoid using self in async move
        let memory_clone = self.memory.clone();

        // Build enhanced system prompt using ReAct framework format
        let enhanced_system_prompt = if !tool_descriptions.is_empty() {
            format!("{}
You are an AI assistant that follows the ReAct (Reasoning and Acting) framework. 
You should think step by step and decide whether to use tools based on user needs.
You should carefully review and when confirming the use of the tool, if there are omissions, errors, or other issues with the parameters, you should reply and remind the user.
Available tools:\n{}\n\nWhen you need to use a tool, please respond in the following JSON format:
            \n{{\"call_tool\": {{\"name\": \"Tool Name\", \"parameters\": {{\"parameter_name\": \"parameter_value\"}}}}}}
        When you don't need to use a tool, please respond in the following JSON format:\n{{\"content\": \"Your answer\"}}
        Please think carefully about whether the user's request requires a tool to be used, and only use tools when necessary.", 
            system_prompt, tool_descriptions)
        } else {
            system_prompt
        };

        // Capture OpenAI model instance in advance to avoid using self in async move
        let openai_model_clone = self.openai_model.clone();

        Box::pin(async move {
            // Check if input is empty
            if input_text.is_empty() {
                let mut return_values = std::collections::HashMap::new();
                return_values.insert("answer".to_string(), "Please enter valid content".to_string());
                // Get model name from OpenAI model, use default value if not available
                let model_name = if let Some(ref openai_model) = openai_model_clone {
                    openai_model.model_name().map(|s| s.to_string()).unwrap_or("unknown".to_string())
                } else {
                    "unknown".to_string()
                };
                return_values.insert("model".to_string(), model_name);
                return Ok(AgentOutput::Finish(AgentFinish { return_values }));
            }

            // Use the passed OpenAI model instance or create a new instance
            let model = if let Some(ref openai_model) = openai_model_clone {
                // Use the passed OpenAI model instance
                openai_model
            } else {
                // If no OpenAI model instance is provided, return an error
                let mut return_values = std::collections::HashMap::new();
                return_values.insert("answer".to_string(), "No OpenAI model provided".to_string());
                return_values.insert("model".to_string(), "unknown".to_string());
                return Ok(AgentOutput::Finish(AgentFinish { return_values }));
            };

            // Build message list
            let mut messages = Vec::new();

            // Get summary content and append to system prompt
            let enhanced_system_prompt_with_summary = {
                let mut enhanced_prompt = enhanced_system_prompt;
                if let Some(memory) = &memory_clone {
                    // Try to get summary content from memory module
                    // Here we use downcast_ref to check if it's CompositeMemory type
                    if let Some(composite_memory) = memory.as_any().downcast_ref::<crate::memory::composite_memory::CompositeMemory>() {
                        // If it's CompositeMemory, call get_summary method to get summary
                        match composite_memory.get_summary().await {
                            Ok(Some(summary)) => {
                                // Append summary content to system prompt
                                enhanced_prompt = format!("{}\n\nPrevious conversation summary: {}", enhanced_prompt, summary);
                                log::info!("Summary appended to system prompt");
                            },
                            Ok(None) => {
                                log::info!("No summary content found");
                            },
                            Err(e) => {
                                log::warn!("Error getting summary: {}", e);
                            }
                        }
                    } else {
                        // If not CompositeMemory, try to get summary from memory variables
                        match memory.load_memory_variables(&std::collections::HashMap::new()).await {
                            Ok(memories) => {
                                if let Some(summary) = memories.get("summary") {
                                    if let Some(summary_str) = summary.as_str() {
                                        // Append summary content to system prompt
                                        enhanced_prompt = format!("{}\n\nPrevious conversation summary: {}", enhanced_prompt, summary_str);
                                        log::info!("Summary retrieved from memory variables and appended to system prompt");
                                    }
                                }
                            },
                            Err(e) => {
                                log::warn!("Error getting summary from memory variables: {}", e);
                            }
                        }
                    }
                }
                enhanced_prompt
            };

            // Add system message
            messages.push(ModelChatMessage::System(ChatMessageContent {
                content: enhanced_system_prompt_with_summary,
                name: None,
                additional_kwargs: std::collections::HashMap::new(),
            }));

            // If there is a memory module, load memory variables and add them to the message list
            if let Some(memory) = &memory_clone {
                match memory.load_memory_variables(&std::collections::HashMap::new()).await {
                    Ok(memories) => {
                        info!("Loaded memory variables: {:?}", memories);
                        if let Some(chat_history) = memories.get("chat_history") {
                            if let serde_json::Value::Array(messages_array) = chat_history {
                                for message in messages_array {
                                    if let serde_json::Value::Object(msg_obj) = message {
                                        let role = msg_obj.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
                                        let content = msg_obj.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                        
                                        // Skip empty content messages
                                        if content.trim().is_empty() {
                                            continue;
                                        }
                                        
                                        // Skip assistant messages containing complete history messages
                                        if role == "assistant" && content.contains("user:") && content.contains("assistant:") {
                                            continue;
                                        }
                                        
                                        // Add debug log
                                        // info!("Loaded message: role={}, content={}", role, content);
                                        
                                        match role {
                                            "human" | "user" => {
                                                // Add debug log
                                                log::info!("Loaded human message: content={}", content);
                                                messages.push(ModelChatMessage::Human(ChatMessageContent {
                                                    content: content.to_string(),
                                                    name: None,
                                                    additional_kwargs: std::collections::HashMap::new(),
                                                }));
                                            },
                                            "ai" | "assistant" => {
                                                // Add debug log
                                                log::info!("Loaded AI message: content={}", content);
                                                messages.push(ModelChatMessage::AIMessage(ChatMessageContent {
                                                    content: content.to_string(),
                                                    name: None,
                                                    additional_kwargs: std::collections::HashMap::new(),
                                                }));
                                            },
                                            "tool" => {
                                                // Handle tool messages
                                                let content_str = content.to_string();
                                                // Add debug log
                                                log::info!("Loaded tool message: content={}", content_str);
                                                messages.push(ModelChatMessage::ToolMessage(ChatMessageContent {
                                                    content: content_str,
                                                    name: None,
                                                    additional_kwargs: std::collections::HashMap::new(),
                                                }));
                                            },
                                            _ => {
                                                // Add debug log
                                                log::info!("Loaded unknown role message: role={}, content={}", role, content);
                                                // Ignore messages with unknown roles
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        // If loading memory fails, log the error but continue execution
                        log::warn!("Failed to load memory variables: {}", e);
                    }
                }
            }

            // Add current user message
            messages.push(ModelChatMessage::Human(ChatMessageContent {
                content: input_text.clone(),
                name: None,
                additional_kwargs: std::collections::HashMap::new(),
            }));
            // info!("Added current user message: role=user, content={}", input_text);
            
            // Add debug log, showing all messages
            log::info!("Messages to be sent to model:");
            for (i, msg) in messages.iter().enumerate() {
                match msg {
                    ModelChatMessage::System(content) => {
                        log::info!("  {}. role=system, content={}", i+1, content.content);
                    },
                    ModelChatMessage::Human(content) => {
                        log::info!("  {}. role=user, content={}", i+1, content.content);
                    },
                    ModelChatMessage::AIMessage(content) => {
                        log::info!("  {}. role=assistant, content={}", i+1, content.content);
                    },
                    ModelChatMessage::ToolMessage(content) => {
                        log::info!("  {}. role=tool, content={}", i+1, content.content);
                    },
                }
            }

            // Call the language model
            let result = model.invoke(messages).await;

            match result {
                Ok(completion) => {
                    // Parse model output
                    let content = match completion.message {
                        ModelChatMessage::AIMessage(content) => content.content,
                        _ => { format!("{},{:?}", "Non-AI message received", completion.message) }
                    };

                    // Get model name from OpenAI model, use default value if not available
                    let model_name = model.model_name().map(|s| s.to_string()).unwrap_or("unknown".to_string());

                    // If there is a memory module, save the current conversation to memory
                    if let Some(memory) = &memory_clone {
                        let mut inputs = std::collections::HashMap::new();
                        inputs.insert("input".to_string(), serde_json::Value::String(input_text.clone()));
                        
                        // Preprocess content, if it's JSON string format, extract the content field
                        let processed_content = if content.starts_with('"') && content.ends_with('"') {
                            // Try to parse as JSON string
                            match serde_json::from_str::<serde_json::Value>(&content) {
                                Ok(serde_json::Value::String(s)) => s,
                                _ => content.clone(),
                            }
                        } else if content.starts_with('{') && content.ends_with('}') {
                            // Try to parse as JSON object
                            match serde_json::from_str::<serde_json::Value>(&content) {
                                Ok(json_obj) => {
                                    // If it's a JSON object, try to extract the content field
                                    if let Some(content_value) = json_obj.get("content") {
                                        if let Some(content_str) = content_value.as_str() {
                                            content_str.to_string()
                                        } else {
                                            content.clone()
                                        }
                                    } else {
                                        content.clone()
                                    }
                                },
                                _ => content.clone(),
                            }
                        } else {
                            content.clone()
                        };
                        
                        let mut outputs = std::collections::HashMap::new();
                        outputs.insert("output".to_string(), serde_json::Value::String(processed_content));
                        
                        if let Err(e) = memory.save_context(&inputs, &outputs).await {
                            log::warn!("Failed to save context to memory: {}", e);
                        }
                    }

                    // Parse model output, determine if tool call is needed
                    // Here should correctly parse the JSON format of model output
                    if let Ok(parsed_output) = parse_model_output(&content) {
                        match parsed_output {
                            AgentOutput::Action(action) => {
                                // Directly return the Action parsed by the model
                                return Ok(AgentOutput::Action(action));
                            }
                            AgentOutput::Finish(_) => {
                                // Directly return the answer
                                let mut return_values = std::collections::HashMap::new();
                                return_values.insert("answer".to_string(), content.clone());
                                return_values.insert("model".to_string(), model_name);
                                return Ok(AgentOutput::Finish(AgentFinish { return_values }));
                            }
                        }
                    } else {
                        // If parsing fails, try to extract tool call information
                        // Check if tool call keywords are included
                        if content.contains("call_tool") {
                            // Try to extract JSON format tool call from content
                            // Here should more intelligently parse tool calls instead of using default tools
                            if let Ok(agent_action) = parse_tool_call_from_content(&content) {
                                Ok(AgentOutput::Action(agent_action))
                            } else {
                                // If unable to parse tool call, directly return the answer
                                let mut return_values = std::collections::HashMap::new();
                                return_values.insert("answer".to_string(), content.clone());
                                return_values.insert("model".to_string(), model_name);
                                Ok(AgentOutput::Finish(AgentFinish { return_values }))
                            }
                        } else {
                            // Directly return the answer
                            let mut return_values = std::collections::HashMap::new();
                            return_values.insert("answer".to_string(), content.clone());
                            return_values.insert("model".to_string(), model_name);
                            Ok(AgentOutput::Finish(AgentFinish { return_values }))
                        }
                    }
                }
                Err(e) => {
                    // Return error message when an error occurs
                    // Get model name from OpenAI model, use default value if not available
                    let model_name = if let Some(ref model) = openai_model_clone {
                        model.model_name().map(|s| s.to_string()).unwrap_or("unknown".to_string())
                    } else {
                        "unknown".to_string()
                    };
                    
                    // Even if model call fails, save user message to memory
                    if let Some(memory) = &memory_clone {
                        let mut inputs = std::collections::HashMap::new();
                        inputs.insert("input".to_string(), serde_json::Value::String(input_text.clone()));
                        
                        let mut outputs = std::collections::HashMap::new();
                        outputs.insert("output".to_string(), serde_json::Value::String(format!("Model invocation failed: {}", e)));
                        
                        if let Err(e) = memory.save_context(&inputs, &outputs).await {
                            log::warn!("Failed to save context to memory: {}", e);
                        }
                    }
                    
                    let mut return_values = std::collections::HashMap::new();
                    return_values.insert("answer".to_string(), format!("Model invocation failed: {}", e));
                    return_values.insert("model".to_string(), model_name);
                    Ok(AgentOutput::Finish(AgentFinish { return_values }))
                }
            }
        })
    }

    fn clone_to_owned(
        &self,
    ) -> Box<dyn Runnable<std::collections::HashMap<String, String>, AgentOutput> + Send + Sync>
    {
        Box::new(self.clone())
    }
}

/// Extract JSON object string from content
fn extract_json_object(content: &str) -> Option<String> {
    // Find the first '{' and the last '}'
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            if end > start {
                // Extract possible JSON object
                let json_str = &content[start..=end];
                
                // Verify if it's a valid JSON object
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if value.is_object() {
                        return Some(json_str.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Parse tool call from content
fn parse_tool_call_from_content(content: &str) -> Result<AgentAction, anyhow::Error> {
    // Try to extract JSON object
    if let Some(json_str) = extract_json_object(content) {
        // Parse JSON
        let value: Value = serde_json::from_str(&json_str)?;
        
        // Check if there's a call_tool field
        if let Some(call_tool) = value.get("call_tool").and_then(|v| v.as_object()) {
            // Extract tool name
            let tool_name = call_tool
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?
                .to_string();
            
            // Extract parameters and convert to string
            let tool_input = call_tool
                .get("parameters")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()))
                .to_string();
            
            // Create AgentAction
            let action = AgentAction {
                tool: tool_name,
                tool_input,
                log: content.to_string(),
                thought: None,
            };
            
            return Ok(action);
        }
    }
    
    // If unable to parse, return error
    Err(anyhow::anyhow!("Failed to parse tool call from content"))
}
