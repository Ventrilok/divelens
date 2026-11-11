--[[
Plugin log, written by Lightroom to DiveLens.log in its log folder
(macOS: ~/Library/Logs/Adobe/Lightroom/LrClassicLogs/).

Part of DiveLens, GPL-3.0-or-later.
]]

local logger = import "LrLogger" ("DiveLens")
logger:enable("logfile")

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

return Log
