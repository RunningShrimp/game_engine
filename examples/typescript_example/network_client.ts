// TypeScript网络客户端示例
// 演示如何使用网络API进行TCP、UDP、WebSocket和HTTP通信

interface NetworkMessage {
    type: string;
    data: any;
    timestamp?: number;
}

interface PlayerInput {
    type: string;
    playerId: number;
    sequence: number;
    keys: {
        w: boolean;
        a: boolean;
        s: boolean;
        d: boolean;
    };
}

// ============================================================================
// TCP客户端示例
// ============================================================================

async function tcpClientExample(): Promise<void> {
    console.log("=== TCP客户端示例 ===");

    try {
        // 连接到服务器
        const success = await Network.tcpConnect("tcp_client_1", "localhost", 8080);
        if (success) {
            console.log("TCP连接成功");

            // 发送数据
            const message = "Hello from TypeScript TCP client!";
            const sent = await Network.tcpSend("tcp_client_1", message);
            console.log(`已发送 ${sent} 字节`);

            // 接收数据
            const response = await Network.tcpReceive("tcp_client_1");
            console.log(`收到响应: ${response}`);

            // 关闭连接
            await Network.tcpClose("tcp_client_1");
            console.log("TCP连接已关闭");
        }
    } catch (error) {
        console.error("TCP客户端错误:", error);
    }
}

// ============================================================================
// WebSocket客户端示例
// ============================================================================

async function websocketClientExample(): Promise<void> {
    console.log("\n=== WebSocket客户端示例 ===");

    try {
        // 连接到WebSocket服务器
        const success = await Network.wsConnect("ws_client_1", "ws://localhost:8080/ws");
        if (success) {
            console.log("WebSocket连接成功");

            // 发送JSON消息
            const message: NetworkMessage = {
                type: "greeting",
                data: { message: "Hello from TypeScript!" },
                timestamp: Date.now()
            };
            await Network.wsSend("ws_client_1", JSON.stringify(message));
            console.log("已发送WebSocket消息");

            // 接收消息
            const response = await Network.wsReceive("ws_client_1");
            console.log(`收到WebSocket消息: ${response}`);

            // 关闭连接
            await Network.wsClose("ws_client_1");
            console.log("WebSocket连接已关闭");
        }
    } catch (error) {
        console.error("WebSocket客户端错误:", error);
    }
}

// ============================================================================
// HTTP客户端示例
// ============================================================================

async function httpClientExample(): Promise<void> {
    console.log("\n=== HTTP客户端示例 ===");

    try {
        // 发送GET请求
        const getResponse = await Network.httpGet("http://localhost:8080/api/status");
        console.log(`HTTP GET响应: ${getResponse}`);

        // 解析JSON响应
        const status = JSON.parse(getResponse);
        console.log("服务器状态:", status);

        // 发送POST请求
        const postData = {
            player_id: 123,
            action: "move",
            position: { x: 100, y: 200, z: 0 }
        };
        const postResponse = await Network.httpPost(
            "http://localhost:8080/api/action",
            JSON.stringify(postData)
        );
        console.log(`HTTP POST响应: ${postResponse}`);
    } catch (error) {
        console.error("HTTP客户端错误:", error);
    }
}

// ============================================================================
// UDP客户端示例
// ============================================================================

async function udpClientExample(): Promise<void> {
    console.log("\n=== UDP客户端示例 ===");

    try {
        // 绑定UDP套接字
        const success = await Network.udpBind("udp_client_1", "localhost", 9090);
        if (success) {
            console.log("UDP套接字绑定成功");

            // 发送数据到目标
            const message = "Hello from TypeScript UDP client!";
            const sent = await Network.udpSendTo(
                "udp_client_1",
                "localhost",
                9091,
                message
            );
            console.log(`已发送 ${sent} 字节`);

            // 接收数据
            const response = await Network.udpReceive("udp_client_1");
            console.log(`收到UDP数据: ${response}`);

            // 关闭套接字
            await Network.udpClose("udp_client_1");
            console.log("UDP套接字已关闭");
        }
    } catch (error) {
        console.error("UDP客户端错误:", error);
    }
}

// ============================================================================
// 游戏网络同步示例
// ============================================================================

async function gameNetworkSync(): Promise<void> {
    console.log("\n=== 游戏网络同步示例 ===");

    try {
        // 连接到游戏服务器
        await Network.tcpConnect("game_client", "game-server.example.com", 7777);

        // 发送玩家输入
        const input: PlayerInput = {
            type: "input",
            playerId: 12345,
            sequence: 1,
            keys: { w: true, a: false, s: false, d: false }
        };

        await Network.tcpSend("game_client", JSON.stringify(input));
        console.log("已发送玩家输入");

        // 接收服务器状态更新
        const stateUpdate = await Network.tcpReceive("game_client");
        const serverState = JSON.parse(stateUpdate);
        console.log("收到服务器状态:", serverState);

        // 清理
        await Network.tcpClose("game_client");
    } catch (error) {
        console.error("游戏网络同步错误:", error);
    }
}

// ============================================================================
// 多人游戏房间示例
// ============================================================================

class MultiplayerRoom {
    private clientId: string;

    constructor(roomId: string, playerId: number) {
        this.clientId = `room_${roomId}_player_${playerId}`;
    }

