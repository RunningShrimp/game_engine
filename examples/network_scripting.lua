-- 网络功能脚本示例
-- 演示如何在Lua脚本中使用游戏引擎的网络功能

-- 1. TCP客户端连接
local function tcp_example()
    -- 连接到服务器
    local result = tcp_connect("client1", "localhost", 8080)
    print("TCP连接结果:", result)

    -- 发送数据
    local send_result = tcp_send("client1", "Hello Server!")
    print("TCP发送结果:", send_result)
end

-- 2. WebSocket连接
local function websocket_example()
    -- 连接到WebSocket服务器
    local result = ws_connect("ws_client1", "ws://localhost:8080/ws")
    print("WebSocket连接结果:", result)
end

-- 3. HTTP请求
local function http_example()
    -- 发送GET请求
    local result = http_get("https://api.example.com/data")
    print("HTTP GET结果:", result)
end

-- 主函数
function main()
    print("=== 游戏引擎网络API示例 ===")

    -- 执行TCP示例
    tcp_example()

    -- 执行WebSocket示例
    websocket_example()

    -- 执行HTTP示例
    http_example()
end

-- 运行主函数
main()
