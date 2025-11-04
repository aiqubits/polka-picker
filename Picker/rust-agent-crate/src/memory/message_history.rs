// Long-term memory implementation, persisting conversation history to file
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json::Value;
use std::pin::Pin;
use std::future::Future;
use log::{info, warn};
use chrono::Utc;

// Chat message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub metadata: Option<Value>,
}

/// Single message record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRecord {
    /// Message role: "system", "user", "assistant", "tool"
    pub role: String,
    /// Message content (without historical conversation)
    pub content: String,
    /// Optional message name
    pub name: Option<String>,
    /// Additional metadata
    pub additional_kwargs: Option<HashMap<String, serde_json::Value>>,
    /// Timestamp (ISO 8601 format)
    pub timestamp: String,
    /// Message sequence number (to ensure order)
    pub sequence_number: u64,
}

/// Session-level message history structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionHistory {
    /// Session ID (identity identifier for the entire conversation)
    pub session_id: String,
    /// Session creation time
    pub created_at: String,
    /// Session last update time
    pub updated_at: String,
    /// Message list (in chronological order)
    pub messages: Vec<ChatMessageRecord>,
    /// Session-level metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// File message history implementation, aligned with LangChain's FileChatMessageHistory
/// Uses JSONL format, one JSON object per line
#[derive(Debug)]
pub struct FileChatMessageHistory {
    /// Session ID
    session_id: String,
    /// File path
    file_path: PathBuf,
    /// In-memory session history
    session_history: Arc<RwLock<ChatSessionHistory>>,
    /// Next message sequence number
    next_sequence_number: Arc<RwLock<u64>>,
}

impl Clone for FileChatMessageHistory {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            file_path: self.file_path.clone(),
            session_history: Arc::clone(&self.session_history),
            next_sequence_number: Arc::clone(&self.next_sequence_number),
        }
    }
}

impl FileChatMessageHistory {
    /// Create a new file message history instance
    pub async fn new(session_id: String, file_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Initialize session history
        let now = Utc::now().to_rfc3339();
        let session_history = ChatSessionHistory {
            session_id: session_id.clone(),
            created_at: now.clone(),
            updated_at: now,
            messages: Vec::new(),
            metadata: None,
        };
        
        let instance = Self {
            session_id: session_id.clone(),
            file_path: file_path.clone(),
            session_history: Arc::new(RwLock::new(session_history)),
            next_sequence_number: Arc::new(RwLock::new(1)),
        };
        
        // Try to load existing session history
        instance.load_session_history().await?;
        
        Ok(instance)
    }
    
