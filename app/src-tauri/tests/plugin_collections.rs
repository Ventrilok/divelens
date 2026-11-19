//! Tests of what "Create Collections" writes, run under Lua 5.1 (the version
//! Lightroom embeds).

mod common;

use common::{WRITES_LUA, lua, run, sample_dive};

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
            dive: DiveTags::new(&dive, Units::default(), "Subsurface"),
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
                collection: "700 - Coral Garden".into(),
                dive: DiveTags::new(&dive, Units::default(), "Subsurface"),
                location: LocationTags::new(site, None, None),
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
