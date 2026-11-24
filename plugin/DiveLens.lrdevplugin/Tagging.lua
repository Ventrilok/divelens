--[[
Turns a "tag the selection" request from the app into the list of changes to
make on each photo. Pure Lua (no Lightroom import) so it is tested outside
Lightroom. ApplyTags.lua and ApplyCollections.lua make the two catalog
writes, through CatalogWrite.lua, the GPS and altitude in the second
(splitGps); what goes in each, photo by photo (tagWrites,
collectionWrites), and what the app is told afterwards (applyReply), is
decided here.

"Create Collections" requests are planned per dive with planCollections;
a photo's GPS altitude (the depth, negative) is written only when the
request carries one, and recorded in the altitude marker (WrittenAltitude,
photoChanges). Otherwise the photo keeps its altitude, unless it is the one
Dive Lens wrote from a depth now cleared or another dive's (altitudeClears,
photoWrites).

A "tag the selection" request that writes a dive on a photo already carrying
another one also clears that photo's own values (retagChanges).

When the dive has a profile, a "tag the selection" request also carries the
per-photo values of the photos taken during the dive (planPhotos), computed
by the app for the selection the plugin described to it (`selection`): it is
written only on exactly those photos (sameSelection). Those values replace
what the other dive left (notOverwritten); the other photos get the dive
data alone. photoWrites puts one photo's changes together.

Request (built by the app, see core/src/tags.rs):
  dive:      { diveId, start, end, duration, maxDepth, tempHigh, tempLow, rating } or null
  location:  { country, city, location, lat, lon } or null
  selection: number of the selection description the values are for, or null
  photos:    { { id, photoDepth, photoTemp, photoElapsed, clockOffset, gpsAltitude }, ... }

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
    if not dive and not location and not value(request.collection) then
        return nil, "nothing to write"
    end
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

--- The hidden field recording the GPS altitude Dive Lens last wrote on a
--- photo, in metres, as text: the altitude marker (Metadata.lua, spec F7).
Tagging.MARKER = "WrittenAltitude"

--- Every reason the plug-in gives in an "applyFailed" message, for both
--- features. A declaration only: the app's own list (APPLY_REASONS in
--- lightroom.rs) and its texts (apply/failure.ts) are checked against it by
--- tests/apply_contract.rs.
Tagging.REASONS = {
    "error",
    "busy",
    "badRequest",
    "gpsNotWritten",
    "noSelection",
    "selectionChanged",
    "staleSelection",
    "staleSearch",
    "parentNotFound",
}

--- DiveLens fields "Tag Selection" and "Create Collections" read on each
--- photo before writing a dive (see retagChanges, photoWrites): the dive
--- identity, then the photo's own values, then the altitude marker.
Tagging.RETAG_FIELDS = {}
for _, list in ipairs { DIVE_IDENTITY, PER_PHOTO, { Tagging.MARKER } } do
    for _, field in ipairs(list) do
        table.insert(Tagging.RETAG_FIELDS, field)
    end
end

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
--- rounded to the decimetre: core/src/tags.rs, altitude). The rule for a
--- photo without an altitude marker (see altitudeClears).
function Tagging.isDerivedAltitude(altitude, depth)
    local m = Tagging.depthMetres(depth)
    if type(altitude) ~= "number" or not m then return false end
    return round(altitude * 10) == -round(m * 10)
end

--- The altitude marker's text for an altitude written, in metres: the
--- number as the link writes numbers (Json.lua), "-18.4".
function Tagging.markerText(altitude)
    return string.format("%.14g", altitude)
end