    /// Load session history from file
    async fn load_session_history(&self) -> Result<()> {
        if !tokio::fs::metadata(&self.file_path).await.is_ok() {
            // File doesn't exist, use default session history
            return Ok(());
        }
        
        let mut file = File::open(&self.file_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        
        if contents.trim().is_empty() {
            return Ok(());
        }
        
        // Try to parse as JSON format session history (entire file is a JSON object)
        match serde_json::from_str::<ChatSessionHistory>(&contents) {
            Ok(session_history) => {
                // Update in-memory session history
                {
                    let mut history = self.session_history.write().await;
                    *history = session_history;
                }
                
                // Update next message sequence number
                {
                    let history = self.session_history.read().await;
                    let next_seq = history.messages.len() as u64 + 1;
                    let mut next_sequence = self.next_sequence_number.write().await;
                    *next_sequence = next_seq;
                }
                
                info!("[FileChatMessageHistory] Loaded session history with {} messages from JSONL format", {
                    let history = self.session_history.read().await;
                    history.messages.len()
                });
            },
            Err(e) => {
                // If parsing fails, try to parse as old format (one JSON object per line)
                warn!("Failed to parse as session history JSON, trying as old format: {}", e);
                
                let mut messages = Vec::new();
                let mut max_sequence_number = 0u64;
                
                for line in contents.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    // Try to parse JSONL format message
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(msg_value) => {
                            // Check if it's an old format message (no sequence_number field)
                            if msg_value.get("sequence_number").is_none() {
                                // Try to migrate from old format
                                if let (Some(role), Some(content)) = (
                                    msg_value.get("role").and_then(|v| v.as_str()),
                                    msg_value.get("content").and_then(|v| v.as_str())
                                ) {
                                    // Skip assistant messages containing complete history
                                    if role == "assistant" && content.contains("user:") && content.contains("assistant:") {
                                        continue;
                                    }
                                    
                                    // Create new format message
                                    let message = ChatMessageRecord {
                                        role: role.to_string(),
                                        content: content.to_string(),
                                        name: msg_value.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                        additional_kwargs: msg_value.get("additional_kwargs").cloned().and_then(|v| {
                                            if v.is_null() {
                                                None
                                            } else {
                                                Some(serde_json::from_value(v).unwrap_or_default())
                                            }
                                        }),
                                        timestamp: msg_value.get("timestamp")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(&Utc::now().to_rfc3339())
                                            .to_string(),
                                        sequence_number: max_sequence_number + 1,
                                    };
                                    
                                    max_sequence_number += 1;
                                    messages.push(message);
                                }
                            } else {
                                // Directly parse as new format message
                                if let Ok(message) = serde_json::from_value::<ChatMessageRecord>(msg_value) {
                                    max_sequence_number = max_sequence_number.max(message.sequence_number);
                                    messages.push(message);
                                }
                            }
                        },
                        Err(e) => {
                            warn!("Failed to parse line in JSONL file: {}, error: {}", line, e);
                        }
                    }
                }
                
                if !messages.is_empty() {
                    // Sort messages by sequence_number
                    messages.sort_by_key(|m| m.sequence_number);
                    
                    // Update session history
                    {
                        let mut history = self.session_history.write().await;
                        history.messages = messages;
                        history.updated_at = Utc::now().to_rfc3339();
                    }
                    
                    // Update next message sequence number
                    {
                        let mut next_sequence = self.next_sequence_number.write().await;
                        *next_sequence = max_sequence_number + 1;
                    }
                    
                    info!("[FileChatMessageHistory] Loaded session history with {} messages from old JSONL format", {
                        let history = self.session_history.read().await;
                        history.messages.len()
                    });
                    
                    // Save as new format
                    self.save_session_history().await?;
                } else {
                    return Err(anyhow::anyhow!("Failed to parse file as either session history JSON or old JSONL format"));
                }
            }
        }
        
        Ok(())
    }
    
    /// Save session history to file (entire session as a JSON object)
    pub async fn save_session_history(&self) -> Result<()> {
        // Get current session history
        let history = {
            let history_guard = self.session_history.read().await;
            history_guard.clone()
        };
        
        // Create temporary file
        let temp_path = self.file_path.with_extension("tmp");
        {
            let mut file = File::create(&temp_path).await?;
            
            // Write entire session history as a JSON object to file
            let json_content = serde_json::to_string_pretty(&history)?;
            file.write_all(json_content.as_bytes()).await?;
            
            file.flush().await?;
        }
        
        // Atomically replace original file
        tokio::fs::rename(&temp_path, &self.file_path).await?;
        
        Ok(())
    }
    
    /// Add user message, aligned with LangChain's add_user_message
    pub async fn add_user_message(&self, content: String) -> Result<()> {
        // Check if message content is empty
        if content.trim().is_empty() {
            return Ok(());
        }
        
        let sequence_number = {
            let mut seq = self.next_sequence_number.write().await;
            let current = *seq;
            *seq += 1;
            current
        };
        
        let message = ChatMessageRecord {
            role: "user".to_string(),
            content,
            name: None,
            additional_kwargs: None,
            timestamp: Utc::now().to_rfc3339(),
            sequence_number,
        };
        
        self.add_message(message).await?;
        Ok(())
    }
    
    /// Add AI message to history
    pub async fn add_ai_message(&self, content: &str) -> Result<()> {
        // Preprocess content, if it's JSON string format, extract the content field
        let processed_content = if content.starts_with('"') && content.ends_with('"') {
            // Try to parse as JSON string
            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(serde_json::Value::String(s)) => s,
                _ => content.to_string(),
            }
        } else if content.starts_with('{') && content.ends_with('}') {
            // Try to parse as JSON object
            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(json_obj) => {
                    // If it's a JSON object, try to extract the content field
                    if let Some(content_value) = json_obj.get("content") {
                        if let Some(content_str) = content_value.as_str() {
                            content_str.to_string()
                        } else {
                            content.to_string()
                        }
                    } else {
                        content.to_string()
                    }
                },
                _ => content.to_string(),
            }
        } else {
            content.to_string()
        };
        
        let sequence_number = {
            let mut seq = self.next_sequence_number.write().await;
            let current = *seq;
            *seq += 1;
            current
        };
        
        let message = ChatMessageRecord {
            role: "assistant".to_string(),
            content: processed_content,
            name: None,
            additional_kwargs: None,
            timestamp: Utc::now().to_rfc3339(),
            sequence_number,
        };
        
        self.add_message(message).await?;
        Ok(())
    }
    
    /// Add message to memory and save to file
    async fn add_message(&self, message: ChatMessageRecord) -> Result<()> {
        // Add to memory
        {
            let mut history = self.session_history.write().await;
            history.messages.push(message.clone());
            history.updated_at = Utc::now().to_rfc3339();
        }
        
        // Save to file
        self.save_session_history().await?;
        
        Ok(())
    }
    
    /// Get all messages, aligned with LangChain's get_messages
    pub async fn get_messages(&self) -> Result<Vec<ChatMessageRecord>> {
        let history = self.session_history.read().await;
        Ok(history.messages.clone())
    }
    
    /// Clear all messages
    pub async fn clear(&self) -> Result<()> {
        // Reset session history
        {
            let mut history = self.session_history.write().await;
            history.messages.clear();
            history.updated_at = Utc::now().to_rfc3339();
        }
        
        // Reset message sequence number
        {
            let mut next_sequence = self.next_sequence_number.write().await;
            *next_sequence = 1;
        }
        
        // Save to file
        self.save_session_history().await?;
        
        Ok(())
    }
}

