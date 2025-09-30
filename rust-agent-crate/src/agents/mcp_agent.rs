use anyhow::anyhow;
use std::pin::Pin;
use std::sync::Arc;
use log::info;

use crate::{
    Agent, AgentAction, AgentFinish, AgentOutput, ChatMessage, ChatMessageContent, ChatModel,
    McpClient, McpToolAdapter, OpenAIChatModel, Runnable, Tool, parse_model_output
};

/// McpAgent 是一个基于 MCP 服务的智能体实现
/// 它能够连接 MCP 服务器，处理用户输入，调用工具，并生成响应
pub struct McpAgent {
    client: Arc<dyn McpClient>,
    tools: Vec<Box<dyn Tool + Send + Sync>>,
    system_prompt: String,
    openai_model: Option<OpenAIChatModel>,
}

impl McpAgent {
    /// 创建一个新的 McpAgent 实例
    pub fn new(client: Arc<dyn McpClient>, system_prompt: String) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt,
            openai_model: None, // 默认不设置OpenAI模型
        }
    }
    
    /// 创建一个新的 McpAgent 实例，并指定 OpenAIChatModel
    pub fn with_openai_model(client: Arc<dyn McpClient>, system_prompt: String, openai_model: OpenAIChatModel) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt,
            openai_model: Some(openai_model),
        }
    }

    /// 向 Agent 添加一个工具
    pub fn add_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.push(tool);
    }
    
    /// 自动从 MCP 客户端获取工具并添加到 Agent
    /// 这个方法会从 MCP 客户端获取所有可用工具，并将它们包装为 McpToolAdapter 后添加到 Agent
    pub async fn auto_add_tools(&mut self) -> Result<(), anyhow::Error> {
        use crate::McpToolAdapter;
        
        // 从 MCP 客户端获取工具列表
        let tools = self.client.get_tools().await?;
        
        // 将每个工具包装为 McpToolAdapter 并添加到 Agent
        for tool in tools {
            let tool_adapter = McpToolAdapter::new(
                self.client.clone(),
                tool
            );
            self.add_tool(Box::new(tool_adapter));
        }
        
        Ok(())
    }
}

impl Agent for McpAgent {
    fn tools(&self) -> Vec<Box<dyn Tool + Send + Sync>> {
        // 返回工具列表的克隆版本
        // 为了解决 Box<dyn Tool> 不能直接克隆的问题，我们创建新的工具适配器实例
        let mut cloned_tools: Vec<Box<dyn Tool + Send + Sync>> = Vec::new();

        // 由于 McpToolAdapter 可以通过 client 和 McpTool 重新创建，
        // 我们遍历现有工具并为每个工具创建新的适配器实例
        for tool in &self.tools {
            // 检查工具是否为 McpToolAdapter 类型
            if let Some(mcp_tool_adapter) = tool.as_any().downcast_ref::<McpToolAdapter>() {
                // 重新创建 McpToolAdapter 实例
                let cloned_adapter = McpToolAdapter::new(
                    mcp_tool_adapter.get_client(),
                    mcp_tool_adapter.get_mcp_tool(),
                );
                cloned_tools.push(Box::new(cloned_adapter));
            } else {
                // 对于其他类型的工具，我们暂时跳过或需要实现其他克隆机制
                // 这里可以添加日志或错误处理
                info!(
                    "Warning: Unable to clone non-McpToolAdapter type tool: {}",
                    tool.name()
                );
            }
        }

        cloned_tools
    }

