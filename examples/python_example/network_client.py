# Python网络客户端示例
# 演示如何使用网络API进行TCP、UDP、WebSocket和HTTP通信

import json
import asyncio
from typing import Dict, Any, Optional
from dataclasses import dataclass
from datetime import datetime

# ============================================================================
# 数据模型
# ============================================================================

@dataclass
class NetworkMessage:
    """网络消息"""
    message_type: str
    data: Any
    timestamp: Optional[float] = None

    def to_dict(self) -> dict:
        return {
            "type": self.message_type,
            "data": self.data,
            "timestamp": self.timestamp or datetime.now().timestamp()
        }

@dataclass
class PlayerInput:
    """玩家输入"""
    message_type: str
    player_id: int
    sequence: int
    keys: Dict[str, bool]

    def to_json(self) -> str:
        return json.dumps({
            "type": self.message_type,
            "player_id": self.player_id,
            "sequence": self.sequence,
            "keys": self.keys
        })

# ============================================================================
# TCP客户端示例
# ============================================================================

def tcp_client_example():
    """TCP客户端示例"""
    print("=== TCP客户端示例 ===")

    try:
        # 连接到服务器
        success = Network.tcp_connect("tcp_client_1", "localhost", 8080)
        if success:
            print("TCP连接成功")

            # 发送数据
            message = "Hello from Python TCP client!"
            sent = Network.tcp_send("tcp_client_1", message)
            print(f"已发送 {sent} 字节")

            # 接收数据
            response = Network.tcp_receive("tcp_client_1")
            print(f"收到响应: {response}")

            # 关闭连接
            Network.tcp_close("tcp_client_1")
            print("TCP连接已关闭")
        else:
            print("TCP连接失败")
    except Exception as e:
        print(f"TCP客户端错误: {e}")

# ============================================================================
# WebSocket客户端示例
# ============================================================================

def websocket_client_example():
    """WebSocket客户端示例"""
    print("\n=== WebSocket客户端示例 ===")

    try:
        # 连接到WebSocket服务器
        success = Network.ws_connect("ws_client_1", "ws://localhost:8080/ws")
        if success:
            print("WebSocket连接成功")

            # 发送JSON消息
            message = NetworkMessage(
                message_type="greeting",
                data={"message": "Hello from Python!"},
                timestamp=datetime.now().timestamp()
            )
            Network.ws_send("ws_client_1", json.dumps(message.to_dict()))
            print("已发送WebSocket消息")

            # 接收消息
            response = Network.ws_receive("ws_client_1")
            print(f"收到WebSocket消息: {response}")

            # 关闭连接
            Network.ws_close("ws_client_1")
            print("WebSocket连接已关闭")
        else:
            print("WebSocket连接失败")
    except Exception as e:
        print(f"WebSocket客户端错误: {e}")

# ============================================================================
# HTTP客户端示例
# ============================================================================

def http_client_example():
    """HTTP客户端示例"""
    print("\n=== HTTP客户端示例 ===")

    try:
        # 发送GET请求
        get_response = Network.http_get("http://localhost:8080/api/status")
        print(f"HTTP GET响应: {get_response}")

        # 解析JSON响应
        status = json.loads(get_response)
        print("服务器状态:", status)

        # 发送POST请求
        post_data = {
            "player_id": 123,
            "action": "move",
            "position": {"x": 100, "y": 200, "z": 0}
        }
        post_response = Network.http_post(
            "http://localhost:8080/api/action",
            json.dumps(post_data)
        )
        print(f"HTTP POST响应: {post_response}")
    except Exception as e:
        print(f"HTTP客户端错误: {e}")

# ============================================================================
# UDP客户端示例
# ============================================================================

