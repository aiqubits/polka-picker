// 基于MCP的AI Agent聊天机器人示例
use std::path::PathBuf;
use rust_agent::{run_agent, OpenAIChatModel, McpClient, SimpleMcpClient, McpTool, McpAgent, SimpleMemory, BaseMemory, CompositeMemory};
use std::sync::Arc;
use std::collections::HashMap;
use chrono;
use serde_json::{Value, json};
use anyhow::Error;

// 初始化日志记录器
use log::LevelFilter;
use env_logger;
use log::{info, error};

#[tokio::main]
async fn main() {
    // 初始化日志记录器
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
    
    info!("=== Rust Agent 使用示例 ===");
    
    // 获取记忆类型配置
    let memory_type = std::env::var("MEMORY_TYPE").unwrap_or_else(|_| "composite".to_string());
    let summary_threshold = std::env::var("SUMMARY_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let recent_messages_count = std::env::var("RECENT_MESSAGES_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    
    info!("使用记忆类型: {}", memory_type);
    info!("摘要阈值: {}", summary_threshold);
    info!("保留最近消息数: {}", recent_messages_count);
    
    // 从环境变量获取API密钥和基本URL
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "OPENAI_API_KEY".to_string());
    let base_url = std::env::var("OPENAI_API_URL").ok();
    let mcp_url = std::env::var("MCP_URL").unwrap_or("http://localhost:8000/mcp".to_string());
    
    // 创建OpenAI模型实例 - 支持Openai兼容 API
    let model = OpenAIChatModel::new(api_key.clone(), base_url)
        .with_model(std::env::var("OPENAI_API_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()))
        .with_temperature(0.7)
        .with_max_tokens(8*1024);
    
    // 初始化MCP客户端
    // 在初始化 MCP 客户端后，自定义工具和工具处理器
    let mut mcp_client = SimpleMcpClient::new(mcp_url.clone());
    
    // 清空默认工具（可选）
    mcp_client.clear_tools();
    
    // 添加自定义工具
    mcp_client.add_tools(vec![
        McpTool {
            name: "get_weather".to_string(),
            description: format!(
                "Get weather information for a specified city. For example: 'What's the weather like in Beijing?'.
                The parameter request body you should extract is: '\"parameters\": {{ \"city\": \"{}\" }}'",
                "city".to_string()),
        },
        McpTool {
            name: "simple_calculate".to_string(),
            description: format!(
                "Execute simple mathematical calculations. For example: 'What is 9.11 plus 9.8?'.
                The parameter request body you should extract is: '\"parameters\": {{ \"expression\": \"{}\" }}'",
                "expression".to_string()),
        },
    ]);
    
    // 注册自定义工具处理器
    mcp_client.register_tool_handler("get_weather".to_string(), |params: HashMap<String, Value>| async move {
        let default_city = Value::String("Shanghai".to_string());
        let city_value = params.get("city").unwrap_or(&default_city);
        let city = city_value.as_str().unwrap_or("Shanghai");
        Ok(json!({
            "city": city,
            "temperature": "25°C",
            "weather": "Sunny",
            "humidity": "40%",
            "updated_at": chrono::Utc::now().to_rfc3339()
        }))
    });
    
    mcp_client.register_tool_handler("simple_calculate".to_string(), |params: HashMap<String, Value>| async move {
        let expression_value = params.get("expression").ok_or_else(|| Error::msg("Missing calculation expression"))?;
        let expression = expression_value.as_str().ok_or_else(|| Error::msg("Expression format error"))?;
        
        // 解析表达式，提取操作数和运算符
        let result = parse_and_calculate(expression)?;
        
        Ok(json!({
            "expression": expression,
            "result": result,
            "calculated_at": chrono::Utc::now().to_rfc3339()
        }))
    });
    
    // 不连接到 MCP 服务器，仅使用本地工具
    info!("Using local tools only, not connecting to MCP server...");

    info!("Using model: {}", model.model_name().map_or("Model not specified", |v| v));
    info!("Using API URL: {}", model.base_url());
    info!("----------------------------------------");
    
    let client_arc: Arc<dyn McpClient> = Arc::new(mcp_client);
    
    // 根据配置创建不同类型的记忆模块实例
    let memory: Box<dyn BaseMemory> = match memory_type.as_str() {
        "simple" => {
            info!("使用SimpleMemory (仅内存记忆)");
            Box::new(SimpleMemory::new())
        },
        "composite" => {
            info!("使用CompositeMemory (组合记忆 - 支持中长期记忆和摘要记忆)");
            // 使用新的简化接口，只需要提供必要的参数
            // session_id将在内部自动生成
            let memory = CompositeMemory::with_basic_params(
                PathBuf::from("./data/memory"),
                summary_threshold,
                recent_messages_count,
            ).await.expect("Failed to create composite memory");
            
            Box::new(memory)
        },
        _ => {
            error!("未知的记忆类型: {}, 使用默认的SimpleMemory", memory_type);
            Box::new(SimpleMemory::new())
        }
    };

    // 创建Agent实例，并传递temperature、max_tokens和memory参数
    let user_system_prompt = "You are an AI assistant that can use tools to answer user questions. Please decide whether to use tools based on the user's needs.".to_string();
    
    let mut agent = McpAgent::with_openai_model_and_memory(
        client_arc.clone(),
        user_system_prompt,
        model.clone(),
        memory
    );
    
    // 尝试从MCP服务器自动获取工具并添加到Agent
    if let Err(e) = agent.auto_add_tools().await {
        error!("Failed to auto add tools from MCP server: {}", e);
    }
    
    println!("基于MCP的AI Agent聊天机器人已启动！");
    println!("记忆类型: {}", memory_type);
    if memory_type == "composite" {
        println!("摘要功能: 已启用 (阈值: {} 条消息)", summary_threshold);
        println!("中长期记忆: 已启用");
    }
    println!("输入'退出'结束对话");
    println!("----------------------------------------");
    println!("Using tools example:");
    let tools = client_arc.get_tools().await.unwrap_or_else(|e| {
        error!("Failed to get tools from MCP server: {}", e);
        // 返回本地工具列表
        vec![
            McpTool {
                name: "get_weather".to_string(),
                description: "Get the weather information for a specified city. For example: 'What's the weather like in Beijing?'".to_string(),
            },
            McpTool {
                name: "simple_calculate".to_string(),
                description: "Perform simple mathematical calculations. For example: 'What is 9.11 plus 9.8?'".to_string(),
            },
        ]
    });
    
    // 打印工具列表
    let mut index = 0;
    for tool in &tools {
        index += 1;

        println!("{index}. {}: {}", tool.name, tool.description);
    }
    
    println!("----------------------------------------");
    // 对话循环
    loop {
        let mut user_input = String::new();
        println!("你: ");
        std::io::stdin().read_line(&mut user_input).expect("读取输入失败");
        println!("");
        let user_input = user_input.trim();
        
        if user_input.to_lowercase() == "退出" || user_input.to_lowercase() == "exit" {
            println!("再见！");
            break;
        }
        
        // 创建输入上下文
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), serde_json::Value::String(user_input.to_string()));
        
        // 运行Agent
        match run_agent(&agent, user_input.to_string()).await {
            Ok(response) => {
                // 尝试解析response为JSON并提取content字段
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response) {
                    if let Some(content) = json_value.get("content").and_then(|v| v.as_str()) {
                        println!("助手: {}", content);
                    } else {
                        println!("助手: {}", response);
                    }
                } else {
                    println!("助手: {}", response);
                }
            },
            Err(e) => {
                println!("助手: 抱歉，处理您的请求时出现错误: {}", e);
            },
        }
        
        info!("----------------------------------------");
    }
    
    // 打印对话历史
    info!("对话历史:");
    if let Some(memory) = agent.get_memory() {
        match memory.load_memory_variables(&HashMap::new()).await {
            Ok(memories) => {
                if let Some(chat_history) = memories.get("chat_history") {
                    if let serde_json::Value::Array(messages) = chat_history {
                        info!("总消息数: {}", messages.len());
                        for (i, message) in messages.iter().enumerate() {
                            if let serde_json::Value::Object(msg) = message {
                                let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                // 限制内容长度以便显示
                                let display_content = if content.len() > 100 {
                                    format!("{}...", &content[..100])
                                } else {
                                    content.to_string()
                                };
                                info!("{}. {}: {}", i + 1, role, display_content);
                            }
                        }
                    }
                }
                
                // 如果有摘要，也打印出来
                if let Some(summary) = memories.get("summary") {
                    if let serde_json::Value::String(summary_text) = summary {
                        info!("对话摘要: {}", summary_text);
                    }
                }
            },
            Err(e) => {
                info!("Failed to load memory variables: {}", e);
            }
        }
    } else {
        info!("No memory available");
    }
    
    // 断开MCP连接
    if let Err(e) = client_arc.disconnect().await {
        error!("Failed to disconnect MCP client: {}", e);
    }
}

