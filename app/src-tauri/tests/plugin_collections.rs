//! Tests of what "Create Collections" writes, run under Lua 5.1 (the version
//! Lightroom embeds).

mod common;

use common::{WRITES_LUA, json_of, lua, run, sample_dive};

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
            collection: Some("700 - Coral Garden".into()),
            dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
            location: Some(LocationTags::new(&dive.site, None, None)),
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
            "collection 700 - Coral Garden common 12",
            "11 PhotoDepth=10.0 m",
            "11 PhotoTemp=25.0°C",
            "11 PhotoElapsed=02:40",
            "11 ClockOffset=+04:01",
            "11 gpsAltitude=-10",
            // The altitude marker: the altitude written, as text.
            "11 WrittenAltitude=-10",
            // At the surface: per-photo values cleared, offset kept, no
            // altitude written.
            "12 PhotoDepth=nil",
            "12 PhotoTemp=nil",
            "12 PhotoElapsed=nil",
            "12 ClockOffset=-00:30",
        ]
    );
}

/// A request with one group: the dive and the collection name as given,
/// the location of the sample dive, and two photos (one in the dive).
fn partial_request(
    collection: Option<&str>,
    dive: bool,
    location: bool,
) -> divelens_core::tags::CollectionsRequest {
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    let sample = sample_dive();
    let point = divelens_core::profile::PhotoPoint {
        elapsed_s: 160,
        depth_m: 12.0,
        temp_c: Some(25.0),
    };
    CollectionsRequest {
        search: 3,
        parent: Vec::new(),
        group: Some("Dive Lens".into()),
        groups: vec![CollectionGroup {
            collection: collection.map(Into::into),
            dive: dive.then(|| DiveTags::new(&sample, Units::default(), "Subsurface")),
            location: location.then(|| LocationTags::new(&sample.site, None, None)),
            photos: vec![PhotoTags::new(
                "1".into(),
                Some(point),
                240,
                Units::default(),
            )],
        }],
    }
}

/// Lua defining `partial()`: plans `input` and writes it on photo "1"
/// (found, nothing before it, no collection there). Returns the planned
/// groups and the writes, or nil and the reason.
const PARTIAL_LUA: &str = r#"
    local Json = require "Json"
    local Tagging = require "Tagging"
    function partial()
        local groups, place = Tagging.planCollections(Json.decode(input), function(iso) return iso end)
        if not groups then return nil, place end
        Tagging.resolvePhotos(groups, { ["1"] = {} })
        return groups, Tagging.collectionWrites(groups, {}, {})
    end
"#;

