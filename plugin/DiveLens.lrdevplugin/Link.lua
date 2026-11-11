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
photo selection whenever it changes. Messages are handled in the link's own
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
local LrSocket = import "LrSocket"
local LrTasks = import "LrTasks"

local Framing = require "Framing"
local Json = require "Json"
local Log = require "Log"

local PROTOCOL = 1
local APP_ID = "com.querton.divelens.desktop" -- the app's bundle identifier
local LINK_FILE = "lightroom-link.json"
local TICK = 0.1           -- seconds between two passes of the link task
local SELECTION_EVERY = 5  -- selection checked every 5 ticks while the app is connected
local PENDING_SCREEN_FOR = 30 -- seconds a menu choice waits for a just-launched app
local SILENCE = 8          -- seconds without news before the app is considered gone

local Link = {}

-- The link state lives in the plugin's global table so the menu scripts, the
-- Plug-in Manager and the shutdown script all see the same link.
local function current()
    return _G.DiveLensLink
end

local function pluginVersion()
    local v = require "Info".VERSION
    return string.format("%d.%d.%d", v.major, v.minor, v.revision)
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

-- Number of selected photos, and a cheap signature of the selection. Nothing
-- when no photo is selected (getTargetPhotos alone would return the whole
-- filmstrip in that case). The app asks for the photos themselves when it
-- needs them, so a large selection is never pushed around.
local function selection()
    local catalog = LrApplication.activeCatalog()
    local active = catalog:getTargetPhoto()
    if not active then return 0, "" end
    local count = #catalog:getTargetPhotos()
    return count, count .. ":" .. tostring(active.localIdentifier)
end

local function pushSelection(s, force)
    local count, signature = selection()
    if not force and signature == s.selectionSignature then return end
    if send(s, { type = "selection", count = count }) then
        s.selectionSignature = signature
    end
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
