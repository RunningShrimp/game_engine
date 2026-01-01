-- 实体API高级用法示例
--
-- 本文件演示实体API的高级功能,包括:
-- - 实体模板系统
-- - 组件验证
-- - 性能优化
-- - 错误处理
-- - 实体生命周期管理

-- ========================================
-- 高级示例1: 自定义实体模板
-- ========================================

-- 创建自定义模板 (通过Rust API)
-- 注意: 这里演示如何在Lua中使用预定义的模板

-- 使用内置模板创建实体
local fast_enemy = create_entity("Enemy")
set_component_data(fast_enemy, "Transform", {
    position = {x = 0, y = 0, z = 0}
})
set_component_data(fast_enemy, "Velocity", {
    linear = {x = 10.0, y = 0.0, z = 0.0}
})

-- ========================================
-- 高级示例2: 实体工厂模式
-- ========================================

-- 创建一个实体工厂
local EntityFactory = {}

function EntityFactory.create_player(x, y)
    local player = create_entity("Player")
    name_entity(player, "Player")

    -- 初始化玩家组件
    add_components(player, {
        Transform = {
            position = {x = x, y = y, z = 0.0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1.0, y = 1.0, z = 1.0}
        },
        Sprite = {
            color = {r = 0.0, g = 1.0, b = 0.0, a = 1.0},
            tex_index = 1,
            layer = 1.0
        },
        Velocity = {
            linear = {x = 0.0, y = 0.0, z = 0.0},
            angular = {x = 0.0, y = 0.0, z = 0.0}
        }
    })

    return player
end

function EntityFactory.create_enemy(x, y, speed)
    local enemy = create_entity("Enemy")

    add_components(enemy, {
        Transform = {
            position = {x = x, y = y, z = 0.0}
        },
        Sprite = {
            color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0}
        },
        Velocity = {
            linear = {x = speed, y = 0.0, z = 0.0}
        }
    })

    return enemy
end

function EntityFactory.create_pickup(x, y, pickup_type)
    local pickup = create_entity("Prop")

    local color = {r = 1.0, g = 1.0, b = 0.0, a = 1.0}
    if pickup_type == "health" then
        color = {r = 0.0, g = 1.0, b = 0.0, a = 1.0}
    elseif pickup_type == "ammo" then
        color = {r = 1.0, g = 0.5, b = 0.0, a = 1.0}
    end

    add_components(pickup, {
        Transform = {
            position = {x = x, y = y, z = 0.0},
            scale = {x = 0.5, y = 0.5, z = 1.0}
        },
        Sprite = {
            color = color,
            tex_index = 3,
            layer = 0.5
        }
    })

    return pickup
end

-- 使用工厂创建实体
local player = EntityFactory.create_player(400, 300)
local enemy1 = EntityFactory.create_enemy(100, 100, 5.0)
local enemy2 = EntityFactory.create_enemy(200, 150, 7.0)
local health_pack = EntityFactory.create_pickup(300, 200, "health")

print("Created entities using factory pattern")

-- ========================================
-- 高级示例3: 组件验证与错误处理
-- ========================================

-- 安全的组件获取函数
local function safe_get_component(entity_id, component_name)
    local success, result = pcall(get_component, entity_id, component_name)

    if success then
        return result
    else
        print("Error getting component: " .. tostring(result))
        return nil
    end
end

-- 安全的组件设置函数
local function safe_set_component_data(entity_id, component_name, data)
    local success, result = pcall(set_component_data, entity_id, component_name, data)

    if not success then
        print("Error setting component data: " .. tostring(result))
        return false
    end

    return true
end

-- 使用安全的组件操作
local transform = safe_get_component(player, "Transform")
if transform then
    print("Player position retrieved safely")
end

safe_set_component_data(player, "Transform", {
    position = {x = 500, y = 400, z = 0}
})

-- ========================================
-- 高级示例4: 实体池管理
-- ========================================

-- 创建对象池来重用实体
local EntityPool = {
    active = {},
    inactive = {},
    max_size = 100
}

function EntityPool:acquire(template_name)
    -- 从池中获取一个实体
    local entity

    if #self.inactive > 0 then
        entity = table.remove(self.inactive)
    else
        if #self.active < self.max_size then
            entity = create_entity(template_name)
        else
            print("Warning: Entity pool at maximum capacity")
            return nil
        end
    end

    table.insert(self.active, entity)
    return entity
end

function EntityPool:release(entity)
    -- 从活动列表移除
    for i, e in ipairs(self.active) do
        if e == entity then
            table.remove(self.active, i)
            break
        end
    end

    -- 重置实体状态并放回非活动列表
    -- (实际应用中可能需要重置组件数据)
    table.insert(self.inactive, entity)
end

function EntityPool:get_active_count()
    return #self.active
end

-- 使用对象池
local pool = EntityPool

-- 从池中获取敌人
for i = 1, 10 do
    local enemy = pool:acquire("Enemy")
    if enemy then
        set_component_data(enemy, "Transform", {
            position = {x = i * 50, y = 100, z = 0}
        })
    end
end

print("Active entities in pool: " .. pool:get_active_count())

-- ========================================
-- 高级示例5: 实体查询优化
-- ========================================

-- 缓存查询结果以提高性能
local QueryCache = {
    cache = {},
    max_age = 1000  -- 毫秒
}

function QueryCache:query(component_name)
    local current_time = Engine and Engine.time() or 0

    -- 检查缓存
    if self.cache[component_name] then
        local cached = self.cache[component_name]
        if current_time - cached.timestamp < self.max_age then
            return cached.results
        end
    end

    -- 执行查询
    local results = find_entities_with_component(component_name)

    -- 更新缓存
    self.cache[component_name] = {
        results = results,
        timestamp = current_time
    }

    return results
