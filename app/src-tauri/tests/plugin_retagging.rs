//! Tests of what "Tag Selection" and "Create Collections" clear or keep on a
//! photo that already carries Dive Lens values, run under Lua 5.1 (the version
//! Lightroom embeds).

mod common;

use common::{WRITES_LUA, lua, run, sample_dive};
use mlua::Lua;

/// What a photo carries before "Tag Selection" or "Create Collections", as
/// the plugin's own write path left it.
enum Before<'a> {
    Nothing,
    Tagged(&'a divelens_core::tags::TagRequest),
    Filed(&'a divelens_core::tags::CollectionsRequest),
    /// Filed by a version that wrote no altitude marker.
    FiledEarlier(&'a divelens_core::tags::CollectionsRequest),
}

/// Lua making the global `current`, what Link.lua reads on photo "1"
/// before writing (photosBefore), from the globals `set_before` sets. It
/// also defines the global `date`, the date format of these tests.
const BEFORE_LUA: &str = r#"
local Json = require "Json"
local Tagging = require "Tagging"
function date(iso) return "date(" .. iso .. ")" end
local photo = {}
local function write(changes)
    for _, c in ipairs(changes) do photo[c.field] = c.value end
end
if tagged then write(assert(Tagging.plan(Json.decode(tagged), date))) end
if filed then
    local groups = assert(Tagging.planCollections(Json.decode(filed), date))
    write(groups[1].common)
    write(groups[1].photos[1].changes)
end
if earlier then photo[Tagging.MARKER] = nil end
if altitude then photo.gpsAltitude = altitude end
current = { gpsAltitude = photo.gpsAltitude }
for _, field in ipairs(Tagging.RETAG_FIELDS) do current[field] = photo[field] end
"#;

/// Sets the globals BEFORE_LUA reads: what the photo carries (`before`),
/// and its GPS altitude when `altitude` replaces it (the camera's, the
/// user's, or the value as Lightroom stored it).
fn set_before(lua: &Lua, before: Before, altitude: Option<f64>) {
    let globals = lua.globals();
    match before {
        Before::Nothing => {}
        Before::Tagged(r) => globals
            .set("tagged", serde_json::to_string(r).unwrap())
            .unwrap(),
        Before::Filed(r) => globals
            .set("filed", serde_json::to_string(r).unwrap())
            .unwrap(),
        Before::FiledEarlier(r) => {
            globals
                .set("filed", serde_json::to_string(r).unwrap())
                .unwrap();
            globals.set("earlier", true).unwrap();
        }
    }
    globals.set("altitude", altitude).unwrap();
    run(lua, BEFORE_LUA);
}

/// The extra changes "Tag Selection" makes on a photo left by `before` when
/// it writes `request`, as "kind field=value" lines. `altitude` replaces the
/// photo's GPS altitude when given (the camera's, or the value as Lightroom
/// stored it).
fn retag(
    before: Before,
    altitude: Option<f64>,
    request: &divelens_core::tags::TagRequest,
) -> Vec<String> {
    retag_lua(before, altitude, request, false)
}

/// Everything "Tag Selection" writes on a photo left by `before` (see
/// `retag`), in its two catalog writes (see `WRITES_LUA`).
fn retag_writes(
    before: Before,
    altitude: Option<f64>,
    request: &divelens_core::tags::TagRequest,
) -> Vec<String> {
    retag_lua(before, altitude, request, true)
}

fn retag_lua(
    before: Before,
    altitude: Option<f64>,
    request: &divelens_core::tags::TagRequest,
    split: bool,
) -> Vec<String> {
    let lua = lua();
    run(&lua, WRITES_LUA);
    set_before(&lua, before, altitude);
    let globals = lua.globals();
    globals.set("split", split).unwrap();
    globals
        .set("input", serde_json::to_string(request).unwrap())
        .unwrap();
    lua.load(
        r#"
        local Json = require "Json"
        local Tagging = require "Tagging"
        local request = Json.decode(input)
        local changes = assert(Tagging.plan(request, date))
        -- The photo is "1": its per-photo values when the request lists it
        -- (taken during the dive).
        local perPhoto, bad = Tagging.planPhotos(request)
        assert(not bad, bad)
        local own = perPhoto and perPhoto.photos["1"]
        -- What Link.lua writes, as it makes it.
        if split then return checked(Tagging.photoWrites(changes, current, own)) end
        assert(not own, "what the other dive leaves, without per-photo values only")
        local extra = Tagging.retagChanges(current, changes)
        local lines = {}
        for i, c in ipairs(extra) do
            lines[i] = ("%s %s=%s"):format(c.kind, c.field, tostring(c.value))
        end
        return lines
        "#,
    )
    .eval()
    .unwrap()
}

/// Everything "Create Collections" writes, with `request`, on photo "1",
/// left by `before` (see `retag`), in its two catalog writes (see
/// `WRITES_LUA`).
fn refile_writes(
    before: Before,
    altitude: Option<f64>,
    request: &divelens_core::tags::CollectionsRequest,
) -> Vec<String> {
    let lua = lua();
    run(&lua, WRITES_LUA);
    set_before(&lua, before, altitude);
    lua.globals()
        .set("input", serde_json::to_string(request).unwrap())
        .unwrap();
    lua.load(
        r#"
        local Json = require "Json"
        local Tagging = require "Tagging"
        local groups = assert(Tagging.planCollections(Json.decode(input), date))
        local p = groups[1].photos[1]
        assert(p.id == "1")
        -- What Link.lua writes, as it makes it.
        return checked(Tagging.photoWrites(groups[1].common, current, p.changes))
        "#,
    )
    .eval()
    .unwrap()
}

use divelens_core::Dive;
use divelens_core::profile::PhotoPoint;
use divelens_core::tags::{
    CollectionGroup, CollectionsRequest, DepthUnit, DiveTags, LocationTags, PhotoTags, TagRequest,
    TempUnit, Units,
};

const CLEARED: [&str; 4] = [
    "plugin PhotoDepth=nil",
    "plugin PhotoTemp=nil",
    "plugin PhotoElapsed=nil",
    "plugin ClockOffset=nil",
];

const MARKER_CLEARED: &str = "plugin WrittenAltitude=nil";

/// The other dive's per-photo values cleared, then its GPS altitude
/// when `altitude`, then the altitude marker when `marker`.
fn cleared(altitude: bool, marker: bool) -> Vec<String> {
    let mut lines: Vec<String> = CLEARED.iter().map(|s| s.to_string()).collect();
    if altitude {
        lines.push("raw gpsAltitude=nil".into());
    }
    if marker {
        lines.push(MARKER_CLEARED.into());
    }
    lines
}

const FEET: Units = Units {
    depth: DepthUnit::Feet,
    temp: TempUnit::Fahrenheit,
};

/// The dive after the sample one, the same day.
fn next_dive() -> Dive {
    let mut d = sample_dive();
    d.id = "B".into();
    d.number = Some(701);
    d.start += chrono::Duration::hours(3);
    d.end += chrono::Duration::hours(3);
    d
}

/// A photo filed by "Create Collections" in `dive`, at `depth_m` (none:
/// at the surface), its depth copied to the altitude.
fn filed(dive: &Dive, source: &str, depth_m: Option<f64>, units: Units) -> CollectionsRequest {
    filing(dive, source, depth_m, units, true)
}

/// "Create Collections" filing photo "1" in `dive` at `depth_m` (none:
/// at the surface, or in a dive without a profile), its depth copied to
/// the altitude when `copy`.
fn filing(
    dive: &Dive,
    source: &str,
    depth_m: Option<f64>,
    units: Units,
    copy: bool,
) -> CollectionsRequest {
    let point = depth_m.map(|depth_m| PhotoPoint {
        elapsed_s: 600,
        depth_m,
        temp_c: Some(26.5),
    });
    let mut request = CollectionsRequest {
        search: 1,
        parent: Vec::new(),
        group: None,
        groups: vec![CollectionGroup {
            collection: "700 - Coral Garden".into(),
            dive: DiveTags::new(dive, units, source),
            location: LocationTags::new(&dive.site, None, None),
            photos: vec![PhotoTags::new("1".into(), point, 240, units)],
        }],
    };
    if copy {
        request.copy_depth_to_altitude();
    }
    request
}

fn tag(dive: &Dive, source: &str) -> TagRequest {
    TagRequest {
        dive: Some(DiveTags::new(dive, Units::default(), source)),
        location: Some(LocationTags::new(&dive.site, None, None)),
        ..TagRequest::default()
    }
}

#[test]
fn another_dive_clears_the_photos_own_values() {
    let a = sample_dive();
    let photo = filed(&a, "MacDive", Some(18.44), Units::default());
    // The altitude Dive Lens wrote (its marker says so): cleared, with
    // the marker.
    assert_eq!(
        retag(Before::Filed(&photo), None, &tag(&next_dive(), "MacDive")),
        cleared(true, true)
    );
    // Changed since (the user, another tool): the altitude stays, the
    // marker, no longer true, goes.
    assert_eq!(
        retag(
            Before::Filed(&photo),
            Some(12.0),
            &tag(&next_dive(), "MacDive")
        ),
        cleared(false, true)
    );
    // Only the source differs: another dive (the same log in another
    // application may have other start times).
    assert_eq!(
        retag(Before::Filed(&photo), None, &tag(&a, "Subsurface")),
        cleared(true, true)
    );
    // Only the number differs, or one side has none.
    let mut renumbered = a.clone();
    renumbered.number = Some(702);
    assert_eq!(
        retag(Before::Filed(&photo), None, &tag(&renumbered, "MacDive")),
        cleared(true, true)
    );
    renumbered.number = None;
    assert_eq!(
        retag(Before::Filed(&photo), None, &tag(&renumbered, "MacDive")),
        cleared(true, true)
    );
    // Only the values the photo has are cleared. Tagged earlier by "Tag
    // Selection": none, so nothing to clear, and the altitude stays even
    // when it equals a depth Dive Lens could have written.
    let tagged = tag(&a, "MacDive");
    assert_eq!(
        retag(
            Before::Tagged(&tagged),
            Some(-18.4),
            &tag(&next_dive(), "MacDive")
        ),
        Vec::<String>::new()
    );
    // Filed in the margin: only a clock offset, no depth.
    let surface = filed(&a, "MacDive", None, Units::default());
    assert_eq!(
        retag(
            Before::Filed(&surface),
            Some(-18.4),
            &tag(&next_dive(), "MacDive")
        ),
        ["plugin ClockOffset=nil"]
    );
    // 18.45 m is written "18.4 m" but its altitude is -18.5: the marker
    // tells it is Dive Lens's, so it is cleared.
    let half = filed(&a, "MacDive", Some(18.45), Units::default());
    assert_eq!(
        retag(Before::Filed(&half), None, &tag(&next_dive(), "MacDive")),
        cleared(true, true)
    );
    // In feet too: "60 ft" is 18.29 m, the altitude written -18.4.
    let feet = filed(&a, "MacDive", Some(18.44), FEET);
    assert_eq!(
        retag(Before::Filed(&feet), None, &tag(&next_dive(), "MacDive")),
        cleared(true, true)
    );
    // And as Lightroom may store the altitude (a float).
    assert_eq!(
        retag(
            Before::Filed(&feet),
            Some(f64::from(-18.4f32)),
            &tag(&next_dive(), "MacDive")
        ),
        cleared(true, true)
    );
}

#[test]
fn without_a_marker_the_altitude_derived_from_the_depth_is_cleared() {
    // Filed by a version without the altitude marker: the old rule, the
    // altitude derived from the depth written, to the decimetre.
    let a = sample_dive();
    let next = tag(&next_dive(), "MacDive");
    let photo = filed(&a, "MacDive", Some(18.44), Units::default());
    assert_eq!(
        retag(Before::FiledEarlier(&photo), None, &next),
        cleared(true, false)
    );
    // The camera's altitude: kept.
    assert_eq!(
        retag(Before::FiledEarlier(&photo), Some(12.0), &next),
        cleared(false, false)
    );
    // Known misses, kept on purpose: 18.45 m is written "18.4 m" but its
    // altitude is -18.5; "60 ft" gives back -18.3, not -18.4.
    let half = filed(&a, "MacDive", Some(18.45), Units::default());
    assert_eq!(
        retag(Before::FiledEarlier(&half), None, &next),
        cleared(false, false)
    );
    let feet = filed(&a, "MacDive", Some(18.44), FEET);
    assert_eq!(
        retag(Before::FiledEarlier(&feet), None, &next),
        cleared(false, false)
    );
}

#[test]
fn the_altitude_marker_decides_whose_the_altitude_is() {
    run(
        &lua(),
        r#"
        local Tagging = require "Tagging"
        local function clears(photo)
            local out = {}
            for _, c in ipairs(Tagging.altitudeClears(photo)) do
                assert(c.value == nil, c.field)
                out[#out + 1] = c.kind .. " " .. c.field
            end
            return table.concat(out, ";")
        end
        local function check(photo, want, why)
            local got = clears(photo)
            assert(got == want, why .. ": " .. got)
        end
        local BOTH, MARKER = "raw gpsAltitude;plugin WrittenAltitude", "plugin WrittenAltitude"

        -- With a marker, it alone decides, whatever the depth.
        check({ gpsAltitude = -18.4, WrittenAltitude = "-18.4", PhotoDepth = "60 ft" }, BOTH, "own")
        check({ gpsAltitude = -18.399999618530273, WrittenAltitude = "-18.4" }, BOTH, "as a float")
        check({ gpsAltitude = -18.404, WrittenAltitude = "-18.4" }, BOTH, "to the centimetre")
        check({ gpsAltitude = 0, WrittenAltitude = "0" }, BOTH, "at the surface")
        check({ gpsAltitude = -18.41, WrittenAltitude = "-18.4" }, MARKER, "a centimetre off")
        check({ gpsAltitude = -18.3, WrittenAltitude = "-18.4", PhotoDepth = "18.3 m" }, MARKER,
            "changed to what the depth gives")
        check({ gpsAltitude = 18.4, WrittenAltitude = "-18.4" }, MARKER, "sign changed")
        check({ WrittenAltitude = "-18.4", PhotoDepth = "18.4 m" }, MARKER, "altitude removed")
        check({ gpsAltitude = -18.4, WrittenAltitude = "deep", PhotoDepth = "18.4 m" }, MARKER,
            "not a number")
        -- Without a marker (an empty one is none): the old rule.
        check({ gpsAltitude = -18.4, PhotoDepth = "18.4 m" }, "raw gpsAltitude", "derived")
        check({ gpsAltitude = -18.4, WrittenAltitude = "", PhotoDepth = "18.4 m" }, "raw gpsAltitude",
            "empty marker")
        check({ gpsAltitude = -18.4, PhotoDepth = "60 ft" }, "", "feet, rounded")
        check({ gpsAltitude = -3 }, "", "camera, no depth")
        check({}, "", "nothing")

        -- The marker's text: the altitude written, read back exactly.
        for _, a in ipairs { -18.4, -10, 0, -0.1, -123.4, -18.45 } do
            assert(tonumber(Tagging.markerText(a)) == a, a)
        end
        assert(Tagging.markerText(-18.4) == "-18.4" and Tagging.markerText(-10) == "-10")
        "#,
    );
}

#[test]
fn the_altitude_cleared_goes_in_the_gps_write() {
    let a = sample_dive();
    let photo = filed(&a, "MacDive", Some(18.44), Units::default());
    let part = |lines: &[String], write: &str| -> Vec<String> {
        lines
            .iter()
            .filter_map(|l| l.strip_prefix(write).map(str::to_string))
            .collect()
    };
    let lines = retag_writes(Before::Filed(&photo), None, &tag(&next_dive(), "MacDive"));
    // Second write: the new site's GPS, then the old dive's altitude.
    assert_eq!(
        part(&lines, "2 "),
        ["raw gps=12.5,-45.25", "raw gpsAltitude=nil"]
    );
    // First write: the request without its GPS, then the other dive's
    // per-photo values and the altitude marker cleared.
    let first = part(&lines, "1 ");
    assert!(first.contains(&"raw country=Atlantis".to_string()));
    assert_eq!(first[first.len() - 5..], cleared(false, true));
    // Dive data only: the altitude alone in the second write.
    let dive_only = TagRequest {
        dive: Some(DiveTags::new(&next_dive(), Units::default(), "MacDive")),
        location: None,
        ..TagRequest::default()
    };
    let lines = retag_writes(Before::Filed(&photo), None, &dive_only);
    assert_eq!(part(&lines, "2 "), ["raw gpsAltitude=nil"]);
    assert_eq!(part(&lines, "1 ")[9..], cleared(false, true));
    // An altitude changed since stays: only the GPS in the second
    // write, the marker cleared in the first.
    let lines = retag_writes(
        Before::Filed(&photo),
        Some(12.0),
        &tag(&next_dive(), "MacDive"),
    );
    assert_eq!(part(&lines, "2 "), ["raw gps=12.5,-45.25"]);
    assert_eq!(
        part(&lines, "1 ").last().map(String::as_str),
        Some(MARKER_CLEARED)
    );
    // Same dive: nothing cleared, the GPS still on its own.
    let lines = retag_writes(Before::Filed(&photo), None, &tag(&a, "MacDive"));
    assert_eq!(part(&lines, "2 "), ["raw gps=12.5,-45.25"]);
    // 9 dive fields, country and location.
    assert_eq!(part(&lines, "1 ").len(), 11);
}

#[test]
fn same_dive_location_only_and_untagged_photos_touch_nothing() {
    let a = sample_dive();
    let photo = filed(&a, "MacDive", Some(18.44), Units::default());
    let none: Vec<String> = Vec::new();
    assert_eq!(
        retag(Before::Filed(&photo), None, &tag(&a, "MacDive")),
        none
    );
    // Same dive, edited in the log since: still the same dive.
    let mut edited = a.clone();
    edited.rating = Some(5);
    edited.max_depth_m = Some(32.0);
    assert_eq!(
        retag(Before::Filed(&photo), None, &tag(&edited, "MacDive")),
        none
    );
    // No number on either side counts as the same number.
    let mut unnumbered = a.clone();
    unnumbered.number = None;
    let photo_unnumbered = filed(&unnumbered, "MacDive", Some(18.44), Units::default());
    assert_eq!(
        retag(
            Before::Filed(&photo_unnumbered),
            None,
            &tag(&unnumbered, "MacDive")
        ),
        none
    );
    // Location only ("Dive data" unchecked), even another dive's site.
    let location_only = TagRequest {
        dive: None,
        location: Some(LocationTags::new(&next_dive().site, None, None)),
        ..TagRequest::default()
    };
    assert_eq!(retag(Before::Filed(&photo), None, &location_only), none);
    // A photo without dive data: nothing to clear, its altitude kept.
    assert_eq!(
        retag(Before::Nothing, Some(-18.4), &tag(&next_dive(), "MacDive")),
        none
    );
    assert_eq!(
        retag(
            Before::Tagged(&location_only),
            None,
            &tag(&next_dive(), "MacDive")
        ),
        none
    );
}

#[test]
fn altitude_cleared_only_when_dive_lens_derived_it() {
    use divelens_core::tags::{DepthUnit, altitude, depth};
    let metres = |m| Some(depth(m, DepthUnit::Metres));
    let feet = |m| Some(depth(m, DepthUnit::Feet));
    // Rounding to the foot: both are written "60 ft", only the first
    // gives back the altitude written from the exact depth.
    assert_eq!(feet(18.288), feet(18.44));
    assert_eq!(feet(18.44).as_deref(), Some("60 ft"));
    assert_eq!(feet(30.48).as_deref(), Some("100 ft"));
    assert_eq!(feet(9.2).as_deref(), Some("30 ft"));
    // The known metres miss: the text rounds down, the altitude up.
    assert_eq!(metres(18.45).as_deref(), Some("18.4 m"));
    assert_eq!(altitude(18.45), -18.5);
    let cases: [(Option<String>, Option<f64>, bool); 17] = [
        (metres(18.44), Some(altitude(18.44)), true),
        // As Lightroom may store it.
        (metres(18.44), Some(altitude(18.44) as f32 as f64), true),
        (metres(18.44), Some(-18.5), false),
        (metres(18.44), Some(-18.3), false),
        (metres(18.44), Some(18.4), false),
        (metres(18.45), Some(altitude(18.45)), false),
        (metres(0.02), Some(altitude(0.02)), true),
        (feet(18.288), Some(altitude(18.288)), true),
        (feet(18.44), Some(altitude(18.44)), false),
        // "100 ft" gives back -30.5; "30 ft" gives -9.1, not -9.2.
        (feet(30.48), Some(altitude(30.48)), true),
        (feet(9.2), Some(altitude(9.2)), false),
        // Before units: a number, in metres.
        (Some("18.4".into()), Some(altitude(18.4)), true),
        (Some(String::new()), Some(-18.4), false),
        (None, Some(-18.4), false),
        (Some("deep".into()), Some(-18.4), false),
        (Some("18,4 m".into()), Some(-18.4), false),
        (metres(18.44), None, false),
    ];
    let lua = lua();
    let derived: mlua::Function = lua
        .load(r#"return require("Tagging").isDerivedAltitude"#)
        .eval()
        .unwrap();
    for (photo_depth, alt, want) in cases {
        let got: bool = derived.call((alt, photo_depth.clone())).unwrap();
        assert_eq!(got, want, "{photo_depth:?} {alt:?}");
    }
}

/// "Tag Selection" of `dive` with per-photo values: photo "1" listed
/// when `inside` (taken during the dive), its depth copied to the
/// altitude when `altitude`.
fn tag_with_values(dive: &Dive, inside: bool, altitude: bool) -> TagRequest {
    let point = PhotoPoint {
        elapsed_s: 160,
        depth_m: 10.0,
        temp_c: Some(25.0),
    };
    let mut request = TagRequest {
        selection: Some(5),
        photos: if inside {
            vec![PhotoTags::new(
                "1".into(),
                Some(point),
                241,
                Units::default(),
            )]
        } else {
            Vec::new()
        },
        ..tag(dive, "MacDive")
    };
    if altitude {
        request.copy_depth_to_altitude();
    }
    request
}

const OWN: [&str; 4] = [
    "plugin PhotoDepth=10.0 m",
    "plugin PhotoTemp=25.0°C",
    "plugin PhotoElapsed=02:40",
    "plugin ClockOffset=+04:01",
];

/// OWN, with the altitude marker of the altitude copied from its depth.
const OWN_ALT: [&str; 5] = [
    "plugin PhotoDepth=10.0 m",
    "plugin PhotoTemp=25.0°C",
    "plugin PhotoElapsed=02:40",
    "plugin ClockOffset=+04:01",
    "plugin WrittenAltitude=-10",
];

/// The first write's lines after the 9 dive fields, country and
/// location, and the second write's lines.
fn tail_and_second(lines: &[String]) -> (Vec<String>, Vec<String>) {
    let first: Vec<String> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("1 ").map(str::to_string))
        .collect();
    assert_eq!(
        first[9..11],
        ["raw country=Atlantis", "raw location=Coral Garden"]
    );
    let second = lines
        .iter()
        .filter_map(|l| l.strip_prefix("2 ").map(str::to_string))
        .collect();
    (first[11..].to_vec(), second)
}

#[test]
fn per_photo_values_in_and_outside_the_dive_same_or_other_dive() {
    let a = sample_dive();
    // Filed by Create Collections in dive A at 18.44 m, its depth copied
    // to the altitude (-18.4).
    let photo = filed(&a, "MacDive", Some(18.44), Units::default());
    let b = next_dive();
    let gps = "raw gps=12.5,-45.25";
    let check = |dive: &Dive, inside: bool, altitude: bool, first: &[&str], second: &[&str]| {
        let lines = retag_writes(
            Before::Filed(&photo),
            None,
            &tag_with_values(dive, inside, altitude),
        );
        let (tail, got) = tail_and_second(&lines);
        assert_eq!(tail, first, "first write, {inside} {altitude}");
        assert_eq!(got, second, "second write, {inside} {altitude}");
    };
    let marker_then_own: Vec<&str> = [MARKER_CLEARED].into_iter().chain(OWN).collect();
    let cleared_and_marker: Vec<&str> = CLEARED.into_iter().chain([MARKER_CLEARED]).collect();
    // Another dive, photo in the dive: its new values replace the old
    // ones (nothing cleared first). The altitude: the new depth with the
    // option, and its marker; else the old one cleared (Dive Lens wrote
    // it), with its marker.
    check(&b, true, true, &OWN_ALT, &[gps, "raw gpsAltitude=-10"]);
    check(
        &b,
        true,
        false,
        &marker_then_own,
        &[gps, "raw gpsAltitude=nil"],
    );
    // Another dive, photo outside: the old dive's values cleared, as
    // without per-photo values; no clock offset written.
    check(
        &b,
        false,
        true,
        &cleared_and_marker,
        &[gps, "raw gpsAltitude=nil"],
    );
    check(
        &b,
        false,
        false,
        &cleared_and_marker,
        &[gps, "raw gpsAltitude=nil"],
    );
    // Same dive, photo in the dive: rewritten with the offset shown.
    check(&a, true, true, &OWN_ALT, &[gps, "raw gpsAltitude=-10"]);
    // Without the option, the altitude it has stays, and its marker.
    check(&a, true, false, &OWN, &[gps]);
    // Same dive, photo outside: its values stay as they are.
    check(&a, false, true, &[], &[gps]);
    check(&a, false, false, &[], &[gps]);
}

#[test]
fn per_photo_values_on_a_photo_without_dive_data() {
    let b = next_dive();
    // Taken during the dive: its values and clock offset, the altitude
    // only with the option.
    let lines = retag_writes(
        Before::Nothing,
        Some(-3.0),
        &tag_with_values(&b, true, false),
    );
    let (tail, second) = tail_and_second(&lines);
    assert_eq!(tail, OWN);
    assert_eq!(
        second,
        ["raw gps=12.5,-45.25"],
        "the camera's altitude stays"
    );
    let lines = retag_writes(
        Before::Nothing,
        Some(-3.0),
        &tag_with_values(&b, true, true),
    );
    assert_eq!(
        tail_and_second(&lines).1,
        ["raw gps=12.5,-45.25", "raw gpsAltitude=-10"]
    );
    // Outside the dive: the dive data alone.
    let lines = retag_writes(Before::Nothing, None, &tag_with_values(&b, false, true));
    let (tail, second) = tail_and_second(&lines);
    assert!(tail.is_empty(), "{tail:?}");
    assert_eq!(second, ["raw gps=12.5,-45.25"]);
    // Dive data only, no location: the altitude alone in the second
    // write, the per-photo values in the first.
    let mut dive_only = tag_with_values(&b, true, true);
    dive_only.location = None;
    let lines = retag_writes(
        Before::Filed(&filed(
            &sample_dive(),
            "MacDive",
            Some(18.44),
            Units::default(),
        )),
        None,
        &dive_only,
    );
    let first: Vec<&String> = lines.iter().filter(|l| l.starts_with("1 ")).collect();
    assert_eq!(first.len(), 9 + OWN_ALT.len());
    assert_eq!(
        lines
            .iter()
            .filter(|l| l.starts_with("2 "))
            .collect::<Vec<_>>(),
        ["2 raw gpsAltitude=-10"]
    );
}

/// The altitude and marker lines of both writes ("1 …" then "2 …").
fn altitude_lines(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|l| l.contains("Altitude"))
        .cloned()
        .collect()
}

#[test]
fn create_collections_clears_the_altitude_it_wrote_from_a_depth_cleared() {
    let a = sample_dive();
    let b = next_dive();
    // Filed in dive A at 18.44 m, its depth copied to the altitude.
    let photo = filed(&a, "MacDive", Some(18.44), Units::default());
    let gone = ["1 plugin WrittenAltitude=nil", "2 raw gpsAltitude=nil"];
    let refiled = |before: Before, altitude: Option<f64>, request: &CollectionsRequest| {
        let lines = refile_writes(before, altitude, request);
        // Its per-photo values always rewritten, never in the GPS write.
        assert!(lines.iter().any(|l| l.starts_with("1 plugin PhotoDepth=")));
        assert!(lines.iter().any(|l| l == "2 raw gps=12.5,-45.25"));
        altitude_lines(&lines)
    };

    // Filed again without a depth (in the surface margin, of another
    // dive or the same one, or in a dive without a profile): the
    // altitude Dive Lens wrote goes with the depth, option on or off.
    for copy in [true, false] {
        let margin_b = filing(&b, "MacDive", None, Units::default(), copy);
        assert_eq!(refiled(Before::Filed(&photo), None, &margin_b), gone);
        let margin_a = filing(&a, "MacDive", None, Units::default(), copy);
        assert_eq!(refiled(Before::Filed(&photo), None, &margin_a), gone);
    }
    let mut no_profile = b.clone();
    no_profile.has_profile = false;
    let flat = filing(&no_profile, "MacDive", None, Units::default(), true);
    assert_eq!(refiled(Before::Filed(&photo), None, &flat), gone);
    // In feet too (the marker decides, not the rounded depth).
    let feet = filed(&a, "MacDive", Some(18.44), FEET);
    assert_eq!(refiled(Before::Filed(&feet), None, &flat), gone);

    // Not Dive Lens's: kept. A camera's altitude on a photo never
    // tagged; one changed since Dive Lens wrote it (its marker cleared).
    let margin = filing(&b, "MacDive", None, Units::default(), true);
    assert!(refiled(Before::Nothing, Some(-3.0), &margin).is_empty());
    assert_eq!(
        refiled(Before::Filed(&photo), Some(12.0), &margin),
        ["1 plugin WrittenAltitude=nil"]
    );

    // Filed again with a depth: the new one with the option, with its
    // marker; without it, the old one is cleared for another dive, as in
    // Tag Selection, and kept for the same dive.
    let new_alt = ["1 plugin WrittenAltitude=-10", "2 raw gpsAltitude=-10"];
    let into_b = |copy| filing(&b, "MacDive", Some(10.0), Units::default(), copy);
    let into_a = |copy| filing(&a, "MacDive", Some(10.0), Units::default(), copy);
    assert_eq!(refiled(Before::Filed(&photo), None, &into_b(true)), new_alt);
    assert_eq!(refiled(Before::Filed(&photo), None, &into_a(true)), new_alt);
    assert_eq!(refiled(Before::Filed(&photo), None, &into_b(false)), gone);
    assert!(refiled(Before::Filed(&photo), None, &into_a(false)).is_empty());
    // The camera's altitude, replaced with the option (as before).
    assert_eq!(refiled(Before::Nothing, Some(-3.0), &into_b(true)), new_alt);

    // Filed by a version without the marker: the old rule, the altitude
    // derived from the old depth, to the decimetre ("60 ft" misses).
    assert_eq!(
        refiled(Before::FiledEarlier(&photo), None, &margin),
        ["2 raw gpsAltitude=nil"]
    );
    assert!(refiled(Before::FiledEarlier(&feet), None, &margin).is_empty());
    assert!(refiled(Before::FiledEarlier(&photo), Some(12.0), &margin).is_empty());
}
