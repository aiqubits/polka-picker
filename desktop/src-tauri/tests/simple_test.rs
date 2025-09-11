// 简单测试文件
#[tokio::test]
async fn simple_test() {
    // 这是一个非常简单的异步测试，只打印一条消息
    println!("Running simple async test");
    // 添加一个短暂的延迟以确保异步代码执行
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    // 一个简单的断言
    assert_eq!(1 + 1, 2);
}