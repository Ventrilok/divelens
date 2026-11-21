#![allow(dead_code)]

//! Shared by the integration tests: the plugin's folder, and a Lua 5.1 state
//! (the version Lightroom embeds) that loads the plugin's modules.

use std::path::PathBuf;

use mlua::Lua;

pub fn plugin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugin/DiveLens.lrdevplugin")
}

/// Lua that finds the plugin's modules with `require`, with a `LOC` that
/// returns the English text, as Lightroom does without a translation.
pub fn lua() -> Lua {
    let lua = Lua::new();
    let path = plugin_dir().join("?.lua");
    lua.load(format!(
        "package.path = {:?}\n\
         LOC = function(s) return (s:gsub('^%$%$%$/[^=]*=', '')) end",
        path.to_str().unwrap()
    ))
    .exec()
    .unwrap();
    lua
}

pub fn run(lua: &Lua, code: &str) {
    if let Err(e) = lua.load(code).exec() {
        panic!("{e}");
    }
}

/// Runs Lua code that returns JSON text, and parses it.
pub fn json_of(lua: &Lua, code: &str) -> serde_json::Value {
    let text: String = match lua.load(code).eval() {
        Ok(text) => text,
        Err(e) => panic!("{e}"),
    };
    serde_json::from_str(&text).unwrap()
}

pub fn sample_dive() -> divelens_core::Dive {
    let start = chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
        .unwrap()
        .and_hms_opt(7, 14, 0)
        .unwrap();
    divelens_core::Dive {
        id: "A".into(),
        number: Some(700),
        start,
        end: start + chrono::Duration::seconds(2100),
        duration_s: 2100,
        max_depth_m: Some(31.2),
        temp_high_c: Some(28.0),
        temp_low_c: None,
        rating: None,
        site: divelens_core::Site {
            id: Some("S".into()),
            name: Some("Coral Garden".into()),
            location: None,
            country: Some("Atlantis".into()),
            lat: Some(12.5),
            lon: Some(-45.25),
        },
        has_profile: true,
        invalid: false,
        notes: None,
    }
}

/// Lua defining `writes(...)`: the two catalog writes the ApplyTags and
/// ApplyCollections envelopes make from lists of planned changes
/// (Tagging.splitGps), as "1 kind field=value" and "2 kind field=value"
/// lines. It checks what must always hold: the GPS and the altitude only in
/// the second write, never in a write with a location text field, and every
/// change kept, once, in its order. `checked(first, second)` makes the same
/// lines and the first two checks, for writes Tagging made itself
/// (Tagging.photoWrites).
pub const WRITES_LUA: &str = r#"
local Tagging = require "Tagging"
local TEXT = { country = true, city = true, location = true }
local GPS = { gps = true, gpsAltitude = true }
local function line(c)
    local v = c.value
    if type(v) == "table" then v = ("%s,%s"):format(v.latitude, v.longitude) end
    return ("%s %s=%s"):format(c.kind, c.field, tostring(v))
end
local function has(list, fields)
    for _, c in ipairs(list) do
        if c.kind == "raw" and fields[c.field] then return true end
    end
    return false
end
function checked(first, second)
    assert(not has(first, GPS), "GPS or altitude in the first write")
    for _, w in ipairs { first, second } do
        assert(not (has(w, GPS) and has(w, TEXT)), "GPS with a location text field")
    end
    local out = {}
    for _, c in ipairs(first) do out[#out + 1] = "1 " .. line(c) end
    for _, c in ipairs(second) do out[#out + 1] = "2 " .. line(c) end
    return out
end
function writes(...)
    local lists = { ... }
    local first, second = Tagging.splitGps(...)
    local out = checked(first, second)
    local i, j = 1, 1
    for _, list in ipairs(lists) do
        for _, c in ipairs(list) do
            if c.kind == "raw" and GPS[c.field] then
                assert(second[j] == c, "second write: change lost or moved")
                j = j + 1
            else
                assert(first[i] == c, "first write: change lost or moved")
                i = i + 1
            end
        end
    end
    assert(first[i] == nil and second[j] == nil, "change added")
    return out
end
"#;
