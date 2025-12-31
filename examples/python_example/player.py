# Python示例：玩家控制脚本
#
# 演示如何使用Python编写游戏脚本

from game_engine import Engine, Entity
from typing import Optional, List, Tuple
import math

# 玩家类
class Player:
    def __init__(self) -> None:
        # 创建玩家实体
        self.entity = Engine.spawn_entity()
        self.entity.name = "Player"

        # 设置初始位置
        self.entity.set_position((0.0, 1.0, 0.0))

        # 配置参数
        self.speed = 5.0
        self.jump_force = 10.0

        Engine.log(f"Player created with ID: {self.entity.id}")

    def update(self, delta_time: float) -> None:
        """更新方法（每帧调用）"""
        # 获取输入
        move_x = Engine.get_input_axis("Horizontal")
        move_z = Engine.get_input_axis("Vertical")

        # 计算新位置
        current_pos = self.entity.get_position()
        new_pos = (
            current_pos[0] + move_x * self.speed * delta_time,
            current_pos[1],
            current_pos[2] + move_z * self.speed * delta_time,
        )

        # 应用位置
        self.entity.set_position(new_pos)

        # 跳跃检测
        if Engine.get_input_button("Jump"):
            self.jump()

    def jump(self) -> None:
        """跳跃方法"""
        current_pos = self.entity.get_position()
        new_pos = (
            current_pos[0],
            current_pos[1] + self.jump_force,
            current_pos[2],
        )
        self.entity.set_position(new_pos)
        Engine.log("Player jumped!")

    def get_speed(self) -> float:
        """获取速度"""
        return self.speed

    def set_speed(self, speed: float) -> None:
        """设置速度"""
        self.speed = speed


# NPC类
class NPC:
    def __init__(self, name: str, behavior: str) -> None:
        self.entity = Engine.spawn_entity()
        self.entity.name = name
        self.behavior = behavior

        self.patrol_points = [
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 0.0),
            (10.0, 0.0, 10.0),
            (0.0, 0.0, 10.0),
        ]
        self.current_point_index = 0

        Engine.log(f"NPC '{name}' created with behavior: {behavior}")

    def update(self, delta_time: float) -> None:
        """更新方法"""
        if self.behavior == "patrol":
            self.patrol(delta_time)
        elif self.behavior == "idle":
            pass  # 什么都不做
        elif self.behavior == "follow":
            self.follow_player(delta_time)
        else:
            Engine.log(f"Unknown behavior: {self.behavior}")

    def patrol(self, delta_time: float) -> None:
        """巡逻行为"""
        target_point = self.patrol_points[self.current_point_index]
        current_pos = self.entity.get_position()

        # 计算方向
        direction = (
            target_point[0] - current_pos[0],
            target_point[1] - current_pos[1],
            target_point[2] - current_pos[2],
        )

        # 归一化并移动
        distance = math.sqrt(
            direction[0] ** 2 +
            direction[1] ** 2 +
            direction[2] ** 2
        )

        if distance > 0.1:
            speed = 2.0
            movement = (
                (direction[0] / distance) * speed * delta_time,
                (direction[1] / distance) * speed * delta_time,
                (direction[2] / distance) * speed * delta_time,
            )

            new_pos = (
                current_pos[0] + movement[0],
                current_pos[1] + movement[1],
                current_pos[2] + movement[2],
            )
            self.entity.set_position(new_pos)
        else:
            # 到达当前点，移动到下一个点
            self.current_point_index = (self.current_point_index + 1) % len(self.patrol_points)

    def follow_player(self, delta_time: float) -> None:
        """跟随玩家行为"""
        player = Engine.find_entity("Player")
        if player is None:
            return

        player_pos = player.get_position()
        current_pos = self.entity.get_position()

        # 计算到玩家的方向
        direction = (
            player_pos[0] - current_pos[0],
            player_pos[1] - current_pos[1],
            player_pos[2] - current_pos[2],
        )

        # 移动向玩家
        distance = math.sqrt(
            direction[0] ** 2 +
            direction[1] ** 2 +
            direction[2] ** 2
        )

        if distance > 2.0:
            speed = 3.0
            movement = (
                (direction[0] / distance) * speed * delta_time,
                (direction[1] / distance) * speed * delta_time,
                (direction[2] / distance) * speed * delta_time,
            )

            new_pos = (
                current_pos[0] + movement[0],
                current_pos[1] + movement[1],
                current_pos[2] + movement[2],
            )
            self.entity.set_position(new_pos)


# 游戏初始化函数
def init_game() -> None:
    """初始化游戏"""
    Engine.log("Initializing game...")

    # 创建玩家
    player = Player()

    # 创建NPC
    guard = NPC("Guard", "patrol")
    villager = NPC("Villager", "idle")

    Engine.log("Game initialized!")


# 游戏更新函数
def update_game(delta_time: float) -> None:
    """更新游戏"""
    # 更新所有实体
    # (引擎会自动调用每个实体的更新方法)
    pass


# 导出游戏接口
class Game:
    """游戏接口类"""

    @staticmethod
    def init() -> None:
        """初始化游戏"""
        init_game()

    @staticmethod
    def update(delta_time: float) -> None:
        """更新游戏"""
        update_game(delta_time)


# 模块导出
__all__ = ['Player', 'NPC', 'Game', 'init_game', 'update_game']
