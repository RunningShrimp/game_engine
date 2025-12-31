// TypeScript示例：玩家控制脚本
//
// 演示如何使用TypeScript编写游戏脚本

import { Engine, Entity } from '@game-engine/core';

// 玩家类
export class Player {
    private entity: Entity;
    private speed: number;
    private jumpForce: number;

    constructor() {
        // 创建玩家实体
        this.entity = Engine.spawnEntity();
        this.entity.name = "Player";

        // 设置初始位置
        this.entity.setPosition({ x: 0, y: 1, z: 0 });

        // 配置参数
        this.speed = 5.0;
        this.jumpForce = 10.0;

        Engine.log(`Player created with ID: ${this.entity.id}`);
    }

    // 更新方法（每帧调用）
    public update(deltaTime: number): void {
        // 获取输入
        const moveX = Engine.getInputAxis("Horizontal");
        const moveZ = Engine.getInputAxis("Vertical");

        // 计算新位置
        const currentPos = this.entity.getPosition();
        const newPos = {
            x: currentPos.x + moveX * this.speed * deltaTime,
            y: currentPos.y,
            z: currentPos.z + moveZ * this.speed * deltaTime,
        };

        // 应用位置
        this.entity.setPosition(newPos);

        // 跳跃检测
        if (Engine.getInputButton("Jump")) {
            this.jump();
        }
    }

    // 跳跃方法
    private jump(): void {
        const currentPos = this.entity.getPosition();
        currentPos.y += this.jumpForce;
        this.entity.setPosition(currentPos);
        Engine.log("Player jumped!");
    }

    // 获取速度
    public getSpeed(): number {
        return this.speed;
    }

    // 设置速度
    public setSpeed(speed: number): void {
        this.speed = speed;
    }
}

// NPC类
export class NPC {
    private entity: Entity;
    private behavior: string;
    private patrolPoints: Vector3[];
    private currentPointIndex: number;

    constructor(name: string, behavior: string) {
        this.entity = Engine.spawnEntity();
        this.entity.name = name;
        this.behavior = behavior;

        this.patrolPoints = [
            { x: 0, y: 0, z: 0 },
            { x: 10, y: 0, z: 0 },
            { x: 10, y: 0, z: 10 },
            { x: 0, y: 0, z: 10 },
        ];
        this.currentPointIndex = 0;

        Engine.log(`NPC '${name}' created with behavior: ${behavior}`);
    }

    public update(deltaTime: number): void {
        switch (this.behavior) {
            case "patrol":
                this.patrol(deltaTime);
                break;
            case "idle":
                // 什么都不做
                break;
            case "follow":
                this.followPlayer(deltaTime);
                break;
            default:
                Engine.log(`Unknown behavior: ${this.behavior}`);
        }
    }

    private patrol(deltaTime: number): void {
        const targetPoint = this.patrolPoints[this.currentPointIndex];
        const currentPos = this.entity.getPosition();

        // 计算方向
        const direction = {
            x: targetPoint.x - currentPos.x,
            y: targetPoint.y - currentPos.y,
            z: targetPoint.z - currentPos.z,
        };

        // 归一化并移动
        const distance = Math.sqrt(
            direction.x * direction.x +
            direction.y * direction.y +
            direction.z * direction.z
        );

        if (distance > 0.1) {
            const speed = 2.0;
            const movement = {
                x: (direction.x / distance) * speed * deltaTime,
                y: (direction.y / distance) * speed * deltaTime,
                z: (direction.z / distance) * speed * deltaTime,
            };

            this.entity.setPosition({
                x: currentPos.x + movement.x,
                y: currentPos.y + movement.y,
                z: currentPos.z + movement.z,
            });
        } else {
            // 到达当前点，移动到下一个点
            this.currentPointIndex = (this.currentPointIndex + 1) % this.patrolPoints.length;
        }
    }

    private followPlayer(deltaTime: number): void {
        const player = Engine.findEntity("Player");
        if (!player) {
            return;
        }

        const playerPos = player.getPosition();
        const currentPos = this.entity.getPosition();

        // 计算到玩家的方向
        const direction = {
            x: playerPos.x - currentPos.x,
            y: playerPos.y - currentPos.y,
            z: playerPos.z - currentPos.z,
        };

        // 移动向玩家
        const distance = Math.sqrt(
            direction.x * direction.x +
            direction.y * direction.y +
            direction.z * direction.z
        );

        if (distance > 2.0) {
            const speed = 3.0;
            const movement = {
                x: (direction.x / distance) * speed * deltaTime,
                y: (direction.y / distance) * speed * deltaTime,
                z: (direction.z / distance) * speed * deltaTime,
            };

            this.entity.setPosition({
                x: currentPos.x + movement.x,
                y: currentPos.y + movement.y,
                z: currentPos.z + movement.z,
            });
        }
    }
}

// 游戏初始化函数
export function initGame(): void {
    Engine.log("Initializing game...");

    // 创建玩家
    const player = new Player();

    // 创建NPC
    const guard = new NPC("Guard", "patrol");
    const villager = new NPC("Villager", "idle");

    Engine.log("Game initialized!");
}

// 游戏更新函数
export function updateGame(deltaTime: number): void {
    // 更新所有实体
    // (引擎会自动调用每个实体的更新方法)
}

// 导出游戏接口
export const Game = {
    init: initGame,
    update: updateGame,
};
