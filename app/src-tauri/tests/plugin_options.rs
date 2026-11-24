//! Every combination of "what to write" options, from the request the core
//! builds to the writes the plug-in plans, under Lua 5.1: only what is ticked
//! is written, whichever of the two paths (collections, selection) is taken.

mod common;

use std::collections::BTreeSet;

use common::{json_of, lua, sample_dive};
use divelens_core::profile::PhotoPoint;
use divelens_core::tags::{
    CollectionGroup, CollectionPlace, CollectionsRequest, DiveTags, Kept, LocationTags, PhotoTags,
    TagRequest, Units,
};

/// The options, as bits 1..32: dive details, altitude copy, place, GPS,
/// collections.
struct Combination {
    dive: bool,
    altitude: bool,
    place: bool,
    gps: bool,
    collections: bool,
}

fn combinations() -> impl Iterator<Item = Combination> {
    (1u8..32).map(|bits| Combination {
        dive: bits & 1 != 0,
        altitude: bits & 2 != 0,
        place: bits & 4 != 0,
        gps: bits & 8 != 0,
        collections: bits & 16 != 0,
    })
}

impl Combination {
    fn kept(&self) -> Kept {
        Kept {
            dive: self.dive,
            place: self.place,
            gps: self.gps,
            collections: self.collections,
        }
    }

    /// The altitude is copied only with the dive details.
    fn copies_altitude(&self) -> bool {
        self.dive && self.altitude
    }

    fn label(&self) -> String {
        format!(
            "dive {} altitude {} place {} gps {} collections {}",
            self.dive, self.altitude, self.place, self.gps, self.collections
        )
    }
}

fn point() -> PhotoPoint {
    PhotoPoint {
        elapsed_s: 160,
        depth_m: 12.0,
        temp_c: Some(25.0),
    }
}

