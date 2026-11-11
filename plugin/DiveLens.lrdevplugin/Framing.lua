--[[
Splits what the link socket receives into messages: one JSON object per line.

Lightroom may hand over a message with or without its final newline, and may
group or split lines; this copes with all three. Pure Lua, no Lightroom
import, so it can be tested outside Lightroom.

Part of DiveLens, GPL-3.0-or-later.
]]

local Framing = {}

function Framing.new()
    return { buffer = "" }
end

--- Adds a received chunk and returns the complete messages it finishes, in
--- order. `isWhole(text)` tells whether a trailing piece without newline is a
--- complete message already (Lightroom stripped the newline).
function Framing.push(state, chunk, isWhole)
    local messages = {}
    local buffer = state.buffer .. chunk
    local start = 1
    while true do
        local nl = buffer:find("\n", start, true)
        if not nl then break end
        local line = buffer:sub(start, nl - 1):gsub("\r$", "")
        if line:find("%S") then messages[#messages + 1] = line end
        start = nl + 1
    end
    local rest = buffer:sub(start):gsub("\r$", "")
    if rest:find("%S") and isWhole and isWhole(rest) then
        messages[#messages + 1] = rest
        rest = ""
    end
    state.buffer = rest
    return messages
end

return Framing
