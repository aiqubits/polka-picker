// MCP Ping 接口使用示例
use rust_agent::{McpClient, SimpleMcpClient, McpServer, SimpleMcpServer};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始 MCP Ping 示例");
    
    // 创建 MCP 服务器
    let server = SimpleMcpServer::new();
    
    // 启动服务器
    let server_address = "127.0.0.1:6000";
    server.start(server_address).await?;
    
    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 创建 MCP 客户端
    let mut client = SimpleMcpClient::new(format!("http://{}", server_address));
    
    // 连接到服务器 - 修复URL格式
    client.connect(&format!("http://{}", server_address)).await?;
    client.set_server_connected(true);
    
    // 执行多次 ping 测试
    for i in 1..=3 {
        let start_time = std::time::Instant::now();
        
        match client.ping().await {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                println!("Ping {} 成功，响应时间: {:?}", i, elapsed);
            }
            Err(e) => {
                println!("Ping {} 失败: {}", i, e);
            }
        }
        
        // 等待 1 秒再进行下一次 ping
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    // 停止服务器
    server.stop().await?;
    
    println!("MCP Ping 示例完成");
    
    Ok(())
}