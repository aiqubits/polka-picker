use crate::tools::Tool;
use std::boxed::Box;
use serde_json::Value;
use std::collections::HashMap;
use crate::agents::{AgentOutput, AgentAction, AgentFinish};

/// Implement fuzzy matching mechanism for tool names, returns the matching tool name
pub fn find_matching_tool_index(tools: &[Box<dyn Tool + Send + Sync>], requested_tool: &str) -> Option<String> {
    // 1. Exact match - prioritize complete matching
    if let Some(tool) = tools.iter().find(|t| { 
        t.name() == requested_tool 
    }) {
        return Some(tool.name().to_string());
    }
    
    // 2. Contains match - check if tool name contains the requested tool name (case insensitive)
    let requested_lower = requested_tool.to_lowercase();
    for tool in tools {
        let tool_name_lower = tool.name().to_lowercase();
        // For example: when requesting "weather", match "get_weather"
        if tool_name_lower.contains(&requested_lower) || requested_lower.contains(&tool_name_lower) {
            return Some(tool.name().to_string());
        }
    }
    
    // 3. Keyword matching - for default simulation tools, weather-related queries, special handling
    if requested_lower.contains("weather") || requested_lower.contains("天气") {
        if let Some(tool) = tools.iter().find(|t| t.name().to_lowercase().contains("weather") || t.name().to_lowercase().contains("天气")) {
            return Some(tool.name().to_string());
        }
    }
    
    // 4. Calculate tool keyword matching - only match calculation tool when input looks like a mathematical expression
    // Check if it contains calculation-related keywords and mathematical operators
    let has_calc_keywords = requested_lower.contains("calculate") || requested_lower.contains("计算") || 
                           requested_lower.contains("plus") || requested_lower.contains("minus") || 
                           requested_lower.contains("times") || requested_lower.contains("divided");
    
    let has_math_operators = requested_lower.contains("+") || requested_lower.contains("-") || 
                            requested_lower.contains("*") || requested_lower.contains("/") || 
                            requested_lower.contains("plus") || requested_lower.contains("minus") || 
                            requested_lower.contains("times") || requested_lower.contains("divided");
    
    if has_calc_keywords && has_math_operators {
        if let Some(tool) = tools.iter().find(|t| t.name().to_lowercase().contains("calculate") || t.name().to_lowercase().contains("计算")) {
            return Some(tool.name().to_string());
        }
    }
    
    None
}

// Independent model output parsing function, avoid referencing self in async blocks
pub fn parse_model_output(content: &str) -> Result<AgentOutput, anyhow::Error> {
    // Try to parse JSON
    if let Ok(value) = serde_json::from_str::<Value>(content) {
        // Check if there is a call_tool field
        if let Some(call_tool) = value.get("call_tool") {
            // Parse tool call
            if let Some(tool_name) = call_tool.get("name") {
                let tool_name = tool_name.as_str().unwrap_or("unknown").to_string();
                
                // Get parameters
                let parameters = call_tool.get("parameters").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
                
                // Convert parameters to string
                let tool_input = parameters.to_string();
                
                return Ok(AgentOutput::Action(AgentAction {
                    tool: tool_name,
                    tool_input,
                    log: "Call tool".to_string(),
                    thought: Some("Call tool based on model output".to_string()),
                }));
            }
        }
        
        // Check if there is a content field
        if let Some(content_value) = value.get("content") {
            let content_text = content_value.as_str().unwrap_or("").to_string();
            let mut return_values = HashMap::new();
            return_values.insert("answer".to_string(), content_text);
            
            return Ok(AgentOutput::Finish(AgentFinish {
                return_values,
            }));
        }
    }
    
    // If parsing fails, return error
    Err(anyhow::anyhow!("Failed to parse model output"))
}
