-- Lua示例：玩家控制脚本
--
-- 演示如何使用Lua编写游戏脚本

-- 玩家类
local Player = {}
Player.__index = Player

function Player.new()
    local self = setmetatable({}, Player)

    -- 创建玩家实体
    self.entity = Engine.spawnEntity()
    self.entity.name = "Player"

    -- 设置初始位置
    self.entity:setPosition({x = 0, y = 1, z = 0})

    -- 配置参数
    self.speed = 5.0
    self.jumpForce = 10.0

    Engine.log("Player created with ID: " .. self.entity.id)

    return self
end

function Player:update(deltaTime)
    -- 获取输入
    local moveX = Engine.getInputAxis("Horizontal")
    local moveZ = Engine.getInputAxis("Vertical")

    -- 计算新位置
    local currentPos = self.entity:getPosition()
    local newPos = {
        x = currentPos.x + moveX * self.speed * deltaTime,
        y = currentPos.y,
        z = currentPos.z + moveZ * self.speed * deltaTime
    }

    -- 应用位置
    self.entity:setPosition(newPos)

    -- 跳跃检测
    if Engine.getInputButton("Jump") then
        self:jump()
    end
end

function Player:jump()
    local currentPos = self.entity:getPosition()
    currentPos.y = currentPos.y + self.jumpForce
    self.entity:setPosition(currentPos)
    Engine.log("Player jumped!")
end

function Player:getSpeed()
    return self.speed
end

function Player:setSpeed(speed)
    self.speed = speed
end

-- NPC类
local NPC = {}
NPC.__index = NPC

function NPC.new(name, behavior)
    local self = setmetatable({}, NPC)

    self.entity = Engine.spawnEntity()
    self.entity.name = name
    self.behavior = behavior

    self.patrolPoints = {
        {x = 0, y = 0, z = 0},
        {x = 10, y = 0, z = 0},
        {x = 10, y = 0, z = 10},
        {x = 0, y = 0, z = 10}
    }
    self.currentPointIndex = 1

    Engine.log("NPC '" .. name .. "' created with behavior: " .. behavior)

    return self
end

function NPC:update(deltaTime)
    if self.behavior == "patrol" then
        self:patrol(deltaTime)
    elseif self.behavior == "idle" then
        -- 什么都不做
    elseif self.behavior == "follow" then
        self:followPlayer(deltaTime)
    else
        Engine.log("Unknown behavior: " .. self.behavior)
    end
end

function NPC:patrol(deltaTime)
    local targetPoint = self.patrolPoints[self.currentPointIndex]
    local currentPos = self.entity:getPosition()

    -- 计算方向
    local direction = {
        x = targetPoint.x - currentPos.x,
        y = targetPoint.y - currentPos.y,
        z = targetPoint.z - currentPos.z
    }

    -- 归一化并移动
    local distance = math.sqrt(
        direction.x * direction.x +
        direction.y * direction.y +
        direction.z * direction.z
    )

    if distance > 0.1 then
        local speed = 2.0
        local movement = {
            x = (direction.x / distance) * speed * deltaTime,
            y = (direction.y / distance) * speed * deltaTime,
            z = (direction.z / distance) * speed * deltaTime
        }

        self.entity:setPosition({
            x = currentPos.x + movement.x,
            y = currentPos.y + movement.y,
            z = currentPos.z + movement.z
        })
    else
        -- 到达当前点，移动到下一个点
        self.currentPointIndex = self.currentPointIndex % #self.patrolPoints + 1
    end
end

function NPC:followPlayer(deltaTime)
    local player = Engine.findEntity("Player")
    if not player then
        return
    end

    local playerPos = player:getPosition()
    local currentPos = self.entity:getPosition()

    -- 计算到玩家的方向
    local direction = {
        x = playerPos.x - currentPos.x,
        y = playerPos.y - currentPos.y,
        z = playerPos.z - currentPos.z
    }

    -- 移动向玩家
    local distance = math.sqrt(
        direction.x * direction.x +
        direction.y * direction.y +
        direction.z * direction.z
    )

    if distance > 2.0 then
        local speed = 3.0
        local movement = {
            x = (direction.x / distance) * speed * deltaTime,
            y = (direction.y / distance) * speed * deltaTime,
            z = (direction.z / distance) * speed * deltaTime
        }

        self.entity:setPosition({
            x = currentPos.x + movement.x,
            y = currentPos.y + movement.y,
            z = currentPos.z + movement.z
        })
    end
end

-- 游戏初始化函数
function initGame()
    Engine.log("Initializing game...")

    -- 创建玩家
    local player = Player.new()

    -- 创建NPC
    local guard = NPC.new("Guard", "patrol")
    local villager = NPC.new("Villager", "idle")

    Engine.log("Game initialized!")

    -- 返回游戏对象
    return {
        player = player,
        guard = guard,
        villager = villager
    }
end

-- 游戏更新函数
function updateGame(game, deltaTime)
    -- 更新所有实体
    if game.player then
        game.player:update(deltaTime)
    end

    if game.guard then
        game.guard:update(deltaTime)
    end

    if game.villager then
        game.villager:update(deltaTime)
    end
end

-- 导出模块
return {
    Player = Player,
    NPC = NPC,
    initGame = initGame,
    updateGame = updateGame
}