// 解析表达式并计算结果
fn parse_and_calculate(expression: &str) -> Result<f64, Error> {
    let expression = expression.replace(" ", "");
    
    // 尝试匹配不同的运算符
    for op_char in ["+", "-", "*", "/"].iter() {
        if let Some(pos) = expression.find(op_char) {
            // 提取左右操作数
            let left_str = &expression[0..pos];
            let right_str = &expression[pos + 1..];
            
            // 转换为浮点数
            let left = left_str.parse::<f64>().map_err(|e| 
                Error::msg(format!("Left operand format error: {}", e)))?;
            let right = right_str.parse::<f64>().map_err(|e| 
                Error::msg(format!("Right operand format error: {}", e)))?;
            
            // 执行相应的运算
            let result = match *op_char {
                "+" => left + right,
                "-" => left - right,
                "*" => left * right,
                "/" => {
                    if right == 0.0 {
                        return Err(Error::msg("除数不能为零"));
                    }
                    left / right
                },
                _ => unreachable!()
            };
            
            return Ok(result);
        }
    }
    
    // 如果没有找到运算符，尝试将整个表达式解析为数字
    if let Ok(number) = expression.parse::<f64>() {
        return Ok(number);
    }
    
    Err(Error::msg(format!("Failed to parse expression: {}", expression)))
}