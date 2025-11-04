// Composite memory module
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::pin::Pin;
use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use async_trait::async_trait;
use log::{info, warn, error};
use std::future::Future;

use crate::memory::base::{BaseMemory, MemoryVariables};
use crate::memory::message_history::{MessageHistoryMemory, ChatMessage};
use crate::memory::summary::SummaryMemory;
use crate::memory::utils::{
    ensure_data_dir_exists, get_data_dir_from_env, get_summary_threshold_from_env,
    get_recent_messages_count_from_env, generate_session_id
};

/// Composite memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeMemoryConfig {
    /// Data directory
    pub data_dir: PathBuf,
    /// Session ID (automatically generated internally)
    pub session_id: Option<String>,
    /// Summary threshold (in token count, 1 token ≈ 4 English characters, 1 token ≈ 1 Chinese character)
    pub summary_threshold: usize,
    /// Number of recent messages to keep (in message count)
    pub recent_messages_count: usize,
    /// Whether to automatically generate summaries
    pub auto_generate_summary: bool,
}

impl Default for CompositeMemoryConfig {
    fn default() -> Self {
        Self {
            data_dir: get_data_dir_from_env(),
            session_id: None, // Will be automatically generated internally
            summary_threshold: get_summary_threshold_from_env(),
            recent_messages_count: get_recent_messages_count_from_env(),
            auto_generate_summary: true,
        }
    }
}

/// Composite memory implementation
/// 
/// This struct combines multiple memory types, providing a unified interface to manage different types of memory.
/// It can simultaneously manage message history and summary memory, and provide intelligent summary generation functionality.
#[derive(Debug, Clone)]
pub struct CompositeMemory {
    /// Configuration
    config: CompositeMemoryConfig,
    /// Message history memory
    message_history: Option<Arc<MessageHistoryMemory>>,
    /// Summary memory
    summary_memory: Option<Arc<SummaryMemory>>,
    /// In-memory memory variables
    memory_variables: Arc<RwLock<MemoryVariables>>,
}

impl CompositeMemory {
    /// Create a new composite memory instance
    pub async fn new() -> Result<Self> {
        Self::with_config(CompositeMemoryConfig::default()).await
    }

    /// Create a composite memory instance with basic parameters
    /// This is the recommended constructor, only requires necessary parameters
    /// session_id will be automatically generated internally
    pub async fn with_basic_params(
        data_dir: PathBuf,
        summary_threshold: usize,
        recent_messages_count: usize,
    ) -> Result<Self> {
        let config = CompositeMemoryConfig {
            data_dir,
            session_id: None, // Will be automatically generated internally
            summary_threshold,
            recent_messages_count,
            auto_generate_summary: true,
        };
        Self::with_config(config).await
    }

