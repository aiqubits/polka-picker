// OpenAI model implementation - based on LangChain design
use super::chat::{ChatCompletion, ChatModel};
use super::message::{ChatMessage, ChatMessageContent, TokenUsage};
use anyhow::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use log::info;
#[derive(Serialize, Deserialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: String,
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

// Token usage details structure - referencing LangChain's InputTokenDetails and OutputTokenDetails
#[derive(Deserialize, Default)]
struct InputTokenDetails {
    audio_tokens: Option<usize>,
    cache_read: Option<usize>,
    reasoning_tokens: Option<usize>,
    // Other possible fields
}

#[derive(Deserialize, Default)]
struct OutputTokenDetails {
    cache_write: Option<usize>,
    reasoning_tokens: Option<usize>,
    // Other possible fields
}

// OpenAI traditional API usage statistics
#[derive(Deserialize, Default)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
    // Extended fields, supporting more details
    input_tokens_details: Option<InputTokenDetails>,
    output_tokens_details: Option<OutputTokenDetails>,
}

// Responses API usage statistics format
#[derive(Deserialize, Default)]
struct OpenAIResponsesUsage {
    input_tokens: Option<usize>,
    output_tokens: Option<usize>,
    total_tokens: Option<usize>,
    // Fields specific to Responses API
    input_tokens_details: Option<InputTokenDetails>,
    output_tokens_details: Option<OutputTokenDetails>,
}

// Generic API response structure - compatible with OpenAI and other providers
#[derive(Deserialize)]
struct OpenAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OpenAIChoice>, // This field is usually required
    usage: Option<OpenAIUsage>,
    // Fields compatible with Responses API
    output: Option<Vec<OpenAIChoice>>,
    // Other possible response fields
}

#[derive(Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

// API type enumeration - supporting traditional Chat Completions API and new Responses API
#[derive(Debug, Clone, Copy)]
enum OpenAIApiType {
    ChatCompletions,
    Responses,
}

// OpenAI model implementation - supporting multiple API formats
#[derive(Clone)]
pub struct OpenAIChatModel {
    client: Client,
    api_key: String,
    base_url: String,
    model_name: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    api_type: OpenAIApiType,
    additional_headers: HashMap<String, String>,
    additional_params: HashMap<String, serde_json::Value>,
}

impl OpenAIChatModel {
    /// Create a new OpenAI chat model instance
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model_name: None,
            temperature: Some(0.7),
            max_tokens: None,
            api_type: OpenAIApiType::ChatCompletions,
            additional_headers: HashMap::new(),
            additional_params: HashMap::new(),
        }
    }

    /// Get model name
    pub fn model_name(&self) -> Option<&String> {
        self.model_name.as_ref()
    }

    /// Get base URL
    pub fn base_url(&self) -> &String {
        &self.base_url
    }

    /// Get temperature parameter
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Get maximum number of tokens
    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    /// Set model name
    pub fn with_model(mut self, model_name: String) -> Self {
        self.model_name = Some(model_name);
        self
    }

    /// Set temperature parameter
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set maximum number of tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set API type (Chat Completions or Responses)
    pub fn with_api_type(mut self, api_type: OpenAIApiType) -> Self {
        self.api_type = api_type;
        self
    }

    /// Add additional request headers
    pub fn with_additional_header(mut self, key: String, value: String) -> Self {
        self.additional_headers.insert(key, value);
        self
    }

    /// Add additional request parameters
    pub fn with_additional_param(mut self, key: String, value: serde_json::Value) -> Self {
        self.additional_params.insert(key, value);
        self
    }

    /// Build request payload - referencing LangChain's _get_request_payload method
    fn _get_request_payload(&self, messages: &[OpenAIMessage]) -> Result<serde_json::Value, Error> {
        Ok(serde_json::json!({"messages": messages}))
    }

    /// Convert message to dictionary format - referencing LangChain's _convert_message_to_dict
    fn _convert_message_to_dict(&self, message: &OpenAIMessage) -> Result<serde_json::Value, Error> {
        Ok(serde_json::to_value(message)?)  
    }

    /// Build Responses API payload - referencing LangChain's _construct_responses_api_payload
    fn _construct_responses_api_payload(&self, messages: &[OpenAIMessage]) -> Result<serde_json::Value, Error> {
        Ok(serde_json::json!({"messages": messages}))
    }

    /// Create usage metadata - referencing LangChain's _create_usage_metadata
    fn _create_usage_metadata(&self, usage: &OpenAIUsage) -> TokenUsage {
        TokenUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }

    /// Create usage metadata for Responses API - referencing LangChain's _create_usage_metadata_responses
    fn _create_usage_metadata_responses(&self, usage: &OpenAIResponsesUsage) -> TokenUsage {
        TokenUsage {
            prompt_tokens: usage.input_tokens.unwrap_or(0),
            completion_tokens: usage.output_tokens.unwrap_or(0),
            total_tokens: usage.total_tokens.unwrap_or(0),
        }
    }

    /// Convert dictionary to message - referencing LangChain's _convert_dict_to_message
    fn _convert_dict_to_message(&self, message_dict: serde_json::Value) -> Result<ChatMessage, Error> {
        // Simple implementation: try to extract role and content from JSON
        let role = message_dict.get("role").and_then(|v| v.as_str()).unwrap_or("assistant");
        let content = message_dict.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
        
        let chat_content = ChatMessageContent {
            content,
            name: None,
            additional_kwargs: HashMap::new(),
        };
        
        match role {
            "system" => Ok(ChatMessage::System(chat_content)),
            "user" => Ok(ChatMessage::Human(chat_content)),
            "assistant" => Ok(ChatMessage::AIMessage(chat_content)),
            "tool" => Ok(ChatMessage::ToolMessage(chat_content)),
            _ => Ok(ChatMessage::AIMessage(chat_content)),
        }
    }
}

