--[[
Plug-in Manager entry. Lightroom shows the version itself; this adds the link
state and a button to open the app. All settings live in the app.

Part of DiveLens, GPL-3.0-or-later.
]]

local Launch = require "Launch"
local Link = require "Link"

return {
    sectionsForTopOfDialog = function(f, _)
        local status = Link.isConnected()
            and LOC "$$$/DiveLens/PluginManager/Connected=DiveLens app connected."
            or LOC "$$$/DiveLens/PluginManager/NotConnected=DiveLens app not connected."
        return {
            {
                title = LOC "$$$/DiveLens/PluginManager/Title=DiveLens",
                f:row {
                    f:static_text {
                        title = LOC "$$$/DiveLens/PluginManager/Text=All DiveLens settings are in the app." .. " " .. status,
                        fill_horizontal = 1,
                    },
                    f:push_button {
                        title = LOC "$$$/DiveLens/PluginManager/Open=Open DiveLens",
                        action = function() Launch.open("dives") end,
                    },
                },
            },
        }
    end,
}
