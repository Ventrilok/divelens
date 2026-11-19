//! Tests of what "Tag Selection" writes, run under Lua 5.1 (the version
//! Lightroom embeds).

mod common;

use common::{WRITES_LUA, lua, run, sample_dive};

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

#[test]
fn tagging_plan_writes_dive_fields_and_clears_missing_ones() {
    use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units};
    let dive = sample_dive();
    let lines = plan(&TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
        ..TagRequest::default()
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
            "plugin Source=Subsurface",
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
        ..TagRequest::default()
    })
    .unwrap();
    assert_eq!(lines, ["raw country=Atlantis", "raw location=Coral Garden"]);
    assert_eq!(
        plan(&TagRequest {
            dive: None,
            location: None,
            ..TagRequest::default()
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

/// The two writes "Tag Selection" makes from `request`, on a photo without
/// dive data.
fn tag_writes(request: &divelens_core::tags::TagRequest) -> Vec<String> {
    let lua = lua();
    run(&lua, WRITES_LUA);
    lua.globals()
        .set("input", serde_json::to_string(request).unwrap())
        .unwrap();
    lua.load(
        r#"
        local Json = require "Json"
        local Tagging = require "Tagging"
        local changes = assert(Tagging.plan(Json.decode(input), function(iso) return "date(" .. iso .. ")" end))
        return writes(changes)
        "#,
    )
    .eval()
    .unwrap()
}

#[test]
fn tagging_writes_gps_in_a_write_of_its_own() {
    use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units};
    let dive = sample_dive();
    let both = TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
        ..TagRequest::default()
    };
    // The same changes as the plan, the GPS moved to the second write.
    let mut planned: Vec<String> = plan(&both).unwrap();
    let gps = planned.pop().unwrap();
    assert_eq!(gps, "raw gps=12.5,-45.25");
    let mut want: Vec<String> = planned.iter().map(|l| format!("1 {l}")).collect();
    want.push(format!("2 {gps}"));
    assert_eq!(tag_writes(&both), want);
    assert!(want.contains(&"1 raw country=Atlantis".to_string()));

    // Location only, with its GPS: country and location first, GPS second.
    let place = TagRequest {
        dive: None,
        location: Some(LocationTags::new(&dive.site, None, None)),
        ..TagRequest::default()
    };
    assert_eq!(
        tag_writes(&place),
        [
            "1 raw country=Atlantis",
            "1 raw location=Coral Garden",
            "2 raw gps=12.5,-45.25"
        ]
    );
    // A site without GPS, or dive data only: no second write.
    let mut site = dive.site.clone();
    site.lat = None;
    site.lon = None;
    let no_gps = TagRequest {
        dive: None,
        location: Some(LocationTags::new(&site, None, None)),
        ..TagRequest::default()
    };
    assert_eq!(
        tag_writes(&no_gps),
        ["1 raw country=Atlantis", "1 raw location=Coral Garden"]
    );
    let dive_only = TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: None,
        ..TagRequest::default()
    };
    assert!(tag_writes(&dive_only).iter().all(|l| l.starts_with("1 ")));
}