end

-- 使用缓存查询
local transform_entities = QueryCache:query("Transform")
print("Cached query found " .. #transform_entities .. " entities")

-- ========================================
-- 高级示例6: 实体关系管理
-- ========================================

-- 创建实体关系图
local EntityGraph = {
    children = {},
    parents = {}
}

function EntityGraph:set_parent(child, parent)
    if not self.children[parent] then
        self.children[parent] = {}
    end

    table.insert(self.children[parent], child)
    self.parents[child] = parent
end

function EntityGraph:get_children(parent)
    return self.children[parent] or {}
end

function EntityGraph:get_parent(child)
    return self.parents[child]
end

function EntityGraph:remove(entity)
    -- 移除作为父级的关系
    if self.children[entity] then
        for _, child in ipairs(self.children[entity]) do
            self.parents[child] = nil
        end
        self.children[entity] = nil
    end

    -- 移除作为子级的关系
    if self.parents[entity] then
        local parent = self.parents[entity]
        for i, child in ipairs(self.children[parent]) do
            if child == entity then
                table.remove(self.children[parent], i)
                break
            end
        end
        self.parents[entity] = nil
    end
end

-- 使用实体关系图
local parent_entity = create_entity("Prop")
local child1 = create_entity("Prop")
local child2 = create_entity("Prop")

EntityGraph:set_parent(child1, parent_entity)
EntityGraph:set_parent(child2, parent_entity)

local children = EntityGraph:get_children(parent_entity)
print("Parent has " .. #children .. " children")

-- ========================================
-- 高级示例7: 实体生命周期事件
-- ========================================

-- 创建实体事件系统
local EntityEvents = {
    on_create = {},
    on_destroy = {},
    on_update = {}
}

function EntityEvents:register_create(callback)
    table.insert(self.on_create, callback)
end

function EntityEvents:register_destroy(callback)
    table.insert(self.on_destroy, callback)
end

function EntityEvents:trigger_create(entity)
    for _, callback in ipairs(self.on_create) do
        callback(entity)
    end
end

function EntityEvents:trigger_destroy(entity)
    for _, callback in ipairs(self.on_destroy) do
        callback(entity)
    end
end

-- 注册事件
EntityEvents:register_create(function(entity)
    print("Entity created: " .. tostring(entity))
end)

EntityEvents:register_destroy(function(entity)
    print("Entity destroyed: " .. tostring(entity))
end)

-- 触发事件
local test_entity = create_entity("Enemy")
EntityEvents:trigger_create(test_entity)

destroy_entity(test_entity)
EntityEvents:trigger_destroy(test_entity)

-- ========================================
-- 高级示例8: 实体数据序列化
-- ========================================

-- 序列化实体数据
local function serialize_entity(entity_id)
    local data = {
        id = entity_id,
        components = {}
    }

    -- 收集所有组件数据
    local component_names = {"Transform", "Sprite", "Velocity"}

    for _, component_name in ipairs(component_names) do
        local component = safe_get_component(entity_id, component_name)
        if component then
            data.components[component_name] = component
        end
    end

    return data
end

-- 打印实体数据
local function print_entity_data(entity_id)
    local data = serialize_entity(entity_id)

    print("Entity ID: " .. tostring(data.id))
    print("Components:")

    for name, component in pairs(data.components) do
        print("  " .. name)
        -- 简化打印,实际应用中需要递归打印嵌套结构
    end
end

-- 序列化玩家实体
print_entity_data(player)

-- ========================================
-- 高级示例9: 批量实体操作
-- ========================================

-- 批量操作函数
local function batch_operation(entities, operation)
    local results = {}

    for i, entity_id in ipairs(entities) do
        local success, result = pcall(operation, entity_id, i)
        if success then
            table.insert(results, {entity = entity_id, result = result})
        else
            print("Batch operation failed for entity " .. tostring(entity_id) .. ": " .. tostring(result))
            table.insert(results, {entity = entity_id, error = result})
        end
    end

    return results
end

-- 批量移动实体
local entities = find_entities_with_component("Transform")
local move_results = batch_operation(entities, function(entity_id, index)
    local transform = get_component(entity_id, "Transform")
    if transform then
        local new_x = transform.position.x + (index * 10)
        set_component_data(entity_id, "Transform", {
            position = {
                x = new_x,
                y = transform.position.y,
                z = transform.position.z
            }
        })
        return true
    end
    return false
end)

print("Batch operation completed: " .. #move_results .. " entities processed")

-- ========================================
-- 高级示例10: 实体查找器模式
-- ========================================

-- 创建实体查找器
local EntityFinder = {}

function EntityFinder.find_by_name_pattern(pattern)
    local results = {}
    -- 注意: 这需要在有实体命名系统的情况下工作
    -- 简化示例
    return results
end

function EntityFinder.find_in_radius(center_x, center_y, radius)
    local entities = find_entities_with_component("Transform")
    local results = {}

    for _, entity_id in ipairs(entities) do
        local transform = get_component(entity_id, "Transform")
        if transform then
            local dx = transform.position.x - center_x
            local dy = transform.position.y - center_y
            local distance = math.sqrt(dx * dx + dy * dy)

            if distance <= radius then
                table.insert(results, {
                    entity = entity_id,
                    distance = distance
                })
            end
        end
    end

    -- 按距离排序
    table.sort(results, function(a, b)
        return a.distance < b.distance
    end)

    return results
end

-- 查找附近的实体
local nearby = EntityFinder.find_in_radius(400, 300, 200)
print("Found " .. #nearby .. " entities within radius")

print("\n=== Advanced Entity API Examples Complete ===")