def udp_client_example():
    """UDP客户端示例"""
    print("\n=== UDP客户端示例 ===")

    try:
        # 绑定UDP套接字
        success = Network.udp_bind("udp_client_1", "localhost", 9090)
        if success:
            print("UDP套接字绑定成功")

            # 发送数据到目标
            message = "Hello from Python UDP client!"
            sent = Network.udp_send_to(
                "udp_client_1",
                "localhost",
                9091,
                message
            )
            print(f"已发送 {sent} 字节")

            # 接收数据
            response = Network.udp_receive("udp_client_1")
            print(f"收到UDP数据: {response}")

            # 关闭套接字
            Network.udp_close("udp_client_1")
            print("UDP套接字已关闭")
        else:
            print("UDP绑定失败")
    except Exception as e:
        print(f"UDP客户端错误: {e}")

# ============================================================================
# 游戏网络同步示例
# ============================================================================

def game_network_sync():
    """游戏网络同步示例"""
    print("\n=== 游戏网络同步示例 ===")

    try:
        # 连接到游戏服务器
        Network.tcp_connect("game_client", "game-server.example.com", 7777)

        # 发送玩家输入
        input_data = PlayerInput(
            message_type="input",
            player_id=12345,
            sequence=1,
            keys={"w": True, "a": False, "s": False, "d": False}
        )

        Network.tcp_send("game_client", input_data.to_json())
        print("已发送玩家输入")

        # 接收服务器状态更新
        state_update = Network.tcp_receive("game_client")
        server_state = json.loads(state_update)
        print("收到服务器状态:", server_state)

        # 清理
        Network.tcp_close("game_client")
    except Exception as e:
        print(f"游戏网络同步错误: {e}")

# ============================================================================
# 多人游戏房间类
# ============================================================================

class MultiplayerRoom:
    """多人游戏房间客户端"""

    def __init__(self, room_id: str, player_id: int):
        self.client_id = f"room_{room_id}_player_{player_id}"
        self.room_id = room_id
        self.player_id = player_id

    async def connect(self, room_server: str) -> bool:
        """连接到房间服务器"""
        success = Network.ws_connect(self.client_id, room_server)
        if success:
            print("已连接到房间服务器")
        return success

    async def join_room(self, room_id: str, player_id: int) -> None:
        """加入房间"""
        join_message = {
            "action": "join",
            "room_id": room_id,
            "player_id": player_id
        }
        Network.ws_send(self.client_id, json.dumps(join_message))
        print(f"正在加入房间 {room_id}...")

    async def send_action(self, action: str, data: Any) -> None:
        """发送游戏动作"""
        message = {
            "action": action,
            "data": data,
            "timestamp": datetime.now().timestamp()
        }
        Network.ws_send(self.client_id, json.dumps(message))

    async def receive_update(self) -> Dict[str, Any]:
        """接收房间更新"""
        update = Network.ws_receive(self.client_id)
        return json.loads(update)

    async def disconnect(self) -> None:
        """断开连接"""
        Network.ws_close(self.client_id)
        print("已离开房间")

async def multiplayer_room_example():
    """多人游戏房间示例"""
    print("\n=== 多人游戏房间示例 ===")

    try:
        room = MultiplayerRoom("room_12345", 12345)

        # 连接到房间服务器
        await room.connect("wss://rooms.example.com/ws")

        # 加入房间
        await room.join_room("room_12345", 12345)

        # 接收房间状态
        room_state = await room.receive_update()
        print("房间状态:", room_state)

        # 发送游戏动作
        await room.send_action("move", {"x": 100, "y": 200, "z": 0})
        print("已发送游戏动作")

        # 接收多个更新
        for i in range(1, 6):
            update = await room.receive_update()
            print(f"房间更新 {i}:", update)

        # 断开连接
        await room.disconnect()
    except Exception as e:
        print(f"多人游戏房间错误: {e}")

# ============================================================================
# 实时多人游戏客户端类
# ============================================================================

