// Agent interface and related structure definitions
use anyhow::Error;
use std::collections::HashMap;
use crate::tools::{ExampleTool, Tool};
use crate::core::Runnable;

// Action executed by Agent (simplified)
#[derive(Clone, Debug)]
pub struct AgentAction {
    pub tool: String,
    pub tool_input: String,
    pub log: String,
    pub thought: Option<String>,
}

// Result when Agent completes execution (simplified)
#[derive(Clone, Debug)]
pub struct AgentFinish {
    pub return_values: HashMap<String, String>,
}

// Unified Agent output type
#[derive(Clone, Debug)]
pub enum AgentOutput {
    Action(AgentAction),
    Finish(AgentFinish),
}

// Minimal Agent interface (separated from Runnable functionality)
pub trait Agent: Send + Sync {
    // Get list of available tools
    fn tools(&self) -> Vec<Box<dyn Tool + Send + Sync>>;
    
    // Execute Agent action (consistent with README)
    fn execute(&self, action: &AgentAction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send + '_>> {
        let tools = self.tools();
        let tool_name = action.tool.clone();
        let tool_input = action.tool_input.clone();
        
        Box::pin(async move {
            // Find the corresponding tool
            for tool in tools {
                if tool.name() == tool_name {
                    return tool.invoke(&tool_input).await;
                }
            }
            Err(Error::msg(format!("The tool {} not found", tool_name)))
        })
    }
    
    // Clone agent instance
    fn clone_agent(&self) -> Box<dyn Agent>;
}

// Agent runner - specifically handles execution logic
pub trait AgentRunner: Runnable<HashMap<String, String>, AgentOutput> {
    // Note: Since it inherits from Runnable, there's no need to redefine the invoke method here
    // This method will be automatically provided when implementing the Runnable trait
}

// Simple Agent implementation
pub struct SimpleAgent {
    tools: Vec<Box<dyn Tool + Send + Sync>>,
}

impl SimpleAgent {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
        }
    }
    
    pub fn add_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.push(tool);
    }
}

impl Agent for SimpleAgent {
    fn tools(&self) -> Vec<Box<dyn Tool + Send + Sync>> {
        // Since Box<dyn Tool> cannot be directly cloned, return an empty vector as a minimal implementation
        Vec::new()
    }
    
    fn clone_agent(&self) -> Box<dyn Agent> {
        let mut cloned = SimpleAgent::new();
        // Clone all tools
        for tool in &self.tools {
            let name = tool.name();
            let description = tool.description();
            let new_tool = Box::new(ExampleTool::new(name.to_string(), description.to_string()));
            cloned.add_tool(new_tool);
        }
        Box::new(cloned)
    }
}

// Simple AgentRunner implementation
pub struct SimpleAgentRunner {
    agent: Box<dyn Agent>,
}

impl SimpleAgentRunner {
    pub fn new(agent: impl Agent + 'static) -> Self {
        Self {
            agent: Box::new(agent),
        }
    }
}

impl Runnable<HashMap<String, String>, AgentOutput> for SimpleAgentRunner {
    fn invoke(&self, inputs: HashMap<String, String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AgentOutput, Error>> + Send>> {
        let inputs_clone = inputs.clone();
        
        Box::pin(async move {
            // This is just a simple implementation, in practice should use LLM to decide whether to call a tool or directly return results
            // For demonstration, we assume if the input contains a "tool" field, call the corresponding tool
            if let Some(tool_name) = inputs_clone.get("tool") {
                let tool_input = inputs_clone.get("input").unwrap_or(&"input empty".to_string()).clone();
                
                Ok(AgentOutput::Action(AgentAction {
                    tool: tool_name.to_string(),
                    tool_input,
                    log: format!("Invoking tool: {}", tool_name),
                    thought: Some("Invoking tool".to_string()),
                }))
            } else {
                // Otherwise return a simple completion result
                let output_text = inputs_clone.get("input").unwrap_or(&"".to_string()).clone();
                let mut return_values = HashMap::new();
                return_values.insert("output".to_string(), format!("Finish processing input: {}", output_text));
                
                Ok(AgentOutput::Finish(AgentFinish {
                    return_values,
                }))
            }
        })
    }
    
    fn clone_to_owned(&self) -> Box<dyn Runnable<HashMap<String, String>, AgentOutput> + Send + Sync> {
        // Use the newly added clone_agent method
        Box::new(SimpleAgentRunner { agent: self.agent.clone_agent() })
    }
}

// Implement Clone trait for SimpleAgent
impl Clone for SimpleAgent {
    fn clone(&self) -> Self {
        let mut agent = SimpleAgent::new();
        // Since Tool trait doesn't require Clone, we need to create new tool instances
        // For ExampleTool, we can directly create new instances
        // This is a minimal implementation
        for tool in &self.tools {
            // Try to get tool name to create a new instance
            let name = tool.name();
            let description = tool.description();
            // Create a new ExampleTool as a clone
            let new_tool = Box::new(ExampleTool::new(name.to_string(), description.to_string()));
            agent.add_tool(new_tool);
        }
        agent
    }
}