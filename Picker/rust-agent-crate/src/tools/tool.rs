// Tool interface and implementation
use anyhow::Error;
use std::pin::Pin;

// Minimal tool interface (aligned with langchain-core)
pub trait Tool: Send + Sync {
    // Basic tool information
    fn name(&self) -> &str;
    
    fn description(&self) -> &str;
    
    // Core execution method
    fn invoke(&self, input: &str) -> Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send + '_>>;
    
    // Add as_any method to support runtime type checking
    fn as_any(&self) -> &dyn std::any::Any;
}

// Toolkit interface
pub trait Toolkit {
    // Get all tools
    fn tools(&self) -> Vec<Box<dyn Tool>>;
}

// Example tool implementation - for demonstration purposes
pub struct ExampleTool {
    name: String,
    description: String,
}

impl ExampleTool {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
        }
    }
}

impl Tool for ExampleTool {
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
            Ok(format!("Tool {} received input: {}", name, input_str))
        })
    }
    
    // Implement as_any method to support runtime type checking
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Example toolkit implementation
pub struct ExampleToolkit {
    tools: Vec<Box<dyn Tool>>,
}

impl ExampleToolkit {
    pub fn new() -> Self {
        let tools: Vec<Box<dyn Tool>> = Vec::new();
        Self {
            tools,
        }
    }
    
    pub fn add_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }
}

impl Toolkit for ExampleToolkit {
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        // Since Box<dyn Tool> cannot be directly cloned, return an empty vector as minimal implementation
        Vec::new()
    }
}

// Implement Clone trait for ExampleToolkit
impl Clone for ExampleToolkit {
    fn clone(&self) -> Self {
        let mut toolkit = ExampleToolkit::new();
        // Since Tool trait doesn't require Clone, we implement cloning by creating new instances
        for tool in &self.tools {
            let name = tool.name();
            let description = tool.description();
            // Create a new ExampleTool as a clone
            let new_tool = Box::new(ExampleTool::new(name.to_string(), description.to_string()));
            toolkit.add_tool(new_tool);
        }
        toolkit
    }
}