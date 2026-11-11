--[[
Minimal JSON encoder and decoder for the link with the DiveLens app.

Covers what the link exchanges: objects, arrays, strings, numbers, booleans
and null. A table is encoded as an array when its keys are exactly 1..n, as an
object otherwise (an empty table is an empty object; use Json.emptyArray for an
empty array). `Json.null` stands for JSON null on both sides.

Part of DiveLens, GPL-3.0-or-later.
]]

local Json = {}

Json.null = setmetatable({}, { __tostring = function() return "null" end })
Json.emptyArray = setmetatable({}, { __tostring = function() return "[]" end })

-- Encoding --------------------------------------------------------------------

local escapes = {
    ['"'] = '\\"', ['\\'] = '\\\\', ['\b'] = '\\b', ['\f'] = '\\f',
    ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t',
}

local function encodeString(s)
    return '"' .. s:gsub('[%c"\\]', function(c)
        return escapes[c] or string.format("\\u%04x", c:byte())
    end) .. '"'
end

local function isArray(t)
    local n = 0
    for _ in pairs(t) do n = n + 1 end
    for i = 1, n do
        if t[i] == nil then return false end
    end
    return n > 0
end

local encodeValue

local function encodeTable(t, seen)
    if t == Json.null then return "null" end
    if t == Json.emptyArray then return "[]" end
    if seen[t] then error("Json.encode: circular table") end
    seen[t] = true
    local parts = {}
    if isArray(t) then
        for i = 1, #t do parts[i] = encodeValue(t[i], seen) end
        seen[t] = nil
        return "[" .. table.concat(parts, ",") .. "]"
    end
    local keys = {}
    for k in pairs(t) do
        if type(k) ~= "string" then error("Json.encode: object key must be a string") end
        keys[#keys + 1] = k
    end
    table.sort(keys) -- stable output, easier to read in logs and tests
    for i, k in ipairs(keys) do
        parts[i] = encodeString(k) .. ":" .. encodeValue(t[k], seen)
    end
    seen[t] = nil
    return "{" .. table.concat(parts, ",") .. "}"
end

encodeValue = function(v, seen)
    local kind = type(v)
    if kind == "nil" then return "null" end
    if kind == "boolean" then return tostring(v) end
    if kind == "number" then
        if v ~= v or v == math.huge or v == -math.huge then
            error("Json.encode: number is not finite")
        end
        if v == math.floor(v) and math.abs(v) < 2 ^ 53 then
            return string.format("%d", v)
        end
        return string.format("%.14g", v)
    end
    if kind == "string" then return encodeString(v) end
    if kind == "table" then return encodeTable(v, seen) end
    error("Json.encode: cannot encode a " .. kind)
end

function Json.encode(value)
    return encodeValue(value, {})
end

-- Decoding --------------------------------------------------------------------

local function decodeError(text, pos, what)
    error(string.format("Json.decode: %s at position %d", what, pos), 0)
end

local function skipSpace(text, pos)
    return text:find("[^ \t\r\n]", pos) or #text + 1
end

local function utf8Char(code)
    if code < 0x80 then return string.char(code) end
    if code < 0x800 then
        return string.char(0xC0 + math.floor(code / 0x40), 0x80 + code % 0x40)
    end
    if code < 0x10000 then
        return string.char(0xE0 + math.floor(code / 0x1000),
            0x80 + math.floor(code / 0x40) % 0x40, 0x80 + code % 0x40)
    end
    return string.char(0xF0 + math.floor(code / 0x40000),
        0x80 + math.floor(code / 0x1000) % 0x40,
        0x80 + math.floor(code / 0x40) % 0x40, 0x80 + code % 0x40)
end

local unescapes = { b = "\b", f = "\f", n = "\n", r = "\r", t = "\t", ['"'] = '"', ["\\"] = "\\", ["/"] = "/" }

local decodeValue

local function decodeString(text, pos)
    -- pos is on the opening quote
    local parts = {}
    local i = pos + 1
    while true do
        local j = text:find('["\\]', i)
        if not j then decodeError(text, pos, "unterminated string") end
        parts[#parts + 1] = text:sub(i, j - 1)
        if text:sub(j, j) == '"' then
            return table.concat(parts), j + 1
        end
        local c = text:sub(j + 1, j + 1)
        if c == "u" then
            local hex = text:sub(j + 2, j + 5)
            if not hex:match("^%x%x%x%x$") then decodeError(text, j, "bad \\u escape") end
            local code = tonumber(hex, 16)
            local nextPos = j + 6
            if code >= 0xD800 and code <= 0xDBFF then
                local low = text:match("^\\u(%x%x%x%x)", nextPos)
                if low then
                    code = 0x10000 + (code - 0xD800) * 0x400 + (tonumber(low, 16) - 0xDC00)
                    nextPos = nextPos + 6
                end
            end
            parts[#parts + 1] = utf8Char(code)
            i = nextPos
        elseif unescapes[c] then
            parts[#parts + 1] = unescapes[c]
            i = j + 2
        else
            decodeError(text, j, "bad escape")
        end
    end
end

local function decodeNumber(text, pos)
    local s = text:match("^-?%d+%.?%d*[eE]?[-+]?%d*", pos)
    local n = s and tonumber(s)
    if not n then decodeError(text, pos, "bad number") end
    return n, pos + #s
end

local literals = { ["true"] = true, ["false"] = false }

local function decodeArray(text, pos)
    local result, n = {}, 0
    pos = skipSpace(text, pos + 1)
    if text:sub(pos, pos) == "]" then return Json.emptyArray, pos + 1 end
    while true do
        local value
        value, pos = decodeValue(text, pos)
        n = n + 1
        result[n] = value
        pos = skipSpace(text, pos)
        local c = text:sub(pos, pos)
        if c == "]" then return result, pos + 1 end
        if c ~= "," then decodeError(text, pos, "expected ',' or ']'") end
        pos = skipSpace(text, pos + 1)
    end
end

local function decodeObject(text, pos)
    local result = {}
    pos = skipSpace(text, pos + 1)
    if text:sub(pos, pos) == "}" then return result, pos + 1 end
    while true do
        if text:sub(pos, pos) ~= '"' then decodeError(text, pos, "expected a key") end
        local key
        key, pos = decodeString(text, pos)
        pos = skipSpace(text, pos)
        if text:sub(pos, pos) ~= ":" then decodeError(text, pos, "expected ':'") end
        local value
        value, pos = decodeValue(text, skipSpace(text, pos + 1))
        result[key] = value
        pos = skipSpace(text, pos)
        local c = text:sub(pos, pos)
        if c == "}" then return result, pos + 1 end
        if c ~= "," then decodeError(text, pos, "expected ',' or '}'") end
        pos = skipSpace(text, pos + 1)
    end
end

decodeValue = function(text, pos)
    pos = skipSpace(text, pos)
    local c = text:sub(pos, pos)
    if c == "{" then return decodeObject(text, pos) end
    if c == "[" then return decodeArray(text, pos) end
    if c == '"' then return decodeString(text, pos) end
    if c == "-" or c:match("%d") then return decodeNumber(text, pos) end
    for word, value in pairs(literals) do
        if text:sub(pos, pos + #word - 1) == word then return value, pos + #word end
    end
    if text:sub(pos, pos + 3) == "null" then return Json.null, pos + 4 end
    decodeError(text, pos, "unexpected character")
end

--- Decodes one JSON document. Raises an error on malformed input.
function Json.decode(text)
    if type(text) ~= "string" then error("Json.decode: expected a string", 0) end
    local value, pos = decodeValue(text, 1)
    pos = skipSpace(text, pos)
    if pos <= #text then decodeError(text, pos, "trailing characters") end
    return value
end

return Json
