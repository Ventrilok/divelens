--[[
Plug-in Manager entry. Lightroom shows the version itself; this adds the link
state, buttons to open the app (at the start of its journey, as the Library
menu does, or on Settings), and the Dive Lens website and source code. All
settings live in the app.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrHttp = import "LrHttp"

local Launch = require "Launch"
local Link = require "Link"

local WEBSITE = "www.querton.com/divelens"
local GITHUB = "github.com/Ventrilok/divelens"

-- An address as shown, with a button opening it in the browser.
local function linkRow(f, text, button, address)
    return f:row {
        f:static_text {
            title = text,
            fill_horizontal = 1,
        },
        f:push_button {
            title = button,
            action = function() LrHttp.openUrlInBrowser("https://" .. address) end,
        },
    }
end

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
                },
                -- The buttons on a row of their own: Lightroom does not wrap
                -- the text, which would be cut off beside two buttons.
                f:row {
                    f:spacer { fill_horizontal = 1 },
                    f:push_button {
                        title = LOC "$$$/DiveLens/PluginManager/Open=Open Dive Lens",
                        action = function() Launch.open("journey") end,
                    },
                    f:push_button {
                        title = LOC "$$$/DiveLens/PluginManager/OpenSettings=Open Settings",
                        action = function() Launch.open("settings") end,
                    },
                },
                linkRow(f,
                    LOC("$$$/DiveLens/PluginManager/Website=Website: ^1", WEBSITE),
                    LOC "$$$/DiveLens/PluginManager/OpenWebsite=Open Website",
                    WEBSITE),
                linkRow(f,
                    LOC("$$$/DiveLens/PluginManager/GitHub=GitHub: ^1", GITHUB),
                    LOC "$$$/DiveLens/PluginManager/OpenGitHub=Open GitHub",
                    GITHUB),
            },
        }
    end,
}
