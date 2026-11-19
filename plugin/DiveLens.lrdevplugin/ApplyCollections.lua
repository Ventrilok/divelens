--[[
Create Collections, for the DiveLens app: finds the photos captured between
two days (find), writes each dive's collection, its data and location on
its photos, and each photo's own values (run), and lists the catalog's
collection sets for the app's settings (listSets). What goes on each photo
is decided by Tagging.lua; the write helpers it shares with Tag Selection
are in CatalogWrite.lua.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrApplication = import "LrApplication"
local LrDate = import "LrDate"
local LrTasks = import "LrTasks"

local CatalogWrite = require "CatalogWrite"
local Json = require "Json"
local Log = require "Log"
local Selection = require "Selection"
local Send = require "Send"
local Tagging = require "Tagging"

local ApplyCollections = {}

-- Every photo captured between two days (inclusive, "YYYY-MM-DD"), whole
-- catalog. Lightroom searches by day only and on the adjusted capture time;
-- the app keeps what fits on the original time. The photos are kept for the
-- following "apply", which names them by id.
function ApplyCollections.find(s, message)
    local id = message.id
    if type(message.from) ~= "string" or type(message.to) ~= "string" then
        return Send.toApp(s, { type = "applyFailed", id = id, reason = "badRequest" })
    end
    local catalog = LrApplication.activeCatalog()
    local photos = catalog:findPhotos {
        sort = "captureTime",
        ascending = true,
        searchDesc = {
            { criteria = "captureTime", operation = "in", value = message.from, value2 = message.to },
            combine = "intersect",
        },
    }
    local byId = {}
    for _, photo in ipairs(photos) do byId[tostring(photo.localIdentifier)] = photo end
    s.search = { id = id, photos = byId }
    local list = Selection.describePhotos(photos)
    local undated = 0
    for _, p in ipairs(list) do
        if not p.time then undated = undated + 1 end
    end
    Log.info("found %d photos from %s to %s (%d without capture time)", #list, message.from, message.to, undated)
    Selection.sendPhotos(s, id, list)
end

-- Writes a "Create Collections" request: each dive's collection where the
-- app's settings say (by default in a "Dive Lens" set at the root, created
-- if needed), its data and location on its photos, and each photo's own
-- values. Two catalog writes under one Undo name, the GPS and altitude in
-- the second (Tagging.splitGps, CatalogWrite.writeGpsLater), so one Undo
-- reverts it all. What each photo carries is read inside the first write,
-- as Tag Selection does: an altitude Dive Lens wrote from a depth now
-- cleared, or from another dive's, is cleared in the second write
-- (Tagging.photoWrites). Always answers the app.
function ApplyCollections.run(s, message)
    local id = message.id
    local function fail(reason, detail)
        Log.warn("collections refused: %s %s", reason, detail or "")
        Send.toApp(s, { type = "applyFailed", id = id, reason = reason, detail = detail })
    end
    local groups, place = Tagging.planCollections(message, CatalogWrite.formatDate)
    if not groups then return fail("badRequest", place) end
    if not (s.search and s.search.id == message.search) then return fail("staleSearch") end
    local catalog = LrApplication.activeCatalog()

    -- "holder" lists what is already there; "container" is what Lightroom's
    -- create functions take (nil for the catalog's root).
    local holder = Tagging.findSet(catalog, place.parent)
    if not holder then return fail("parentNotFound", table.concat(place.parent, " › ")) end
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
            if reason then return fail(reason, detail) end
            set = Tagging.findChild(holder:getChildCollectionSets(), place.group)
            if not set then return fail("error", "collection set not created") end
        end
        holder, container = set, set
    end
    local existing = {}
    for _, c in ipairs(holder:getChildCollections()) do existing[c:getName()] = true end

    -- The photos found, in the catalog.
    local total, found = Tagging.resolvePhotos(groups, s.search.photos)
    local undoName = LOC "$$$/DiveLens/Undo/CreateCollections=Create Collections (Dive Lens)"
    local progress = Tagging.progress(total, CatalogWrite.PROGRESS_EVERY)
    local results, later = {}, {}
    local ok, status = LrTasks.pcall(function()
        return catalog:withWriteAccessDo(undoName, function()
            local before = CatalogWrite.photosBefore(catalog, found)
            local planned = Tagging.collectionWrites(groups, before, existing)
            results, later = planned.results, planned.later
            for gi, g in ipairs(groups) do
                local group = planned.groups[gi]
                -- One step per photo requested: written when found.
                for _, step in ipairs(group.steps) do
                    if step.photo then CatalogWrite.write(step.photo, step.changes) end
                    CatalogWrite.advance(s, id, total, progress)
                end
                if #group.photos > 0 then
                    local collection = catalog:createCollection(g.collection, container, true)
                    collection:addPhotos(group.photos)
                end
            end
        end, { timeout = CatalogWrite.WRITE_TIMEOUT })
    end)
    -- Straight on to the second write: nothing sent or yielded in between.
    local second = ok and status == "executed" and CatalogWrite.writeGpsLater(catalog, undoName, later)
    s.lastHeard = LrDate.currentTime()
    local reply = Tagging.applyReply(id, { ok = ok, status = status, second = second },
        { type = "collectionsApplied", id = id, groups = results })
    if reply.reason == "error" or reply.reason == "busy" then return fail(reply.reason, reply.detail) end
    if reply.reason == "gpsNotWritten" then
        -- The first write stays: collections, dive data and location text.
        Log.warn("collections written but not the GPS and altitude of %d photos: %s", #later, second.error or "Lightroom busy")
    else
        CatalogWrite.logSecondWrite(later, second)
        Log.info("collections written: %d groups, %d photos (GPS altitude cleared on %d)", #results, total, second.cleared)
    end
    Send.toApp(s, reply)
end

-- The catalog's collection sets, for the app's settings.
function ApplyCollections.listSets(s, message)
    local sets = Tagging.describeSets(LrApplication.activeCatalog(), Json.emptyArray)
    Send.toApp(s, { type = "sets", id = message.id, sets = sets })
end

return ApplyCollections