/// Lua defining `values(changes, id, out)`: adds "id field" to `out` for each
/// change that writes a value (not a clearing).
const VALUES_LUA: &str = r#"
    function values(changes, id, out)
        for _, c in ipairs(changes) do
            if c.value ~= nil then out[#out + 1] = id .. " " .. c.field end
        end
    end
"#;

fn written(out: &serde_json::Value) -> BTreeSet<String> {
    // An empty list is encoded as null by the plug-in's JSON.
    out["written"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn has(set: &BTreeSet<String>, entry: &str) -> bool {
    set.contains(entry)
}

/// Every combination of the "Create Collections" path: the request `keep`
/// leaves, planned by the plug-in, writes only what is ticked.
#[test]
fn every_option_combination_writes_only_what_is_ticked_catalogue() {
    let sample = sample_dive();
    let mut tested = 0;
    for c in combinations() {
        // One group, two photos: "1" at 12 m during the dive, "2" at the surface.
        let mut request = CollectionsRequest {
            search: 3,
            parent: Vec::new(),
            group: Some("Dive Lens".into()),
            groups: vec![CollectionGroup {
                collection: Some("700 - Coral Garden".into()),
                dive: Some(DiveTags::new(&sample, Units::default(), "Subsurface")),
                location: Some(LocationTags::new(&sample.site, None, None)),
                photos: vec![
                    PhotoTags::new("1".into(), Some(point()), 240, Units::default()),
                    PhotoTags::new("2".into(), None, 240, Units::default()),
                ],
            }],
        };
        if c.copies_altitude() {
            request.copy_depth_to_altitude();
        }
        request.keep(c.kept(), None);
        if request.is_empty() {
            // Only "altitude" ticked: the engine refuses it.
            assert!(
                !c.dive && !c.place && !c.gps && !c.collections,
                "{}",
                c.label()
            );
            continue;
        }
        tested += 1;
        let lua = lua();
        lua.globals()
            .set("input", serde_json::to_string(&request).unwrap())
            .unwrap();
        let out = json_of(
            &lua,
            &format!(
                r#"
                {VALUES_LUA}
                local Json = require "Json"
                local Tagging = require "Tagging"
                local groups = assert(Tagging.planCollections(Json.decode(input), function(iso) return iso end))
                Tagging.resolvePhotos(groups, {{ ["1"] = {{ id = "1" }}, ["2"] = {{ id = "2" }} }})
                local plan = Tagging.collectionWrites(groups, {{}}, {{}})
                local out = {{}}
                for _, step in ipairs(plan.groups[1].steps) do
                    values(step.changes, step.photo.id, out)
                end
                for _, later in ipairs(plan.later) do
                    values(later.changes, later.photo.id, out)
                end
                return Json.encode {{
                    written = out,
                    collection = plan.results[1].collection,
                    count = plan.results[1].count,
                }}
                "#
            ),
        );
        let w = written(&out);
        let l = c.label();
        assert_eq!(has(&w, "1 StartDate"), c.dive, "StartDate: {l}");
        assert_eq!(has(&w, "2 StartDate"), c.dive, "StartDate: {l}");
        assert_eq!(has(&w, "1 PhotoDepth"), c.dive, "PhotoDepth: {l}");
        assert!(!has(&w, "2 PhotoDepth"), "depth at the surface: {l}");
        assert_eq!(
            has(&w, "1 gpsAltitude"),
            c.copies_altitude(),
            "gpsAltitude: {l}"
        );
        assert_eq!(
            has(&w, "1 WrittenAltitude"),
            c.copies_altitude(),
            "WrittenAltitude: {l}"
        );
        assert!(!has(&w, "2 gpsAltitude"), "altitude at the surface: {l}");
        assert_eq!(has(&w, "1 country"), c.place, "country: {l}");
        assert_eq!(has(&w, "1 location"), c.place, "location: {l}");
        assert_eq!(has(&w, "1 gps"), c.gps, "gps: {l}");
        assert_eq!(has(&w, "2 gps"), c.gps, "gps: {l}");
        assert_eq!(
            out["collection"].is_string(),
            c.collections,
            "collection: {l}"
        );
        assert_eq!(out["count"], 2, "{l}");
    }
    assert_eq!(tested, 30);
}

/// Every combination of the "Tag Selection" path.
#[test]
fn every_option_combination_writes_only_what_is_ticked_selection() {
    let sample = sample_dive();
    let mut tested = 0;
    for c in combinations() {
        // The selection holds "p1", taken during the dive at 12 m, and "p2",
        // taken outside it: only "p1" is listed with per-photo values.
        let mut request = TagRequest {
            dive: Some(DiveTags::new(&sample, Units::default(), "Subsurface")),
            location: Some(LocationTags::new(&sample.site, None, None)),
            collection: Some(CollectionPlace {
                name: "700 - Coral Garden".into(),
                parent: vec!["Trips".into()],
                group: Some("Dive Lens".into()),
            }),
            selection: Some(12),
            photos: vec![PhotoTags::new(
                "p1".into(),
                Some(point()),
                240,
                Units::default(),
            )],
        };
        if c.copies_altitude() {
            request.copy_depth_to_altitude();
        }
        request.keep(c.kept());
        if request.is_empty() {
            assert!(
                !c.dive && !c.place && !c.gps && !c.collections,
                "{}",
                c.label()
            );
            continue;
        }
        tested += 1;
        let lua = lua();
        lua.globals()
            .set("input", serde_json::to_string(&request).unwrap())
            .unwrap();
        let out = json_of(
            &lua,
            &format!(
                r#"
                {VALUES_LUA}
                local Json = require "Json"
                local Tagging = require "Tagging"
                local request = Json.decode(input)
                local changes = assert(Tagging.plan(request, function(iso) return iso end))
                local perPhoto, why = Tagging.planPhotos(request)
                assert(perPhoto or not why, why)
                local place = Tagging.planCollection(request)
                local p1, p2 = {{ id = "p1" }}, {{ id = "p2" }}
                local plan = Tagging.tagWrites(changes, {{ p1, p2 }}, {{}}, perPhoto, function(p) return p.id end)
                local out, depthChanges = {{}}, {{}}
                for _, w in ipairs(plan.writes) do
                    values(w.changes, w.photo.id, out)
                    for _, ch in ipairs(w.changes) do
                        if ch.field == "PhotoDepth" then depthChanges[#depthChanges + 1] = w.photo.id end
                    end
                end
                for _, l in ipairs(plan.later) do
                    values(l.changes, l.photo.id, out)
                end
                return Json.encode {{
                    written = out,
                    depthChanges = depthChanges,
                    collection = place and place.name or Json.null,
                    withValues = plan.withValues,
                }}
                "#
            ),
        );
        let w = written(&out);
        let l = c.label();
        assert_eq!(has(&w, "p1 StartDate"), c.dive, "StartDate: {l}");
        assert_eq!(has(&w, "p2 StartDate"), c.dive, "StartDate: {l}");
        assert_eq!(has(&w, "p1 PhotoDepth"), c.dive, "PhotoDepth: {l}");
        assert!(
            !out["depthChanges"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .any(|id| id == "p2"),
            "the photo outside the dive got a PhotoDepth: {l}"
        );
        assert!(!has(&w, "p2 PhotoDepth"), "{l}");
        assert_eq!(
            has(&w, "p1 gpsAltitude"),
            c.copies_altitude(),
            "gpsAltitude: {l}"
        );
        assert_eq!(
            has(&w, "p1 WrittenAltitude"),
            c.copies_altitude(),
            "WrittenAltitude: {l}"
        );
        assert!(!has(&w, "p2 gpsAltitude"), "altitude outside the dive: {l}");
        assert_eq!(has(&w, "p1 country"), c.place, "country: {l}");
        assert_eq!(has(&w, "p1 location"), c.place, "location: {l}");
        assert_eq!(has(&w, "p1 gps"), c.gps, "gps: {l}");
        assert_eq!(has(&w, "p2 gps"), c.gps, "gps: {l}");
        assert_eq!(
            out["collection"].is_string(),
            c.collections,
            "collection: {l}"
        );
        assert_eq!(out["withValues"], u8::from(c.dive), "{l}");
    }
    assert_eq!(tested, 30);
}