    async connect(roomServer: string): Promise<boolean> {
        const success = await Network.wsConnect(this.clientId, roomServer);
        if (success) {
            console.log("已连接到房间服务器");
        }
        return success;
    }

    async joinRoom(roomId: string, playerId: number): Promise<void> {
        const joinMessage = {
            action: "join",
            room_id: roomId,
            player_id: playerId
        };
        await Network.wsSend(this.clientId, JSON.stringify(joinMessage));
        console.log(`正在加入房间 ${roomId}...`);
    }

    async sendAction(action: string, data: any): Promise<void> {
        const message = {
            action,
            data,
            timestamp: Date.now()
        };
        await Network.wsSend(this.clientId, JSON.stringify(message));
    }

    async receiveUpdate(): Promise<any> {
        const update = await Network.wsReceive(this.clientId);
        return JSON.parse(update);
    }

    async disconnect(): Promise<void> {
        await Network.wsClose(this.clientId);
        console.log("已离开房间");
    }
}

async function multiplayerRoomExample(): Promise<void> {
    console.log("\n=== 多人游戏房间示例 ===");

    try {
        const room = new MultiplayerRoom("room_12345", 12345);

        // 连接到房间服务器
        await room.connect("wss://rooms.example.com/ws");

        // 加入房间
        await room.joinRoom("room_12345", 12345);

        // 接收房间状态
        const roomState = await room.receiveUpdate();
        console.log("房间状态:", roomState);

        // 发送游戏动作
        await room.sendAction("move", { x: 100, y: 200, z: 0 });
        console.log("已发送游戏动作");

        // 接收多个更新
        for (let i = 1; i <= 5; i++) {
            const update = await room.receiveUpdate();
            console.log(`房间更新 ${i}:`, update);
        }

        // 断开连接
        await room.disconnect();
    } catch (error) {
        console.error("多人游戏房间错误:", error);
    }
}

// ============================================================================
// 实时多人游戏类
// ============================================================================

class RealtimeGameClient {
    private tcpClient: string;
    private wsClient: string;
    private playerId: number;
    private sequence: number = 0;

    constructor(server: string, port: number, playerId: number) {
        this.tcpClient = `game_tcp_${playerId}`;
        this.wsClient = `game_ws_${playerId}`;
        this.playerId = playerId;
    }

    async connect(): Promise<void> {
        // TCP连接用于游戏数据
        await Network.tcpConnect(this.tcpClient, "localhost", 7777);

        // WebSocket连接用于实时事件
        await Network.wsConnect(this.wsClient, "ws://localhost:7777/events");

        console.log(`玩家 ${this.playerId} 已连接到游戏服务器`);
    }

    async sendInput(keys: { w: boolean; a: boolean; s: boolean; d: boolean }): Promise<void> {
        const input: PlayerInput = {
            type: "input",
            playerId: this.playerId,
            sequence: this.sequence++,
            keys
        };

        await Network.tcpSend(this.tcpClient, JSON.stringify(input));
    }

    async receiveState(): Promise<any> {
        const stateData = await Network.tcpReceive(this.tcpClient);
        return JSON.parse(stateData);
    }

    async sendChat(message: string): Promise<void> {
        const chatMsg = {
            type: "chat",
            playerId: this.playerId,
            message,
            timestamp: Date.now()
        };

        await Network.wsSend(this.wsClient, JSON.stringify(chatMsg));
    }

    async receiveEvent(): Promise<any> {
        const eventData = await Network.wsReceive(this.wsClient);
        return JSON.parse(eventData);
    }

    async disconnect(): Promise<void> {
        await Network.tcpClose(this.tcpClient);
        await Network.wsClose(this.wsClient);
        console.log(`玩家 ${this.playerId} 已断开连接`);
    }
}

async function realtimeGameExample(): Promise<void> {
    console.log("\n=== 实时多人游戏示例 ===");

    try {
        const gameClient = new RealtimeGameClient("localhost", 7777, 12345);

        // 连接到服务器
        await gameClient.connect();

        // 模拟游戏循环
        for (let i = 0; i < 10; i++) {
            // 发送输入
            await gameClient.sendInput({
                w: i % 2 === 0,
                a: i % 3 === 0,
                s: false,
                d: i % 5 === 0
            });

            // 接收状态
            const state = await gameClient.receiveState();
            console.log(`帧 ${i}: 服务器状态`, state);

            // 延迟
            await new Promise(resolve => setTimeout(resolve, 100));
        }

        // 发送聊天消息
        await gameClient.sendChat("Hello everyone!");

        // 接收事件
        const event = await gameClient.receiveEvent();
        console.log("收到事件:", event);

        // 断开连接
        await gameClient.disconnect();
    } catch (error) {
        console.error("实时游戏错误:", error);
    }
}

// ============================================================================
// 主函数
// ============================================================================

async function main(): Promise<void> {
    console.log("TypeScript网络客户端示例");
    console.log("========================");

    // 运行所有示例
    await tcpClientExample();
    await websocketClientExample();
    await httpClientExample();
    await udpClientExample();
    await gameNetworkSync();
    await multiplayerRoomExample();
    await realtimeGameExample();

    console.log("\n所有示例执行完毕!");
}

// 运行主函数
main().catch(console.error);
