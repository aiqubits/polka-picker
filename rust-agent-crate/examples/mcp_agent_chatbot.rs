// 基于MCP的AI Agent聊天机器人示例
use rust_agent::{run_agent, ChatModel, OpenAIChatModel, McpClient, SimpleMcpClient, McpTool, McpAgent};
use std::sync::Arc;
use std::collections::HashMap;
use chrono;
use serde_json::{Value, json};
use anyhow::Error;

#[tokio::main]
async fn main() {
    println!("=== Rust Agent 使用示例 ===");
    // 从环境变量获取API密钥和基本URL
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "OPENAI_API_KEY".to_string());
    let base_url = std::env::var("OPENAI_API_URL").ok();
    let mcp_url = std::env::var("MCP_URL").unwrap_or("http://localhost:8000/mcp".to_string());
    
    // 创建OpenAI模型实例 - 支持MoonShot API
    let model = OpenAIChatModel::new(api_key.clone(), base_url)
        .with_model("kimi-k2-0905-preview".to_string())
        .with_temperature(0.6)
        .with_max_tokens(100*1024);
    
    // 初始化MCP客户端
    // 在初始化 MCP 客户端后，自定义工具和工具处理器
    let mut mcp_client = SimpleMcpClient::new(mcp_url.clone());
    
    // 清空默认工具（可选）
    mcp_client.clear_tools();
    
    // 添加自定义工具
    mcp_client.add_tools(vec![
        McpTool {
            name: "get_weather".to_string(),
            description: "获取指定城市的天气信息。例如：'北京的天气怎么样？'".to_string(),
        },
        McpTool {
            name: "search_knowledge".to_string(),
            description: "搜索知识库信息。例如：'什么是人工智能？'".to_string(),
        },
        McpTool {
            name: "simple_calculate".to_string(),
            description: "执行简单的数学计算。例如：'9.11加9.8等于多少？'".to_string(),
        },
    ]);
    
    // 注册自定义工具处理器
    mcp_client.register_tool_handler("get_weather".to_string(), |params: HashMap<String, Value>| async move {
        let default_city = Value::String("上海".to_string());
        let city_value = params.get("city").unwrap_or(&default_city);
        let city = city_value.as_str().unwrap_or("上海");
        Ok(json!({
            "city": city,
            "temperature": "25°C",
            "weather": "晴",
            "humidity": "40%",
            "updated_at": chrono::Utc::now().to_rfc3339()
        }))
    });
    
    mcp_client.register_tool_handler("search_knowledge".to_string(), |params: HashMap<String, Value>| async move {
        let default_query = Value::String("".to_string());
        let query_value = params.get("query").unwrap_or(&default_query);
        let query = query_value.as_str().unwrap_or("");
        Ok(json!({
            "query": query,
            "results": [
                format!("关于'{}'的详细信息1", query),
                format!("关于'{}'的详细信息2", query)
            ],
            "source": "示例知识库"
        }))
    });
    
    mcp_client.register_tool_handler("simple_calculate".to_string(), |params: HashMap<String, Value>| async move {
        let expression_value = params.get("expression").ok_or_else(|| Error::msg("缺少计算表达式"))?;
        let expression = expression_value.as_str().ok_or_else(|| Error::msg("表达式格式错误"))?;
        
        // 解析表达式，提取操作数和运算符
        let result = parse_and_calculate(expression)?;
        
        Ok(json!({
            "expression": expression,
            "result": result,
            "calculated_at": chrono::Utc::now().to_rfc3339()
        }))
    });

// 解析表达式并计算结果
fn parse_and_calculate(expression: &str) -> Result<f64, Error> {
    println!("原始表达式: {}", expression);
    let expression = expression.replace(" ", "");
    
    // 尝试匹配不同的运算符
    for op_char in ["+", "-", "*", "/"].iter() {
        if let Some(pos) = expression.find(op_char) {
            // 提取左右操作数
            let left_str = &expression[0..pos];
            let right_str = &expression[pos + 1..];
            
            // 转换为浮点数
            let left = left_str.parse::<f64>().map_err(|e| 
                Error::msg(format!("左侧操作数格式错误: {}", e)))?;
            let right = right_str.parse::<f64>().map_err(|e| 
                Error::msg(format!("右侧操作数格式错误: {}", e)))?;
            
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
    
    Err(Error::msg(format!("无法解析表达式: {}", expression)))
}
    
    // 连接到 MCP 服务器
    if let Err(e) = mcp_client.connect(&mcp_url).await {
        println!("MCP连接失败: {}", e);
        println!("使用模拟工具继续...");
    }

    println!("使用模型: {}", model.model_name().unwrap_or("未指定"));
    println!("使用API URL: {}", model.base_url());
    println!("----------------------------------------");
    
    let mcp_client_arc: Arc<dyn McpClient> = match mcp_client.clone() {
        boxed_client => boxed_client.into(),
    };
    
    // 创建Agent实例，并传递temperature和max_tokens参数
    let mut agent = McpAgent::new(
        mcp_client_arc.clone(),
        model.model_name().unwrap_or("kimi-k2-0905-preview").to_string(),
        "你是一个AI助手，可以使用工具来回答用户问题。请根据用户需求决定是否使用工具。".to_string()
    )
    .with_temperature(0.6f32)  // 使用与模型相同的温度设置，显式指定为f32类型
    .with_max_tokens(100*1024);  // 使用与模型相同的最大令牌数
    
    // 自动从MCP客户端获取工具并添加到Agent
    if let Err(e) = agent.auto_add_tools().await {
        println!("自动添加工具到 McpAgent 失败: {}", e);
    }
    
    println!("基于MCP的AI Agent聊天机器人已启动！");
    println!("输入'退出'结束对话");
    println!("----------------------------------------");
    println!("使用工具示例：");
    let tools = mcp_client.get_tools().await.unwrap_or_else(|e| {
        println!("获取工具列表失败: {}", e);
        vec![]
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
        println!("你: ");
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input).expect("读取输入失败");
        println!("");
        let user_input = user_input.trim();
        
        if user_input.to_lowercase() == "退出" || user_input.to_lowercase() == "exit" {
            println!("再见！");
            break;
        }
        
        // 运行Agent
        match run_agent(&agent, user_input.to_string()).await {
            Ok(response) => {
                println!("助手: ");
                println!("{}", response);
            },
            Err(e) => {
                println!("发生错误: {}", e);
            },
        }
        
        println!("----------------------------------------");
    }
    
    // 断开MCP连接
    if let Err(e) = mcp_client_arc.disconnect().await {
        println!("MCP断开连接失败: {}", e);
    }
}