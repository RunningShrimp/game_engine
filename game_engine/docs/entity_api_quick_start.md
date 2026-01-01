# 实体API快速使用指南

## 概述

实体API允许脚本创建和管理游戏实体，支持组件操作、查询和批量处理。

## 基础用法

### 1. 创建实体

```lua
-- 创建空实体
local entity = create_entity()

-- 使用模板创建
local enemy = create_entity("Enemy")  -- 内置模板: Enemy, Player, Prop
```

### 2. 添加组件

```lua
-- 添加单个组件
add_component(entity, "Transform", {
    position = {x = 100, y = 200, z = 0},
    rotation = {x = 0, y = 0, z = 0, w = 1},
    scale = {x = 1, y = 1, z = 1}
})

-- 批量添加组件
add_components(entity, {
    Transform = {
        position = {x = 0, y = 0, z = 0}
    },
    Sprite = {
        color = {r = 1, g = 0, b = 0, a = 1},
        tex_index = 0,
        layer = 0.0
    }
})
```

### 3. 查询实体

```lua
-- 按组件查询
local enemies = find_entities_with_component("Transform")

-- 按名称查询
local player = find_entity_by_name("Player")

-- 命名实体
name_entity(entity, "MyEnemy")
```

### 4. 操作组件

```lua
-- 检查组件
if has_component(entity, "Transform") then
    -- 处理
end

-- 获取组件数据
local transform = get_component(entity, "Transform")
print(transform.position.x, transform.position.y)

-- 修改组件数据
set_component_data(entity, "Transform", {
    position = {x = 100, y = 200, z = 0}
})

-- 移除组件
remove_component(entity, "Velocity")
```

### 5. 销毁实体

```lua
destroy_entity(entity)
```

## 支持的组件类型

### Transform组件

```lua
{
    position = {x = 0, y = 0, z = 0},  -- 位置
    rotation = {x = 0, y = 0, z = 0, w = 1},  -- 四元数旋转
    scale = {x = 1, y = 1, z = 1}  -- 缩放
}
```

### Sprite组件

```lua
{
    color = {r = 1, g = 1, b = 1, a = 1},  -- RGBA颜色
    tex_index = 0,  -- 纹理索引
    layer = 0.0  -- 渲染层级
}
```

### Velocity组件

```lua
{
    linear = {x = 0, y = 0, z = 0},  -- 线性速度
    angular = {x = 0, y = 0, z = 0}  -- 角速度
}
```

## 完整示例

### 示例1: 创建移动的敌人

```lua
-- 创建敌人
local enemy = create_entity("Enemy")

-- 设置初始位置
set_component_data(enemy, "Transform", {
    position = {x = 400, y = 100, z = 0}
})

-- 设置移动速度
set_component_data(enemy, "Velocity", {
    linear = {x = 2, y = 0, z = 0}
})
```

### 示例2: 批量创建道具

```lua
-- 创建多个道具
for i = 1, 10 do
    local prop = create_entity("Prop")

    add_components(prop, {
        Transform = {
            position = {x = i * 50, y = 100, z = 0}
        },
        Sprite = {
            color = {r = 1, g = 1, b = 0, a = 1},
            tex_index = 2,
            layer = 0.5
        }
    })
end
```

### 示例3: 查询和操作

```lua
-- 查询所有有Transform的实体
local entities = find_entities_with_component("Transform")

-- 遍历并移动
for i, entity_id in ipairs(entities) do
    local transform = get_component(entity_id, "Transform")

    -- 向右移动
    set_component_data(entity_id, "Transform", {
        position = {
            x = transform.position.x + 10,
            y = transform.position.y,
            z = transform.position.z
        }
    })
end
```

## 常见模式

### 实体工厂

```lua
local function create_player(x, y)
    local player = create_entity("Player")
    name_entity(player, "Player")

    add_components(player, {
        Transform = {position = {x = x, y = y, z = 0}},
        Sprite = {color = {r = 0, g = 1, b = 0, a = 1}}
    })

    return player
end
```

### 安全组件操作

```lua
local function safe_get_position(entity)
    local transform = get_component(entity, "Transform")
    if transform then
        return transform.position.x, transform.position.y
    end
    return nil, nil
end
```

### 批量操作

```lua
local function move_all_entities(dx, dy)
    local entities = find_entities_with_component("Transform")

    for _, entity_id in ipairs(entities) do
        local transform = get_component(entity_id, "Transform")
        if transform then
            set_component_data(entity_id, "Transform", {
                position = {
                    x = transform.position.x + dx,
                    y = transform.position.y + dy,
                    z = transform.position.z
                }
            })
        end
    end
end
```

## 注意事项

1. **线程安全**: 所有API调用都是线程安全的
2. **错误处理**: 建议使用pcall包装API调用
3. **性能**: 大量实体操作时考虑使用批量API
4. **生命周期**: 实体销毁后ID会被回收，不要保存已销毁实体的引用

## 进阶用法

查看完整示例:
- `examples/entity_api_lua.lua` - 基础用法
- `examples/entity_api_advanced.lua` - 高级特性
- `examples/entity_api_game.lua` - 完整游戏示例

## API参考

### 实体管理
- `create_entity([template])` - 创建实体
- `destroy_entity(entity)` - 销毁实体
- `name_entity(entity, name)` - 命名实体
- `find_entity_by_name(name)` - 查找实体

### 组件操作
- `add_component(entity, type, data)` - 添加组件
- `add_components(entity, components)` - 批量添加
- `remove_component(entity, type)` - 移除组件
- `has_component(entity, type)` - 检查组件
- `get_component(entity, type)` - 获取组件
- `set_component_data(entity, type, data)` - 设置数据

### 查询
- `find_entities_with_component(type)` - 按组件查询
- `query_entities()` - 创建查询构建器

## 获取帮助

- 查看Rust文档: `cargo doc --open`
- 查看示例代码: `examples/`目录
- 查看测试: `src/scripting/entity_api.rs`测试部分
