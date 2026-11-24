--[[
The Lightroom photo selection, for the DiveLens app. The plugin pushes the
current selection whenever it changes, with its first thumbnails, and
describes it photo by photo when the app asks (Tag Selection's per-photo
values). The link task (Link.lua) calls push and sendThumbnails on its
ticks, describe on a "describeSelection" message and count on a
"countPhotos" message (the size of the whole catalog).

Part of DiveLens, GPL-3.0-or-later.
]]

local LrApplication = import "LrApplication"
local LrDate = import "LrDate"
local LrStringUtils = import "LrStringUtils"
local LrTasks = import "LrTasks"

local Json = require "Json"
local Log = require "Log"
local Send = require "Send"
local Tagging = require "Tagging"

local THUMBNAILS = 6       -- thumbnails of the selection shown by the app
local THUMBNAIL_SIZE = 160 -- pixels, minimum; Lightroom may return larger
local THUMBNAILS_PER_TICK = 2

local Selection = {}

-- The selection: photos (none when nothing is selected: getTargetPhotos alone
-- would return the whole filmstrip) and a cheap signature to notice changes.
local function selection()
    local catalog = LrApplication.activeCatalog()
    local active = catalog:getTargetPhoto()
    if not active then return {}, "" end
    local photos = catalog:getTargetPhotos()
    local signature = table.concat({
        #photos, tostring(active.localIdentifier),
        tostring(photos[1].localIdentifier), tostring(photos[#photos].localIdentifier),
    }, ":")
    return photos, signature
end

-- Photos as the app knows them: id, wall-clock original capture time (the
-- adjusted one when there is none), camera model and serial number.
function Selection.describePhotos(photos)
    local catalog = LrApplication.activeCatalog()
    local raw = catalog:batchGetRawMetadata(photos, { "dateTimeOriginalISO8601", "dateTimeISO8601" })
    local formatted = catalog:batchGetFormattedMetadata(photos, { "cameraMake", "cameraModel", "cameraSerialNumber" })
    local list = {}
    for i, photo in ipairs(photos) do
        local r, f = raw[photo] or {}, formatted[photo] or {}
        list[i] = Tagging.describePhoto(photo.localIdentifier,
            r.dateTimeOriginalISO8601 or r.dateTimeISO8601,
            f.cameraMake, f.cameraModel, f.cameraSerialNumber)
    end
    return list
end

-- The selected `photos` as described to the app (describePhotos), and as a
-- set (Tagging.photoSet). Lightroom's metadata is read once per selection:
-- kept with the photos it describes, and reused while the selection is
-- exactly those photos (for the live selection and Selection.describe).
local function describedSelection(s, photos)
    local kept = s.selectionDescribed
    if kept and Tagging.sameSelection(kept.set, photos, Tagging.photoId) then return kept.list, kept.set end
    local list = #photos > 0 and Selection.describePhotos(photos) or {}
    local set = Tagging.photoSet(photos, Tagging.photoId)
    s.selectionDescribed = { set = set, list = list }
    return list, set
end

-- Asks Lightroom for the first thumbnails. Callbacks only queue the result;
-- the link task sends it. Older requests are released (no longer wanted).
local function requestThumbnails(s, photos, version)
    s.thumbnailRequests = {}
    for i = 1, math.min(THUMBNAILS, #photos) do
        s.thumbnailRequests[i] = photos[i]:requestJpegThumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE, function(jpeg)
            if jpeg and version == s.selectionVersion then
                s.thumbnailOutbox[#s.thumbnailOutbox + 1] = { version = version, index = i, jpeg = jpeg }
            end
        end)
    end
end

function Selection.sendThumbnails(s)
    for _ = 1, THUMBNAILS_PER_TICK do
        local item = table.remove(s.thumbnailOutbox, 1)
        if not item then return end
        if item.version == s.selectionVersion then
            Send.toApp(s, {
                type = "thumbnail",
                version = item.version,
                index = item.index,
                jpeg = LrStringUtils.encodeBase64(item.jpeg),
            })
        end
    end
end

function Selection.push(s, force)
    local photos, signature = selection()
    if not force and signature == s.selectionSignature then return end
    s.selectionVersion = s.selectionVersion + 1
    s.thumbnailOutbox = {}
    local from, to, cameras = Tagging.timeRange((describedSelection(s, photos)))
    if Send.toApp(s, {
        type = "selection", count = #photos, version = s.selectionVersion, from = from, to = to,
        cameras = (cameras and #cameras > 0) and cameras or Json.emptyArray,
    }) then
        s.selectionSignature = signature
        requestThumbnails(s, photos, s.selectionVersion)
    end
end

-- Sends `list` (described photos) to the app as "photos" messages of at
-- most PHOTOS_PER_MESSAGE, the last one marked done.
local PHOTOS_PER_MESSAGE = 200

function Selection.sendPhotos(s, id, list)
    local i = 1
    repeat
        local chunk = {}
        for j = i, math.min(i + PHOTOS_PER_MESSAGE - 1, #list) do chunk[#chunk + 1] = list[j] end
        i = i + PHOTOS_PER_MESSAGE
        local done = i > #list
        Send.toApp(s, { type = "photos", id = id, photos = #chunk > 0 and chunk or Json.emptyArray, done = done })
        if not done then LrTasks.yield() end
    until done
    s.lastHeard = LrDate.currentTime()
end

-- The photos selected now, described to the app for Tag Selection's
-- per-photo values, and kept: an "apply" naming this description writes
-- only if the selection is still exactly these photos (ApplyTags.run).
function Selection.describe(s, message)
    local photos = selection()
    local list, set = describedSelection(s, photos)
    s.described = { id = message.id, set = set }
    Log.debug("selection described: %d photos", #list)
    Selection.sendPhotos(s, message.id, list)
end

-- The number of photos in the whole catalog, for the app's status bar.
function Selection.count(s, message)
    local count = #LrApplication.activeCatalog():getAllPhotos()
    Send.toApp(s, { type = "catalogSize", id = message.id, count = count })
end

return Selection
