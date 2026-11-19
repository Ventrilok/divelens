--[[
What Tag Selection (ApplyTags.lua) and Create Collections
(ApplyCollections.lua) share to write in the catalog: Lightroom's date
format, one photo's changes, what the photos carry before a write, the
progress messages to the app, and the second catalog write of an "Apply",
the GPS positions and altitudes. What goes on each photo is decided by
Tagging.lua.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrDate = import "LrDate"
local LrTasks = import "LrTasks"

local Log = require "Log"
local Send = require "Send"
local Tagging = require "Tagging"

local CatalogWrite = {}

CatalogWrite.WRITE_TIMEOUT = 10 -- seconds to wait for Lightroom's write access
CatalogWrite.PROGRESS_EVERY = 50 -- photos between two progress messages

-- Lightroom's own date format for StartDate and EndDate, exactly as the
-- previous plugin wrote them (the time zone does not matter: the same one
-- is used both ways, so the wall-clock time is kept).
function CatalogWrite.formatDate(iso)
    local y, mo, d, h, mi, sec = Tagging.dateParts(iso)
    if not y then return nil end
    local t = LrDate.timeFromComponents(y, mo, d, h, mi, sec)
    return LrDate.formatShortDate(t) .. " " .. LrDate.formatMediumTime(t)
end

-- Writes one planned change on a photo, inside a catalog write.
local function writeChange(photo, change)
    if change.kind == "plugin" then
        photo:setPropertyForPlugin(_PLUGIN, change.field, change.value)
    else
        photo:setRawMetadata(change.field, change.value)
    end
end

function CatalogWrite.write(photo, changes)
    for _, change in ipairs(changes) do writeChange(photo, change) end
end

-- Writes a photo's GPS position and altitude (the second list of
-- Tagging.splitGps), inside the second catalog write. Returns whether its
-- GPS altitude was cleared (Dive Lens's own, from a depth cleared or
-- another dive's, Tagging.altitudeClears), and whether Lightroom refused to
-- clear it. When in doubt, the altitude is left: one Lightroom refuses to
-- clear stays, and the rest of the write goes on. Its marker, cleared in
-- the first write, is then missing: the next write falls back to the rule
-- for photos without a marker, which keeps it unless it matches the depth.
local function writeGps(photo, changes)
    local cleared, refused = false, false
    for _, change in ipairs(changes) do
        if Tagging.isAltitudeClear(change) then
            local ok, err = LrTasks.pcall(writeChange, photo, change)
            if ok then
                cleared = true
            else
                refused = true
                Log.warn("GPS altitude kept on photo %s: %s", tostring(photo.localIdentifier), tostring(err))
            end
        else
            writeChange(photo, change)
        end
    end
    return cleared, refused
end

-- Tells the app how far the first write is, when `progress`
-- (Tagging.progress) has something new to tell after one more photo.
function CatalogWrite.advance(s, id, total, progress)
    local done = progress.advance()
    if done then
        Send.toApp(s, { type = "progress", id = id, done = done, total = total })
        LrTasks.yield()
    end
end

-- The second catalog write of an "Apply": the GPS positions and altitudes,
-- `later` = { { photo = photo, changes = GPS changes }, ... }, under the
-- same Undo name as the first. Lightroom's Undo does not revert them when
-- they are written with a location text field (spec section 3). On macOS,
-- writes made in a row "without any intervening user interaction" form one
-- Undo step, so one Undo reverts both: the caller starts it right after the
-- first, sending and yielding nothing in between, and it never yields or
-- sends either (no progress), so Lightroom gets no chance to handle a click
-- between or during the two writes.
-- Returns { written = true, cleared = n, refused = n } (altitudes cleared,
-- and altitudes Lightroom refused to clear), or { written = false, error =
-- message, or nil when Lightroom kept the catalog busy }
-- (Tagging.secondWriteResult).
function CatalogWrite.writeGpsLater(catalog, undoName, later)
    local cleared, refused = 0, 0
    if #later == 0 then return { written = true, cleared = 0, refused = 0 } end
    local ok, status = LrTasks.pcall(function()
        return catalog:withWriteAccessDo(undoName, function()
            for _, item in ipairs(later) do
                local c, r = writeGps(item.photo, item.changes)
                if c then cleared = cleared + 1 end
                if r then refused = refused + 1 end
            end
        end, { timeout = CatalogWrite.WRITE_TIMEOUT })
    end)
    return Tagging.secondWriteResult(ok, status, cleared, refused)
end

function CatalogWrite.logSecondWrite(later, second)
    if #later == 0 then
        Log.info("no GPS or altitude to write: no second write")
    elseif second.refused == 0 then
        Log.info("GPS and altitude written on %d photos, in a second write", #later)
    else
        Log.info("second write on %d photos: GPS and altitude written on %d, altitude left on %d (Lightroom refused to clear it)",
            #later, #later - second.refused, second.refused)
    end
end

-- What the photos carry before a dive is written on them, for
-- Tagging.photoWrites: { [photo] = { RETAG_FIELDS..., gpsAltitude } },
-- the altitude marker among the RETAG_FIELDS.
function CatalogWrite.photosBefore(catalog, photos)
    if #photos == 0 then return {} end
    local fields = catalog:batchGetPropertyForPlugin(photos, _PLUGIN, Tagging.RETAG_FIELDS)
    local raw = catalog:batchGetRawMetadata(photos, { "gpsAltitude" })
    local before = {}
    for _, photo in ipairs(photos) do
        local f = fields[photo] or {}
        local current = { gpsAltitude = (raw[photo] or {}).gpsAltitude }
        for _, field in ipairs(Tagging.RETAG_FIELDS) do current[field] = f[field] end
        before[photo] = current
    end
    return before
end

return CatalogWrite
