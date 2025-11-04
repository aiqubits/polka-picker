// Summary memory implementation, generates summaries when token count exceeds threshold
use std::path::PathBuf;
use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json::{json, Value};
use std::pin::Pin;
use std::future::Future;
use log::info;
use uuid;
use chrono;
use crate::ChatMessage;
use std::sync::Arc;

// Import FileChatMessageHistory
use crate::memory::message_history::{FileChatMessageHistory, ChatMessageRecord, MessageHistoryMemory};
// Import utility functions
use crate::memory::utils::estimate_text_tokens;
// Import common models
use crate::{ChatModel, OpenAIChatModel, ModelChatMessage, ChatMessageContent};

/// Summary data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryData {
    /// Session ID
    pub session_id: String,
    /// Summary update sequence number, used for incremental summary updates
    pub sequence_number: u64,
    /// Summary content
    pub summary: Option<String>,
    /// Token count (approximately equal to message count * 4)
    pub token_count: usize,
    /// Last update time
    pub last_updated: String,
}

impl Default for SummaryData {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            sequence_number: 0,
            summary: None,
            token_count: 0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Summary memory implementation
/// 
/// This struct is responsible for generating and managing conversation summaries.
/// It can automatically generate summaries when the conversation reaches a certain length,
/// and load previous summaries when needed.
#[derive(Debug)]
pub struct SummaryMemory {
    /// Session ID
    session_id: String,
    /// Data directory
    data_dir: PathBuf,
    /// Summary threshold (in token count, 1 token ≈ 4 English characters, 1 token ≈ 1 Chinese character)
    summary_threshold: usize,
    /// Summary prompt template
    summary_prompt_template: String,
    /// Number of recent messages to keep (in message count)
    recent_messages_count: usize,
    /// Shared message history memory (optional)
    message_history: Option<Arc<MessageHistoryMemory>>,
}

impl Clone for SummaryMemory {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            data_dir: self.data_dir.clone(),
            summary_threshold: self.summary_threshold,
            summary_prompt_template: self.summary_prompt_template.clone(),
            recent_messages_count: self.recent_messages_count,
            message_history: self.message_history.clone(),
        }
    }
}

impl SummaryMemory {
    /// Create a new summary memory instance
    pub async fn new(session_id: String, data_dir: PathBuf, summary_threshold: usize) -> Result<Self> {
        // Ensure data directory exists
        tokio::fs::create_dir_all(&data_dir).await?;
        
        Ok(Self {
            session_id,
            data_dir,
            summary_threshold,
            summary_prompt_template: "Please provide a concise summary of the following conversation. Focus on the main topics discussed, key decisions made, and any important outcomes.\n\nConversation:\n{chat_history}\n\nSummary:".to_string(),
            recent_messages_count: crate::memory::utils::get_recent_messages_count_from_env(),
            message_history: None,
        })
    }
    
    /// Create a new summary memory instance with shared message history
    pub async fn new_with_shared_history(
        session_id: String, 
        data_dir: PathBuf, 
        summary_threshold: usize,
        message_history: Arc<MessageHistoryMemory>
    ) -> Result<Self> {
        // Ensure data directory exists
        tokio::fs::create_dir_all(&data_dir).await?;
        
        Ok(Self {
            session_id,
            data_dir,
            summary_threshold,
            summary_prompt_template: "Please provide a concise summary of the following conversation. Focus on the main topics discussed, key decisions made, and any important outcomes.\n\nConversation:\n{chat_history}\n\nSummary:".to_string(),
            recent_messages_count: crate::memory::utils::get_recent_messages_count_from_env(),
            message_history: Some(message_history),
        })
    }
    
    /// Set summary prompt template
    pub fn with_summary_prompt_template(mut self, template: String) -> Self {
        self.summary_prompt_template = template;
        self
    }
    
    /// Set the number of recent messages to keep
    pub fn with_recent_messages_count(mut self, count: usize) -> Self {
        self.recent_messages_count = count;
        self
    }
    
    /// Get summary file path
    fn get_summary_file_path(&self) -> PathBuf {
        self.data_dir.join(format!("{}_summary.json", self.session_id))
    }
    