    fn execute(
        &self,
        _action: &AgentAction,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + Send + '_>,
    > {
        Box::pin(async move {
            // 在实际应用中，这里应该有一个机制来查找和调用工具
            // 由于我们不能克隆工具列表，这里简化实现
            Err(anyhow!("Tool execution functionality is not implemented yet"))
        })
    }

    fn clone_agent(&self) -> Box<dyn Agent> {
        // 创建一个新的 McpAgent 实例，复制基本字段，但不复制工具（简化实现）
        let new_agent = McpAgent::new(
            self.client.clone(),
            self.system_prompt.clone(),
        );

        // 注意：这里我们不复制工具，因为 Box<dyn Tool> 不能直接克隆
        Box::new(new_agent)
    }
}

impl Clone for McpAgent {
    fn clone(&self) -> Self {
        // 创建一个新的 McpAgent 实例，但不复制工具列表（简化实现）
        Self {
            client: Arc::clone(&self.client),
            tools: Vec::new(), // 不复制工具，因为 Box<dyn Tool> 不能直接克隆
            system_prompt: self.system_prompt.clone(),
            openai_model: self.openai_model.clone(), // 克隆OpenAI模型实例
        }
    }
}

impl Runnable<std::collections::HashMap<String, String>, AgentOutput> for McpAgent {
    fn invoke(
        &self,
        input: std::collections::HashMap<String, String>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<AgentOutput, anyhow::Error>> + Send>> {
        // 提前捕获系统提示词
        let system_prompt = self.system_prompt.clone();
        let input_text = input
            .get("input")
            .cloned()
            .unwrap_or_default()
            .to_string()
            .trim()
            .to_string();

        // 提前捕获工具描述，避免在async move中使用self
        let tool_descriptions: String = if !self.tools.is_empty() {
            let mut descriptions = String::new();
            for tool in &self.tools {
                descriptions.push_str(&format!("- {}: {}\n", tool.name(), tool.description()));
            }
            descriptions
        } else {
            String::new()
        };

        // 构建增强的系统提示词
        let enhanced_system_prompt = if !tool_descriptions.is_empty() {
            // 否则返回{{\"content\": \"回答内容\"}}
            format!("{}\n\nAvailable tools:\n{}\n\nPlease decide whether to use the tool based on user needs. If tools are needed, please return in JSON format:{{\"call_tool\": {{\"name\": \"Tool Name\", \"parameters\": {{...}}}}}}, else return{{}}", system_prompt, tool_descriptions)
        } else {
            system_prompt
        };

        // 提前捕获OpenAI模型实例，避免在async move中使用self
        let openai_model_clone = self.openai_model.clone();

        Box::pin(async move {
            // 检查输入是否为空
            if input_text.is_empty() {
                let mut return_values = std::collections::HashMap::new();
                return_values.insert("answer".to_string(), "Please enter valid content".to_string());
                // 从OpenAI模型获取模型名称，如果没有则使用默认值
                let model_name = if let Some(ref openai_model) = openai_model_clone {
                    openai_model.model_name().map(|s| s.to_string()).unwrap_or("unknown".to_string())
                } else {
                    "unknown".to_string()
                };
                return_values.insert("model".to_string(), model_name);
                return Ok(AgentOutput::Finish(AgentFinish { return_values }));
            }

            // 使用传入的OpenAI模型实例或创建新的实例
            let model = if let Some(ref openai_model) = openai_model_clone {
                // 使用传入的OpenAI模型实例
                openai_model
            } else {
                // 如果没有提供OpenAI模型实例，则返回错误
                let mut return_values = std::collections::HashMap::new();
                return_values.insert("answer".to_string(), "No OpenAI model provided".to_string());
                return_values.insert("model".to_string(), "unknown".to_string());
                return Ok(AgentOutput::Finish(AgentFinish { return_values }));
            };

            // 构建消息列表
            let mut messages = Vec::new();

            // 添加系统消息
            messages.push(ChatMessage::System(ChatMessageContent {
                content: enhanced_system_prompt,
                name: None,
                additional_kwargs: std::collections::HashMap::new(),
            }));

            // 添加用户消息
            messages.push(ChatMessage::Human(ChatMessageContent {
                content: input_text.clone(),
                name: None,
                additional_kwargs: std::collections::HashMap::new(),
            }));

            // 调用语言模型
            let result = model.invoke(messages).await;

            match result {
                Ok(completion) => {
                    // 解析模型输出
                    let content = match completion.message {
                        ChatMessage::AIMessage(content) => content.content,
                        _ => { format!("{},{:?}", "Non-AI message received", completion.message) }
                    };

                    // 从OpenAI模型获取模型名称，如果没有则使用默认值
                    let model_name = model.model_name().map(|s| s.to_string()).unwrap_or("unknown".to_string());

                    // 解析模型输出，判断是否需要调用工具
                    // 这里应该正确解析模型输出的JSON格式
                    if let Ok(parsed_output) = parse_model_output(&content) {
                        match parsed_output {
                            AgentOutput::Action(action) => {
                                // 直接返回模型解析出的Action
                                return Ok(AgentOutput::Action(action));
                            }
                            AgentOutput::Finish(_) => {
                                // 直接返回回答
                                let mut return_values = std::collections::HashMap::new();
                                return_values.insert("answer".to_string(), content.clone());
                                return_values.insert("model".to_string(), model_name);
                                return Ok(AgentOutput::Finish(AgentFinish { return_values }));
                            }
                        }
                    } else {
                        // 如果解析失败，检查是否包含工具调用关键词
                        if content.contains("call_tool") && content.contains("get_weather") {
                            // 尝试从内容中提取JSON格式的工具调用
                            // 这里简化实现，直接返回一个默认的工具调用
                            Ok(AgentOutput::Action(AgentAction {
                                tool: "default_tool".to_string(),
                                tool_input: "{\"default_input\": \"Default Input\"}".to_string(),
                                log: "Call default tool".to_string(),
                                thought: Some("Model output parsing failed, call default tool".to_string()),
                            }))
                        } else {
                            // 直接返回回答
                            let mut return_values = std::collections::HashMap::new();
                            return_values.insert("answer".to_string(), content.clone());
                            return_values.insert("model".to_string(), model_name);
                            Ok(AgentOutput::Finish(AgentFinish { return_values }))
                        }
                    }
                }
                Err(e) => {
                    // 出错时返回错误信息
                    // 从OpenAI模型获取模型名称，如果没有则使用默认值
                    let model_name = if let Some(ref model) = openai_model_clone {
                        model.model_name().map(|s| s.to_string()).unwrap_or("unknown".to_string())
                    } else {
                        "unknown".to_string()
                    };
                    let mut return_values = std::collections::HashMap::new();
                    return_values.insert("answer".to_string(), format!("Model invocation failed: {}", e));
                    return_values.insert("model".to_string(), model_name);
                    Ok(AgentOutput::Finish(AgentFinish { return_values }))
                }
            }
        })
    }

    fn clone_to_owned(
        &self,
    ) -> Box<dyn Runnable<std::collections::HashMap<String, String>, AgentOutput> + Send + Sync>
    {
        Box::new(self.clone())
    }
}
