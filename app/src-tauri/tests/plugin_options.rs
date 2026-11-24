//! Every combination of "what to write" options, from the request the core
//! builds to the writes the plug-in plans, under Lua 5.1: only what is ticked
//! is written, whichever of the two paths (collections, selection) is taken.

mod common;

use common::{json_of, lua, sample_dive};
use divelens_core::Dive;
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

/// Lua defining `record(changes, id, out)`: adds "id field written" or
/// "id field cleared" to `out` for every change, a clearing (nil value)
/// included, and `BEFORE`, what photo "1" carries before the write: another
/// dive, with its own values and an altitude marker matching its altitude.
const RECORD_LUA: &str = r#"
    function record(changes, id, out)
        for _, c in ipairs(changes) do
            out[#out + 1] = id .. " " .. c.field .. (c.value ~= nil and " written" or " cleared")
        end
    end
    BEFORE = {
        Source = "MacDive", StartDate = "2025-12-30T08:00:00", DiveId = "699",
        PhotoDepth = "3.0 m", PhotoTemp = "20.0°C", PhotoElapsed = "01:00",
        ClockOffset = "+00:00", WrittenAltitude = "-3", gpsAltitude = -3,
    }
"#;

/// Where the two photos are, for the expectation.
#[derive(Clone, Copy)]
enum Path {
    /// "Create Collections": photo 2 is at the surface, in the dive's group.
    Catalogue,
    /// "Tag Selection": photo 2 was taken outside the dive.
    Selection,
}

/// Every write ("photo field written|cleared") the options must give, the
/// sample dive having no low temperature and no rating, and photo 1 (12 m,
/// carrying another dive) being the only photo with a depth.
fn expected(c: &Combination, path: Path, p1: &str, p2: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut add =
        |photo: &str, field: &str, state: &str| out.push(format!("{photo} {field} {state}"));
    for photo in [p1, p2] {
        if c.dive {
            // Dive details ticked: the dive's fields on both photos; what the
            // dive has not (low temperature, rating) is cleared.
            for field in [
                "DiveId",
                "StartDate",
                "EndDate",
                "Duration",
                "MaxDepth",
                "TempHigh",
                "Source",
            ] {
                add(photo, field, "written");
            }
            for field in ["TempLow", "Rating"] {
                add(photo, field, "cleared");
            }
        }
        if c.place {
            // Place ticked: the site's country and name (it has no city).
            add(photo, "country", "written");
            add(photo, "location", "written");
        }
        if c.gps {
            // GPS ticked: the site's position.
            add(photo, "gps", "written");
        }
    }
    if c.dive {
        // Photo 1, taken during the dive, gets its own values, replacing
        // those of the other dive it carried.
        for field in ["PhotoDepth", "PhotoTemp", "PhotoElapsed", "ClockOffset"] {
            add(p1, field, "written");
        }
        if c.copies_altitude() {
            // Altitude ticked with the dive: the depth copied, with its marker.
            add(p1, "gpsAltitude", "written");
            add(p1, "WrittenAltitude", "written");
        } else {
            // No new altitude: the other dive's own altitude and marker go.
            add(p1, "gpsAltitude", "cleared");
            add(p1, "WrittenAltitude", "cleared");
        }
        if let Path::Catalogue = path {
            // A photo at the surface of the dive's group: its clock offset
            // written, its per-photo values written empty.
            for field in ["PhotoDepth", "PhotoTemp", "PhotoElapsed"] {
                add(p2, field, "cleared");
            }
            add(p2, "ClockOffset", "written");
        }
    }
    // Dive details unticked: no Dive Lens field at all, and nothing cleared
    // on photo 1 (the other dive's values stay) - nothing more is added.
    out.sort();
    out
}

fn recorded(out: &serde_json::Value) -> Vec<String> {
    // An empty list is encoded as null by the plug-in's JSON.
    let mut list: Vec<String> = out["written"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    list.sort();
    list
}

/// The "Create Collections" request for `dive`: one group, two photos, "1"
/// at 12 m during the dive, "2" at the surface.
fn catalogue_request(dive: &Dive) -> CollectionsRequest {
    CollectionsRequest {
        search: 3,
        parent: Vec::new(),
        group: Some("Dive Lens".into()),
        groups: vec![CollectionGroup {
            collection: Some("700 - Coral Garden".into()),
            dive: Some(DiveTags::new(dive, Units::default(), "Subsurface")),
            location: Some(LocationTags::new(&dive.site, None, None)),
            photos: vec![
                PhotoTags::new("1".into(), Some(point()), 240, Units::default()),
                PhotoTags::new("2".into(), None, 240, Units::default()),
            ],
        }],
    }
}

/// What the plug-in writes for a "Create Collections" request: every
/// change recorded (`written`), the group's collection and photo count.
fn planned_catalogue(request: &CollectionsRequest) -> serde_json::Value {
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(request).unwrap())
        .unwrap();
    json_of(
        &lua,
        &format!(
            r#"
            {RECORD_LUA}
            local Json = require "Json"
            local Tagging = require "Tagging"
            local groups = assert(Tagging.planCollections(Json.decode(input), function(iso) return iso end))
            local p1, p2 = {{ id = "1" }}, {{ id = "2" }}
            Tagging.resolvePhotos(groups, {{ ["1"] = p1, ["2"] = p2 }})
            local plan = Tagging.collectionWrites(groups, {{ [p1] = BEFORE }}, {{}})
            local out = {{}}
            for _, step in ipairs(plan.groups[1].steps) do
                record(step.changes, step.photo.id, out)
            end
            for _, later in ipairs(plan.later) do
                record(later.changes, later.photo.id, out)
            end
            return Json.encode {{
                written = out,
                collection = plan.results[1].collection,
                count = plan.results[1].count,
            }}
            "#
        ),
    )
}

/// The "Tag Selection" request for `dive`: the selection holds "p1", taken
/// during the dive at 12 m, and "p2", taken outside it: only "p1" is listed
/// with per-photo values.
fn selection_request(dive: &Dive) -> TagRequest {
    TagRequest {
        dive: Some(DiveTags::new(dive, Units::default(), "Subsurface")),
        location: Some(LocationTags::new(&dive.site, None, None)),
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
    }
}

/// What the plug-in writes for a "Tag Selection" request on "p1" and "p2":
/// every change recorded (`written`), the collection and the number of
/// photos with their own values.
fn planned_selection(request: &TagRequest) -> serde_json::Value {
    let lua = lua();
    lua.globals()
        .set("input", serde_json::to_string(request).unwrap())
        .unwrap();
    json_of(
        &lua,
        &format!(
            r#"
            {RECORD_LUA}
            local Json = require "Json"
            local Tagging = require "Tagging"
            local request = Json.decode(input)
            local changes = assert(Tagging.plan(request, function(iso) return iso end))
            local perPhoto, why = Tagging.planPhotos(request)
            assert(perPhoto or not why, why)
            local place = Tagging.planCollection(request)
            local p1, p2 = {{ id = "p1" }}, {{ id = "p2" }}
            local plan = Tagging.tagWrites(changes, {{ p1, p2 }}, {{ [p1] = BEFORE }}, perPhoto, function(p) return p.id end)
            local out = {{}}
            for _, w in ipairs(plan.writes) do record(w.changes, w.photo.id, out) end
            for _, l in ipairs(plan.later) do record(l.changes, l.photo.id, out) end
            return Json.encode {{
                written = out,
                collection = place and place.name or Json.null,
                withValues = plan.withValues,
            }}
            "#
        ),
    )
}

/// Every combination of the "Create Collections" path: the request `keep`
/// leaves, planned by the plug-in, writes exactly what is ticked.
#[test]
fn every_option_combination_writes_only_what_is_ticked_catalogue() {
    let sample = sample_dive();
    let mut tested = 0;
    for c in combinations() {
        let mut request = catalogue_request(&sample);
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
        let out = planned_catalogue(&request);
        let l = c.label();
        assert_eq!(
            recorded(&out),
            expected(&c, Path::Catalogue, "1", "2"),
            "{l}"
        );
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
        let mut request = selection_request(&sample);
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
        let out = planned_selection(&request);
        let l = c.label();
        let got = recorded(&out);
        assert!(
            !got.iter()
                .any(|w| w.starts_with("p2 Photo") || w.starts_with("p2 ClockOffset")),
            "the photo outside the dive got a per-photo value: {l}"
        );
        assert_eq!(got, expected(&c, Path::Selection, "p1", "p2"), "{l}");
        assert_eq!(
            out["collection"].is_string(),
            c.collections,
            "collection: {l}"
        );
        assert_eq!(out["withValues"], u8::from(c.dive), "{l}");
    }
    assert_eq!(tested, 30);
}

/// A site with no country, but a name and a position, with the place
/// ticked: the country is neither written nor cleared (a country set by
/// hand stays), every other field is written as usual, in both paths.
#[test]
fn a_site_without_a_country_leaves_the_country_alone() {
    let mut sample = sample_dive();
    sample.site.country = None;
    // What a site with a country writes, without its country.
    let without_country = |mut writes: Vec<String>| {
        writes.retain(|w| !w.ends_with(" country written"));
        writes
    };
    let untouched = |writes: &[String]| !writes.iter().any(|w| w.contains(" country "));
    let mut tested = 0;
    for c in combinations().filter(|c| c.place) {
        let l = c.label();

        let mut catalogue = catalogue_request(&sample);
        if c.copies_altitude() {
            catalogue.copy_depth_to_altitude();
        }
        catalogue.keep(c.kept(), None);
        let got = recorded(&planned_catalogue(&catalogue));
        assert!(untouched(&got), "catalogue, country touched: {l}");
        for photo in ["1", "2"] {
            assert!(got.contains(&format!("{photo} location written")), "{l}");
            assert_eq!(got.contains(&format!("{photo} gps written")), c.gps, "{l}");
        }
        let want = without_country(expected(&c, Path::Catalogue, "1", "2"));
        assert_eq!(got, want, "catalogue: {l}");

        let mut selection = selection_request(&sample);
        if c.copies_altitude() {
            selection.copy_depth_to_altitude();
        }
        selection.keep(c.kept());
        let got = recorded(&planned_selection(&selection));
        assert!(untouched(&got), "selection, country touched: {l}");
        for photo in ["p1", "p2"] {
            assert!(got.contains(&format!("{photo} location written")), "{l}");
            assert_eq!(got.contains(&format!("{photo} gps written")), c.gps, "{l}");
        }
        let want = without_country(expected(&c, Path::Selection, "p1", "p2"));
        assert_eq!(got, want, "selection: {l}");
        tested += 1;
    }
    assert_eq!(tested, 16);
}
