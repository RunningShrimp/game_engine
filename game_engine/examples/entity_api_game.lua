-- 实体API游戏场景示例
--
-- 本文件展示如何在实际游戏场景中使用实体API,
-- 包括完整的游戏循环、实体管理和交互系统

-- ========================================
-- 游戏场景: 太空射击游戏
-- ========================================

-- 游戏状态管理
local GameState = {
    player = nil,
    enemies = {},
    bullets = {},
    particles = {},
    score = 0,
    wave = 1,
    game_over = false
}

-- ========================================
-- 玩家系统
-- ========================================

local function create_player()
    local player = create_entity("Player")
    name_entity(player, "Player")

    add_components(player, {
        Transform = {
            position = {x = 400, y = 500, z = 0},
            rotation = {x = 0, y = 0, z = 0, w = 1},
            scale = {x = 1.5, y = 1.5, z = 1.0}
        },
        Sprite = {
            color = {r = 0.0, g = 0.8, b = 1.0, a = 1.0},
            tex_index = 10,
            layer = 10.0
        },
        Velocity = {
            linear = {x = 0.0, y = 0.0, z = 0.0},
            angular = {x = 0.0, y = 0.0, z = 0.0}
        }
    })

    -- 添加玩家元数据 (可以通过组件或其他方式存储)
    player_hp = 100
    player_speed = 300.0
    fire_rate = 0.2
    last_fire_time = 0

    GameState.player = player
    print("Player created at center")

    return player
end

local function update_player(delta_time)
    if not GameState.player or GameState.game_over then
        return
    end

    -- 获取当前位置
    local transform = get_component(GameState.player, "Transform")
    if not transform then return end

    local velocity = get_component(GameState.player, "Velocity")
    if not velocity then return end

    -- 简单的移动逻辑 (实际应用中会从输入系统获取)
    local dx = 0
    local dy = 0

    -- 模拟自动移动
    local time = Engine and Engine.time() or 0
    dx = math.sin(time * 2) * player_speed * delta_time
    dy = math.cos(time * 3) * player_speed * delta_time * 0.5

    -- 更新位置
    local new_x = transform.position.x + dx
    local new_y = transform.position.y + dy

    -- 边界限制
    new_x = math.max(50, math.min(750, new_x))
    new_y = math.max(50, math.min(550, new_y))

    -- 应用新位置
    set_component_data(GameState.player, "Transform", {
        position = {x = new_x, y = new_y, z = 0}
    })
end

