--[[
Turns a "tag the selection" request from the app into the list of changes to
make on each photo. Pure Lua (no Lightroom import) so it is tested outside
Lightroom; Link.lua applies the changes in two catalog writes, the GPS and
altitude in the second (splitGps).

"Create Collections" requests are planned per dive with planCollections;
a photo's GPS altitude (the depth, negative) is written only when the
request carries one, otherwise the photo keeps its altitude.

A "tag the selection" request that writes a dive on a photo already carrying
another one also clears that photo's own values (retagChanges).

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

-- Photo's own values (spec F6, section 7): they belong to the dive they were
-- computed in.
local PER_PHOTO = { "PhotoDepth", "PhotoTemp", "PhotoElapsed", "ClockOffset" }

-- The fields that tell one dive from another (spec F8).
local DIVE_IDENTITY = { "Source", "StartDate", "DiveId" }

--- DiveLens fields "Tag Selection" reads on each photo before writing a dive
--- (see retagChanges).
Tagging.RETAG_FIELDS = { "Source", "StartDate", "DiveId", "PhotoDepth", "PhotoTemp", "PhotoElapsed", "ClockOffset" }

local FT_PER_M = 3.28084 -- as core/src/tags.rs

-- A field's value, nil when empty: a field emptied in Lightroom counts as unset.
local function filled(v)
    if type(v) ~= "string" or v == "" then return nil end
    return v
end

-- Nearest integer, halves away from zero (as Rust's f64::round).
local function round(x)
    if x < 0 then return -math.floor(-x + 0.5) end
    return math.floor(x + 0.5)
end

--- Depth in metres from a PhotoDepth value: "18.4 m", "60 ft", or a bare
--- number (metres, the format before units); nil when it is none of these.
function Tagging.depthMetres(s)
    if type(s) ~= "string" then return nil end
    local number, unit = s:match("^%s*(%-?%d+%.?%d*)%s*(%a*)%s*$")
    local n = tonumber(number)
    if not n then return nil end
    if unit == "" or unit == "m" then return n end
    if unit == "ft" then return n / FT_PER_M end
    return nil
end

--- True when the GPS `altitude` (metres) is, to the decimetre, the one Dive
--- Lens derives from the PhotoDepth value `depth` (the depth, negative,
--- rounded to the decimetre: core/src/tags.rs, altitude).
function Tagging.isDerivedAltitude(altitude, depth)
    local m = Tagging.depthMetres(depth)
    if type(altitude) ~= "number" or not m then return false end
    return round(altitude * 10) == -round(m * 10)
end

--- True when `changes` (from Tagging.plan) write a dive, not only a location.
function Tagging.writesDive(changes)
    for _, c in ipairs(changes) do
        if c.kind == "plugin" and c.field == "StartDate" then return true end
    end
    return false
end

--- Extra changes for one photo of a "Tag Selection" request (`changes`, from
--- Tagging.plan) when the photo already carries another dive (spec F8): its
--- own values belonged to that dive and are cleared (only those it has, so
--- an empty list means nothing to clear). Its GPS altitude is cleared only
--- when it is exactly the one Dive Lens derived from the old depth: the
--- field is shared with the camera and the user, so when in doubt it stays.
---
--- `photo` is what the photo carries before the write: the RETAG_FIELDS
--- values and `gpsAltitude` (metres or nil). Nothing for the same dive
--- (source, start and number all equal), a photo without dive data, or a
--- request writing only a location.
function Tagging.retagChanges(photo, changes)
    if not Tagging.writesDive(changes) then return {} end
    photo = photo or {}
    local new = {}
    for _, c in ipairs(changes) do
        if c.kind == "plugin" then new[c.field] = filled(c.value) end
    end
    local hasDive, same = false, true
    for _, field in ipairs(DIVE_IDENTITY) do
        local old = filled(photo[field])
        if old then hasDive = true end
        if old ~= new[field] then same = false end
    end
    if not hasDive or same then return {} end
    local extra = {}
    for _, field in ipairs(PER_PHOTO) do
        if filled(photo[field]) then
            extra[#extra + 1] = { kind = "plugin", field = field, value = nil }
        end
    end
    if Tagging.isDerivedAltitude(photo.gpsAltitude, filled(photo.PhotoDepth)) then
        extra[#extra + 1] = { kind = "raw", field = "gpsAltitude", value = nil }
    end
    return extra
end

-- Lightroom fields that go in the second catalog write.
local GPS_FIELDS = { gps = true, gpsAltitude = true }

--- Splits planned changes (from plan, planCollections or retagChanges; any
--- number of lists, taken in turn) between an "Apply"'s two catalog writes:
--- the first gets the Dive Lens fields and the location text (country,
--- city, location), the second the GPS position and altitude, set or
--- cleared. Lightroom's Undo does not revert a GPS or an altitude written in
--- the same write as a location text field; written in a write of their
--- own, right after, they are reverted with the rest (spec section 3).
--- Returns the two lists, each in the order given.
function Tagging.splitGps(...)
    local first, second = {}, {}
    for i = 1, select("#", ...) do
        for _, c in ipairs((select(i, ...))) do
            if c.kind == "raw" and GPS_FIELDS[c.field] then
                second[#second + 1] = c
            else
                first[#first + 1] = c
            end
        end
    end
    return first, second
end

--- Progress of an "Apply"'s first write, told in photos, `total` in all.
--- Call `advance()` after each photo: every `every` photos it returns the
--- number done, nil otherwise, and never `total` (the app's answer tells the
--- end). The second write tells nothing: it must not pause (see Link.lua,
--- writeGpsLater), so the count stays where the first write left it.
function Tagging.progress(total, every)
    local done = 0
    local p = {}
    function p.advance()
        done = done + 1
        if done % every ~= 0 or done >= total then return nil end
        return done
    end
    return p
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