--- Changes for a photo whose depth Dive Lens clears, or replaces with
--- another dive's (`photo`: what it carries before the write, RETAG_FIELDS
--- and `gpsAltitude`). The GPS altitude is shared with the camera and the
--- user: it is cleared only when it is Dive Lens's own, and when in doubt
--- it stays (spec F7, F8).
---   - With a marker: the altitude is Dive Lens's own when it still equals
---     the marker, to the centimetre (Lightroom may store it as a float):
---     the altitude and the marker are cleared. Otherwise it was changed
---     since (by hand, by another tool, or a second write that did not
---     happen): it stays, and the marker, no longer true, is cleared. A
---     marker that is not a number counts as changed.
---   - Without a marker (photos written before the marker, or never by
---     Dive Lens): cleared when it is the one derived from the photo's
---     depth (isDerivedAltitude).
--- The changes go in the two writes as splitGps puts them: the marker in
--- the first, the altitude in the second.
function Tagging.altitudeClears(photo)
    local altitude = photo.gpsAltitude
    local marker = filled(photo[Tagging.MARKER])
    local clearMarker = { kind = "plugin", field = Tagging.MARKER, value = nil }
    local clearAltitude = { kind = "raw", field = "gpsAltitude", value = nil }
    if marker then
        local written = tonumber(marker)
        if type(altitude) == "number" and written and round(altitude * 100) == round(written * 100) then
            return { clearAltitude, clearMarker }
        end
        return { clearMarker }
    end
    if Tagging.isDerivedAltitude(altitude, filled(photo.PhotoDepth)) then
        return { clearAltitude }
    end
    return {}
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
--- when it is Dive Lens's own (altitudeClears).
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
    for _, c in ipairs(Tagging.altitudeClears(photo)) do extra[#extra + 1] = c end
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
--- end). The second write tells nothing: it must not pause (see
--- CatalogWrite.writeGpsLater), so the count stays where the first write
--- left it.
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

--- Why a catalog write (Lightroom's withWriteAccessDo, run under pcall:
--- `ok`, `status`) wrote nothing, or nil when it was executed: "error" and
--- the error's text, or "busy" when Lightroom kept the catalog busy.
function Tagging.writeFailure(ok, status)
    if not ok then return "error", tostring(status) end
    if status ~= "executed" then return "busy" end
    return nil
end

--- True when a change of the second write clears the GPS altitude: the one
--- write there Lightroom may refuse, which then leaves the altitude
--- (CatalogWrite.lua, writeGps).
function Tagging.isAltitudeClear(change)
    return change.field == "gpsAltitude" and change.value == nil
end

--- What the second write of an "Apply" did, from `ok` and `status` (as for
--- writeFailure), and `cleared` and `refused` (altitudes cleared, and
--- altitudes Lightroom refused to clear): { written = true, cleared,
--- refused }, or { written = false, error = message, or nil when Lightroom
--- kept the catalog busy }.
function Tagging.secondWriteResult(ok, status, cleared, refused)
    if not ok then return { written = false, error = tostring(status) } end
    if status ~= "executed" then return { written = false } end
    return { written = true, cleared = cleared, refused = refused }
end

--- The answer to the "Apply" `id`, once both writes are over. `outcome`:
--- `ok` and `status` of the first write (as for writeFailure), and
--- `second`, the second write's result (secondWriteResult), false when the
--- first wrote nothing. Nothing written: "applyFailed", "error" or "busy";
--- the first write only (dive data and location text, no GPS):
--- "applyFailed", "gpsNotWritten"; both: `done`, as it is.
function Tagging.applyReply(id, outcome, done)
    local reason, detail = Tagging.writeFailure(outcome.ok, outcome.status)
    if reason then return { type = "applyFailed", id = id, reason = reason, detail = detail } end
    local second = outcome.second
    if not second.written then
        return { type = "applyFailed", id = id, reason = "gpsNotWritten", detail = second.error }
    end
    return done
end

