--[[
Plugin log, written by Lightroom to DiveLens.log in its log folder
(macOS: ~/Library/Logs/Adobe/Lightroom/LrClassicLogs/).

Part of DiveLens, GPL-3.0-or-later.
]]

local logger = import "LrLogger" ("DiveLens")

-- What each level of the app's setting writes: "normal" leaves debug lines
-- out, "detailed" writes them too.
local LEVELS = {
    normal = { fatal = "logfile", error = "logfile", warn = "logfile", info = "logfile" },
    detailed = { fatal = "logfile", error = "logfile", warn = "logfile", info = "logfile", debug = "logfile", trace = "logfile" },
}
logger:enable(LEVELS.normal)

local function format(msg, ...)
    if select("#", ...) == 0 then return msg end
    local args = { ... }
    for i = 1, select("#", ...) do args[i] = tostring(args[i]) end
    return string.format(msg, unpack(args))
end

local Log = {}

function Log.debug(msg, ...) logger:debug(format(msg, ...)) end
function Log.info(msg, ...) logger:info(format(msg, ...)) end
function Log.warn(msg, ...) logger:warn(format(msg, ...)) end
function Log.error(msg, ...) logger:error(format(msg, ...)) end

--- Follows the app's log level setting ("normal" or "detailed"); returns
--- false for an unknown level, which changes nothing.
function Log.setLevel(level)
    local actions = LEVELS[level]
    if not actions then return false end
    logger:enable(actions)
    return true
end

return Log