#[test]
fn a_group_without_a_dive_writes_only_the_location() {
    let lua = lua();
    let request = partial_request(Some("700 - Coral Garden"), false, true);
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    let fields: Vec<String> = lua
        .load(format!(
            r#"
            {PARTIAL_LUA}
            local groups, plan = partial()
            assert(groups, plan)
            -- Its own values (clock offset, depth) are ignored.
            assert(groups[1].photos[1].changes == nil)
            assert(not groups[1].writesDive)
            local out = {{}}
            for _, c in ipairs(plan.groups[1].steps[1].changes) do
                out[#out + 1] = c.kind .. " " .. c.field
            end
            for _, c in ipairs(plan.later[1].changes) do
                out[#out + 1] = c.kind .. " " .. c.field
            end
            return out
            "#
        ))
        .eval()
        .unwrap();
    assert!(
        fields.iter().all(|f| f.starts_with("raw ")),
        "a Dive Lens field written: {fields:?}"
    );
    let mut sorted = fields.clone();
    sorted.sort();
    assert!(sorted.contains(&"raw country".to_string()), "{fields:?}");
    assert!(sorted.contains(&"raw gps".to_string()), "{fields:?}");
    assert!(
        sorted
            .iter()
            .all(|f| ["raw country", "raw city", "raw location", "raw gps"].contains(&f.as_str())),
        "{fields:?}"
    );
}

#[test]
fn a_group_without_a_collection_creates_none() {
    let lua = lua();
    let request = partial_request(None, true, true);
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    let out = json_of(
        &lua,
        &format!(
            r#"
            {PARTIAL_LUA}
            local groups, plan = partial()
            assert(groups, plan)
            local r = plan.results[1]
            assert(r.collection == nil and r.created == false and r.count == 1)
            assert(#plan.groups[1].photos == 1)
            return Json.encode({{ writesDive = groups[1].writesDive,
                names = Tagging.namesCollections(groups) }})
            "#
        ),
    );
    assert_eq!(
        out,
        serde_json::json!({ "writesDive": true, "names": false })
    );
}

#[test]
fn a_group_with_nothing_is_refused() {
    let lua = lua();
    let request = partial_request(None, false, false);
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    let out: Vec<String> = lua
        .load(format!(
            r#"
            {PARTIAL_LUA}
            local groups, why = partial()
            return {{ tostring(groups), why }}
            "#
        ))
        .eval()
        .unwrap();
    assert_eq!(out, ["nil", "nothing to write"]);
}

#[test]
fn collections_write_gps_and_altitude_in_a_write_of_their_own() {
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    let dive = sample_dive();
    let point = divelens_core::profile::PhotoPoint {
        elapsed_s: 160,
        depth_m: 10.0,
        temp_c: Some(25.0),
    };
    let request = |site: &divelens_core::Site, altitude: bool| {
        let mut request = CollectionsRequest {
            search: 3,
            parent: Vec::new(),
            group: Some("Dive Lens".into()),
            groups: vec![CollectionGroup {
                collection: Some("700 - Coral Garden".into()),
                dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
                location: Some(LocationTags::new(site, None, None)),
                photos: vec![
                    PhotoTags::new("11".into(), Some(point), 241, Units::default()),
                    PhotoTags::new("12".into(), None, -30, Units::default()),
                ],
            }],
        };
        if altitude {
            request.copy_depth_to_altitude();
        }
        request
    };
    // Each photo's two writes, the first one counted (its lines are checked
    // by `writes`: the plan's, without the GPS and the altitude).
    let writes = |request: &CollectionsRequest| -> Vec<String> {
        let lua = lua();
        run(&lua, WRITES_LUA);
        lua.globals()
            .set("input", serde_json::to_string(request).unwrap())
            .unwrap();
        lua.load(
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            local groups = assert(Tagging.planCollections(Json.decode(input), function(iso) return iso end))
            local out = {}
            for _, p in ipairs(groups[1].photos) do
                local first = 0
                for _, l in ipairs(writes(groups[1].common, p.changes)) do
                    if l:sub(1, 2) == "1 " then
                        first = first + 1
                        assert(not l:find("gps"), l)
                    else
                        out[#out + 1] = p.id .. " " .. l
                    end
                end
                out[#out + 1] = p.id .. " first write: " .. first
            end
            return out
            "#,
        )
        .eval()
        .unwrap()
    };
    assert_eq!(
        writes(&request(&dive.site, true)),
        [
            "11 2 raw gps=12.5,-45.25",
            "11 2 raw gpsAltitude=-10",
            // 11 dive and location fields, 4 per-photo values, and the
            // altitude marker: in the first write, never with the altitude.
            "11 first write: 16",
            // At the surface: no altitude, no marker.
            "12 2 raw gps=12.5,-45.25",
            "12 first write: 15",
        ]
    );
    // Without site GPS nor the altitude option: no second write.
    let mut site = dive.site.clone();
    site.lat = None;
    site.lon = None;
    assert_eq!(
        writes(&request(&site, false)),
        ["11 first write: 15", "12 first write: 15"]
    );
    // The altitude alone.
    assert_eq!(
        writes(&request(&site, true)),
        [
            "11 2 raw gpsAltitude=-10",
            "11 first write: 16",
            "12 first write: 15"
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
fn collection_writes_step_every_requested_photo() {
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    use serde_json::json;
    let dive = sample_dive();
    let point = divelens_core::profile::PhotoPoint {
        elapsed_s: 160,
        depth_m: 10.0,
        temp_c: Some(25.0),
    };
    let group = |collection: &str, photos: Vec<PhotoTags>| CollectionGroup {
        collection: Some(collection.into()),
        dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
        photos,
    };
    let request = CollectionsRequest {
        search: 3,
        parent: Vec::new(),
        group: Some("Dive Lens".into()),
        groups: vec![
            group(
                "700 - Coral Garden",
                vec![
                    PhotoTags::new("11".into(), Some(point), 241, Units::default()),
                    PhotoTags::new("12".into(), None, -30, Units::default()),
                ],
            ),
            group(
                "701 - North Reef",
                vec![PhotoTags::new("21".into(), None, 0, Units::default())],
            ),
        ],
    };
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(&request).unwrap())
        .unwrap();
    let text: String = match lua
        .load(
            r#"
            local Json = require "Json"
            local Tagging = require "Tagging"
            local groups = assert(Tagging.planCollections(Json.decode(input), function(iso) return iso end))
            -- Only photo 11 is still in the catalog.
            local p11 = { id = "11" }
            local total, found = Tagging.resolvePhotos(groups, { ["11"] = p11 })
            assert(groups[1].photos[1].photo == p11 and groups[1].photos[2].photo == nil)
            local function ids(list)
                local out = {}
                for i, p in ipairs(list) do out[i] = p.id end
                return #out > 0 and out or Json.emptyArray
            end
            -- The writes as photo ids and numbers of changes.
            local function summary(plan)
                local out = { groups = {}, later = {}, results = plan.results }
                for gi, g in ipairs(plan.groups) do
                    local steps = {}
                    for i, s in ipairs(g.steps) do
                        steps[i] = s.photo and { photo = s.photo.id, changes = #s.changes } or Json.null
                    end
                    out.groups[gi] = { steps = steps, photos = ids(g.photos) }
                end
                for i, l in ipairs(plan.later) do out.later[i] = { photo = l.photo.id, changes = #l.changes } end
                return out
            end
            return Json.encode {
                total = total,
                found = ids(found),
                first = summary(Tagging.collectionWrites(groups, {}, { ["700 - Coral Garden"] = true })),
                second = summary(Tagging.collectionWrites(groups, {}, { ["701 - North Reef"] = true })),
            }
            "#,
        )
        .eval()
    {
        Ok(text) => text,
        Err(e) => panic!("{e}"),
    };
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    // One step per photo requested, found or not (progress counts them
    // all); a collection counts only the photos found.
    let groups = json!([
        {
            "steps": [{ "photo": "11", "changes": 15 }, null],
            "photos": ["11"],
        },
        { "steps": [null], "photos": [] },
    ]);
    let later = json!([{ "photo": "11", "changes": 1 }]);
    assert_eq!(
        value,
        json!({
            "total": 3,
            "found": ["11"],
            "first": {
                "groups": groups,
                "later": later,
                "results": [
                    { "collection": "700 - Coral Garden", "count": 1, "created": false },
                    { "collection": "701 - North Reef", "count": 0, "created": false },
                ],
            },
            // Created only with a photo found and the name not there.
            "second": {
                "groups": groups,
                "later": later,
                "results": [
                    { "collection": "700 - Coral Garden", "count": 1, "created": true },
                    { "collection": "701 - North Reef", "count": 0, "created": false },
                ],
            },
        })
    );
}

/// Runs Lua on `groups` (name, photo ids found) and returns the plan as JSON.
const GROUPS_LUA: &str = r#"
    local Json = require "Json"
    local Tagging = require "Tagging"
    local function group(name, photos)
        local list = {}
        for i, photo in ipairs(photos) do list[i] = { id = photo.id, photo = photo, changes = {} } end
        return { collection = name, common = {}, photos = list }
    end
"#;

#[test]
fn a_group_without_photo_found_is_not_created() {
    let out = json_of(
        &lua(),
        &format!(
            r#"{GROUPS_LUA}
        local plan = Tagging.collectionWrites({{ group("101 - Coral Garden", {{}}) }}, {{}}, {{}})
        return Json.encode(plan.results)"#
        ),
    );
    assert_eq!(
        out,
        serde_json::json!([{ "collection": "101 - Coral Garden", "count": 0, "created": false }])
    );
}

#[test]
fn a_second_group_of_the_same_new_name_is_completed() {
    let out = json_of(
        &lua(),
        &format!(
            r#"{GROUPS_LUA}
        local groups = {{
            group("101 - Coral Garden", {{ {{ id = "1" }} }}),
            group("101 - Coral Garden", {{ {{ id = "2" }} }}),
        }}
        local plan = Tagging.collectionWrites(groups, {{}}, {{}})
        return Json.encode(plan.results)"#
        ),
    );
    assert_eq!(
        out,
        serde_json::json!([
            { "collection": "101 - Coral Garden", "count": 1, "created": true },
            { "collection": "101 - Coral Garden", "count": 1, "created": false },
        ])
    );
}

#[test]
fn collection_writes_count_replaced_dive_values() {
    let out = json_of(
        &lua(),
        &format!(
            r#"{GROUPS_LUA}
        local other, same, bare = {{ id = "1" }}, {{ id = "2" }}, {{ id = "3" }}
        local common = {{
            {{ kind = "plugin", field = "Source", value = "Subsurface" }},
            {{ kind = "plugin", field = "StartDate", value = "2026-01-02T10:00:00" }},
            {{ kind = "plugin", field = "DiveId", value = "A" }},
        }}
        local g = group("101 - Coral Garden", {{ other, same, bare }})
        g.common = common
        -- Photo 1 carries another dive, photo 2 the same dive, photo 3 none.
        local before = {{
            [other] = {{ Source = "Subsurface", StartDate = "2026-01-01T09:00:00", DiveId = "B", PhotoDepth = "5" }},
            [same] = {{ Source = "Subsurface", StartDate = "2026-01-02T10:00:00", DiveId = "A", PhotoDepth = "5" }},
            [bare] = {{}},
        }}
        local plan = Tagging.collectionWrites({{ g }}, before, {{}})
        return Json.encode({{ cleared = plan.cleared }})"#
        ),
    );
    assert_eq!(out, serde_json::json!({ "cleared": 1 }));
}
