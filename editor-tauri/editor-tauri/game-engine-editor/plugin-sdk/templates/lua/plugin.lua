--[[
    {{plugin-name}}

    {{description}}
--]]

local plugin = {
    name = "{{plugin-name}}",
    version = "0.1.0",
    apiVersion = "0.1.0",
    description = "{{description}}",
    author = "{{author}}"
}

function plugin:on_load(context)
    print(string.format("✓ Plugin '%s' loaded!", self.name))

    -- TODO: Initialize your plugin here
end

function plugin:on_update(context, delta_time)
    -- Called every frame
    -- TODO: Implement update logic
end

function plugin:on_unload(context)
    print(string.format("✓ Plugin '%s' unloaded!", self.name))

    -- TODO: Cleanup your plugin here
end

return plugin
