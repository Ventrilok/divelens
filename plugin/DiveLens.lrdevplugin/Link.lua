--[[
Local link with the DiveLens app.

Lightroom plugins can only listen on a local socket, one direction per socket
(LrSocket), so the plugin opens two and the app connects to both:
  - "receive": messages from the app to the plugin;
  - "send":    messages from the plugin to the app.
Ports are chosen by the system and published, with a one-time token, in a
small "link file" in the app's settings folder. The app reads it, connects,
and says hello with the token; nothing is answered before that.

Each message is one JSON object on one line. The plugin pushes the current
photo selection whenever it changes, and describes it photo by photo when
the app asks (Tag Selection's per-photo values). Its hello carries the previous plugin's
settings (same plug-in identifier, so the same preferences) for the app to
take over once. Messages are handled in the link's own
task, never inside socket callbacks, so they may use the catalog.

When the app goes away, both sockets are opened again for the next app
launch. The link stays up as long as Lightroom runs.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrApplication = import "LrApplication"
local LrDate = import "LrDate"
local LrFileUtils = import "LrFileUtils"
local LrFunctionContext = import "LrFunctionContext"
local LrMD5 = import "LrMD5"
local LrPathUtils = import "LrPathUtils"
local LrPrefs = import "LrPrefs"
local LrSocket = import "LrSocket"
local LrStringUtils = import "LrStringUtils"
local LrTasks = import "LrTasks"

local Framing = require "Framing"
local Json = require "Json"
local Log = require "Log"
local Tagging = require "Tagging"

local PROTOCOL = 6
local APP_ID = "com.querton.divelens.desktop" -- the app's bundle identifier
local LINK_FILE = "lightroom-link.json"
local TICK = 0.1           -- seconds between two passes of the link task
local SELECTION_EVERY = 5  -- selection checked every 5 ticks while the app is connected
local PENDING_SCREEN_FOR = 30 -- seconds a menu choice waits for a just-launched app
local SILENCE = 8          -- seconds without news before the app is considered gone
local THUMBNAILS = 6       -- thumbnails of the selection shown by the app
local THUMBNAIL_SIZE = 160 -- pixels, minimum; Lightroom may return larger
local THUMBNAILS_PER_TICK = 2

local Link = {}

-- The link state lives in the plugin's global table so the menu scripts, the
-- Plug-in Manager and the shutdown script all see the same link.
local function current()
    return _G.DiveLensLink
end

-- The full version, pre-release label included (Info.lua's `display`).
local function pluginVersion()
    return require "Info".VERSION.display
end

-- Link file -------------------------------------------------------------------

--- Folder where the app keeps its settings (Tauri's app config folder).
function Link.appSettingsFolder()
    if WIN_ENV then
        local appData = os.getenv and os.getenv("APPDATA")
        if not appData or appData == "" then
            appData = LrPathUtils.child(LrPathUtils.child(LrPathUtils.getStandardFilePath("home"), "AppData"), "Roaming")
        end
        return LrPathUtils.child(appData, APP_ID)
    end
    local support = LrPathUtils.child(LrPathUtils.child(LrPathUtils.getStandardFilePath("home"), "Library"), "Application Support")
    return LrPathUtils.child(support, APP_ID)
end

local function linkFilePath()
    return LrPathUtils.child(Link.appSettingsFolder(), LINK_FILE)
end

local function writeLinkFile(s)
    if not (s.ports.receive and s.ports.send) then return end
    local content = Json.encode {
        protocol = PROTOCOL,
        receivePort = s.ports.receive,
        sendPort = s.ports.send,
        token = s.token,
        plugin = pluginVersion(),
    }
    if content == s.linkFileContent then return end
    local folder = Link.appSettingsFolder()
    LrFileUtils.createAllDirectories(folder)
    local file, err = io.open(linkFilePath(), "w")
    if not file then
        Log.error("cannot write the link file %s: %s", linkFilePath(), err)
        return
    end
    file:write(content)
    file:close()
    s.linkFileContent = content
    Log.info("link ready: receive port %s, send port %s", s.ports.receive, s.ports.send)
end

-- Removes the link file if it is still ours: after a plugin reload, the new
-- link may already have written its own.
local function removeLinkFile(s)
    local path = linkFilePath()
    local file = io.open(path, "r")
    if not file then return end
    local content = file:read("*a")
    file:close()
    if content == s.linkFileContent then LrFileUtils.delete(path) end
end

local function makeToken()
    math.randomseed(os.time())
    local seed = table.concat({ tostring(LrDate.currentTime()), tostring(math.random()), tostring(math.random()), _PLUGIN.path }, "|")
    return LrMD5.digest(seed)
end

-- Sending ---------------------------------------------------------------------

local function send(s, message)
    if not (s.sender and s.senderConnected) then return false end
    local ok, err = LrTasks.pcall(function() s.sender:send(Json.encode(message) .. "\n") end)
    if not ok then
        Log.warn("send failed: %s", err)
        return false
    end
    return true
end

local function catalogName()
    local path = LrApplication.activeCatalog():getPath()
    return LrPathUtils.removeExtension(LrPathUtils.leafName(path))
end

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
local function describePhotos(photos)
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

-- Earliest and latest capture times of the selection, wall-clock
-- "YYYY-MM-DDTHH:MM:SS" (or nil), overall and per camera (the app applies
-- each camera's remembered clock offset).
local function timeRange(photos)
    if #photos == 0 then return nil end
    local first, last
    local cameras, order = {}, {}
    for _, p in ipairs(describePhotos(photos)) do
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

local function sendThumbnails(s)
    for _ = 1, THUMBNAILS_PER_TICK do
        local item = table.remove(s.thumbnailOutbox, 1)
        if not item then return end
        if item.version == s.selectionVersion then
            send(s, {
                type = "thumbnail",
                version = item.version,
                index = item.index,
                jpeg = LrStringUtils.encodeBase64(item.jpeg),
            })
        end
    end
end

local function pushSelection(s, force)
    local photos, signature = selection()
    if not force and signature == s.selectionSignature then return end
    s.selectionVersion = s.selectionVersion + 1
    s.thumbnailOutbox = {}
    local from, to, cameras = timeRange(photos)
    if send(s, {
        type = "selection", count = #photos, version = s.selectionVersion, from = from, to = to,
        cameras = (cameras and #cameras > 0) and cameras or Json.emptyArray,
    }) then
        s.selectionSignature = signature
        requestThumbnails(s, photos, s.selectionVersion)
    end
end

-- Writing ---------------------------------------------------------------------

local WRITE_TIMEOUT = 10 -- seconds to wait for Lightroom's write access
local PROGRESS_EVERY = 50 -- photos between two progress messages

-- Lightroom's own date format for StartDate and EndDate, exactly as the
-- previous plugin wrote them (the time zone does not matter: the same one
-- is used both ways, so the wall-clock time is kept).
local function formatDate(iso)
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

local function write(photo, changes)
    for _, change in ipairs(changes) do writeChange(photo, change) end
end

-- Writes a photo's GPS position and altitude (the second list of
-- Tagging.splitGps), inside the second catalog write. Returns whether its
-- GPS altitude was cleared (a photo that carried another dive,
-- Tagging.retagChanges), and whether Lightroom refused to clear it. When in
-- doubt, the altitude is left: one Lightroom refuses to clear stays, and
-- the rest of the write goes on.
local function writeGps(photo, changes)
    local cleared, refused = false, false
    for _, change in ipairs(changes) do
        if change.field == "gpsAltitude" and change.value == nil then
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
local function advance(s, id, total, progress)
    local done = progress.advance()
    if done then
        send(s, { type = "progress", id = id, done = done, total = total })
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
-- message, or nil when Lightroom kept the catalog busy }.
local function writeGpsLater(catalog, undoName, later)
    local cleared, refused = 0, 0
    if #later == 0 then return { written = true, cleared = 0, refused = 0 } end
    local ok, status = LrTasks.pcall(function()
        return catalog:withWriteAccessDo(undoName, function()
            for _, item in ipairs(later) do
                local c, r = writeGps(item.photo, item.changes)
                if c then cleared = cleared + 1 end
                if r then refused = refused + 1 end
            end
        end, { timeout = WRITE_TIMEOUT })
    end)
    if not ok then return { written = false, error = tostring(status) } end
    if status ~= "executed" then return { written = false } end
    return { written = true, cleared = cleared, refused = refused }
end

local function logSecondWrite(later, second)
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
-- Tagging.retagChanges: { [photo] = { RETAG_FIELDS..., gpsAltitude } }.
local function photosBefore(catalog, photos)
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

local function photoId(photo)
    return tostring(photo.localIdentifier)
end

-- Tags the photos selected in Lightroom now (not when the app last looked),
-- in two catalog writes under one Undo name, the GPS and altitude in the
-- second (Tagging.splitGps, writeGpsLater), so one Undo reverts it all. A
-- photo that carried another dive also loses that dive's own values
-- (Tagging.retagChanges), read from the photo inside the first write; its
-- altitude, when cleared, in the second. With per-photo values
-- (Tagging.planPhotos), the selection must be exactly the one described to
-- the app (describeSelection): otherwise nothing is written. The photos
-- taken during the dive get their values, and the other dive's are not
-- cleared on them first (Tagging.notOverwritten). Always answers the app.
local function applyTags(s, message)
    local id = message.id
    local function fail(reason, detail)
        Log.warn("tagging refused: %s %s", reason, detail or "")
        send(s, { type = "applyFailed", id = id, reason = reason, detail = detail })
    end
    local changes, why = Tagging.plan(message, formatDate)
    if not changes then return fail("badRequest", why) end
    local perPhoto, bad = Tagging.planPhotos(message)
    if bad then return fail("badRequest", bad) end
    local catalog = LrApplication.activeCatalog()
    -- Never the whole filmstrip: getTargetPhotos returns it when nothing is selected.
    if not catalog:getTargetPhoto() then return fail("noSelection") end
    local photos = catalog:getTargetPhotos()
    if perPhoto then
        local described = s.described
        if not (described and described.id == perPhoto.selection) then return fail("staleSelection") end
        if not Tagging.sameSelection(described, photos, photoId) then return fail("selectionChanged") end
    end
    local total = #photos
    local undoName = LOC "$$$/DiveLens/Undo/TagSelection=Tag Selection (Dive Lens)"
    local progress = Tagging.progress(total, PROGRESS_EVERY)
    local cleared, withValues, later = 0, 0, {}
    local ok, status = LrTasks.pcall(function()
        return catalog:withWriteAccessDo(undoName, function()
            local before = Tagging.writesDive(changes) and photosBefore(catalog, photos) or {}
            for _, photo in ipairs(photos) do
                local extra = before[photo] and Tagging.retagChanges(before[photo], changes) or {}
                if #extra > 0 then cleared = cleared + 1 end
                local own = perPhoto and perPhoto.photos[photoId(photo)]
                if own then
                    withValues = withValues + 1
                    extra = Tagging.notOverwritten(extra, own)
                end
                local now, afterwards = Tagging.splitGps(changes, extra, own or {})
                write(photo, now)
                if #afterwards > 0 then later[#later + 1] = { photo = photo, changes = afterwards } end
                advance(s, id, total, progress)
            end
        end, { timeout = WRITE_TIMEOUT })
    end)
    -- Straight on to the second write: nothing sent or yielded in between.
    local second = ok and status == "executed" and writeGpsLater(catalog, undoName, later)
    s.lastHeard = LrDate.currentTime() -- the app's pings waited during the writes
    if not ok then return fail("error", tostring(status)) end
    if status ~= "executed" then return fail("busy") end
    if not second.written then
        -- The first write stays: dive data and location text, without GPS.
        Log.warn("tagged %d photos but not the GPS and altitude of %d: %s", total, #later, second.error or "Lightroom busy")
        send(s, { type = "applyFailed", id = id, reason = "gpsNotWritten", detail = second.error })
        return
    end
    logSecondWrite(later, second)
    Log.info("tagged %d photos, per-photo values on %d; another dive's values cleared or replaced on %d (GPS altitude cleared on %d)",
        total, withValues, cleared, second.cleared)
    send(s, { type = "applied", id = id, count = total })
end

-- Sends `list` (described photos) to the app as "photos" messages of at
-- most PHOTOS_PER_MESSAGE, the last one marked done.
local PHOTOS_PER_MESSAGE = 200

local function sendPhotos(s, id, list)
    local i = 1
    repeat
        local chunk = {}
        for j = i, math.min(i + PHOTOS_PER_MESSAGE - 1, #list) do chunk[#chunk + 1] = list[j] end
        i = i + PHOTOS_PER_MESSAGE
        local done = i > #list
        send(s, { type = "photos", id = id, photos = #chunk > 0 and chunk or Json.emptyArray, done = done })
        if not done then LrTasks.yield() end
    until done
    s.lastHeard = LrDate.currentTime()
end

-- The photos selected now, described to the app for Tag Selection's
-- per-photo values, and kept: an "apply" naming this description writes
-- only if the selection is still exactly these photos (applyTags).
local function describeSelection(s, message)
    local photos = selection()
    local byId = {}
    for _, photo in ipairs(photos) do byId[photoId(photo)] = photo end
    s.described = { id = message.id, count = #photos, photos = byId }
    local list = #photos > 0 and describePhotos(photos) or {}
    Log.debug("selection described: %d photos", #list)
    sendPhotos(s, message.id, list)
end

-- Create Collections ----------------------------------------------------------

-- Every photo captured between two days (inclusive, "YYYY-MM-DD"), whole
-- catalog. Lightroom searches by day only and on the adjusted capture time;
-- the app keeps what fits on the original time. The photos are kept for the
-- following "apply", which names them by id.
local function findPhotos(s, message)
    local id = message.id
    if type(message.from) ~= "string" or type(message.to) ~= "string" then
        return send(s, { type = "applyFailed", id = id, reason = "badRequest" })
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
    local list = describePhotos(photos)
    local undated = 0
    for _, p in ipairs(list) do
        if not p.time then undated = undated + 1 end
    end
    Log.info("found %d photos from %s to %s (%d without capture time)", #list, message.from, message.to, undated)
    sendPhotos(s, id, list)
end

local function findChild(list, name)
    for _, item in ipairs(list) do
        if item:getName() == name then return item end
    end
end

-- Writes a "Create Collections" request: each dive's collection where the
-- app's settings say (by default in a "Dive Lens" set at the root, created
-- if needed), its data and location on its photos, and each photo's own
-- values. Two catalog writes under one Undo name, the GPS and altitude in
-- the second (Tagging.splitGps, writeGpsLater), so one Undo reverts it all.
-- Always answers the app.
local function applyCollections(s, message)
    local id = message.id
    local function fail(reason, detail)
        Log.warn("collections refused: %s %s", reason, detail or "")
        send(s, { type = "applyFailed", id = id, reason = reason, detail = detail })
    end
    local groups, place = Tagging.planCollections(message, formatDate)
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
        local set = findChild(holder:getChildCollectionSets(), place.group)
        if not set then
            local ok, status = LrTasks.pcall(function()
                return catalog:withWriteAccessDo(place.group, function()
                    catalog:createCollectionSet(place.group, container, true)
                end, { timeout = WRITE_TIMEOUT })
            end)
            if not ok then return fail("error", tostring(status)) end
            if status ~= "executed" then return fail("busy") end
            set = findChild(holder:getChildCollectionSets(), place.group)
            if not set then return fail("error", "collection set not created") end
        end
        holder, container = set, set
    end
    local existing = {}
    for _, c in ipairs(holder:getChildCollections()) do existing[c:getName()] = true end

    -- Each photo found: its changes for the first write, and its GPS
    -- position and altitude for the second (Tagging.splitGps).
    local total, later = 0, {}
    for _, g in ipairs(groups) do
        total = total + #g.photos
        for _, p in ipairs(g.photos) do
            p.photo = s.search.photos[p.id]
            if p.photo then
                local gps
                p.now, gps = Tagging.splitGps(g.common, p.changes)
                if #gps > 0 then later[#later + 1] = { photo = p.photo, changes = gps } end
            end
        end
    end
    local undoName = LOC "$$$/DiveLens/Undo/CreateCollections=Create Collections (Dive Lens)"
    local progress = Tagging.progress(total, PROGRESS_EVERY)
    local results = {}
    local ok, status = LrTasks.pcall(function()
        return catalog:withWriteAccessDo(undoName, function()
            for gi, g in ipairs(groups) do
                local photos = {}
                for _, p in ipairs(g.photos) do
                    if p.photo then
                        write(p.photo, p.now)
                        photos[#photos + 1] = p.photo
                    end
                    advance(s, id, total, progress)
                end
                if #photos > 0 then
                    local collection = catalog:createCollection(g.collection, container, true)
                    collection:addPhotos(photos)
                end
                results[gi] = { collection = g.collection, count = #photos, created = not existing[g.collection] }
            end
        end, { timeout = WRITE_TIMEOUT })
    end)
    -- Straight on to the second write: nothing sent or yielded in between.
    local second = ok and status == "executed" and writeGpsLater(catalog, undoName, later)
    s.lastHeard = LrDate.currentTime()
    if not ok then return fail("error", tostring(status)) end
    if status ~= "executed" then return fail("busy") end
    if not second.written then
        -- The first write stays: collections, dive data and location text.
        Log.warn("collections written but not the GPS and altitude of %d photos: %s", #later, second.error or "Lightroom busy")
        send(s, { type = "applyFailed", id = id, reason = "gpsNotWritten", detail = second.error })
        return
    end
    logSecondWrite(later, second)
    Log.info("collections written: %d groups, %d photos", #results, total)
    send(s, { type = "collectionsApplied", id = id, groups = results })
end

-- The catalog's collection sets, for the app's settings.
local function listSets(s, message)
    local sets = Tagging.describeSets(LrApplication.activeCatalog(), Json.emptyArray)
    send(s, { type = "sets", id = message.id, sets = sets })
end

-- Settings of the previous plugin, which had the same identifier (so the
-- same preferences), for the app to take over once; nil when there are none.
local function legacySettings()
    local prefs = LrPrefs.prefsForPlugin()
    local path, order = prefs.MacDivePath, prefs.DisplayLogOrder
    if type(path) ~= "string" then path = nil end
    if type(order) ~= "string" then order = nil end
    if not (path or order) then return nil end
    return { macDivePath = path, displayLogOrder = order }
end

-- Receiving -------------------------------------------------------------------

local function isWholeMessage(text)
    return (pcall(Json.decode, text))
end

local function handle(s, text)
    local ok, message = pcall(Json.decode, text)
    if not ok or type(message) ~= "table" then
        Log.warn("ignored a malformed message")
        return
    end
    if message.type == "hello" then
        if message.token ~= s.token then
            Log.warn("app hello refused: wrong token")
            send(s, { type = "error", reason = "token" })
            return
        end
        s.authed = true
        Log.info("app %s connected", tostring(message.app))
        send(s, {
            type = "hello",
            protocol = PROTOCOL,
            plugin = pluginVersion(),
            lightroom = LrApplication.versionString(),
            catalog = catalogName(),
            legacy = legacySettings(),
        })
        pushSelection(s, true)
        local pending = s.pendingScreen
        s.pendingScreen = nil
        if pending and LrDate.currentTime() - pending.at < PENDING_SCREEN_FOR then
            send(s, { type = "show", screen = pending.screen })
        end
        return
    end
    if not s.authed then return end
    if message.type == "ping" then
        send(s, { type = "pong" })
    elseif message.type == "logLevel" then
        if Log.setLevel(message.level) then Log.info("log level: %s", tostring(message.level)) end
    elseif message.type == "apply" then
        applyTags(s, message)
    elseif message.type == "findPhotos" or message.type == "applyCollections" or message.type == "listSets"
        or message.type == "describeSelection" then
        -- The app waits for an answer: an error still gets one.
        local handlers = {
            findPhotos = findPhotos, applyCollections = applyCollections, listSets = listSets,
            describeSelection = describeSelection,
        }
        local handler = handlers[message.type]
        local ok, err = LrTasks.pcall(handler, s, message)
        if not ok then
            Log.error("%s failed: %s", message.type, err)
            s.lastHeard = LrDate.currentTime()
            send(s, { type = "applyFailed", id = message.id, reason = "error", detail = tostring(err) })
        end
    else
        Log.debug("ignored message type %s", tostring(message.type))
    end
end

-- Sockets ---------------------------------------------------------------------

-- Socket callbacks never touch the sockets themselves: they only ask for a
-- reset, which the link task performs. A reset closes both sockets and opens
-- two fresh ones, so the next app launch can connect. Callbacks of closed
-- sockets are ignored thanks to the generation number.

local function requestReset(s, generation, reason)
    if s.running and generation == s.generation and not s.resetReason then
        s.resetReason = reason
    end
end

local function closeSockets(s)
    s.generation = s.generation + 1
    for _, name in ipairs { "sender", "receiver" } do
        local socket = s[name]
        s[name] = nil
        if socket then pcall(function() socket:close() end) end
    end
    s.senderConnected = false
    s.receiverConnected = false
    s.authed = false
    s.selectionSignature = nil
    s.search = nil -- photo ids belong to one app session
    s.described = nil
    s.thumbnailRequests = {}
    s.thumbnailOutbox = {}
    s.framing = Framing.new()
    s.inbox = {}
    s.ports = {}
end

local function bindSockets(s)
    local generation = s.generation
    local function live() return s.running and generation == s.generation end
    local function onError(socket, err, which)
        if not live() then return end
        if err == "timeout" then
            socket:reconnect() -- nobody connected yet: keep waiting
        else
            requestReset(s, generation, which .. " socket: " .. tostring(err))
        end
    end

    s.receiver = LrSocket.bind {
        functionContext = s.context,
        plugin = _PLUGIN,
        address = "localhost",
        port = 0,
        mode = "receive",
        onConnecting = function(_, port)
            if not live() then return end
            s.ports.receive = port
            writeLinkFile(s)
        end,
        onConnected = function()
            if not live() then return end
            s.receiverConnected = true
            s.lastHeard = LrDate.currentTime()
        end,
        onMessage = function(_, chunk)
            if not live() or type(chunk) ~= "string" then return end
            s.lastHeard = LrDate.currentTime()
            for _, text in ipairs(Framing.push(s.framing, chunk, isWholeMessage)) do
                s.inbox[#s.inbox + 1] = text
            end
        end,
        onClosed = function()
            requestReset(s, generation, "app closed its link")
        end,
        onError = function(socket, err) onError(socket, err, "receive") end,
    }

    s.sender = LrSocket.bind {
        functionContext = s.context,
        plugin = _PLUGIN,
        address = "localhost",
        port = 0,
        mode = "send",
        onConnecting = function(_, port)
            if not live() then return end
            s.ports.send = port
            writeLinkFile(s)
        end,
        onConnected = function()
            if not live() then return end
            s.senderConnected = true
            s.lastHeard = LrDate.currentTime()
        end,
        onMessage = function() end, -- never used: the app writes on the other socket
        onClosed = function()
            requestReset(s, generation, "app closed its link")
        end,
        onError = function(socket, err) onError(socket, err, "send") end,
    }
end

-- The app pings every 2 seconds. A connected socket that stays silent means
-- the app is gone even if Lightroom did not report it (or connected to one
-- socket only): start afresh.
local function checkSilence(s)
    if (s.senderConnected or s.receiverConnected)
        and LrDate.currentTime() - (s.lastHeard or 0) > SILENCE then
        requestReset(s, s.generation, "no news from the app")
    end
end

-- Link task -------------------------------------------------------------------

local function run(s)
    LrFunctionContext.callWithContext("DiveLens link", function(context)
        s.context = context
        Log.info("DiveLens plugin %s, Lightroom %s, link file %s", pluginVersion(), LrApplication.versionString(), linkFilePath())
        Log.debug("Lightroom appData folder: %s", LrPathUtils.getStandardFilePath("appData"))
        bindSockets(s)
        local tick = 0
        while s.running do
            if s.resetReason then
                Log.info("link reset: %s", s.resetReason)
                s.resetReason = nil
                closeSockets(s)
                bindSockets(s)
            end
            local inbox = s.inbox
            s.inbox = {}
            for _, text in ipairs(inbox) do
                local ok, err = LrTasks.pcall(handle, s, text)
                if not ok then Log.error("message failed: %s", err) end
            end
            tick = tick + 1
            if s.authed and tick % SELECTION_EVERY == 0 then
                local ok, err = LrTasks.pcall(pushSelection, s, false)
                if not ok then Log.error("selection failed: %s", err) end
            end
            if s.authed then sendThumbnails(s) end
            checkSilence(s)
            LrTasks.sleep(TICK)
        end
        send(s, { type = "bye" })
        closeSockets(s)
        removeLinkFile(s)
        Log.info("link closed")
    end)
end

-- Public ----------------------------------------------------------------------

--- Starts the link (does nothing if it already runs).
function Link.start()
    local old = current()
    if old and old.running then return end
    local s = {
        running = true,
        stopped = false,
        token = makeToken(),
        ports = {},
        inbox = {},
        framing = Framing.new(),
        authed = false,
        generation = 0,
        selectionVersion = 0,
        thumbnailRequests = {},
        thumbnailOutbox = {},
    }
    _G.DiveLensLink = s
    LrTasks.startAsyncTask(function()
        local ok, err = LrTasks.pcall(run, s)
        if not ok then Log.error("link stopped on error: %s", err) end
        s.running = false
        s.stopped = true
    end, "DiveLens link")
end

--- Asks the link to stop; it says goodbye to the app and removes the link file.
function Link.stop()
    local s = current()
    if s then s.running = false end
end

function Link.isStopped()
    local s = current()
    return not s or s.stopped
end

--- True when the app is connected and has said hello.
function Link.isConnected()
    local s = current()
    return s ~= nil and s.running and s.authed
end

--- Asks the connected app to come to the front on `screen`. Returns false when
--- no app is connected. Used from tasks other than the link's.
function Link.show(screen)
    local s = current()
    if not (s and s.running and s.authed) then return false end
    return send(s, { type = "show", screen = screen })
end

--- Remembers a screen to show as soon as an app connects (the app was just
--- launched and may not have connected yet).
function Link.showWhenConnected(screen)
    local s = current()
    if s then s.pendingScreen = { screen = screen, at = LrDate.currentTime() } end
end

return Link
