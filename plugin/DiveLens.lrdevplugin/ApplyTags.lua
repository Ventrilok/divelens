--[[
Tag Selection's "Apply", for an "apply" message from the DiveLens app:
writes a dive, its site and each photo's own values on the photos selected
in Lightroom (ApplyTags.run). What goes on each photo is decided by
Tagging.lua; the write helpers it shares with Create Collections are in
CatalogWrite.lua.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrApplication = import "LrApplication"

local CatalogWrite = require "CatalogWrite"
local Log = require "Log"
local Send = require "Send"
local Tagging = require "Tagging"

local ApplyTags = {}

-- Tags the photos selected in Lightroom now (not when the app last looked),
-- in two catalog writes under one Undo name, the GPS and altitude in the
-- second (Tagging.splitGps, CatalogWrite.writeGpsLater), so one Undo
-- reverts it all. A photo that carried another dive also loses that dive's
-- own values (Tagging.retagChanges), read from the photo inside the first
-- write; its altitude, when cleared, in the second, and its altitude
-- marker in the first, with the other Dive Lens fields. When the request
-- names a selection (Tagging.planPhotos: per-photo values, or none with the
-- dive details unticked), it must be exactly the one described to the app
-- (Selection.describe): otherwise nothing is written.
-- The photos taken during the dive get their values, and the other dive's
-- are not cleared on them first (Tagging.notOverwritten). When the request
-- names a collection (Tagging.planCollection), the selected photos also go
-- in it, made when missing, in the first write; where it goes is looked up
-- before any photo is written (CatalogWrite.holder). Always answers the app.
function ApplyTags.run(s, message)
    local id = message.id
    s.applyStage = { id = id, stage = "before" }
    local function fail(reason, detail)
        Log.warn("tagging refused: %s %s", reason, detail or "")
        Send.toApp(s, { type = "applyFailed", id = id, reason = reason, detail = detail })
    end
    local changes, why = Tagging.plan(message, CatalogWrite.formatDate)
    if not changes then return fail("badRequest", why) end
    local perPhoto, bad = Tagging.planPhotos(message)
    if bad then return fail("badRequest", bad) end
    local place, badPlace = Tagging.planCollection(message)
    if badPlace then return fail("badRequest", badPlace) end
    local catalog = LrApplication.activeCatalog()
    -- Never the whole filmstrip: getTargetPhotos returns it when nothing is selected.
    if not catalog:getTargetPhoto() then return fail("noSelection") end
    local photos = catalog:getTargetPhotos()
    local refusal = Tagging.selectionRefusal(perPhoto, s.described, photos, Tagging.photoId)
    if refusal then return fail(refusal) end
    local total = #photos

    -- Where the collection goes, looked up before any photo is written:
    -- a missing set refuses the whole request. "existing" lists what is
    -- already there, so the answer says made or completed.
    local container
    local existing = {}
    if place then
        local holder
        holder, container = CatalogWrite.holder(catalog, place, fail)
        if not holder then return end
        for _, c in ipairs(holder:getChildCollections()) do existing[c:getName()] = true end
    end

    CatalogWrite.apply(s, id, {
        catalog = catalog,
        undoName = LOC "$$$/DiveLens/Undo/TagSelection=Tag Selection (Dive Lens)",
        total = total,
        fail = fail,
        before = function(cat)
            return Tagging.writesDive(changes) and CatalogWrite.photosBefore(cat, photos) or {}
        end,
        writeFirst = function(cat, before, advance)
            local planned = Tagging.tagWrites(changes, photos, before, perPhoto, Tagging.photoId)
            for _, w in ipairs(planned.writes) do
                CatalogWrite.write(w.photo, w.changes)
                advance()
            end
            if place then
                -- All the selected photos: Lightroom ignores those already in it.
                cat:createCollection(place.name, container, true):addPhotos(photos)
            end
            return { later = planned.later, cleared = planned.cleared, withValues = planned.withValues }
        end,
        done = function()
            return {
                type = "applied",
                id = id,
                count = total,
                collection = place and Tagging.selectionCollection(place.name, total, existing) or nil,
            }
        end,
        log = function(result, second)
            local later = result.later
            if not second.written then
                -- The first write stays: dive data and location text, without GPS.
                Log.warn("tagged %d photos but not the GPS and altitude of %d: %s", total, #later, second.error or "Lightroom busy")
            else
                CatalogWrite.logSecondWrite(later, second)
                Log.info("tagged %d photos, per-photo values on %d; another dive's values cleared or replaced on %d (GPS altitude cleared on %d)",
                    total, result.withValues, result.cleared, second.cleared)
            end
        end,
    })
end

return ApplyTags
