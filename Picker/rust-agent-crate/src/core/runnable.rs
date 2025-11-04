// Runnable interface definition - core concept of the framework
use std::collections::HashMap;
use std::pin::Pin;
use futures::stream::Stream;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Runnable interface definition
pub trait Runnable<I: Send + 'static, O: Send + 'static>: Send + Sync {
    // Core async call method (main entry point)
    fn invoke(&self, input: I) -> Pin<Box<dyn std::future::Future<Output = Result<O, anyhow::Error>> + Send>>;
    
    // Call variant with configuration - optional implementation
    fn invoke_with_config(
        &self, 
        input: I, 
        _config: Option<HashMap<String, Value>>
    ) -> Pin<Box<dyn std::future::Future<Output = Result<O, anyhow::Error>> + Send>> {
        self.invoke(input)
    }
    
    // Async batch processing for multiple inputs
    fn batch(&self, inputs: Vec<I>) -> Pin<Box<dyn std::future::Future<Output = Vec<Result<O, anyhow::Error>>> + Send>> {
        let self_clone = self.clone_to_owned();
        Box::pin(async move {
            // Provide default implementation, specific components can override for optimization
            futures::future::join_all(inputs.into_iter().map(|input| {
                let self_clone_inner = self_clone.clone_to_owned();
                async move {
                    self_clone_inner.invoke(input).await
                }
            })).await
        })
    }
    
    // Variant of batch processing - optional implementation
    // Temporarily simplified implementation to avoid complex async composition issues
    fn batch_with_config(
        &self, 
        inputs: Vec<I>, 
        _config: Option<HashMap<String, Value>>
    ) -> Pin<Box<dyn std::future::Future<Output = Vec<Result<O, anyhow::Error>>> + Send>> {
        // Directly call the batch method
        self.batch(inputs)
    }
    
    // Stream processing interface - synchronous implementation
    fn stream(&self, input: I) -> Box<dyn Stream<Item = Result<O, anyhow::Error>> + Send> {
        // Simple implementation: use futures' stream::once to wrap a single result
        let self_clone = self.clone_to_owned();
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<O, anyhow::Error>>(1);
        
        // Execute invoke in a separate task and send the result
        tokio::spawn(async move {
            let result = self_clone.invoke(input).await;
            let _ = tx.send(result).await;
        });
        
        // Convert mpsc receiver to Stream
        Box::new(tokio_stream::wrappers::ReceiverStream::new(rx))
    }
    
    // Async stream processing - optional implementation
    fn astream(
        &self, 
        _input: I
    ) -> Pin<Box<dyn std::future::Future<Output = Box<dyn Stream<Item = Result<O, anyhow::Error>> + Send>> + Send>> {
        let _self_clone = self.clone_to_owned();
        
        Box::pin(async move {
            // Simple implementation: return an empty stream
            let (_tx, rx) = mpsc::channel(10);
            // Create and return an empty stream, add explicit type conversion
            let stream: Box<dyn Stream<Item = Result<O, anyhow::Error>> + Send> = Box::new(ReceiverStream::new(rx));
            stream
        })
    }
    
    // Helper method for astream default implementation, needs to be provided when implementing
    fn clone_to_owned(&self) -> Box<dyn Runnable<I, O> + Send + Sync>;
}

// Runnable extension trait
pub trait RunnableExt<I: Send + 'static, O: Send + 'static> {
    fn pipe<NextO: Send + 'static>(
        self: Box<Self>,
        next: impl Runnable<O, NextO> + Send + Sync + 'static
    ) -> impl Runnable<I, NextO> + Send + Sync
    where
        Self: Sized + 'static + Send + Sync;
}

// Provide extension methods for Runnable
impl<T: Runnable<I, O> + ?Sized, I: Send + 'static, O: Send + 'static> RunnableExt<I, O> for T {
    fn pipe<NextO: Send + 'static>(
        self: Box<Self>,
        next: impl Runnable<O, NextO> + Send + Sync + 'static
    ) -> impl Runnable<I, NextO> + Send + Sync
    where
        Self: Sized + 'static + Send + Sync,
    {
        // Call the pipe function to combine two Runnables
        pipe(*self, next)
    }
}

