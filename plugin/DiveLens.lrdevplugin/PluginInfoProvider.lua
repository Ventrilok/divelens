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
            and LOC "$$$/DiveLens/PluginManager/Connected=Dive Lens app connected."
            or LOC "$$$/DiveLens/PluginManager/NotConnected=Dive Lens app not connected."
        return {
            {
                title = LOC "$$$/DiveLens/PluginManager/Title=Dive Lens",
                f:row {
                    f:static_text {
                        title = LOC "$$$/DiveLens/PluginManager/Text=All Dive Lens settings are in the app." .. " " .. status,
                        fill_horizontal = 1,
                    },
                    f:push_button {
                        title = LOC "$$$/DiveLens/PluginManager/Open=Open Dive Lens",
                        action = function() Launch.open("collections") end,
                    },
                },
            },
        }
    end,
}