/// MessageHistoryMemory implementation, implementing BaseMemory trait
#[derive(Debug)]
pub struct MessageHistoryMemory {
    /// Session ID
    session_id: String,
    /// Data directory
    data_dir: PathBuf,
    /// File message history
    chat_history: FileChatMessageHistory,
    /// Default number of recent messages to get
    default_recent_count: usize,
}

impl Clone for MessageHistoryMemory {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            data_dir: self.data_dir.clone(),
            chat_history: self.chat_history.clone(),
            default_recent_count: self.default_recent_count,
        }
    }
}

impl MessageHistoryMemory {
    /// Create a new MessageHistoryMemory instance
    pub async fn new(session_id: String, data_dir: PathBuf) -> Result<Self> {
        // Use default recent message count
        let default_recent_count = crate::memory::utils::get_recent_messages_count_from_env();
        Self::new_with_recent_count(session_id, data_dir, default_recent_count).await
    }
    
    /// Create a new MessageHistoryMemory instance with specified recent message count
    pub async fn new_with_recent_count(session_id: String, data_dir: PathBuf, recent_count: usize) -> Result<Self> {
        // Ensure data directory exists
        tokio::fs::create_dir_all(&data_dir).await?;
        
        // Create file message history, using JSONL format
        let file_path = data_dir.join(format!("{}_history.jsonl", session_id));
        let chat_history = FileChatMessageHistory::new(session_id.clone(), file_path).await?;
        
        Ok(Self {
            session_id,
            data_dir,
            chat_history,
            default_recent_count: recent_count,
        })
    }
    
    /// Get session ID
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Get recent messages
    pub async fn get_recent_messages(&self, count: usize) -> Result<Vec<ChatMessageRecord>> {
        let messages = self.chat_history.get_messages().await?;
        
        // Get recent messages
        let messages_len = messages.len();
        let recent_messages: Vec<ChatMessageRecord> = if messages_len > count {
            messages.into_iter().skip(messages_len - count).collect()
        } else {
            messages
        };
        
        Ok(recent_messages)
    }
    
    /// Get recent messages using default count
    pub async fn get_default_recent_messages(&self) -> Result<Vec<ChatMessageRecord>> {
        self.get_recent_messages(self.default_recent_count).await
    }
    
    /// Get total message count
    pub async fn get_message_count(&self) -> Result<usize> {
        let messages = self.chat_history.get_messages().await?;
        Ok(messages.len())
    }
    
    /// Keep only the most recent N messages
    pub async fn keep_recent_messages(&self, count: usize) -> Result<()> {
        let messages = self.chat_history.get_messages().await?;
        
        if messages.len() <= count {
            return Ok(());
        }
        
        // Get recent messages
        let messages_len = messages.len();
        let recent_messages: Vec<ChatMessageRecord> = if messages_len > count {
            messages.into_iter().skip(messages_len - count).collect()
        } else {
            messages
        };
        
        // Update session history
        {
            let mut history = self.chat_history.session_history.write().await;
            history.messages = recent_messages;
            history.updated_at = Utc::now().to_rfc3339();
        }
        
        // Save to file
        self.chat_history.save_session_history().await?;
        
        Ok(())
    }
    
