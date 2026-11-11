--[[
Turns a "tag the selection" request from the app into the list of changes to
make on each photo. Pure Lua (no Lightroom import) so it is tested outside
Lightroom; Link.lua applies the changes inside one catalog write.

Request (built by the app, see core/src/tags.rs):
  dive:     { diveId, start, end, duration, maxDepth, tempHigh, tempLow, rating } or null
  location: { country, city, location, lat, lon } or null

Rules:
  - DiveLens fields describe one dive: a missing value clears the field.
  - Lightroom location fields: a missing value leaves the photo untouched;
    GPS is written only when both coordinates are present.
  - Start and end are formatted with `formatDate` (Lightroom's date format,
    as the previous plugin did).

Part of DiveLens, GPL-3.0-or-later.
]]

local Json = require "Json"

local Tagging = {}

local function value(v)
    if v == nil or v == Json.null then return nil end
    return v
end

local function text(v)
    v = value(v)
    if type(v) ~= "string" then return nil end
    return v
end

--- Returns the changes as a list of { kind = "plugin" | "raw", field, value },
--- or nil and a reason when the request is not usable.
function Tagging.plan(request, formatDate)
    if type(request) ~= "table" then return nil, "bad request" end
    local dive = value(request.dive)
    local location = value(request.location)
    if not dive and not location then return nil, "nothing to write" end
    local changes = {}
    local function plugin(field, v) changes[#changes + 1] = { kind = "plugin", field = field, value = v } end
    local function raw(field, v) changes[#changes + 1] = { kind = "raw", field = field, value = v } end

    if dive then
        if type(dive) ~= "table" or not text(dive.start) or not text(dive["end"]) then
            return nil, "bad dive"
        end
        plugin("DiveId", text(dive.diveId))
        plugin("StartDate", formatDate(dive.start))
        plugin("EndDate", formatDate(dive["end"]))
        plugin("Duration", text(dive.duration))
        plugin("MaxDepth", text(dive.maxDepth))
        plugin("TempHigh", text(dive.tempHigh))
        plugin("TempLow", text(dive.tempLow))
        plugin("Rating", text(dive.rating))
    end

    if location then
        if type(location) ~= "table" then return nil, "bad location" end
        for _, pair in ipairs { { "country", "country" }, { "city", "city" }, { "location", "location" } } do
            local v = text(location[pair[1]])
            if v then raw(pair[2], v) end
        end
        local lat, lon = value(location.lat), value(location.lon)
        if type(lat) == "number" and type(lon) == "number" then
            raw("gps", { latitude = lat, longitude = lon })
        end
    end
    return changes
end

--- Wall-clock "YYYY-MM-DDTHH:MM:SS" to its parts, or nil.
function Tagging.dateParts(s)
    local y, mo, d, h, mi, sec = tostring(s):match("^(%d+)-(%d+)-(%d+)T(%d+):(%d+):(%d+)$")
    if not y then return nil end
    return tonumber(y), tonumber(mo), tonumber(d), tonumber(h), tonumber(mi), tonumber(sec)
end

return Tagging
