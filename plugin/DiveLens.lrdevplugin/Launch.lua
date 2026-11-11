--[[
Opens the DiveLens app on a given screen: brings the running app to the front
when it is connected, launches it otherwise.

The app ships inside the plugin folder. During development, the release build
of the repository is used when the plugin is loaded from the repository.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrDialogs = import "LrDialogs"
local LrFileUtils = import "LrFileUtils"
local LrPathUtils = import "LrPathUtils"
local LrTasks = import "LrTasks"

local Link = require "Link"
local Log = require "Log"

local Launch = {}

local function childPath(base, ...)
    for _, part in ipairs({ ... }) do base = LrPathUtils.child(base, part) end
    return base
end

local function candidates()
    local plugin = _PLUGIN.path
    local repo = LrPathUtils.parent(LrPathUtils.parent(plugin))
    if WIN_ENV then
        return {
            childPath(plugin, "DiveLens.exe"),
            childPath(repo, "target", "release", "divelens.exe"),
        }
    end
    return {
        childPath(plugin, "DiveLens.app"),
        childPath(repo, "target", "release", "bundle", "macos", "DiveLens.app"),
    }
end

--- Path of the app, or nil plus the expected path when it is missing.
function Launch.findApp()
    local list = candidates()
    for _, path in ipairs(list) do
        if LrFileUtils.exists(path) then return path end
    end
    return nil, list[1]
end

local function shellQuote(s)
    return "'" .. s:gsub("'", "'\\''") .. "'"
end

local function command(app, screen)
    if WIN_ENV then
        -- Lightroom runs this through cmd /c, which strips one pair of outer quotes.
        return '"start "" "' .. app .. '" --screen ' .. screen .. '"'
    end
    return "open -a " .. shellQuote(app) .. " --args --screen " .. screen
end

--- Opens the app on `screen` ("dives", "collections" or "tag").
function Launch.open(screen)
    LrTasks.startAsyncTask(function()
        if Link.show(screen) then return end
        local app, expected = Launch.findApp()
        if not app then
            Log.error("app not found, expected at %s", expected)
            LrDialogs.message(
                LOC "$$$/DiveLens/Launch/NotFound/Title=DiveLens app not found",
                LOC("$$$/DiveLens/Launch/NotFound/Message=The DiveLens app should be in the plug-in folder: ^1. Please reinstall DiveLens.", expected),
                "critical")
            return
        end
        Link.showWhenConnected(screen)
        local cmd = command(app, screen)
        Log.info("launching the app: %s", cmd)
        local status = LrTasks.execute(cmd)
        if status ~= 0 then
            Log.error("app launch failed with status %s", status)
            LrDialogs.message(
                LOC "$$$/DiveLens/Launch/Failed/Title=Cannot open DiveLens",
                LOC("$$$/DiveLens/Launch/Failed/Message=The DiveLens app could not be started (code ^1). See the DiveLens log.", tostring(status)),
                "critical")
        end
    end, "DiveLens launch")
end

return Launch