impl ChatModel for OpenAIChatModel {
    fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    fn base_url(&self) -> String {
        self.base_url.to_string()
    }

    fn invoke(&self, messages: Vec<ChatMessage>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ChatCompletion, Error>> + Send + '_>> {
        let messages = messages;
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();
        let model_name = self.model_name.clone();
        let temperature = self.temperature;
        let max_tokens = self.max_tokens;
        let additional_headers = self.additional_headers.clone();
        let additional_params = self.additional_params.clone();

        Box::pin(async move {
            // Convert message format
            let openai_messages: Vec<OpenAIMessage> = messages
                .into_iter()
                .map(|msg| match msg {
                    ChatMessage::System(content) => OpenAIMessage {
                        role: "system".to_string(),
                        content: content.content,
                        name: content.name,
                        tool_call_id: None,
                    },
                    ChatMessage::Human(content) => OpenAIMessage {
                        role: "user".to_string(),
                        content: content.content,
                        name: content.name,
                        tool_call_id: None,
                    },
                    ChatMessage::AIMessage(content) => OpenAIMessage {
                        role: "assistant".to_string(),
                        content: content.content,
                        name: content.name,
                        tool_call_id: None,
                    },
                    ChatMessage::ToolMessage(content) => {
                        info!("Converting tool message: role=tool, content={}", content.content);
                        // Add tool_call_id for tool messages
                        let tool_call_id = content.additional_kwargs.get("tool_call_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default_tool_call_id").to_string();
                        OpenAIMessage {
                            role: "tool".to_string(),
                            content: content.content,
                            name: content.name,
                            tool_call_id: Some(tool_call_id),
                        }
                    },
                })
                .collect();

            // Build request body
            let mut request_body = serde_json::json!({
                "messages": openai_messages,
                "model": model_name.clone().unwrap_or("".to_string()),
            });

            // Add optional parameters
            if let Some(temp) = temperature {
                request_body["temperature"] = serde_json::json!(temp);
            }
            if let Some(max) = max_tokens {
                request_body["max_tokens"] = serde_json::json!(max);
            }
            
            // Add additional parameters
            for (key, value) in additional_params {
                request_body[key] = value;
            }

            // Build complete API path, concatenating base_url with specific endpoint
            let api_url = format!("{}/chat/completions", base_url);
            
            // Build request
            let mut request = client.post(&api_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json");

            // Add additional request headers
            for (key, value) in additional_headers {
                request = request.header(key, value);
            }
            
            // Send request
            let response = request.json(&request_body).send().await?;
            
            // Check response status
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await?;
                return Err(Error::msg(format!("API request failed: {} - {}", status, error_text)));
            }

            // Parse response
            let response: OpenAIResponse = response.json().await?;

            // Handle response
            let chat_message = match response.choices.first() {
                Some(choice) => {
                    let message = &choice.message;
                    match message.role.as_str() {
                        "assistant" => ChatMessage::AIMessage(ChatMessageContent {
                            content: message.content.clone(),
                            name: message.name.clone(),
                            additional_kwargs: HashMap::new(),
                        }),
                        _ => {
                            return Err(Error::msg(format!("Unexpected message role: {}", message.role)));
                        }
                    }
                },
                None => {
                    // Try to use output field (Responses API)
                    match &response.output {
                        Some(outputs) => {
                            match outputs.first() {
                                Some(choice) => {
                                    let message = &choice.message;
                                    ChatMessage::AIMessage(ChatMessageContent {
                                        content: message.content.clone(),
                                        name: message.name.clone(),
                                        additional_kwargs: HashMap::new(),
                                    })
                                },
                                None => return Err(Error::msg("No output returned from API")),
                            }
                        },
                        None => return Err(Error::msg("No choices or output returned from API")),
                    }
                },
            };

            // Convert usage statistics
            let usage = match &response.usage {
                Some(openai_usage) => {
                    Some(TokenUsage {
                        prompt_tokens: openai_usage.prompt_tokens,
                        completion_tokens: openai_usage.completion_tokens,
                        total_tokens: openai_usage.total_tokens,
                    })
                },
                None => None,
            };

            let model_name_str = response.model.as_deref().unwrap_or("unknown");
            Ok(ChatCompletion {
                message: chat_message,
                usage,
                model_name: model_name_str.to_string(),
            })
        })
    }
}