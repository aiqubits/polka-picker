// Memory system module
pub mod base;
pub mod message_history;
pub mod summary;
pub mod utils;
pub mod composite_memory;

// Export main types and traits
pub use base::{BaseMemory, SimpleMemory, MemoryVariables};
pub use message_history::{MessageHistoryMemory, ChatMessage, ChatMessageRecord};
pub use summary::{SummaryMemory, SummaryData};
pub use utils::*;
pub use composite_memory::{CompositeMemory, CompositeMemoryConfig};