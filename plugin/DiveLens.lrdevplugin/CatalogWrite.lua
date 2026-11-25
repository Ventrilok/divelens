--[[
What the selection path (ApplyTags.lua) and the catalog path
(ApplyCollections.lua) share to write in the catalog: the one Undo name of
their writes, Lightroom's date format, one photo's changes, what the photos
carry before a write, the progress messages to the app, the envelope of an
"Apply" (the first catalog write and the one answer to the app) and its
second catalog write, the GPS positions and altitudes. What goes on each
photo is decided by Tagging.lua.

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

-- The Undo name of both writes of an "Apply", whichever the feature:
-- Lightroom's Edit menu then reads "Undo Dive Lens" (spec section 3). The
-- app's texts quote it (tests/apply_contract.rs).
CatalogWrite.UNDO_NAME = LOC "$$$/DiveLens/Undo/DiveLens=Dive Lens"

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

-- The envelope both features write through, for the "apply" `id`: the first
-- catalog write, the second one right after it, and the one answer to the
-- app. It keeps `s.applyStage` ("before" is set by the feature at the very
-- start of its run; "afterFirst" and "replied" here), which Link.lua's
-- protected handler reads when an unexpected error ends the request
-- (Tagging.crashReply). `job`:
--   catalog    the catalog the feature already works on
--   undoName   the Undo name of both writes (CatalogWrite.UNDO_NAME)
--   total      photos requested, found or not, for the progress messages
--   fail       function(reason, detail): the feature's refusal (logs, then
--              tells the app "applyFailed"), used when the first write fails or
--              is busy (a second-write failure goes out as "gpsNotWritten")
--   before     optional function(catalog) -> what the photos carry before,
--              read inside the first write; absent = nothing is read
--   writeFirst function(catalog, before, advance) -> result: the first write's
--              body; calls advance() once per step, and returns
--              { later = the GPS changes for the second write, ... } plus
--              whatever done and log need
--   done       function(result) -> the message when everything was written
--   log        function(result, second): the log lines once the app has its
--              answer; second.written is false when the GPS did not go
function CatalogWrite.apply(s, id, job)
    local catalog = job.catalog
    local progress = Tagging.progress(job.total, CatalogWrite.PROGRESS_EVERY)
    local function advance() CatalogWrite.advance(s, id, job.total, progress) end
    local result
    local ok, status = LrTasks.pcall(function()
        return catalog:withWriteAccessDo(job.undoName, function()
            local before = job.before and job.before(catalog) or {}
            result = job.writeFirst(catalog, before, advance)
        end, { timeout = CatalogWrite.WRITE_TIMEOUT })
    end)
    -- Straight on to the second write: nothing sent or yielded in between.
    if ok and status == "executed" then s.applyStage.stage = "afterFirst" end
    local second = ok and status == "executed" and CatalogWrite.writeGpsLater(catalog, job.undoName, result.later)
    s.lastHeard = LrDate.currentTime() -- the app's pings waited during the writes
    local reply = Tagging.applyReply(id, { ok = ok, status = status, second = second },
        ok and status == "executed" and job.done(result))
    if reply.reason == "error" or reply.reason == "busy" then return job.fail(reply.reason, reply.detail) end
    Send.toApp(s, reply)
    s.applyStage.stage = "replied"
    job.log(result, second)
end

-- Where a collection goes, looked up (and its group set created when
-- needed) before the write: `place` = { parent = set names from the root,
-- group = set name or nil } (Tagging.planCollections, planCollection).
-- Returns `holder` (what lists the collections already there) and
-- `container` (what Lightroom's create functions take: nil for the
-- catalog's root), or nil after calling `fail(reason, detail)`.
function CatalogWrite.holder(catalog, place, fail)
    local holder = Tagging.findSet(catalog, place.parent)
    if not holder then
        fail("parentNotFound", table.concat(place.parent, " › "))
        return nil
    end
    local container = #place.parent > 0 and holder or nil

    -- The group set is created on its own the first time: a set created
    -- inside a write cannot be used before that write ends.
    if place.group then
        local set = Tagging.findChild(holder:getChildCollectionSets(), place.group)
        if not set then
            local ok, status = LrTasks.pcall(function()
                return catalog:withWriteAccessDo(place.group, function()
                    catalog:createCollectionSet(place.group, container, true)
                end, { timeout = CatalogWrite.WRITE_TIMEOUT })
            end)
            local reason, detail = Tagging.writeFailure(ok, status)
            if reason then
                fail(reason, detail)
                return nil
            end
            set = Tagging.findChild(holder:getChildCollectionSets(), place.group)
            if not set then
                fail("error", "collection set not created")
                return nil
            end
        end
        holder, container = set, set
    end
    return holder, container
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
