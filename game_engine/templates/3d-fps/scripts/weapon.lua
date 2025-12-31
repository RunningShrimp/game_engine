-- Weapon System Script
--
-- Handles weapon behavior and shooting mechanics

local weapon = {
    name = "Pistol",
    damage = 25,
    fire_rate = 0.5,
    ammo = 12,
    max_ammo = 12,
    reserve_ammo = 60,
    is_reloading = false,
    reload_time = 2.0,
    last_shot = 0
}

function weapon.init()
    print("Weapon initialized: " .. weapon.name)
end

function weapon.shoot(current_time)
    if weapon.is_reloading then
        return false
    end

    if weapon.ammo <= 0 then
        print("Out of ammo! Reloading...")
        weapon.reload()
        return false
    end

    if current_time - weapon.last_shot < weapon.fire_rate then
        return false
    end

    -- Fire weapon
    weapon.ammo = weapon.ammo - 1
    weapon.last_shot = current_time

    print("Bang! Ammo: " .. weapon.ammo .. "/" .. weapon.max_ammo)

    -- TODO: Apply damage, play sound, spawn muzzle flash

    return true
end

function weapon.reload()
    if weapon.reserve_ammo <= 0 or weapon.ammo == weapon.max_ammo then
        return
    end

    weapon.is_reloading = true
    print("Reloading...")

    -- TODO: Play reload animation

    -- Simulate reload time (in real game, use coroutine/timer)
    weapon.ammo = weapon.max_ammo
    weapon.reserve_ammo = weapon.reserve_ammo - (weapon.max_ammo - weapon.ammo)
    weapon.is_reloading = false

    print("Reloaded!")
end

function weapon.get_ammo()
    return weapon.ammo, weapon.reserve_ammo
end

return weapon
