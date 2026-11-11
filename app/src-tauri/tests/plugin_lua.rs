//! Tests of the Lightroom plugin's pure-Lua parts, run under Lua 5.1 (the
//! version Lightroom embeds). The parts that need Lightroom itself are checked
//! by hand in Lightroom.

use std::path::PathBuf;

use mlua::Lua;

fn plugin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugin/DiveLens.lrdevplugin")
}

fn lua() -> Lua {
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

fn run(lua: &Lua, code: &str) {
    if let Err(e) = lua.load(code).exec() {
        panic!("{e}");
    }
}

#[test]
fn runs_under_plain_lua_5_1() {
    run(
        &lua(),
        r#"assert(_VERSION == "Lua 5.1" and jit == nil, _VERSION)"#,
    );
}

#[test]
fn json_encodes_what_rust_reads() {
    let lua = lua();
    let text: String = lua
        .load(
            r#"
            local Json = require "Json"
            return Json.encode {
                type = "selection", count = 3, photos = { 12, 13, 2000000001 },
                catalog = 'Plongées "2026"\n\tété', ratio = 0.25, ok = true,
                none = Json.null, empty = Json.emptyArray, nested = { a = { b = -1.5e-3 } },
            }
            "#,
        )
        .eval()
        .unwrap();
    assert!(!text.contains('\n'), "one message per line: {text}");
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "type": "selection", "count": 3, "photos": [12, 13, 2000000001u64],
            "catalog": "Plongées \"2026\"\n\tété", "ratio": 0.25, "ok": true,
            "none": null, "empty": [], "nested": { "a": { "b": -0.0015 } },
        })
    );
}

#[test]
fn json_decodes_what_rust_writes() {
    let lua = lua();
    let message = serde_json::json!({
        "type": "hello", "token": "a1b2", "app": "0.1.0", "protocol": 1,
        "text": "Été \u{1F420} \\ \"quoted\" / tab\t", "list": [1, 2.5, -3, true, null, []],
        "empty": {},
    });
    lua.globals()
        .set("input", serde_json::to_string(&message).unwrap())
        .unwrap();
    run(
        &lua,
        r#"
        local Json = require "Json"
        local m = Json.decode(input)
        assert(m.type == "hello" and m.token == "a1b2" and m.protocol == 1)
        assert(m.text == "Été \240\159\144\160 \\ \"quoted\" / tab\t", m.text)
        assert(m.list[1] == 1 and m.list[2] == 2.5 and m.list[3] == -3 and m.list[4] == true)
        assert(m.list[5] == Json.null and m.list[6] == Json.emptyArray)
        assert(next(m.empty) == nil)
        -- And back again, unchanged.
        assert(Json.decode(Json.encode(m)).text == m.text)
        "#,
    );
}

#[test]
fn json_rejects_malformed_input() {
    let lua = lua();
    run(
        &lua,
        r#"
        local Json = require "Json"
        for _, bad in ipairs { '', '{', '{"a":}', '{"a":1', '[1,]', '{"a":1} x', 'nul', '"open' } do
            assert(not pcall(Json.decode, bad), "accepted: " .. bad)
        end
        assert(not pcall(Json.encode, { [1] = 1, [3] = 3 }), "sparse array")
        assert(not pcall(Json.encode, 0 / 0), "NaN")
        "#,
    );
}

#[test]
fn framing_splits_lines_whatever_lightroom_hands_over() {
    let lua = lua();
    run(
        &lua,
        r#"
        local Framing = require "Framing"
        local Json = require "Json"
        local function whole(text) return (pcall(Json.decode, text)) end
        local function same(got, want)
            assert(#got == #want, ("got %d messages, want %d"):format(#got, #want))
            for i = 1, #want do assert(got[i] == want[i], got[i]) end
        end

        -- Several lines at once, CRLF, blank lines.
        local f = Framing.new()
        same(Framing.push(f, '{"a":1}\r\n\n{"b":2}\n', whole), { '{"a":1}', '{"b":2}' })

        -- A line split across chunks.
        f = Framing.new()
        same(Framing.push(f, '{"a":', whole), {})
        same(Framing.push(f, '1}\n{"b"', whole), { '{"a":1}' })
        same(Framing.push(f, ':2}\n', whole), { '{"b":2}' })

        -- Lightroom stripped the newline: a complete message is still taken.
        f = Framing.new()
        same(Framing.push(f, '{"type":"ping"}', whole), { '{"type":"ping"}' })
        assert(f.buffer == "")
        "#,
    );
}

#[test]
fn plugin_version_matches_app_version() {
    let lua = lua();
    let version: String = lua
        .load(
            r#"
            local v = require "Info".VERSION
            local numbers = string.format("%d.%d.%d", v.major, v.minor, v.revision)
            assert(v.display == numbers, "display " .. tostring(v.display) .. " vs " .. numbers)
            return numbers
            "#,
        )
        .eval()
        .unwrap();
    let package: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../package.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(version, package["version"].as_str().unwrap());
}

#[test]
fn plugin_keeps_its_identifier_and_fields() {
    let lua = lua();
    run(
        &lua,
        r#"
        assert(require "Info".LrToolkitIdentifier == "com.querton.divelens")
        local metadata = require "Metadata"
        assert(metadata.schemaVersion == 1)
        local ids = {}
        for _, field in ipairs(metadata.metadataFieldsForPhotos) do
            assert(field.dataType == "string" and field.version == 1, field.id)
            ids[#ids + 1] = field.id
        end
        assert(table.concat(ids, ",") == "DiveId,StartDate,EndDate,Duration,MaxDepth,TempHigh,TempLow,Rating",
            table.concat(ids, ","))
        "#,
    );
}