    /// Add ChatMessage to history
    pub async fn add_message(&self, message: &ChatMessage) -> Result<()> {
        // Check if message content is empty
        if message.content.trim().is_empty() {
            return Ok(());
        }
        
        let sequence_number = {
            let mut seq = self.chat_history.next_sequence_number.write().await;
            let current = *seq;
            *seq += 1;
            current
        };
        
        let record = ChatMessageRecord {
            role: message.role.clone(),
            content: message.content.clone(),
            name: None,
            additional_kwargs: if let Some(metadata) = &message.metadata {
                let filtered_kwargs: HashMap<String, serde_json::Value> = metadata.as_object()
                    .unwrap_or(&serde_json::Map::new())
                    .iter()
                    .filter(|(k, _)| k != &"type") // Filter out special fields
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                Some(filtered_kwargs)
            } else {
                None
            },
            timestamp: message.timestamp.clone(),
            sequence_number,
        };
        
        self.chat_history.add_message(record).await?;
        Ok(())
    }
    
    /// Get the most recent N messages, return ChatMessage type
    pub async fn get_recent_chat_messages(&self, count: usize) -> Result<Vec<ChatMessage>> {
        let records = self.get_recent_messages(count).await?;
        
        // Convert to ChatMessage
        let messages: Result<Vec<ChatMessage>> = records.into_iter().map(|record| {
            Ok(ChatMessage {
                id: uuid::Uuid::new_v4().to_string(), // Generate new ID
                role: record.role,
                content: record.content,
                timestamp: record.timestamp,
                metadata: record.additional_kwargs.map(|kwargs| {
                    let mut map = serde_json::Map::new();
                    for (k, v) in kwargs {
                        map.insert(k, v);
                    }
                    serde_json::Value::Object(map)
                }),
            })
        }).collect();
        
        messages
    }
    
}

// Implement BaseMemory trait, compatible with existing system
use crate::memory::base::BaseMemory;

impl BaseMemory for MessageHistoryMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec!["chat_history".to_string()]
    }
    
    fn load_memory_variables<'a>(&'a self, _inputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>, Error>> + Send + 'a>> {
        Box::pin(async move {
            // Load messages from file, but only return recent messages
            // Use default configured message count
            let messages = self.get_default_recent_messages().await?;
            
            // Convert to format compatible with SimpleMemory
            let mut history_array = Vec::new();
            for msg in messages {
                let mut msg_obj = serde_json::Map::new();
                msg_obj.insert("role".to_string(), serde_json::Value::String(msg.role));
                msg_obj.insert("content".to_string(), serde_json::Value::String(msg.content));
                
                if let Some(kwargs) = msg.additional_kwargs {
                    for (k, v) in kwargs {
                        msg_obj.insert(k, v);
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
            // Save user message
            if let Some(input_value) = inputs.get("input") {
                if let Some(content) = input_value.as_str() {
                    self.chat_history.add_user_message(content.to_string()).await?;
                }
            }
            
            // Save AI response
            if let Some(output_value) = outputs.get("output") {
                if let Some(content) = output_value.as_str() {
                    // Preprocess content, if it's JSON string format, extract the content field
                    let processed_content = if content.starts_with('"') && content.ends_with('"') {
                        // Try to parse as JSON string
                        match serde_json::from_str::<serde_json::Value>(content) {
                            Ok(serde_json::Value::String(s)) => s,
                            _ => content.to_string(),
                        }
                    } else if content.starts_with('{') && content.ends_with('}') {
                        // Try to parse as JSON object
                        match serde_json::from_str::<serde_json::Value>(content) {
                            Ok(json_obj) => {
                                // If it's a JSON object, try to extract the content field
                                if let Some(content_value) = json_obj.get("content") {
                                    if let Some(content_str) = content_value.as_str() {
                                        content_str.to_string()
                                    } else {
                                        content.to_string()
                                    }
                                } else {
                                    content.to_string()
                                }
                            },
                            _ => content.to_string(),
                        }
                    } else {
                        content.to_string()
                    };
                    
                    self.chat_history.add_ai_message(&processed_content).await?;
                }
            }
            
            Ok(())
        })
    }
    
    fn clear<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        Box::pin(async move {
            self.chat_history.clear().await?;
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
        // Simplified implementation: estimate token count based on character count
        // In actual applications, a more precise token calculator can be used
        let count = self.session_id.len() + self.data_dir.to_string_lossy().len();
        Ok(count)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}