local function player_fire()
    if not GameState.player or GameState.game_over then
        return
    end

    local time = Engine and Engine.time() or 0
    if time - last_fire_time < fire_rate then
        return
    end

    last_fire_time = time

    -- 获取玩家位置
    local transform = get_component(GameState.player, "Transform")
    if not transform then return end

    -- 创建子弹
    local bullet = create_entity("Prop")
    name_entity(bullet, "Bullet" .. #GameState.bullets)

    add_components(bullet, {
        Transform = {
            position = {
                x = transform.position.x,
                y = transform.position.y - 20,
                z = 0
            },
            scale = {x = 0.3, y = 0.8, z = 1.0}
        },
        Sprite = {
            color = {r = 1.0, g = 1.0, b = 0.0, a = 1.0},
            tex_index = 20,
            layer = 9.0
        },
        Velocity = {
            linear = {x = 0.0, y = -800.0, z = 0.0}
        }
    })

    table.insert(GameState.bullets, {
        entity = bullet,
        lifetime = 2.0
    })

    -- 创建发射粒子效果
    create_muzzle_flash(transform.position.x, transform.position.y)
end

-- ========================================
-- 敌人系统
-- ========================================

local function spawn_enemy(x, y, enemy_type)
    local enemy = create_entity("Enemy")
    local enemy_name = "Enemy" .. #GameState.enemies
    name_entity(enemy, enemy_name)

    local color = {r = 1.0, g = 0.0, b = 0.0, a = 1.0}
    local scale = 1.0
    local health = 1
    local speed = 100.0

    if enemy_type == "fast" then
        color = {r = 1.0, g = 0.5, b = 0.0, a = 1.0}
        scale = 0.8
        speed = 200.0
    elseif enemy_type == "tank" then
        color = {r = 0.5, g = 0.0, b = 0.5, a = 1.0}
        scale = 1.5
        health = 3
        speed = 50.0
    end

    add_components(enemy, {
        Transform = {
            position = {x = x, y = y, z = 0},
            scale = {x = scale, y = scale, z = 1.0}
        },
        Sprite = {
            color = color,
            tex_index = 11,
            layer = 8.0
        },
        Velocity = {
            linear = {x = 0.0, y = speed, z = 0.0}
        }
    })

    table.insert(GameState.enemies, {
        entity = enemy,
        health = health,
        max_health = health,
        enemy_type = enemy_type,
        score_value = enemy_type == "tank" and 300 or (enemy_type == "fast" and 200 or 100)
    })

    return enemy
end

local function spawn_wave(wave_number)
    local enemy_count = 5 + wave_number * 2

    for i = 1, enemy_count do
        local x = 100 + (i % 8) * 80
        local y = -50 - math.floor(i / 8) * 60

        local enemy_type = "normal"
        if wave_number >= 3 and i % 3 == 0 then
            enemy_type = "fast"
        elseif wave_number >= 5 and i % 5 == 0 then
            enemy_type = "tank"
        end

        spawn_enemy(x, y, enemy_type)
    end

    print("Wave " .. wave_number .. ": Spawned " .. enemy_count .. " enemies")
end

local function update_enemies(delta_time)
    for i = #GameState.enemies, 1, -1 do
        local enemy_data = GameState.enemies[i]
        local transform = get_component(enemy_data.entity, "Transform")

        if not transform then
            table.remove(GameState.enemies, i)
            goto continue
        end

        -- 向下移动
        local new_y = transform.position.y + 50 * delta_time
        set_component_data(enemy_data.entity, "Transform", {
            position = {
                x = transform.position.x,
                y = new_y,
                z = 0
            }
        })

        -- 检查是否超出屏幕
        if new_y > 650 then
            destroy_entity(enemy_data.entity)
            table.remove(GameState.enemies, i)
            player_hp = player_hp - 10
            print("Enemy reached bottom! Player HP: " .. player_hp)
        end

        ::continue::
    end
end

-- ========================================
-- 子弹系统
-- ========================================

local function update_bullets(delta_time)
    for i = #GameState.bullets, 1, -1 do
        local bullet_data = GameState.bullets[i]
        local transform = get_component(bullet_data.entity, "Transform")

        if not transform then
            table.remove(GameState.bullets, i)
            goto continue
        end

        -- 更新子弹位置
        local velocity = get_component(bullet_data.entity, "Velocity")
        if velocity then
            local new_y = transform.position.y + velocity.linear.y * delta_time
            set_component_data(bullet_data.entity, "Transform", {
                position = {
                    x = transform.position.x,
                    y = new_y,
                    z = 0
                }
            })
        end

        -- 更新生命周期
        bullet_data.lifetime = bullet_data.lifetime - delta_time

        -- 移除超出屏幕或过期的子弹
        if transform.position.y < -50 or bullet_data.lifetime <= 0 then
            destroy_entity(bullet_data.entity)
            table.remove(GameState.bullets, i)
        end

        ::continue::
    end
end

-- ========================================
-- 碰撞检测系统
-- ========================================

local function check_collisions()
    -- 子弹与敌人的碰撞
    for i = #GameState.bullets, 1, -1 do
        local bullet_data = GameState.bullets[i]
        local bullet_transform = get_component(bullet_data.entity, "Transform")

        if not bullet_transform then goto continue end

        for j = #GameState.enemies, 1, -1 do
            local enemy_data = GameState.enemies[j]
            local enemy_transform = get_component(enemy_data.entity, "Transform")

            if not enemy_transform then goto continue2 end

            -- 简单的距离检测
            local dx = bullet_transform.position.x - enemy_transform.position.x
            local dy = bullet_transform.position.y - enemy_transform.position.y
            local distance = math.sqrt(dx * dx + dy * dy)

            if distance < 30 then
                -- 命中!
                enemy_data.health = enemy_data.health - 1

                -- 创建击中效果
                create_hit_effect(enemy_transform.position.x, enemy_transform.position.y)

                -- 移除子弹
                destroy_entity(bullet_data.entity)
                table.remove(GameState.bullets, i)

                -- 检查敌人是否死亡
                if enemy_data.health <= 0 then
                    GameState.score = GameState.score + enemy_data.score_value
                    create_explosion(enemy_transform.position.x, enemy_transform.position.y)
                    destroy_entity(enemy_data.entity)
                    table.remove(GameState.enemies, j)
                    print("Enemy destroyed! Score: " .. GameState.score)
                end

                goto continue
            end

            ::continue2::
        end

        ::continue::
    end

    -- 敌人与玩家的碰撞
    if GameState.player then
        local player_transform = get_component(GameState.player, "Transform")
        if player_transform then
            for i = #GameState.enemies, 1, -1 do
                local enemy_data = GameState.enemies[i]
                local enemy_transform = get_component(enemy_data.entity, "Transform")

                if enemy_transform then
                    local dx = player_transform.position.x - enemy_transform.position.x
                    local dy = player_transform.position.y - enemy_transform.position.y
                    local distance = math.sqrt(dx * dx + dy * dy)

                    if distance < 40 then
                        -- 玩家被撞击
                        player_hp = player_hp - 20
                        create_explosion(enemy_transform.position.x, enemy_transform.position.y)
                        destroy_entity(enemy_data.entity)
                        table.remove(GameState.enemies, i)
                        print("Player hit! HP: " .. player_hp)

                        if player_hp <= 0 then
                            game_over()
                        end
                    end
                end
            end
        end
    end
end

-- ========================================
-- 特效系统
-- ========================================

local function create_muzzle_flash(x, y)
    local flash = create_entity("Prop")
    add_components(flash, {
        Transform = {
            position = {x = x, y = y, z = 0},
            scale = {x = 0.5, y = 0.5, z = 1.0}
        },
        Sprite = {
            color = {r = 1.0, g = 1.0, b = 0.5, a = 1.0},
            tex_index = 30,
            layer = 11.0
        }
    })

    table.insert(GameState.particles, {
        entity = flash,
        lifetime = 0.1,
        type = "muzzle_flash"
    })
end

local function create_hit_effect(x, y)
    local hit = create_entity("Prop")
    add_components(hit, {
        Transform = {
            position = {x = x, y = y, z = 0},
            scale = {x = 0.3, y = 0.3, z = 1.0}
        },
        Sprite = {
            color = {r = 1.0, g = 0.5, b = 0.0, a = 1.0},
            tex_index = 31,
            layer = 12.0
        }
    })

    table.insert(GameState.particles, {
        entity = hit,
        lifetime = 0.2,
        type = "hit"
    })
end

local function create_explosion(x, y)
    -- 创建多个粒子
    for i = 1, 8 do
        local particle = create_entity("Prop")
        local angle = (i / 8) * math.pi * 2
        local speed = 150 + math.random() * 100

        add_components(particle, {
            Transform = {
                position = {x = x, y = y, z = 0},
                scale = {x = 0.4, y = 0.4, z = 1.0}
            },
            Sprite = {
                color = {
                    r = 1.0,
                    g = 0.3 + math.random() * 0.4,
                    b = 0.0,
                    a = 1.0
                },
                tex_index = 32,
                layer = 13.0
            },
            Velocity = {
                linear = {
                    x = math.cos(angle) * speed,
                    y = math.sin(angle) * speed,
                    z = 0
                }
            }
        })

        table.insert(GameState.particles, {
            entity = particle,
            lifetime = 0.5 + math.random() * 0.3,
            type = "explosion"
        })
    end
end

local function update_particles(delta_time)
    for i = #GameState.particles, 1, -1 do
        local particle_data = GameState.particles[i]
        particle_data.lifetime = particle_data.lifetime - delta_time

        if particle_data.lifetime <= 0 then
            destroy_entity(particle_data.entity)
            table.remove(GameState.particles, i)
        else
            -- 淡出效果
            local sprite = get_component(particle_data.entity, "Sprite")
            if sprite then
                local alpha = particle_data.lifetime
                -- 实际应用中需要更新color的alpha值
            end
        end
    end
end

-- ========================================
-- 游戏循环
-- ========================================

local function game_over()
    GameState.game_over = true
    print("GAME OVER!")
    print("Final Score: " .. GameState.score)
    print("Wave Reached: " .. GameState.wave)
end

local function reset_game()
    -- 清理所有实体
    for _, bullet_data in ipairs(GameState.bullets) do
        destroy_entity(bullet_data.entity)
    end
    for _, enemy_data in ipairs(GameState.enemies) do
        destroy_entity(enemy_data.entity)
    end
    for _, particle_data in ipairs(GameState.particles) do
        destroy_entity(particle_data.entity)
    end

    if GameState.player then
        destroy_entity(GameState.player)
    end

    -- 重置状态
    GameState.player = nil
    GameState.enemies = {}
    GameState.bullets = {}
    GameState.particles = {}
    GameState.score = 0
    GameState.wave = 1
    GameState.game_over = false
    player_hp = 100

    print("Game reset!")
end

local function update_game(delta_time)
    if GameState.game_over then
        return
    end

    -- 更新玩家
    update_player(delta_time)

    -- 自动射击
    player_fire()

    -- 更新敌人
    update_enemies(delta_time)

    -- 更新子弹
    update_bullets(delta_time)

    -- 更新粒子
    update_particles(delta_time)

    -- 碰撞检测
    check_collisions()

    -- 检查波次完成
    if #GameState.enemies == 0 then
        GameState.wave = GameState.wave + 1
        print("Wave " .. GameState.wave .. " starting!")
        spawn_wave(GameState.wave)
    end
end

-- ========================================
-- 游戏初始化
-- ========================================

local function init_game()
    print("=== Space Shooter Game ===")
    print("Initializing game...")

    create_player()
    spawn_wave(1)

    print("Game started!")
    print("HP: " .. player_hp)
    print("Score: " .. GameState.score)
end

-- 启动游戏
init_game()

-- ========================================
-- 模拟游戏循环 (实际应用中由引擎调用)
-- ========================================

local delta_time = 0.016  -- 60 FPS

for frame = 1, 300 do  -- 运行5秒 (300帧)
    update_game(delta_time)

    if frame % 60 == 0 then
        print(string.format("Time: %.1fs, Score: %d, Enemies: %d, HP: %d",
            frame * delta_time,
            GameState.score,
            #GameState.enemies,
            player_hp))
    end

    if GameState.game_over then
        break
    end
end

print("\n=== Game Demo Complete ===")
print("Final Stats:")
print("  Score: " .. GameState.score)
print("  Wave: " .. GameState.wave)
print("  Remaining Enemies: " .. #GameState.enemies)
print("  Active Bullets: " .. #GameState.bullets)
print("  Active Particles: " .. #GameState.particles)
