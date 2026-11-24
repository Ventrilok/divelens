//! Tests of the plug-in's JSON and message framing, run under Lua 5.1 (the
//! version Lightroom embeds).

mod common;

use common::{lua, run, sample_dive};

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
fn large_collections_request_is_framed_quickly() {
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    let dive = sample_dive();
    let photos = (0..2000)
        .map(|i| PhotoTags::new(i.to_string(), None, 240, Units::default()))
        .collect();
    let request = CollectionsRequest {
        search: 1,
        parent: Vec::new(),
        group: None,
        groups: vec![CollectionGroup {
            collection: Some("700 - Coral Garden".into()),
            dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
            location: Some(LocationTags::new(&dive.site, None, None)),
            photos,
        }],
    };
    let line = serde_json::to_string(&request).unwrap() + "\n";
    assert!(line.len() > 150_000, "{}", line.len());
    let lua = lua();
    lua.globals().set("input", line).unwrap();
    let started = std::time::Instant::now();
    run(
        &lua,
        r#"
        local Framing = require "Framing"
        local Json = require "Json"
        local state, got = Framing.new(), {}
        local function whole(text) return (pcall(Json.decode, text)) end
        for i = 1, #input, 4096 do
            for _, m in ipairs(Framing.push(state, input:sub(i, i + 4095), whole)) do got[#got + 1] = m end
        end
        assert(#got == 1, #got)
        assert(#Json.decode(got[1]).groups[1].photos == 2000)
        "#,
    );
    assert!(
        started.elapsed().as_secs_f64() < 2.0,
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn the_link_knows_the_count_message() {
    let link = std::fs::read_to_string(common::plugin_dir().join("Link.lua")).unwrap();
    // Line by line: a Windows checkout may end lines with CRLF.
    assert!(
        link.lines()
            .map(str::trim)
            .any(|l| l == "countPhotos = Selection.count,"),
        "Link.lua must dispatch countPhotos to Selection.count"
    );
}
