-- Lua网络客户端示例
-- 演示如何使用网络API进行TCP、UDP、WebSocket和HTTP通信

-- ============================================================================
-- TCP客户端示例
-- ============================================================================

function tcp_client_example()
    print("=== TCP客户端示例 ===")

    -- 连接到服务器
    local success = Network.tcp_connect("tcp_client_1", "localhost", 8080)
    if success then
        print("TCP连接成功")

        -- 发送数据
        local sent = Network.tcp_send("tcp_client_1", "Hello from Lua TCP client!")
        print("已发送 " .. sent .. " 字节")

        -- 接收数据
        local response = Network.tcp_receive("tcp_client_1")
        print("收到响应: " .. response)

        -- 关闭连接
        Network.tcp_close("tcp_client_1")
        print("TCP连接已关闭")
    else
        print("TCP连接失败")
    end
end

-- ============================================================================
-- WebSocket客户端示例
-- ============================================================================

function websocket_client_example()
    print("\n=== WebSocket客户端示例 ===")

    -- 连接到WebSocket服务器
    local success = Network.ws_connect("ws_client_1", "ws://localhost:8080/ws")
    if success then
        print("WebSocket连接成功")

        -- 发送消息
        Network.ws_send("ws_client_1", '{"type":"greeting","message":"Hello from Lua!"}')
        print("已发送WebSocket消息")

        -- 接收消息
        local response = Network.ws_receive("ws_client_1")
        print("收到WebSocket消息: " .. response)

        -- 关闭连接
        Network.ws_close("ws_client_1")
        print("WebSocket连接已关闭")
    else
        print("WebSocket连接失败")
    end
end

-- ============================================================================
-- HTTP客户端示例
-- ============================================================================

function http_client_example()
    print("\n=== HTTP客户端示例 ===")

    -- 发送GET请求
    local response = Network.http_get("http://localhost:8080/api/status")
    print("HTTP GET响应: " .. response)

    -- 发送POST请求
    local post_data = '{"player_id":123,"action":"move"}'
    response = Network.http_post("http://localhost:8080/api/action", post_data)
    print("HTTP POST响应: " .. response)
end

-- ============================================================================
-- UDP客户端示例
-- ============================================================================

function udp_client_example()
    print("\n=== UDP客户端示例 ===")

    -- 绑定UDP套接字
    local success = Network.udp_bind("udp_client_1", "localhost", 9090)
    if success then
        print("UDP套接字绑定成功")

        -- 发送数据到目标
        local sent = Network.udp_send_to(
            "udp_client_1",
            "localhost",
            9091,
            "Hello from Lua UDP client!"
        )
        print("已发送 " .. sent .. " 字节")

        -- 接收数据
        local response = Network.udp_receive("udp_client_1")
        print("收到UDP数据: " .. response)

        -- 关闭套接字
        Network.udp_close("udp_client_1")
        print("UDP套接字已关闭")
    else
        print("UDP绑定失败")
    end
end

-- ============================================================================
-- 游戏网络同步示例
-- ============================================================================

function game_network_sync()
    print("\n=== 游戏网络同步示例 ===")

    -- 连接到游戏服务器
    Network.tcp_connect("game_client", "game-server.example.com", 7777)

    -- 发送玩家输入
    local input = {
        type = "input",
        player_id = 12345,
        sequence = 1,
        keys = { w = true, a = false, s = false, d = false }
    }

    -- 将表序列化为JSON
    local input_json = '{"type":"input","player_id":12345,"sequence":1}'
    Network.tcp_send("game_client", input_json)
    print("已发送玩家输入")

    -- 接收服务器状态更新
    local state_update = Network.tcp_receive("game_client")
    print("收到服务器状态: " .. state_update)

    -- 清理
    Network.tcp_close("game_client")
end

-- ============================================================================
-- 多人游戏房间示例
-- ============================================================================

function multiplayer_room()
    print("\n=== 多人游戏房间示例 ===")

    -- 连接到房间服务器
    Network.ws_connect("room_client", "wss://rooms.example.com/ws")

    -- 加入房间
    local join_msg = '{"action":"join","room_id":"room_12345","player_id":12345}'
    Network.ws_send("room_client", join_msg)
    print("正在加入房间...")

    -- 接收房间状态
    local room_state = Network.ws_receive("room_client")
    print("房间状态: " .. room_state)

    -- 发送游戏动作
    local action_msg = '{"action":"move","x":100,"y":200}'
    Network.ws_send("room_client", action_msg)
    print("已发送游戏动作")

    -- 保持连接并接收更新
    for i = 1, 5 do
        local update = Network.ws_receive("room_client")
        print("房间更新 " .. i .. ": " .. update)
    end

    -- 离开房间
    Network.ws_close("room_client")
    print("已离开房间")
end

-- ============================================================================
-- 主函数
-- ============================================================================

function main()
    print("Lua网络客户端示例")
    print("====================")

    -- 运行所有示例
    tcp_client_example()
    websocket_client_example()
    http_client_example()
    udp_client_example()
    game_network_sync()
    multiplayer_room()

    print("\n所有示例执行完毕!")
end

-- 运行主函数
main()