--- The answer to the "Apply" `id` when an unexpected Lua error ended the
--- request, so the app is never left waiting. `record` is the plug-in's
--- apply record as it is, nil or `{ id = ..., stage = ... }`; it counts only
--- when its `id` is this request's (the app's ids restart at each launch),
--- otherwise the request is taken as "before". `stage` says how far the apply
--- got: "before" (the first catalog write had not run: "applyFailed",
--- "error", nothing written), "afterFirst" (the first write ran, the GPS and
--- altitude may not have: "applyFailed", "gpsNotWritten", never "nothing
--- written"), "replied" (the app already has its answer: nil, send nothing).
--- `err` is the error text, given as the detail.
function Tagging.crashReply(id, record, err)
    local stage = type(record) == "table" and record.id == id and record.stage or "before"
    if stage == "replied" then return nil end
    local reason = stage == "afterFirst" and "gpsNotWritten" or "error"
    return { type = "applyFailed", id = id, reason = reason, detail = tostring(err) }
end

-- One photo's own values, as the app sent them (core/src/tags.rs,
-- PhotoTags): its per-photo fields (a missing value clears the field), its
-- clock offset, and its GPS altitude when there is one to write, with the
-- altitude marker recording it. Returns the photo's id and its changes, or
-- nil and a reason.
--
-- The marker is a Dive Lens field, so splitGps puts it in the first write,
-- with the other Dive Lens fields; the altitude goes in the second (spec
-- section 3). Never the marker in the second write: Lightroom's Undo of a
-- GPS or an altitude has only been checked written alone. If the second
-- write does not happen, the marker and the altitude disagree, and
-- altitudeClears then leaves the altitude: the safe side. One Undo reverts
-- both writes, marker included.
local function photoChanges(photo)
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
        changes[#changes + 1] = { kind = "plugin", field = Tagging.MARKER, value = Tagging.markerText(altitude) }
    end
    return photo.id, changes
end

--- The per-photo values of a "Tag Selection" request:
--- { selection = number of the selection described, photos = { [id] =
--- changes } } for the photos taken during the dive; nil when the request
--- carries none (no `selection`: no dive, or a dive without a profile); nil
--- and a reason when it is not usable.
function Tagging.planPhotos(request)
    if type(request) ~= "table" then return nil, "bad request" end
    local selection = value(request.selection)
    if selection == nil then return nil end
    if type(selection) ~= "number" then return nil, "bad selection" end
    if not value(request.dive) then return nil, "bad dive" end
    local byId = {}
    for _, photo in ipairs(value(request.photos) or {}) do
        local id, changes = photoChanges(photo)
        if not id then return nil, changes end
        byId[id] = changes
    end
    return { selection = selection, photos = byId }
end

--- `extra` (from retagChanges) without what `own`, a photo's per-photo
--- changes (planPhotos), writes again: a photo taken during the dive gets
--- its new values instead of having the other dive's cleared. The other
--- dive's GPS altitude and its marker are still cleared when no new one is
--- written (the copy to the altitude is off), as for the photos outside the
--- dive.
function Tagging.notOverwritten(extra, own)
    local set = {}
    for _, c in ipairs(own or {}) do set[c.kind .. ":" .. c.field] = true end
    local kept = {}
    for _, c in ipairs(extra) do
        if not set[c.kind .. ":" .. c.field] then kept[#kept + 1] = c end
    end
    return kept
end

-- True when a photo's own values (`own`, from photoChanges) give it a depth.
local function writesDepth(own)
    for _, c in ipairs(own) do
        if c.kind == "plugin" and c.field == "PhotoDepth" then return filled(c.value) ~= nil end
    end
    return false
end

--- One photo of a "Tag Selection" or "Create Collections" write, as
--- tagWrites and collectionWrites plan it: `changes` (from plan, or a group's common changes) for every
--- photo; what the other dive the photo carried left (retagChanges, from
--- `before`, what the photo carries; nil: nothing read); and the photo's
--- own values, `own` (from planPhotos or planCollections; nil: none, the
--- photo is not in the dive), which replace what the other dive left. When
--- `own` clears the photo's depth (Create Collections: a photo now at the
--- surface, or in a dive without a profile), the altitude Dive Lens derived
--- from the old depth goes too, even for the same dive (altitudeClears).
--- Returns the changes of the first catalog write, those of the second (GPS
--- and altitude, Tagging.splitGps), and whether the photo carried another
--- dive's values (cleared or replaced).
function Tagging.photoWrites(changes, before, own)
    local extra = before and Tagging.retagChanges(before, changes) or {}
    local replaced = #extra > 0
    if before and own and not writesDepth(own) then
        local listed = {}
        for _, c in ipairs(extra) do listed[c.kind .. ":" .. c.field] = true end
        for _, c in ipairs(Tagging.altitudeClears(before)) do
            if not listed[c.kind .. ":" .. c.field] then extra[#extra + 1] = c end
        end
    end
    if own then extra = Tagging.notOverwritten(extra, own) end
    local first, second = Tagging.splitGps(changes, extra, own or {})
    return first, second, replaced
end

--- A Lightroom photo's id, as the app knows it: its local identifier, as
--- text. The `idOf` of photoSet, sameSelection and the writes.
function Tagging.photoId(photo)
    return tostring(photo.localIdentifier)
end

--- The photos of a selection as a multiset, for sameSelection: { count =
--- number of photos, ids = { [id] = times it appears } }. `idOf` gives a
--- photo's id.
function Tagging.photoSet(photos, idOf)
    local ids = {}
    for _, photo in ipairs(photos) do
        local id = idOf(photo)
        ids[id] = (ids[id] or 0) + 1
    end
    return { count = #photos, ids = ids }
end

--- True when `photos`, those selected now, are exactly the photos of `set`
--- (photoSet), each as many times: the selection described to the app for
--- its per-photo values. `idOf` gives a photo's id.
function Tagging.sameSelection(set, photos, idOf)
    if type(set) ~= "table" or #photos ~= set.count then return false end
    local now = Tagging.photoSet(photos, idOf).ids
    for id, n in pairs(now) do
        if set.ids[id] ~= n then return false end
    end
    for id, n in pairs(set.ids) do
        if now[id] ~= n then return false end
    end
    return true
end

--- Why "Tag Selection" writes nothing on `photos`, those selected now, or
--- nil. With per-photo values (`perPhoto`, from planPhotos), the selection
--- must be exactly the one described to the app (`described` = { id, set
--- }, kept by Selection.describe): "staleSelection" when the
--- values are for another description, or none was made;
--- "selectionChanged" when the photos are not those described. `idOf`
--- gives a photo's id.
function Tagging.selectionRefusal(perPhoto, described, photos, idOf)
    if not perPhoto then return nil end
    if not (described and described.id == perPhoto.selection) then return "staleSelection" end
    if not Tagging.sameSelection(described.set, photos, idOf) then return "selectionChanged" end
    return nil
end

--- The two catalog writes of a "Tag Selection", planned photo by photo
--- (photoWrites) in the order of `photos`: `changes` (from plan) for every
--- photo, what each carries before (`before` = { [photo] = ... }, read
--- inside the first write; empty when not read) and its own values
--- (`perPhoto`, from planPhotos, or nil; `idOf` gives a photo's id).
--- Returns { writes = { { photo, changes }, ... }, the first write's, one
--- per photo, even with nothing to write; later = { { photo, changes },
--- ... }, the second write's, for the photos with a GPS position or
--- altitude to write; cleared = photos that carried another dive's values;
--- withValues = photos with their own values }.
function Tagging.tagWrites(changes, photos, before, perPhoto, idOf)
    local plan = { writes = {}, later = {}, cleared = 0, withValues = 0 }
    for _, photo in ipairs(photos) do
        local own = perPhoto and perPhoto.photos[idOf(photo)]
        if own then plan.withValues = plan.withValues + 1 end
        local now, afterwards, replaced = Tagging.photoWrites(changes, before[photo], own)
        if replaced then plan.cleared = plan.cleared + 1 end
        plan.writes[#plan.writes + 1] = { photo = photo, changes = now }
        if #afterwards > 0 then plan.later[#plan.later + 1] = { photo = photo, changes = afterwards } end
    end
    return plan
end

--- Changes for one group of a "Create Collections" request:
--- { collection = name or nil, common = changes for every photo,
---   photos = { { id = photo id, changes = per-photo changes or nil }, ... },
---   writesDive = true when the group writes a dive },
--- or nil and a reason. A group may lack its dive (then only its location
--- is written, and the photos' own values, which belong to a dive, are
--- ignored: `changes` is nil, only the id is required) or its collection
--- (then none is made), not all three.
local function planGroup(group, formatDate)
    if type(group) ~= "table" then return nil, "bad group" end
    local name = value(group.collection)
    if name ~= nil and type(name) ~= "string" then return nil, "bad group" end
    local dive, location = value(group.dive), value(group.location)
    if not dive and not location and name == nil then return nil, "nothing to write" end
    local common = {}
    if dive or location then
        local why
        common, why = Tagging.plan({ dive = dive, location = location }, formatDate)
        if not common then return nil, why end
    end
    local list = value(group.photos)
    if type(list) ~= "table" then return nil, "bad group" end
    local photos = {}
    for _, photo in ipairs(list) do
        local id, changes
        if dive then
            id, changes = photoChanges(photo)
        elseif type(photo) == "table" and text(photo.id) then
            id = photo.id
        else
            id, changes = nil, "bad photo"
        end
        if not id then return nil, changes end
        photos[#photos + 1] = { id = id, changes = changes }
    end
    return { collection = name, common = common, photos = photos, writesDive = dive ~= nil }
end

--- The collection a "Tag Selection" request asks for (core/src/tags.rs,
--- TagRequest.collection): { name, parent = list of set names from the
--- root, group = set name or nil }; nil without one; nil and "bad
--- collection" when it is unusable (same checks as planCollections).
function Tagging.planCollection(request)
    local c = value(request.collection)
    if c == nil then return nil end
    if type(c) ~= "table" or not text(c.name) or c.name == "" then return nil, "bad collection" end
    local parent = {}
    local names = value(c.parent) or {}
    if type(names) ~= "table" then return nil, "bad collection" end
    for i, name in ipairs(names) do
        if not text(name) or name == "" then return nil, "bad collection" end
        parent[i] = name
    end
    local group = text(c.group)
    if group == "" then group = nil end
    return { name = c.name, parent = parent, group = group }
end

--- What "Tag Selection" tells the app about its collection: `count` photos
--- were put in `name`; `existing` = { [name] = true } for the collections
--- already in that place, which is then completed, not created.
function Tagging.selectionCollection(name, count, existing)
    return { collection = name, count = count, created = not existing[name] and count > 0 }
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

--- True when at least one planned group (planCollections) has a collection
--- to make: only then is the collection set looked for or created.
function Tagging.namesCollections(groups)
    for _, g in ipairs(groups) do
        if g.collection then return true end
    end
    return false
end

--- The photos found (resolvePhotos) of the groups that write a dive: the
--- only ones whose previous dive data is read, to clear what it left.
function Tagging.divePhotos(groups)
    local found = {}
    for _, g in ipairs(groups) do
        if g.writesDive then
            for _, p in ipairs(g.photos) do
                if p.photo then found[#found + 1] = p.photo end
            end
        end
    end
    return found
end

--- Finds the photos of planned groups (planCollections) in `byId`, the
--- photos of the search the request names by id: sets `photo` on each
--- planned photo, nil when it is no longer there. Returns the number of
--- photos requested, found or not, and the list of those found.
function Tagging.resolvePhotos(groups, byId)
    local total, found = 0, {}
    for _, g in ipairs(groups) do
        total = total + #g.photos
        for _, p in ipairs(g.photos) do
            p.photo = byId[p.id]
            if p.photo then found[#found + 1] = p.photo end
        end
    end
    return total, found
end

--- The two catalog writes of a "Create Collections", planned group by
--- group and photo by photo (photoWrites): `groups` from planCollections,
--- their photos found (resolvePhotos); `before` = { [photo] = what it
--- carries }, read inside the first write; `existing` = { [name] = true }
--- for the collections already there. Returns:
---   groups  = { [gi] = { steps, photos } }: one step per photo requested,
---             found or not, { photo, changes } for the first write, or {}
---             for a photo not found; `photos`, those found, for the
---             collection;
---   later   = { { photo, changes }, ... }: the second write's, for the
---             photos with a GPS position or altitude to write;
---   results = { [gi] = { collection (nil without a name), count = photos
---             found, created } }, as
---             the app is told: `created` only for the group that really
---             makes its collection: a name, a photo found, the name neither there
---             nor made by an earlier group (that one is completed);
---   cleared = photos that carried another dive's values (as tagWrites).
function Tagging.collectionWrites(groups, before, existing)
    local plan = { groups = {}, later = {}, results = {}, cleared = 0 }
    local made = {}
    for gi, g in ipairs(groups) do
        local steps, photos = {}, {}
        for _, p in ipairs(g.photos) do
            local step = {}
            if p.photo then
                -- Its changes for the first write, and its GPS position
                -- and altitude for the second.
                -- A group without a dive reads nothing and clears nothing
                -- (p.changes is nil): another dive's values stay.
                local now, gps, replaced = Tagging.photoWrites(g.common, p.changes and before[p.photo], p.changes)
                if replaced then plan.cleared = plan.cleared + 1 end
                step.photo, step.changes = p.photo, now
                if #gps > 0 then plan.later[#plan.later + 1] = { photo = p.photo, changes = gps } end
                photos[#photos + 1] = p.photo
            end
            steps[#steps + 1] = step
        end
        plan.groups[gi] = { steps = steps, photos = photos }
        local created = g.collection ~= nil and #photos > 0
            and not existing[g.collection] and not made[g.collection]
        if created then made[g.collection] = true end
        plan.results[gi] = { collection = g.collection, count = #photos, created = created }
    end
    return plan
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

--- The item of `list` (collections or collection sets) named `name`, or nil.
function Tagging.findChild(list, name)
    for _, item in ipairs(list) do
        if item:getName() == name then return item end
    end
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

--- Earliest and latest capture times of the selection, wall-clock
--- "YYYY-MM-DDTHH:MM:SS" (or nil), overall and per camera (the app applies
--- each camera's remembered clock offset). `list`: the photos described
--- (describePhoto). Returns nil for no photo; otherwise the first time, the
--- last, and the cameras in the order first seen, { { model, serial, from,
--- to }, ... }.
function Tagging.timeRange(list)
    if #list == 0 then return nil end
    local first, last
    local cameras, order = {}, {}
    for _, p in ipairs(list) do
        local t = p.time
        if t then
            if not first or t < first then first = t end
            if not last or t > last then last = t end
            local key = (p.model or "") .. "|" .. (p.serial or "")
            local c = cameras[key]
            if not c then
                c = { model = p.model, serial = p.serial, from = t, to = t }
                cameras[key] = c
                order[#order + 1] = c
            end
            if t < c.from then c.from = t end
            if t > c.to then c.to = t end
        end
    end
    return first, last, order
end

--- Wall-clock "YYYY-MM-DDTHH:MM:SS" to its parts, or nil.
function Tagging.dateParts(s)
    local y, mo, d, h, mi, sec = tostring(s):match("^(%d+)-(%d+)-(%d+)T(%d+):(%d+):(%d+)$")
    if not y then return nil end
    return tonumber(y), tonumber(mo), tonumber(d), tonumber(h), tonumber(mi), tonumber(sec)
end

return Tagging
