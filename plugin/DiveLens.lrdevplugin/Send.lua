--[[
Sends a message to the DiveLens app, on the link's "send" socket (Link.lua):
one JSON object on one line. Returns false when no app is connected or the
send failed.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrTasks = import "LrTasks"

local Json = require "Json"
local Log = require "Log"

local Send = {}

function Send.toApp(s, message)
    if not (s.sender and s.senderConnected) then return false end
    local ok, err = LrTasks.pcall(function() s.sender:send(Json.encode(message) .. "\n") end)
    if not ok then
        Log.warn("send failed: %s", err)
        return false
    end
    return true
end

return Send