    /// Load context from memory
    pub async fn load_context(&self) -> Result<Vec<String>> {
        // Load summary
        let summary_data = self.load_summary().await?;
        
        // Load message history
        let messages = if let Some(ref history) = self.message_history {
            // Use shared message history
            history.get_recent_messages(self.recent_messages_count).await?
        } else {
            // Create new FileChatMessageHistory instance
            let file_path = self.data_dir.join(format!("{}_history.jsonl", self.session_id));
            let chat_history = FileChatMessageHistory::new(self.session_id.clone(), file_path).await?;
            chat_history.get_messages().await?
        };
        
        // Build context vector
        let mut context = Vec::new();
        
        // Add summary (if exists)
        if let Some(summary) = summary_data.summary {
            context.push(format!("Previous conversation summary: {}", summary));
        }
        
        // Add recent messages
        for msg in messages {
            context.push(format!("{}: {}", msg.role, msg.content));
        }
        
        Ok(context)
    }
    
    /// Load summary
    pub async fn load_summary(&self) -> Result<SummaryData> {
        let file_path = self.get_summary_file_path();
        
        if !tokio::fs::metadata(&file_path).await.is_ok() {
            return Ok(SummaryData {
                session_id: self.session_id.clone(),
                sequence_number: 0,
                summary: None,
                token_count: 0,
                last_updated: chrono::Utc::now().to_rfc3339(),
            });
        }
        
        let contents = tokio::fs::read_to_string(&file_path).await?;
        let summary_data: SummaryData = serde_json::from_str(&contents)?;
        
        Ok(summary_data)
    }
    
    /// Save summary
    async fn save_summary(&self, summary: &str, sequence_number: u64) -> Result<()> {
        let file_path = self.get_summary_file_path();
        
        // Calculate token count for the summary
        let token_count = estimate_text_tokens(summary);
        
        let summary_data = SummaryData {
            session_id: self.session_id.clone(),
            sequence_number,
            summary: Some(summary.to_string()),
            token_count,
            last_updated: chrono::Utc::now().to_rfc3339(),
        };
        
        let json = serde_json::to_string(&summary_data)?;
        tokio::fs::write(&file_path, json).await?;
        
        Ok(())
    }
    
    /// Generate summary
    async fn generate_summary(&self, messages: &[ChatMessageRecord]) -> Result<(String, u64)> {
        info!("Generating summary for {} messages", messages.len());

        // Convert messages to text format
        let mut chat_text = String::new();
        for msg in messages {
            let role = if msg.role == "user" { "User" } else { "Assistant" };
            chat_text.push_str(&format!("{}: {}\n", role, msg.content));
        }
        
        // Use summary prompt template
        let summary_prompt = self.summary_prompt_template.replace("{chat_history}", &chat_text);
        
        // Get API key and base URL from environment variables
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "OPENAI_API_KEY".to_string());
        let base_url = std::env::var("OPENAI_API_URL").ok();
        
        // // Check if API key is valid, return error if invalid
        // if api_key == "OPENAI_API_KEY" || api_key.is_empty() || api_key.starts_with("mock_api") {
        //     return Err(anyhow::anyhow!("OpenAI API key is not configured or is invalid. Please set the OPENAI_API_KEY environment variable."));
        // }
        
