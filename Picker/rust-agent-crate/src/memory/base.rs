// Basic memory interface definition
use anyhow::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use std::pin::Pin;
use std::future::Future;
use log::info;

// Memory variable type alias
pub type MemoryVariables = HashMap<String, Value>;

// Minimal memory abstraction interface
pub trait BaseMemory: Send + Sync {
    // Get memory variable names
    fn memory_variables(&self) -> Vec<String>;
    
    // Core method: load memory variables
    fn load_memory_variables<'a>(&'a self, inputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>, Error>> + Send + 'a>>;
    
    // Core method: save context
    fn save_context<'a>(&'a self, inputs: &'a HashMap<String, Value>, outputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
    
    // Optional method: clear memory
    fn clear<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
    
    // Clone method
    fn clone_box(&self) -> Box<dyn BaseMemory>;
    
    // New method: get session ID
    fn get_session_id(&self) -> Option<&str>;
    
    // New method: set session ID
    fn set_session_id(&mut self, session_id: String);
    
    // New method: get token count
    fn get_token_count(&self) -> Result<usize, Error>;
    
    // New method: get Any reference for type conversion
    fn as_any(&self) -> &dyn std::any::Any;
}

// Simple memory implementation, similar to Langchain's ConversationBufferMemory
#[derive(Debug)]
pub struct SimpleMemory {
    memories: Arc<RwLock<HashMap<String, Value>>>,
    memory_key: String,
    session_id: Option<String>,
}

impl Clone for SimpleMemory {
    fn clone(&self) -> Self {
        Self {
            memories: Arc::clone(&self.memories),
            memory_key: self.memory_key.clone(),
            session_id: self.session_id.clone(),
        }
    }
}

impl SimpleMemory {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            memory_key: "chat_history".to_string(),
            session_id: None,
        }
    }
    
    pub fn with_memory_key(memory_key: String) -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            memory_key,
            session_id: None,
        }
    }
    
    pub fn with_memories(memories: HashMap<String, Value>) -> Self {
        Self {
            memories: Arc::new(RwLock::new(memories)),
            memory_key: "chat_history".to_string(),
            session_id: None,
        }
    }
    
    pub async fn add_message(&self, message: Value) -> Result<(), Error> {
        let mut memories = self.memories.write().await;
        let chat_history = memories.entry(self.memory_key.clone()).or_insert_with(|| Value::Array(vec![]));
        
        if let Value::Array(ref mut arr) = chat_history {
            arr.push(message);
        } else {
            *chat_history = Value::Array(vec![message]);
        }
        
        Ok(())
    }
    
    pub fn get_memory_key(&self) -> String {
        self.memory_key.clone()
    }
}

impl Default for SimpleMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseMemory for SimpleMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }
    
    fn load_memory_variables<'a>(&'a self, _inputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>, Error>> + Send + 'a>> {
        let memories = Arc::clone(&self.memories);
        Box::pin(async move {
            let memories = memories.read().await;
            Ok(memories.clone())
        })
    }
    
    fn save_context<'a>(&'a self, inputs: &'a HashMap<String, Value>, outputs: &'a HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        let memories = Arc::clone(&self.memories);
        let input_clone = inputs.clone();
        let output_clone = outputs.clone();
        let memory_key = self.memory_key.clone();
        
        Box::pin(async move {
            let mut memories = memories.write().await;
            
            // Get or create chat history array
            let chat_history = memories.entry(memory_key.clone()).or_insert_with(|| Value::Array(vec![]));
            
            // Ensure chat_history is an array type
            if !chat_history.is_array() {
                *chat_history = Value::Array(vec![]);
            }
            
            // Add input as human message or tool message to chat history
            if let Some(input_value) = input_clone.get("input") {
                let user_message = serde_json::json!({
                        "role": "human",
                        "content": input_value
                    });
                
                if let Value::Array(ref mut arr) = chat_history {
                    info!("Adding to chat history: {:?}", user_message);
                    arr.push(user_message);
                }
            }
            
            // Add output as AI message to chat history
            if let Some(output_value) = output_clone.get("output") {
                let ai_message = serde_json::json!({
                    "role": "ai",
                    "content": output_value
                });
                
                if let Value::Array(ref mut arr) = chat_history {
                    arr.push(ai_message);
                }
            }
            
            Ok(())
        })
    }
    
    fn clear<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        let memories = Arc::clone(&self.memories);
        Box::pin(async move {
            let mut memories = memories.write().await;
            memories.clear();
            Ok(())
        })
    }
    
    fn clone_box(&self) -> Box<dyn BaseMemory> {
        Box::new(self.clone())
    }
    
    fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
    
    fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }
    
    fn get_token_count(&self) -> Result<usize, Error> {
        // Simplified implementation: estimate token count based on character count
        let count = self.memory_key.len() + self.session_id.as_ref().map(|s| s.len()).unwrap_or(0);
        Ok(count)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Implement Clone trait for Box<dyn BaseMemory>
impl Clone for Box<dyn BaseMemory> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}