class RealtimeGameClient:
    """实时多人游戏客户端"""

    def __init__(self, server: str, port: int, player_id: int):
        self.tcp_client = f"game_tcp_{player_id}"
        self.ws_client = f"game_ws_{player_id}"
        self.player_id = player_id
        self.sequence = 0

    async def connect(self) -> None:
        """连接到服务器"""
        # TCP连接用于游戏数据
        Network.tcp_connect(self.tcp_client, "localhost", 7777)

        # WebSocket连接用于实时事件
        Network.ws_connect(self.ws_client, "ws://localhost:7777/events")

        print(f"玩家 {self.player_id} 已连接到游戏服务器")

    async def send_input(self, keys: Dict[str, bool]) -> None:
        """发送玩家输入"""
        input_data = PlayerInput(
            message_type="input",
            player_id=self.player_id,
            sequence=self.sequence,
            keys=keys
        )
        self.sequence += 1

        Network.tcp_send(self.tcp_client, input_data.to_json())

    async def receive_state(self) -> Dict[str, Any]:
        """接收服务器状态"""
        state_data = Network.tcp_receive(self.tcp_client)
        return json.loads(state_data)

    async def send_chat(self, message: str) -> None:
        """发送聊天消息"""
        chat_msg = {
            "type": "chat",
            "player_id": self.player_id,
            "message": message,
            "timestamp": datetime.now().timestamp()
        }

        Network.ws_send(self.ws_client, json.dumps(chat_msg))

    async def receive_event(self) -> Dict[str, Any]:
        """接收事件"""
        event_data = Network.ws_receive(self.ws_client)
        return json.loads(event_data)

    async def disconnect(self) -> None:
        """断开连接"""
        Network.tcp_close(self.tcp_client)
        Network.ws_close(self.ws_client)
        print(f"玩家 {self.player_id} 已断开连接")

async def realtime_game_example():
    """实时多人游戏示例"""
    print("\n=== 实时多人游戏示例 ===")

    try:
        game_client = RealtimeGameClient("localhost", 7777, 12345)

        # 连接到服务器
        await game_client.connect()

        # 模拟游戏循环
        for i in range(10):
            # 发送输入
            await game_client.send_input({
                "w": i % 2 == 0,
                "a": i % 3 == 0,
                "s": False,
                "d": i % 5 == 0
            })

            # 接收状态
            state = await game_client.receive_state()
            print(f"帧 {i}: 服务器状态", state)

            # 延迟
            await asyncio.sleep(0.1)

        # 发送聊天消息
        await game_client.send_chat("Hello everyone!")

        # 接收事件
        event = await game_client.receive_event()
        print("收到事件:", event)

        # 断开连接
        await game_client.disconnect()
    except Exception as e:
        print(f"实时游戏错误: {e}")

# ============================================================================
# 异步网络辅助函数
# ============================================================================

async def wait_for_connection(client_id: str, timeout: float = 5.0) -> bool:
    """等待连接建立"""
    start_time = datetime.now().timestamp()
    while (datetime.now().timestamp() - start_time) < timeout:
        status = Network.get_connection_status()
        # 检查连接状态（简化示例）
        await asyncio.sleep(0.1)
    return True

async def retry_network_call(func, max_retries: int = 3, delay: float = 1.0):
    """重试网络调用"""
    for attempt in range(max_retries):
        try:
            return await func()
        except Exception as e:
            if attempt == max_retries - 1:
                raise
            print(f"重试 {attempt + 1}/{max_retries}...")
            await asyncio.sleep(delay)

# ============================================================================
# 主函数
# ============================================================================

async def main_async():
    """异步主函数"""
    print("Python网络客户端示例")
    print("======================")

    # 同步示例
    tcp_client_example()
    websocket_client_example()
    http_client_example()
    udp_client_example()
    game_network_sync()

    # 异步示例
    await multiplayer_room_example()
    await realtime_game_example()

    print("\n所有示例执行完毕!")

def main():
    """主函数"""
    # 运行异步示例
    asyncio.run(main_async())

if __name__ == "__main__":
    main()
