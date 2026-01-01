-- 实体API Lua使用示例
--
-- 本文件演示如何使用脚本系统的实体API来创建、查询和管理游戏实体

-- ========================================
-- 示例1: 创建基础实体
-- ========================================

-- 创建一个空实体
local empty_entity = create_entity()
print("Created empty entity: " .. tostring(empty_entity))

-- 使用预设模板创建实体
local enemy = create_entity("Enemy")
print("Created enemy entity: " .. tostring(enemy))

local player = create_entity("Player")
print("Created player entity: " .. tostring(player))

-- ========================================
-- 示例2: 组件操作
-- ========================================

-- 为空实体添加Transform组件
add_component(empty_entity, "Transform", {
    position = {x = 10.0, y = 20.0, z = 0.0},
    rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
    scale = {x = 1.0, y = 1.0, z = 1.0}
})

-- 添加Sprite组件
add_component(empty_entity, "Sprite", {
    color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0},
    tex_index = 0,
    layer = 0.0
})

-- 添加Velocity组件
add_component(empty_entity, "Velocity", {
    linear = {x = 5.0, y = 0.0, z = 0.0},
    angular = {x = 0.0, y = 0.0, z = 0.0}
})

-- ========================================
-- 示例3: 批量添加组件
-- ========================================

-- 批量添加多个组件
local prop = create_entity()
add_components(prop, {
    Transform = {
        position = {x = 50.0, y = 30.0, z = 0.0},
        rotation = {x = 0.0, y = 0.0, z = 0.0, w = 1.0},
        scale = {x = 2.0, y = 2.0, z = 1.0}
    },
    Sprite = {
        color = {r = 1.0, g = 1.0, b = 0.0, a = 1.0},
        tex_index = 2,
        layer = 0.5
    }
})

-- ========================================
-- 示例4: 实体查询
-- ========================================

-- 查询所有包含Transform组件的实体
local entities_with_transform = find_entities_with_component("Transform")
print("Found " .. #entities_with_transform .. " entities with Transform component")

-- 查询所有包含Sprite组件的实体
local entities_with_sprite = find_entities_with_component("Sprite")
print("Found " .. #entities_with_sprite .. " entities with Sprite component")

-- ========================================
-- 示例5: 实体命名与查找
-- ========================================

-- 为实体命名
name_entity(enemy, "BossEnemy")
name_entity(player, "MainPlayer")

-- 按名称查找实体
local boss = find_entity_by_name("BossEnemy")
if boss then
    print("Found boss entity: " .. tostring(boss))
end

-- ========================================
-- 示例6: 组件数据获取与修改
-- ========================================

-- 获取Transform组件数据
local transform = get_component(empty_entity, "Transform")
if transform then
    print("Entity position:")
    print("  X: " .. transform.position.x)
    print("  Y: " .. transform.position.y)
    print("  Z: " .. transform.position.z)
end

-- 修改Transform组件数据
set_component_data(empty_entity, "Transform", {
    position = {x = 100.0, y = 200.0, z = 0.0}
})

-- 获取修改后的数据
local updated_transform = get_component(empty_entity, "Transform")
if updated_transform then
    print("Updated position:")
    print("  X: " .. updated_transform.position.x)
    print("  Y: " .. updated_transform.position.y)
end

-- ========================================
-- 示例7: 组件检查
-- ========================================

-- 检查实体是否有特定组件
if has_component(empty_entity, "Transform") then
    print("Entity has Transform component")
end

if has_component(empty_entity, "Sprite") then
    print("Entity has Sprite component")
end

if not has_component(empty_entity, "Velocity") then
    print("Entity does not have Velocity component")
end

-- ========================================
-- 示例8: 组件移除
-- ========================================

-- 移除组件
remove_component(empty_entity, "Velocity")
if not has_component(empty_entity, "Velocity") then
    print("Velocity component removed successfully")
end

-- ========================================
-- 示例9: 复杂查询 (简化版本)
-- ========================================

-- 查询所有带Transform的实体
local all_entities = find_entities_with_component("Transform")

-- 在Lua中手动过滤
local visible_entities = {}
for i, entity_id in ipairs(all_entities) do
    local transform = get_component(entity_id, "Transform")
    if transform and transform.position.x > 50.0 then
        table.insert(visible_entities, entity_id)
    end
end

print("Found " .. #visible_entities .. " visible entities (x > 50)")

-- ========================================
-- 示例10: 实体销毁
-- ========================================

-- 创建临时实体
local temp_entity = create_entity("Prop")
print("Created temporary entity: " .. tostring(temp_entity))

-- 销毁实体
destroy_entity(temp_entity)
print("Temporary entity destroyed")

-- ========================================
-- 示例11: 游戏场景构建
-- ========================================

-- 创建一个简单的游戏场景
local function build_game_scene()
    -- 创建玩家
    local player = create_entity("Player")
    name_entity(player, "Player")
    set_component_data(player, "Transform", {
        position = {x = 400.0, y = 300.0, z = 0.0}
    })

    -- 创建多个敌人
    for i = 1, 5 do
        local enemy = create_entity("Enemy")
        name_entity(enemy, "Enemy" .. i)
        set_component_data(enemy, "Transform", {
            position = {
                x = 100.0 * i,
                y = 100.0 * i,
                z = 0.0
            }
        })
        set_component_data(enemy, "Sprite", {
            color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0}
        })
    end

    -- 创建一些道具
    for i = 1, 3 do
        local prop = create_entity("Prop")
        name_entity(prop, "Prop" .. i)
        set_component_data(prop, "Transform", {
            position = {
                x = 200.0 * i,
                y = 150.0,
                z = 0.0
            }
        })
    end

    print("Game scene built successfully!")
end

-- 构建场景
build_game_scene()

-- ========================================
-- 示例12: 实体遍历与操作
-- ========================================

-- 遍历所有实体并打印位置
local function print_all_entities_position()
    local entities = find_entities_with_component("Transform")
    print("Total entities with Transform: " .. #entities)

    for i, entity_id in ipairs(entities) do
        local transform = get_component(entity_id, "Transform")
        if transform then
            print(string.format("Entity %d position: (%.1f, %.1f, %.1f)",
                i,
                transform.position.x,
                transform.position.y,
                transform.position.z
            ))
        end
    end
end

print_all_entities_position()

-- ========================================
-- 示例13: 动态实体管理
-- ========================================

-- 创建一个简单的生成器
local function spawn_enemy(x, y)
    local enemy = create_entity("Enemy")
    set_component_data(enemy, "Transform", {
        position = {x = x, y = y, z = 0.0}
    })
    set_component_data(enemy, "Velocity", {
        linear = {x = 2.0, y = 0.0, z = 0.0}
    })
    return enemy
end

-- 生成一波敌人
local wave_enemies = {}
for i = 1, 10 do
    local enemy = spawn_enemy(i * 50, 100)
    table.insert(wave_enemies, enemy)
end

print("Spawned enemy wave with " .. #wave_enemies .. " enemies")

-- ========================================
-- 示例14: 组件数据批量更新
-- ========================================

-- 批量移动所有敌人
local entities = find_entities_with_component("Transform")
for i, entity_id in ipairs(entities) do
    local transform = get_component(entity_id, "Transform")
    if transform then
        local new_x = transform.position.x + 10.0
        local new_y = transform.position.y + 5.0
        set_component_data(entity_id, "Transform", {
            position = {x = new_x, y = new_y, z = transform.position.z}
        })
    end
end

print("All entities moved!")

print("\n=== Entity API Examples Complete ===")
