// Callback handler interface definition
use crate::agents::{AgentAction, AgentFinish};

// Minimal callback system (aligned with langchain-core)
pub trait CallbackHandler: Send + Sync {
    // LLM related callbacks (core)
    fn on_llm_start(&self, _model_name: &str, _prompts: &[String]) {}
    
    fn on_llm_new_token(&self, _token: &str) {}
    
    fn on_llm_end(&self, _model_name: &str) {}
    
    fn on_llm_error(&self, _model_name: &str, _error: &str) {}
    
    // Tool related callbacks (core)
    fn on_tool_start(&self, _tool_name: &str, _input: &str) {}
    
    fn on_tool_end(&self, _tool_name: &str, _output: &str) {}
    
    fn on_tool_error(&self, _tool_name: &str, _error: &str) {}
    
    // Chain related callbacks (core)
    fn on_chain_start(&self, _chain_name: &str) {}
    
    fn on_chain_end(&self, _chain_name: &str) {}
    
    fn on_chain_error(&self, _chain_name: &str, _error: &str) {}
    
    // Agent related callbacks
    fn on_agent_action(&self, _action: &AgentAction) {}
    
    fn on_agent_finish(&self, _finish: &AgentFinish) {}
}