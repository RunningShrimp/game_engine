-- Enemy AI Script
--
-- Basic enemy behavior for FPS game

local enemy = {
    health = 100,
    speed = 3.0,
    detection_range = 20.0,
    attack_range = 5.0,
    damage = 10,
    state = "idle",
    target = nil,
    position = { x = 0, y = 0, z = 0 }
}

function enemy.init(x, y, z)
    enemy.position.x = x
    enemy.position.y = y
    enemy.position.z = z
    enemy.state = "patrol"
end

function enemy.update(dt, player_position)
    -- Calculate distance to player
    local dx = player_position.x - enemy.position.x
    local dz = player_position.z - enemy.position.z
    local distance = math.sqrt(dx * dx + dz * dz)

    -- State machine
    if enemy.state == "patrol" then
        if distance < enemy.detection_range then
            enemy.state = "chase"
            enemy.target = player_position
            print("Enemy spotted player!")
        end
    elseif enemy.state == "chase" then
        if distance < enemy.attack_range then
            enemy.state = "attack"
        elseif distance > enemy.detection_range * 1.5 then
            enemy.state = "patrol"
            enemy.target = nil
        else
            -- Move towards player
            local move_x = (dx / distance) * enemy.speed * dt
            local move_z = (dz / distance) * enemy.speed * dt
            enemy.position.x = enemy.position.x + move_x
            enemy.position.z = enemy.position.z + move_z
        end
    elseif enemy.state == "attack" then
        if distance > enemy.attack_range then
            enemy.state = "chase"
        else
            -- Attack player
            enemy.attack()
        end
    end
end

function enemy.attack()
    -- TODO: Apply damage to player
    print("Enemy attacks for " .. enemy.damage .. " damage!")
end

function enemy.take_damage(amount)
    enemy.health = enemy.health - amount
    print("Enemy takes " .. amount .. " damage. Health: " .. enemy.health)

    if enemy.health <= 0 then
        enemy.die()
    end
end

function enemy.die()
    print("Enemy defeated!")
    -- TODO: Play death animation, drop loot
end

return enemy