// Utility function: create a pipeline connecting two Runnables
pub fn pipe<I: Send + 'static, O1: Send + 'static, O2: Send + 'static>(
    first: impl Runnable<I, O1> + Send + Sync + 'static,
    second: impl Runnable<O1, O2> + Send + Sync + 'static
) -> Box<dyn Runnable<I, O2> + Send + Sync> {
    // Implement composition logic: create a struct that implements Runnable
    // Wrap two components and execute them in sequence
    struct PipeImpl<I: Send + 'static, O1: Send + 'static, O2: Send + 'static> {
        first: Box<dyn Runnable<I, O1> + Send + Sync>,
        second: Box<dyn Runnable<O1, O2> + Send + Sync>,
    }
    
    impl<I: Send + 'static, O1: Send + 'static, O2: Send + 'static> Runnable<I, O2> for PipeImpl<I, O1, O2> {
        fn invoke(&self, input: I) -> Pin<Box<dyn std::future::Future<Output = Result<O2, anyhow::Error>> + Send>> {
            let first_clone = self.first.clone_to_owned();
            let second_clone = self.second.clone_to_owned();
            
            Box::pin(async move {
                let intermediate = first_clone.invoke(input).await?;
                second_clone.invoke(intermediate).await
            })
        }
        
        fn clone_to_owned(&self) -> Box<dyn Runnable<I, O2> + Send + Sync> {
            // Note: this implementation assumes components can be cloned, actual implementation may need adjustment
            Box::new(PipeImpl {
                first: self.first.clone_to_owned(),
                second: self.second.clone_to_owned(),
            })
        }
    }
    
    // Send and Sync will be automatically derived because internal fields are already Send + Sync
    
    Box::new(PipeImpl {
        first: Box::new(first),
        second: Box::new(second),
    })
}

// RunnableSequence struct
pub struct RunnableSequence<I, O> {
    // In actual implementation, this needs to store the various components in the chain
    // For example: for a simple two-component chain
    // first: Box<dyn Runnable<I, O1> + Send + Sync>,
    // second: Box<dyn Runnable<O1, O> + Send + Sync>,
    
    // Actual implementation may be more complex, depending on the supported chain length
    inner: Box<dyn Runnable<I, O> + Send + Sync>,
}

// Helper methods for RunnableSequence
impl<I: Send + 'static, O: Send + 'static> RunnableSequence<I, O> {
    pub fn new(runnable: impl Runnable<I, O> + Send + Sync + 'static) -> Self {
        // In actual implementation, need to store runnable in the struct
        Self {
            inner: Box::new(runnable),
        }
    }
}

// Implement Runnable interface for RunnableSequence
impl<I: 'static + Send, O: 'static + Send> Runnable<I, O> for RunnableSequence<I, O> {
    fn invoke(&self, input: I) -> Pin<Box<dyn std::future::Future<Output = Result<O, anyhow::Error>> + Send>> {
        let inner = self.inner.clone_to_owned();
        inner.invoke(input)
    }
    
    fn clone_to_owned(&self) -> Box<dyn Runnable<I, O> + Send + Sync> {
        Box::new(RunnableSequence {
            inner: self.inner.clone_to_owned(),
        })
    }
}

// Example implementation of clone_to_owned method for Box<dyn Runnable>
impl<I: Send + 'static, O: Send + 'static> Runnable<I, O> for Box<dyn Runnable<I, O> + Send + Sync> {
    fn invoke(&self, input: I) -> Pin<Box<dyn std::future::Future<Output = Result<O, anyhow::Error>> + Send>> {
        let self_clone = self.clone_to_owned();
        Box::pin(async move {
            self_clone.invoke(input).await
        })
    }
    
    fn clone_to_owned(&self) -> Box<dyn Runnable<I, O> + Send + Sync> {
        (**self).clone_to_owned()
    }
}