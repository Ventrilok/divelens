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
        assert(metadata.schemaVersion == 2)
        local ids = {}
        for _, field in ipairs(metadata.metadataFieldsForPhotos) do
            assert(field.dataType == "string" and field.version == 1, field.id)
            ids[#ids + 1] = field.id
        end
        assert(table.concat(ids, ",") == "DiveId,StartDate,EndDate,Duration,MaxDepth,TempHigh,TempLow,Rating,"
            .. "PhotoDepth,PhotoTemp,PhotoElapsed,ClockOffset",
            table.concat(ids, ","))
        "#,
    );
}

/// The plan the plugin makes from a request built by the app, as
/// "kind field=value" lines (value "nil" clears the field).
fn plan(request: &divelens_core::tags::TagRequest) -> Result<Vec<String>, String> {
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(request).unwrap())
        .unwrap();
    let out: mlua::Table = lua
        .load(
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            local changes, why = Tagging.plan(Json.decode(input), function(iso)
                return "date(" .. iso .. ")"
            end)
            if not changes then return { error = why } end
            local lines = {}
            for i, c in ipairs(changes) do
                local v = c.value
                if type(v) == "table" then v = ("%s,%s"):format(v.latitude, v.longitude) end
                lines[i] = ("%s %s=%s"):format(c.kind, c.field, tostring(v))
            end
            return { lines = lines }
            "#,
        )
        .eval()
        .unwrap();
    if let Ok(why) = out.get::<String>("error") {
        return Err(why);
    }
    Ok(out.get("lines").unwrap())
}

fn sample_dive() -> divelens_core::Dive {
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
    }
}

#[test]
fn tagging_plan_writes_dive_fields_and_clears_missing_ones() {
    use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units};
    let dive = sample_dive();
    let lines = plan(&TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default())),
        location: Some(LocationTags::new(&dive.site, None, None)),
    })
    .unwrap();
    assert_eq!(
        lines,
        [
            "plugin DiveId=700",
            "plugin StartDate=date(2025-12-31T07:14:00)",
            "plugin EndDate=date(2025-12-31T07:49:00)",
            "plugin Duration=35:00",
            "plugin MaxDepth=31.2 m",
            "plugin TempHigh=28.0°C",
            "plugin TempLow=nil",
            "plugin Rating=nil",
            // No city: the photo's city is left as it is.
            "raw country=Atlantis",
            "raw location=Coral Garden",
            "raw gps=12.5,-45.25",
        ]
    );
}

#[test]
fn tagging_plan_location_only_and_bad_requests() {
    use divelens_core::tags::{LocationTags, TagRequest};
    let mut site = sample_dive().site;
    site.lat = None;
    site.lon = None;
    let lines = plan(&TagRequest {
        dive: None,
        location: Some(LocationTags::new(&site, None, None)),
    })
    .unwrap();
    assert_eq!(lines, ["raw country=Atlantis", "raw location=Coral Garden"]);
    assert_eq!(
        plan(&TagRequest {
            dive: None,
            location: None
        }),
        Err("nothing to write".into())
    );
}

#[test]
fn tagging_reads_wall_clock_dates() {
    let lua = lua();
    run(
        &lua,
        r#"
        local Tagging = require "Tagging"
        local y, mo, d, h, mi, s = Tagging.dateParts("2025-12-31T07:04:09")
        assert(y == 2025 and mo == 12 and d == 31 and h == 7 and mi == 4 and s == 9)
        assert(Tagging.dateParts("31/12/2025") == nil)
        "#,
    );
}

#[test]
fn every_plugin_file_compiles_under_lua_5_1() {
    let lua = lua();
    for entry in std::fs::read_dir(plugin_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "lua") {
            let code = std::fs::read_to_string(&path).unwrap();
            if let Err(e) = lua
                .load(&code)
                .set_name(path.display().to_string())
                .into_function()
            {
                panic!("{e}");
            }
        }
    }
}

#[test]
fn photos_are_described_with_wall_clock_times() {
    let lua = lua();
    run(
        &lua,
        r#"
        local Tagging = require "Tagging"
        local w = Tagging.wallClock
        assert(w("2025-12-31T07:14:09") == "2025-12-31T07:14:09")
        assert(w("2025-12-31T07:14:09.250") == "2025-12-31T07:14:09")
        -- The camera's time zone is dropped: times stay wall-clock.
        assert(w("2025-12-31T07:14:09+04:00") == "2025-12-31T07:14:09")
        assert(w("2025-12-31T07:14") == "2025-12-31T07:14:00")
        assert(w("2025-12-31") == nil and w(nil) == nil)

        local p = Tagging.describePhoto(4012, "2025-12-31T07:14:09.5", "OM Digital", " TG-7 ", "  ")
        assert(p.id == "4012" and p.time == "2025-12-31T07:14:09")
        assert(p.model == "TG-7" and p.serial == nil)
        local q = Tagging.describePhoto(1, nil, "GoPro", nil, "C3441325")
        assert(q.model == "GoPro" and q.serial == "C3441325" and q.time == nil)
        "#,
    );
}

