// Message type definitions
use serde::Deserialize;
use std::collections::HashMap;
use serde_json::Value;

// Message content structure
#[derive(Clone, Debug)]
pub struct ChatMessageContent {
    pub content: String,
    pub name: Option<String>,
    // OpenAI API tool_call_id parameter
    pub additional_kwargs: HashMap<String, Value>,
}

// Simplified message type system (aligned with langchain-core)
#[derive(Clone, Debug)]
pub enum ChatMessage {
    System(ChatMessageContent),
    Human(ChatMessageContent),
    AIMessage(ChatMessageContent),
    ToolMessage(ChatMessageContent),
}

// Token usage statistics
#[derive(Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}