// Chat model interface and related structure definitions
use anyhow::Error;
use crate::models::message::{ChatMessage, TokenUsage};

// Simplified chat completion structure
pub struct ChatCompletion {
    pub message: ChatMessage,
    pub usage: Option<TokenUsage>,
    pub model_name: String,
}

// Chat model interface
pub trait ChatModel: Send + Sync {
    // Basic model information
    fn model_name(&self) -> Option<&str> {
        None
    }

    // Model base URL
    fn base_url(&self) -> String {
        "https://api.openai.com/v1".to_string()
    }
    
    // Core method: handle chat messages
    fn invoke(&self, messages: Vec<ChatMessage>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ChatCompletion, Error>> + Send + '_>> {
        let _messages = messages;
        Box::pin(async move {
            Err(Error::msg("The model does not implement the invoke method"))
        })
    }
}