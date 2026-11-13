--[[
Turns a "tag the selection" request from the app into the list of changes to
make on each photo. Pure Lua (no Lightroom import) so it is tested outside
Lightroom; Link.lua applies the changes inside one catalog write.

"Create Collections" requests are planned per dive with planCollections;
a photo's GPS altitude (the depth, negative) is written only when the
request carries one, otherwise the photo keeps its altitude.

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
        plugin("Source", text(dive.source))
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

--- Changes for one dive of a "Create Collections" request:
--- { collection = name, common = changes for every photo,
---   photos = { { id = photo id, changes = per-photo changes }, ... } },
--- or nil and a reason.
local function planGroup(group, formatDate)
    if type(group) ~= "table" or not text(group.collection) then return nil, "bad group" end
    local common, why = Tagging.plan({ dive = group.dive, location = group.location }, formatDate)
    if not common then return nil, why end
    if not value(group.dive) then return nil, "bad dive" end
    local photos = {}
    for _, photo in ipairs(value(group.photos) or {}) do
        if type(photo) ~= "table" or not text(photo.id) or not text(photo.clockOffset) then
            return nil, "bad photo"
        end
        local changes = {
            { kind = "plugin", field = "PhotoDepth", value = text(photo.photoDepth) },
            { kind = "plugin", field = "PhotoTemp", value = text(photo.photoTemp) },
            { kind = "plugin", field = "PhotoElapsed", value = text(photo.photoElapsed) },
            { kind = "plugin", field = "ClockOffset", value = text(photo.clockOffset) },
        }
        local altitude = value(photo.gpsAltitude)
        if type(altitude) == "number" then
            changes[#changes + 1] = { kind = "raw", field = "gpsAltitude", value = altitude }
        end
        photos[#photos + 1] = { id = photo.id, changes = changes }
    end
    return { collection = group.collection, common = common, photos = photos }
end

--- Plans a "Create Collections" request (core/src/tags.rs, CollectionsRequest):
--- a list of groups (see planGroup) and where the collections go
--- ({ parent = list of set names from the root, group = set name or nil }),
--- or nil and a reason.
function Tagging.planCollections(request, formatDate)
    if type(request) ~= "table" or type(value(request.groups)) ~= "table" then
        return nil, "bad request"
    end
    local parent = {}
    for i, name in ipairs(value(request.parent) or {}) do
        if not text(name) or name == "" then return nil, "bad parent" end
        parent[i] = name
    end
    local group = text(request.group)
    if group == "" then group = nil end
    local groups = {}
    for i, g in ipairs(request.groups) do
        local planned, why = planGroup(g, formatDate)
        if not planned then return nil, why end
        groups[i] = planned
    end
    if #groups == 0 then return nil, "nothing to write" end
    return groups, { parent = parent, group = group }
end

--- Follows `names` down from `root` (the catalog or a collection set): the
--- collection set found, or nil. `names` empty: `root` itself.
function Tagging.findSet(root, names)
    local current = root
    for _, name in ipairs(names) do
        local found
        for _, child in ipairs(current:getChildCollectionSets()) do
            if child:getName() == name then
                found = child
                break
            end
        end
        if not found then return nil end
        current = found
    end
    return current
end

--- The collection sets under `root`, as a tree { name, children }.
function Tagging.describeSets(root, emptyArray)
    local list = {}
    for _, set in ipairs(root:getChildCollectionSets()) do
        list[#list + 1] = { name = set:getName(), children = Tagging.describeSets(set, emptyArray) }
    end
    if #list == 0 then return emptyArray end
    return list
end

--- Wall-clock "YYYY-MM-DDTHH:MM:SS" from one of Lightroom's ISO 8601 capture
--- times ("2025-12-31T07:14:09", with or without fraction, seconds or time
--- zone; the zone is dropped: times are wall-clock, like the dives'), or nil.
function Tagging.wallClock(iso)
    if type(iso) ~= "string" then return nil end
    local date, h, mi, rest = iso:match("^(%d%d%d%d%-%d%d%-%d%d)T(%d%d):(%d%d)(.*)$")
    if not date then return nil end
    local sec = rest:match("^:(%d%d)") or "00"
    return string.format("%sT%s:%s:%s", date, h, mi, sec)
end

--- One photo as sent to the app: Lightroom id, wall-clock capture time,
--- camera model and serial number (nil when unknown).
function Tagging.describePhoto(id, iso, make, model, serial)
    local function clean(v)
        if type(v) ~= "string" then return nil end
        v = v:gsub("^%s+", ""):gsub("%s+$", "")
        if v == "" then return nil end
        return v
    end
    return {
        id = tostring(id),
        time = Tagging.wallClock(iso),
        model = clean(model) or clean(make),
        serial = clean(serial),
    }
end

--- Wall-clock "YYYY-MM-DDTHH:MM:SS" to its parts, or nil.
function Tagging.dateParts(s)
    local y, mo, d, h, mi, sec = tostring(s):match("^(%d+)-(%d+)-(%d+)T(%d+):(%d+):(%d+)$")
    if not y then return nil end
    return tonumber(y), tonumber(mo), tonumber(d), tonumber(h), tonumber(mi), tonumber(sec)
end

return Tagging