#[test]
fn collections_plan_writes_dive_data_and_per_photo_values() {
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    let dive = sample_dive();
    let point = divelens_core::profile::PhotoPoint {
        elapsed_s: 160,
        depth_m: 10.0,
        temp_c: Some(25.0),
    };
    let mut request = CollectionsRequest {
        search: 3,
        parent: vec!["Trips".into(), "2026".into()],
        group: Some("Dive Lens".into()),
        groups: vec![CollectionGroup {
            collection: "700 - Coral Garden".into(),
            dive: DiveTags::new(&dive, Units::default()),
            location: LocationTags::new(&dive.site, None, None),
            photos: vec![
                PhotoTags::new("11".into(), Some(point), 241, Units::default()),
                PhotoTags::new("12".into(), None, -30, Units::default()),
            ],
        }],
    };
    request.copy_depth_to_altitude();
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    let out: Vec<String> = lua
        .load(
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            local groups, place = Tagging.planCollections(Json.decode(input), function(iso) return iso end)
            assert(groups, place)
            local lines = { "place " .. table.concat(place.parent, ">") .. " / " .. tostring(place.group) }
            for _, g in ipairs(groups) do
                lines[#lines + 1] = "collection " .. g.collection .. " common " .. #g.common
                for _, p in ipairs(g.photos) do
                    for _, c in ipairs(p.changes) do
                        lines[#lines + 1] = ("%s %s=%s"):format(p.id, c.field, tostring(c.value))
                    end
                end
            end
            assert(Tagging.planCollections({ groups = {} }) == nil)
            -- Root without a group set; a blank set name is no set.
            local request = Json.decode(input)
            request.parent, request.group = Json.emptyArray, ""
            local _, root = Tagging.planCollections(request, tostring)
            assert(#root.parent == 0 and root.group == nil)
            request.parent = { "Trips", "" }
            assert(Tagging.planCollections(request, tostring) == nil)
            assert(Tagging.planCollections({ groups = { { collection = "x" } } }, tostring) == nil)
            return lines
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!(
        out,
        [
            "place Trips>2026 / Dive Lens",
            "collection 700 - Coral Garden common 11",
            "11 PhotoDepth=10.0 m",
            "11 PhotoTemp=25.0°C",
            "11 PhotoElapsed=02:40",
            "11 ClockOffset=+04:01",
            "11 gpsAltitude=-10",
            // At the surface: per-photo values cleared, offset kept, altitude
            // untouched.
            "12 PhotoDepth=nil",
            "12 PhotoTemp=nil",
            "12 PhotoElapsed=nil",
            "12 ClockOffset=-00:30",
        ]
    );
}

#[test]
fn collection_sets_are_found_by_path_and_listed() {
    let lua = lua();
    run(
        &lua,
        r#"
        local Json = require "Json"
        local Tagging = require "Tagging"
        local function set(name, children)
            return {
                getName = function() return name end,
                getChildCollectionSets = function() return children or {} end,
            }
        end
        local y2026 = set("2026")
        local catalog = set(nil, { set("Dive Lens"), set("Trips", { set("2025"), y2026 }) })
        assert(Tagging.findSet(catalog, {}) == catalog)
        assert(Tagging.findSet(catalog, { "Trips", "2026" }) == y2026)
        assert(Tagging.findSet(catalog, { "Trips", "2027" }) == nil)
        assert(Tagging.findSet(catalog, { "2026" }) == nil, "only from the root")
        local tree = Tagging.describeSets(catalog, Json.emptyArray)
        assert(Json.encode(tree) == '[{"children":[],"name":"Dive Lens"},'
            .. '{"children":[{"children":[],"name":"2025"},{"children":[],"name":"2026"}],"name":"Trips"}]',
            Json.encode(tree))
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
            collection: "700 - Coral Garden".into(),
            dive: DiveTags::new(&dive, Units::default()),
            location: LocationTags::new(&dive.site, None, None),
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