#[test]
fn progress_tells_the_first_write_only() {
    let lua = lua();
    let reported = |total: u32| -> Vec<u32> {
        lua.load(format!(
            r#"
            local p = require("Tagging").progress({total}, 50)
            local out = {{}}
            for _ = 1, {total} do
                local done = p.advance()
                if done then out[#out + 1] = done end
            end
            return out
            "#
        ))
        .eval()
        .unwrap()
    };
    // Every 50 photos, never the last one: the app's answer tells the end.
    // The second write tells nothing (it must not pause).
    assert_eq!(reported(120), [50, 100]);
    assert_eq!(reported(100), [50]);
    assert_eq!(reported(30), Vec::<u32>::new());
}

#[test]
fn tag_selection_plans_per_photo_values_and_checks_the_selection() {
    use divelens_core::profile::PhotoPoint;
    use divelens_core::tags::{DiveTags, PhotoTags, TagRequest, Units};
    let dive = sample_dive();
    let point = PhotoPoint {
        elapsed_s: 160,
        depth_m: 10.0,
        temp_c: None,
    };
    let request = TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        selection: Some(12),
        photos: vec![PhotoTags::new(
            "41".into(),
            Some(point),
            -30,
            Units::default(),
        )],
        ..TagRequest::default()
    };
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    lua.globals()
        .set(
            "location_only",
            serde_json::to_string(&TagRequest {
                location: Some(divelens_core::tags::LocationTags::new(
                    &dive.site, None, None,
                )),
                ..TagRequest::default()
            })
            .unwrap(),
        )
        .unwrap();
    run(
        &lua,
        r#"
        local Json = require "Json"
        local Tagging = require "Tagging"
        local plan = assert(Tagging.planPhotos(Json.decode(input)))
        assert(plan.selection == 12)
        local own = plan.photos["41"]
        local lines = {}
        for i, c in ipairs(own) do lines[i] = ("%s %s=%s"):format(c.kind, c.field, tostring(c.value)) end
        assert(table.concat(lines, ";") == "plugin PhotoDepth=10.0 m;plugin PhotoTemp=nil;"
            .. "plugin PhotoElapsed=02:40;plugin ClockOffset=-00:30", table.concat(lines, ";"))
        -- No selection named: no per-photo values, as before.
        local none, why = Tagging.planPhotos(Json.decode(location_only))
        assert(none == nil and why == nil)
        -- Not usable: nothing is written.
        local bad = Json.decode(input)
        bad.photos[1].clockOffset = Json.null
        assert(select(2, Tagging.planPhotos(bad)) == "bad photo")
        bad = Json.decode(input)
        bad.selection = "12"
        assert(select(2, Tagging.planPhotos(bad)) == "bad selection")
        bad = Json.decode(input)
        bad.dive = Json.null
        assert(select(2, Tagging.planPhotos(bad)) == "bad dive")

        -- Written only on exactly the photos described to the app, each as
        -- many times.
        local function photo(n) return { localIdentifier = n } end
        local function id(p) return tostring(p.localIdentifier) end
        local described = Tagging.photoSet({ photo(41), photo(42) }, id)
        assert(described.count == 2 and described.ids["41"] == 1)
        assert(Tagging.sameSelection(described, { photo(42), photo(41) }, id))
        assert(not Tagging.sameSelection(described, { photo(41) }, id), "one photo less")
        assert(not Tagging.sameSelection(described, { photo(41), photo(42), photo(43) }, id), "one more")
        assert(not Tagging.sameSelection(described, { photo(41), photo(43) }, id), "another one")
        assert(not Tagging.sameSelection(described, { photo(41), photo(41) }, id), "one twice, one missing")
        local twice = Tagging.photoSet({ photo(41), photo(41), photo(42) }, id)
        assert(Tagging.sameSelection(twice, { photo(41), photo(42), photo(41) }, id))
        assert(not Tagging.sameSelection(twice, { photo(41), photo(42), photo(42) }, id), "counts differ")
        assert(not Tagging.sameSelection(nil, { photo(41) }, id), "none described")
        assert(Tagging.sameSelection(Tagging.photoSet({}, id), {}, id))

        -- One photo's writes: its own values replace what the other dive
        -- left; it is told whether it carried another dive's values.
        local changes = assert(Tagging.plan(Json.decode(input), function(iso) return iso end))
        local other = { Source = "MacDive", StartDate = "another start", DiveId = "699", PhotoDepth = "3.0 m" }
        local first, second, replaced = Tagging.photoWrites(changes, other, own)
        assert(replaced and #second == 0)
        local fields = {}
        for _, c in ipairs(first) do fields[#fields + 1] = c.field .. "=" .. tostring(c.value) end
        local text = table.concat(fields, ";")
        assert(not text:find("PhotoDepth=nil", 1, true), text)
        assert(text:find("PhotoDepth=10.0 m", 1, true) and text:find("ClockOffset=-00:30", 1, true), text)
        first, second, replaced = Tagging.photoWrites(changes, other, nil)
        assert(replaced and first[#first].field == "PhotoDepth" and first[#first].value == nil)
        first, second, replaced = Tagging.photoWrites(changes, nil, nil)
        assert(not replaced and #first == #changes and #second == 0)

        -- Only the changes a photo's own values write again are dropped.
        local extra = {
            { kind = "plugin", field = "PhotoDepth" }, { kind = "plugin", field = "ClockOffset" },
            { kind = "raw", field = "gpsAltitude" },
        }
        local kept = Tagging.notOverwritten(extra, own)
        assert(#kept == 1 and kept[1].field == "gpsAltitude")
        own[#own + 1] = { kind = "raw", field = "gpsAltitude", value = -10 }
        assert(#Tagging.notOverwritten(extra, own) == 0)
        assert(#Tagging.notOverwritten(extra, nil) == 3)
        "#,
    );
}

#[test]
fn dive_values_reach_the_plugin_by_name_with_real_values() {
    use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units};
    let mut dive = sample_dive();
    dive.temp_low_c = Some(24.5);
    dive.rating = Some(4);
    dive.site.location = Some("North Reef".into());
    let lines = plan(&TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
        ..TagRequest::default()
    })
    .unwrap();
    assert!(
        lines.contains(&"plugin TempLow=24.5°C".to_string()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"plugin Rating=★★★★".to_string()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"raw city=North Reef".to_string()),
        "{lines:?}"
    );
}

/// What `lua` returns, a JSON text (Json.encode), as a value to compare.
fn json_of(lua: &mlua::Lua, code: &str) -> serde_json::Value {
    let text: String = match lua.load(code).eval() {
        Ok(text) => text,
        Err(e) => panic!("{e}"),
    };
    serde_json::from_str(&text).unwrap()
}

#[test]
fn apply_reply_says_why_nothing_or_part_was_written() {
    use serde_json::json;
    let lua = lua();
    let reply = |outcome: &str| {
        json_of(
            &lua,
            &format!(
                r#"
                local Json = require "Json"
                local Tagging = require "Tagging"
                local done = {{ type = "applied", id = 7, count = 3 }}
                return Json.encode(Tagging.applyReply(7, {outcome}, done))
                "#
            ),
        )
    };
    // The first write failed, or Lightroom kept the catalog busy: nothing
    // written.
    assert_eq!(
        reply(r#"{ ok = false, status = "boom", second = false }"#),
        json!({ "type": "applyFailed", "id": 7, "reason": "error", "detail": "boom" })
    );
    assert_eq!(
        reply(r#"{ ok = true, status = "aborted", second = false }"#),
        json!({ "type": "applyFailed", "id": 7, "reason": "busy" })
    );
    // The first write stays, without the GPS and altitude.
    assert_eq!(
        reply(
            r#"{ ok = true, status = "executed", second = { written = false, error = "locked" } }"#
        ),
        json!({ "type": "applyFailed", "id": 7, "reason": "gpsNotWritten", "detail": "locked" })
    );
    assert_eq!(
        reply(r#"{ ok = true, status = "executed", second = { written = false } }"#),
        json!({ "type": "applyFailed", "id": 7, "reason": "gpsNotWritten" })
    );
    // Both written: the answer the app waits for, as it is.
    assert_eq!(
        reply(
            r#"{ ok = true, status = "executed", second = { written = true, cleared = 0, refused = 0 } }"#
        ),
        json!({ "type": "applied", "id": 7, "count": 3 })
    );
    run(
        &lua,
        r#"
        local Tagging = require "Tagging"
        local done = { type = "collectionsApplied", id = 8, groups = {} }
        local outcome = { ok = true, status = "executed", second = { written = true } }
        assert(rawequal(Tagging.applyReply(8, outcome, done), done))
        "#,
    );
}

#[test]
fn second_write_result_keeps_lightroom_refusals() {
    use serde_json::json;
    let lua = lua();
    assert_eq!(
        json_of(
            &lua,
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            return Json.encode {
                failed = Tagging.secondWriteResult(false, "catalog locked", 0, 0),
                busy = Tagging.secondWriteResult(true, "aborted", 0, 0),
                written = Tagging.secondWriteResult(true, "executed", 2, 1),
                clears = {
                    Tagging.isAltitudeClear({ kind = "raw", field = "gpsAltitude" }),
                    Tagging.isAltitudeClear({ kind = "raw", field = "gpsAltitude", value = -18 }),
                    Tagging.isAltitudeClear({ kind = "raw", field = "gps" }),
                },
            }
            "#,
        ),
        json!({
            "failed": { "written": false, "error": "catalog locked" },
            // Lightroom kept the catalog busy: no error to tell.
            "busy": { "written": false },
            // Two altitudes cleared, one Lightroom refused to clear.
            "written": { "written": true, "cleared": 2, "refused": 1 },
            "clears": [true, false, false],
        })
    );
}

#[test]
fn selection_refusal_checks_the_described_selection() {
    use serde_json::json;
    let lua = lua();
    assert_eq!(
        json_of(
            &lua,
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            local function photo(n) return { localIdentifier = n } end
            local id = Tagging.photoId
            local described = { id = 12, set = Tagging.photoSet({ photo(41), photo(42) }, id) }
            local perPhoto = { selection = 12, photos = {} }
            local function refusal(...) return Tagging.selectionRefusal(...) or Json.null end
            return Json.encode {
                noValues = refusal(nil, nil, { photo(41) }, id),
                notDescribed = refusal(perPhoto, nil, { photo(41), photo(42) }, id),
                otherDescription = refusal({ selection = 11, photos = {} }, described, { photo(41), photo(42) }, id),
                otherPhotos = refusal(perPhoto, described, { photo(41), photo(43) }, id),
                samePhotos = refusal(perPhoto, described, { photo(42), photo(41) }, id),
            }
            "#,
        ),
        json!({
            // No per-photo values: written on whatever is selected.
            "noValues": null,
            "notDescribed": "staleSelection",
            "otherDescription": "staleSelection",
            "otherPhotos": "selectionChanged",
            "samePhotos": null,
        })
    );
}

#[test]
fn tag_writes_keeps_photo_order_and_counts() {
    use divelens_core::profile::PhotoPoint;
    use divelens_core::tags::{DiveTags, LocationTags, PhotoTags, TagRequest, Units};
    use serde_json::json;
    let dive = sample_dive();
    let point = PhotoPoint {
        elapsed_s: 160,
        depth_m: 10.0,
        temp_c: None,
    };
    let request = TagRequest {
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
        selection: Some(12),
        photos: vec![PhotoTags::new(
            "p1".into(),
            Some(point),
            -30,
            Units::default(),
        )],
    };
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    assert_eq!(
        json_of(
            &lua,
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            local request = Json.decode(input)
            local changes = assert(Tagging.plan(request, function(iso) return iso end))
            local perPhoto = assert(Tagging.planPhotos(request))
            local p1, p2 = { id = "p1" }, { id = "p2" }
            local function idOf(p) return p.id end
            -- Each photo's writes as its id and number of changes.
            local function summary(plan)
                local writes, later = {}, {}
                for i, w in ipairs(plan.writes) do writes[i] = { photo = w.photo.id, changes = #w.changes } end
                for i, l in ipairs(plan.later) do later[i] = { photo = l.photo.id, changes = #l.changes } end
                return { writes = writes, later = later, cleared = plan.cleared, withValues = plan.withValues }
            end
            local other = { Source = "MacDive", StartDate = "another start", DiveId = "699", PhotoDepth = "3.0 m" }
            return Json.encode {
                fresh = summary(Tagging.tagWrites(changes, { p1, p2 }, {}, perPhoto, idOf)),
                reversed = summary(Tagging.tagWrites(changes, { p2, p1 }, {}, perPhoto, idOf)),
                retagged = summary(Tagging.tagWrites(changes, { p1, p2 }, { [p2] = other }, perPhoto, idOf)),
            }
            "#,
        ),
        json!({
            // 11 dive and location fields for both, and the first photo's 4
            // own values; the GPS of both in the second write.
            "fresh": {
                "writes": [{ "photo": "p1", "changes": 15 }, { "photo": "p2", "changes": 11 }],
                "later": [{ "photo": "p1", "changes": 1 }, { "photo": "p2", "changes": 1 }],
                "cleared": 0,
                "withValues": 1,
            },
            "reversed": {
                "writes": [{ "photo": "p2", "changes": 11 }, { "photo": "p1", "changes": 15 }],
                "later": [{ "photo": "p2", "changes": 1 }, { "photo": "p1", "changes": 1 }],
                "cleared": 0,
                "withValues": 1,
            },
            // The second photo carried another dive: its depth is cleared.
            "retagged": {
                "writes": [{ "photo": "p1", "changes": 15 }, { "photo": "p2", "changes": 12 }],
                "later": [{ "photo": "p1", "changes": 1 }, { "photo": "p2", "changes": 1 }],
                "cleared": 1,
                "withValues": 1,
            },
        })
    );
}
