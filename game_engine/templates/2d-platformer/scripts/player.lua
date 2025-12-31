-- Player Controller Script
--
-- Handles player input and movement

local player = {
    x = 0,
    y = 0,
    velocity_x = 0,
    velocity_y = 0,
    speed = 200,
    jump_force = 400,
    is_grounded = false
}

function player.init(x, y)
    player.x = x
    player.y = y
    player.velocity_x = 0
    player.velocity_y = 0
end

function player.update(dt)
    -- Horizontal movement
    if input.is_down("left") then
        player.velocity_x = -player.speed
    elseif input.is_down("right") then
        player.velocity_x = player.speed
    else
        player.velocity_x = 0
    end

    -- Jump
    if input.is_pressed("jump") and player.is_grounded then
        player.velocity_y = player.jump_force
        player.is_grounded = false
    end

    -- Apply gravity
    player.velocity_y = player.velocity_y - 900 * dt

    -- Update position
    player.x = player.x + player.velocity_x * dt
    player.y = player.y + player.velocity_y * dt
end

function player.get_position()
    return player.x, player.y
end

return player
