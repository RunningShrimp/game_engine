// 网络功能JavaScript示例
// 演示如何在JavaScript脚本中使用游戏引擎的网络功能

// 1. TCP客户端连接
async function tcpExample() {
    console.log("=== TCP客户端示例 ===");

    // 连接到服务器
    const result = await tcp_connect("client1", "localhost", 8080);
    console.log("TCP连接结果:", result);

    // 发送数据
    const sendResult = await tcp_send("client1", "Hello Server!");
    console.log("TCP发送结果:", sendResult);
}

// 2. WebSocket连接
async function websocketExample() {
    console.log("=== WebSocket示例 ===");

    // 连接到WebSocket服务器
    const result = await ws_connect("ws_client1", "ws://localhost:8080/ws");
    console.log("WebSocket连接结果:", result);
}

// 3. HTTP请求
async function httpExample() {
    console.log("=== HTTP请求示例 ===");

    // 发送GET请求
    const result = await http_get("https://api.example.com/data");
    console.log("HTTP GET结果:", result);
}

// 主函数
async function main() {
    console.log("=== 游戏引擎网络API示例 ===");

    // 执行TCP示例
    await tcpExample();

    // 执行WebSocket示例
    await websocketExample();

    // 执行HTTP示例
    await httpExample();
}

// 运行主函数
main().catch(console.error);