        // Create OpenAI model instance
        let model = crate::OpenAIChatModel::new(api_key.clone(), base_url)
            .with_model(std::env::var("OPENAI_API_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()))
            .with_temperature(0.3)
            .with_max_tokens(1024);
        
        // Build message list
        let model_messages = vec![
            crate::ModelChatMessage::System(crate::ChatMessageContent {
                content: "You are a helpful assistant that creates concise summaries of conversations.".to_string(),
                name: None,
                additional_kwargs: std::collections::HashMap::new(),
            }),
            crate::ModelChatMessage::Human(crate::ChatMessageContent {
                content: summary_prompt,
                name: None,
                additional_kwargs: std::collections::HashMap::new(),
            }),
        ];
        
        // Call model to generate summary
        let response = model.invoke(model_messages).await?;
        
        // Extract response content
        let summary = match response.message {
            crate::ModelChatMessage::AIMessage(content) => content.content,
            _ => return Err(anyhow::anyhow!("Expected AI message response")),
        };

        // Get the sequence number of the last message as the summary update sequence number
        let last_sequence_number = messages.last()
            .map(|msg| msg.sequence_number)
            .unwrap_or(0);

        // Save summary to file
        self.save_summary(&summary, last_sequence_number).await?;
        
        Ok((summary, last_sequence_number))
    }
    
    /// Check if summary needs to be generated and generate if needed
    pub async fn check_and_generate_summary(&self) -> Result<bool> {
        // Load current summary data to get the last sequence number
        let summary_data = self.load_summary().await?;
        let last_summary_sequence = summary_data.sequence_number;
        
        // Load message history
        let messages = if let Some(ref message_history) = self.message_history {
            // Get all messages
            message_history.get_recent_messages(usize::MAX).await?
        } else {
            return Ok(false);
        };
        
        // If no messages, no need to generate summary
        if messages.is_empty() {
            return Ok(false);
        }
        
        // Filter messages to only include those after the last summary sequence number
        let messages_to_summarize: Vec<ChatMessageRecord> = messages
            .into_iter()
            .filter(|msg| msg.sequence_number > last_summary_sequence)
            .collect();
        
        // If no new messages since last summary, no need to generate summary
        if messages_to_summarize.is_empty() {
            return Ok(false);
        }
        
        // Calculate total tokens in new messages
        let mut chat_text = String::new();
        for msg in &messages_to_summarize {
            chat_text.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }

        let total_tokens = estimate_text_tokens(&chat_text);
        
        // If token count exceeds threshold, generate summary
        if total_tokens > self.summary_threshold {
            info!("[SummaryMemory] Generating summary... ({} new messages, {} tokens)", messages_to_summarize.len(), total_tokens);
            
            // Generate summary
            let (summary, _) = self.generate_summary(&messages_to_summarize).await?;
            
            // Get the sequence number of the last message
            let last_sequence = messages_to_summarize.last().map(|m| m.sequence_number).unwrap_or(0);
            
            // Save summary
            self.save_summary(&summary, last_sequence).await?;
            
            // Keep only recent messages
            if let Some(ref message_history) = self.message_history {
                message_history.keep_recent_messages(self.recent_messages_count).await?;
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Get session ID
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> Result<Value> {
        // Load summary
        let summary_data = self.load_summary().await?;
        
        // Load message history
        let file_path = self.data_dir.join(format!("{}_history.jsonl", self.session_id.clone()));
        let chat_history = FileChatMessageHistory::new(self.session_id.clone(), file_path).await?;
        let messages = chat_history.get_messages().await?;
        
        // Calculate total tokens in messages
        let mut chat_text = String::new();
        for msg in &messages {
            chat_text.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
        let token_count = estimate_text_tokens(&chat_text);
        
        let stats = json!({
            "session_id": self.session_id,
            "summary_threshold": self.summary_threshold,
            "recent_messages_count": self.recent_messages_count,
            "message_count": messages.len(),
            "token_count": token_count,
            "has_summary": summary_data.summary.is_some(),
            "summary_token_count": summary_data.token_count,
            "last_updated": summary_data.last_updated
        });
        
        Ok(stats)
    }
}

// Implement BaseMemory trait, compatible with existing system
use crate::memory::base::BaseMemory;

impl BaseMemory for SummaryMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec!["chat_history".to_string()]
    }
    
    fn load_memory_variables<'a>(&'a self, _inputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>, Error>> + Send + 'a>> {
        let session_id = self.session_id.clone();
        let data_dir = self.data_dir.clone();
        let summary_threshold = self.summary_threshold;
        let recent_messages_count = self.recent_messages_count;
        let use_shared_history = self.message_history.is_some();
        
        Box::pin(async move {
            // Load summary
            let summary_memory = SummaryMemory {
                session_id: session_id.clone(),
                data_dir: data_dir.clone(),
                summary_threshold,
                summary_prompt_template: String::new(),
                recent_messages_count,
                message_history: None, // We'll handle this separately
            };
            
            let summary_data = summary_memory.load_summary().await?;
            
            // Load message history
            let messages = if use_shared_history {
                // This is a simplified approach - in a real implementation, we would need to pass the shared instance
                // For now, we'll create a new instance but this should be improved
                let file_path = data_dir.join(format!("{}_history.jsonl", session_id.clone()));
                let chat_history = FileChatMessageHistory::new(session_id.clone(), file_path).await?;
                chat_history.get_messages().await?
            } else {
                let file_path = data_dir.join(format!("{}_history.jsonl", session_id.clone()));
                let chat_history = FileChatMessageHistory::new(session_id.clone(), file_path).await?;
                chat_history.get_messages().await?
            };
            
            // Convert to new format: system_prompt + chat_message
            let mut history_array = Vec::new();
            
            // Build system prompt: basic_system_prompt + user_system_prompt + summary_prompt
            let mut system_prompt_parts = Vec::new();
            
            // Add basic system prompt
            system_prompt_parts.push("You are a helpful assistant that provides accurate and concise answers.".to_string());
            
            // Add user system prompt (if any)
            if let Some(user_system_prompt) = std::env::var("USER_SYSTEM_PROMPT").ok() {
                system_prompt_parts.push(user_system_prompt);
            }
            
            // Add summary (if any)
            if let Some(summary) = summary_data.summary {
                system_prompt_parts.push(format!("Previous conversation summary: {}", summary));
            }
            
            // Combine system prompt
            let combined_system_prompt = system_prompt_parts.join("\n\n");
            
            // Add system prompt to history
            let mut system_msg_obj = serde_json::Map::new();
            system_msg_obj.insert("role".to_string(), serde_json::Value::String("system".to_string()));
            system_msg_obj.insert("content".to_string(), serde_json::Value::String(combined_system_prompt));
            history_array.push(serde_json::Value::Object(system_msg_obj));
            
            // Add recent messages (chat_message)
            let len = messages.len();
            let start = if len > recent_messages_count {
                len - recent_messages_count
            } else {
                0
            };
            
            for msg in &messages[start..] {
                let mut msg_obj = serde_json::Map::new();
                msg_obj.insert("role".to_string(), serde_json::Value::String(msg.role.clone()));
                msg_obj.insert("content".to_string(), serde_json::Value::String(msg.content.clone()));
                
                if let Some(name) = &msg.name {
                    msg_obj.insert("name".to_string(), serde_json::Value::String(name.clone()));
                }
                
                if let Some(kwargs) = &msg.additional_kwargs {
                    for (k, v) in kwargs {
                        msg_obj.insert(k.clone(), v.clone());
                    }
                }
                
                history_array.push(serde_json::Value::Object(msg_obj));
            }
            
            let mut result = HashMap::new();
            result.insert("chat_history".to_string(), serde_json::Value::Array(history_array));
            
            Ok(result)
        })
    }
    
    fn save_context<'a>(&'a self, inputs: &'a HashMap<String, Value>, outputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        Box::pin(async move {
            // Extract user and assistant messages
            let mut user_message: Option<String> = None;
            let mut assistant_message: Option<String> = None;
            
            // Check inputs for user message
            if let Some(input_value) = inputs.get("input") {
                if let Some(s) = input_value.as_str() {
                    user_message = Some(s.to_string());
                }
            }
            
            // Check outputs for assistant message
            if let Some(output_value) = outputs.get("output") {
                if let Some(s) = output_value.as_str() {
                    assistant_message = Some(s.to_string());
                }
            }
            
            // Add messages to shared message history if available
            if let Some(ref message_history) = self.message_history {
                if let Some(user_msg) = user_message {
                    let chat_msg = ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        role: "user".to_string(),
                        content: user_msg,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: None,
                    };
                    message_history.add_message(&chat_msg).await?;
                }
                
                if let Some(assistant_msg) = assistant_message {
                    let chat_msg = ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        role: "assistant".to_string(),
                        content: assistant_msg,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        metadata: None,
                    };
                    message_history.add_message(&chat_msg).await?;
                }
                info!("save_context");
                // Note: Removed check_and_generate_summary() call to avoid duplicate summary generation
                // Summary generation is now handled by CompositeMemory::add_message
            }
            
            Ok(())
        })
    }
    
    fn clear<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        let session_id = self.session_id.clone();
        let data_dir = self.data_dir.clone();
        
        Box::pin(async move {
            // Clear message history
            let file_path = data_dir.join(format!("{}_history.jsonl", session_id.clone()));
            let chat_history = FileChatMessageHistory::new(session_id.clone(), file_path).await?;
            chat_history.clear().await?;
            
            // Clear summary file
            let summary_path = data_dir.join(format!("{}_summary.json", session_id.clone()));
            if tokio::fs::metadata(&summary_path).await.is_ok() {
                tokio::fs::remove_file(&summary_path).await?;
            }
            
            Ok(())
        })
    }
    
    fn clone_box(&self) -> Box<dyn BaseMemory> {
        Box::new(self.clone())
    }
    
    fn get_session_id(&self) -> Option<&str> {
        Some(&self.session_id)
    }
    
    fn set_session_id(&mut self, session_id: String) {
        self.session_id = session_id;
    }
    
    fn get_token_count(&self) -> Result<usize, Error> {
        // Use common function to estimate token count
        let text = format!("{}:{}", self.session_id, self.data_dir.to_string_lossy());
        Ok(estimate_text_tokens(&text))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}