    /// Create a composite memory instance with configuration
    pub async fn with_config(config: CompositeMemoryConfig) -> Result<Self> {
        // Ensure data directory exists
        ensure_data_dir_exists(&config.data_dir).await?;

        // Automatically generate session ID (if not provided)
        let session_id = config.session_id.clone()
            .unwrap_or_else(|| generate_session_id());

        // Always create message history memory
        let history = MessageHistoryMemory::new_with_recent_count(
            session_id.clone(),
            config.data_dir.clone(),
            config.recent_messages_count
        ).await?;
        let message_history = Some(Arc::new(history));

        // Always create summary memory with shared message history
        let summary = SummaryMemory::new_with_shared_history(
            session_id.clone(),
            config.data_dir.clone(),
            config.summary_threshold,
            message_history.clone().unwrap() // We just created it, so it's safe to unwrap
        ).await?;
        let summary_memory = Some(Arc::new(summary));

        Ok(Self {
            config,
            message_history,
            summary_memory,
            memory_variables: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a composite memory instance with session ID
    pub async fn with_session_id(session_id: String) -> Result<Self> {
        let mut config = CompositeMemoryConfig::default();
        config.session_id = Some(session_id);
        Self::with_config(config).await
    }

    /// Add message to memory
    pub async fn add_message(&self, message: ChatMessage) -> Result<()> {
        // Add to message history (always enabled)
        if let Some(ref history) = self.message_history {
            history.add_message(&message).await?;
        }

        // Check if summary generation is needed (always enabled)
        if self.config.auto_generate_summary {
            info!("Checking if summary generation is needed...");
            // Directly call SummaryMemory's check_and_generate_summary method
            // This avoids duplicate implementation of summary generation logic and simplifies the call chain
            if let Some(ref summary) = self.summary_memory {
                summary.check_and_generate_summary().await?;
                
                // Clean up old messages
                if let Some(ref history) = self.message_history {
                    let keep_count = self.config.recent_messages_count;
                    history.keep_recent_messages(keep_count).await?;
                }
            }
        }

        Ok(())
    }

    /// Get message count
    pub async fn get_message_count(&self) -> Result<usize> {
        if let Some(ref history) = self.message_history {
            history.get_message_count().await
        } else {
            Ok(0)
        }
    }

    /// Get the most recent N messages
    pub async fn get_recent_messages(&self, count: usize) -> Result<Vec<ChatMessage>> {
        if let Some(ref history) = self.message_history {
            history.get_recent_chat_messages(count).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Clean up old messages
    pub async fn cleanup_old_messages(&self) -> Result<()> {
        if let Some(ref history) = self.message_history {
            history.keep_recent_messages(self.config.recent_messages_count).await?;
        }
        Ok(())
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> Result<Value> {
        let mut stats = json!({
            "config": {
                "summary_threshold": self.config.summary_threshold,
                "recent_messages_count": self.config.recent_messages_count,
                "auto_generate_summary": self.config.auto_generate_summary,
            }
        });

        // Add message history statistics (always enabled)
        if let Some(ref history) = self.message_history {
            let message_count: usize = history.get_message_count().await?;
            stats["message_history"] = json!({
                "enabled": true,
                "message_count": message_count,
            });
        }

        // Add summary memory statistics (always enabled)
        if let Some(ref summary) = self.summary_memory {
            let summary_data = summary.load_summary().await?;
            stats["summary_memory"] = json!({
                "enabled": true,
                "has_summary": summary_data.summary.is_some(),
                "token_count": summary_data.token_count,
                "last_updated": summary_data.last_updated,
            });
        }

        Ok(stats)
    }

    /// Get summary content
    pub async fn get_summary(&self) -> Result<Option<String>> {
        if let Some(ref summary) = self.summary_memory {
            let summary_data = summary.load_summary().await?;
            Ok(summary_data.summary)
        } else {
            Ok(None)
        }
    }
}

// Implement as_any method for CompositeMemory's BaseMemory trait
impl CompositeMemory {
    /// Get Any reference for type conversion
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl BaseMemory for CompositeMemory {
    fn memory_variables(&self) -> Vec<String> {
        // Return all memory variables
        let mut vars = Vec::new();
        
        // Add base memory variables
        vars.extend_from_slice(&["chat_history".to_string(), "summary".to_string(), "input".to_string(), "output".to_string()]);
        
        // Add configuration related variables
        vars.push("config".to_string());
        
        vars
    }

    fn load_memory_variables<'a>(&'a self, inputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send + 'a>> {
        Box::pin(async move {
            let mut result = HashMap::new();

            // Load chat history (always enabled)
            if let Some(ref history) = self.message_history {
                let messages = history.get_recent_chat_messages(
                    self.config.recent_messages_count
                ).await?;
                
                let history_json = serde_json::to_value(&messages)?;
                result.insert("chat_history".to_string(), history_json);
            }

            // Load summary (always enabled)
            if let Some(ref summary) = self.summary_memory {
                let summary_data = summary.load_summary().await?;
                
                if let Some(summary_text) = summary_data.summary {
                    result.insert("summary".to_string(), json!(summary_text));
                }
            }

            // Add input
            if let Some(input) = inputs.get("input") {
                result.insert("input".to_string(), input.clone());
            }

            // Add output
            if let Some(output) = inputs.get("output") {
                result.insert("output".to_string(), output.clone());
            }

            // Add configuration information
            result.insert("config".to_string(), serde_json::to_value(&self.config)?);

            // Update internal memory variables
            *self.memory_variables.write().await = result.clone();

            Ok(result)
        })
    }

    fn save_context<'a>(&'a self, inputs: &'a HashMap<String, Value>, outputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Extract input and output
            let input = inputs.get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("");
                
            let output = outputs.get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Create user message
            if !input.is_empty() {
                let user_message = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: "user".to_string(),
                    content: input.to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    metadata: None,
                };
                
                // Add directly to message history without triggering summary generation
                if let Some(ref history) = self.message_history {
                    history.add_message(&user_message).await?;
                }
            }

            // Create assistant message
            if !output.is_empty() {
                let assistant_message = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: "assistant".to_string(),
                    content: output.to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    metadata: None,
                };
                
                // Add directly to message history without triggering summary generation
                if let Some(ref history) = self.message_history {
                    history.add_message(&assistant_message).await?;
                }
            }

            // Check for summary generation only once after all messages are added
            if self.config.auto_generate_summary {
                info!("Checking if summary generation is needed...");
                if let Some(ref summary) = self.summary_memory {
                    summary.check_and_generate_summary().await?;
                    
                    // Keep only recent messages after summary generation
                    if let Some(ref history) = self.message_history {
                        let keep_count = self.config.recent_messages_count;
                        history.keep_recent_messages(keep_count).await?;
                    }
                }
            }

            // Update internal memory variables
            let mut memory_vars = self.memory_variables.write().await;
            
            if let Some(input_val) = inputs.get("input") {
                memory_vars.insert("input".to_string(), input_val.clone());
            }
            
            if let Some(output_val) = outputs.get("output") {
                memory_vars.insert("output".to_string(), output_val.clone());
            }

            Ok(())
        })
    }

    fn clear<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Clear message history (always enabled)
            if let Some(ref history) = self.message_history {
                history.clear().await?;
            }

            // Clear summary memory (always enabled)
            if let Some(ref summary) = self.summary_memory {
                summary.clear().await?;
            }

            // Clear internal memory variables
            self.memory_variables.write().await.clear();

            Ok(())
        })
    }

    fn clone_box(&self) -> Box<dyn BaseMemory> {
        Box::new(self.clone())
    }

    fn get_session_id(&self) -> Option<&str> {
        self.config.session_id.as_deref()
    }

    fn set_session_id(&mut self, session_id: String) {
        self.config.session_id = Some(session_id);
    }

    fn get_token_count(&self) -> Result<usize, Error> {
        // This is a simplified implementation, actual applications may need more precise calculation
        let mut count = 0;
        
        // Estimate configuration token count
        if let Ok(config_json) = serde_json::to_value(&self.config) {
            count += crate::memory::utils::estimate_json_token_count(&config_json);
        }
        
        // Estimate memory variables token count
        if let Ok(memory_vars) = self.memory_variables.try_read() {
            if let Ok(vars_json) = serde_json::to_value(&*memory_vars) {
                count += crate::memory::utils::estimate_json_token_count(&vars_json);
            }
        }
        
        Ok(count)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::memory::message_history::ChatMessage;

    #[tokio::test]
    async fn test_composite_memory_new() {
        let memory = CompositeMemory::new().await;
        assert!(memory.is_ok());
    }

    #[tokio::test]
    async fn test_composite_memory_with_session_id() {
        let session_id = "test_session";
        let memory = CompositeMemory::with_session_id(session_id.to_string()).await;
        assert!(memory.is_ok());
        
        let memory = memory.unwrap();
        assert_eq!(memory.get_session_id(), Some(session_id));
    }

    #[tokio::test]
    async fn test_add_message() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = CompositeMemoryConfig::default();
        config.data_dir = temp_dir.path().to_path_buf();
        config.auto_generate_summary = false; // Disable auto summary for testing
        
        let memory = CompositeMemory::with_config(config).await.unwrap();
        
        let message = ChatMessage {
            id: "test_id".to_string(),
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: None,
        };
        
        let result = memory.add_message(message).await;
        assert!(result.is_ok());
        
        let count = memory.get_message_count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_save_context() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = CompositeMemoryConfig::default();
        config.data_dir = temp_dir.path().to_path_buf();
        config.auto_generate_summary = false; // Disable auto summary for testing
        
        let memory = CompositeMemory::with_config(config).await.unwrap();
        
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), json!("Hello"));
        
        let mut outputs = HashMap::new();
        outputs.insert("output".to_string(), json!("Hi there!"));
        
        let result = memory.save_context(&inputs, &outputs).await;
        assert!(result.is_ok());
        
        let count = memory.get_message_count().await.unwrap();
        assert_eq!(count, 2); // User message and assistant message
    }

    #[tokio::test]
    async fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = CompositeMemoryConfig::default();
        config.data_dir = temp_dir.path().to_path_buf();
        config.auto_generate_summary = false; // Disable auto summary for testing
        
        let memory = CompositeMemory::with_config(config).await.unwrap();
        
        // Add some messages
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), json!("Hello"));
        
        let mut outputs = HashMap::new();
        outputs.insert("output".to_string(), json!("Hi there!"));
        
        memory.save_context(&inputs, &outputs).await.unwrap();
        
        // Verify messages have been added
        let count = memory.get_message_count().await.unwrap();
        assert_eq!(count, 2);
        
        // Clear memory
        let result = memory.clear().await;
        assert!(result.is_ok());
        
        // Verify messages have been cleared
        let count = memory.get_message_count().await.unwrap();
        assert_eq!(count, 0);
    }
}