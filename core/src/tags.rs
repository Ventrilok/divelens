//! What DiveLens writes on photos in Lightroom, as text values.
//!
//! Formats follow the previous plugin so existing catalogs stay consistent
//! (compatibility contract, spec section 4), with the V1 changes decided in
//! the spec: depth written with its unit, zero and negative temperatures
//! written too.
//!
//! Start and end stay wall-clock date-times here: the plugin formats them
//! with Lightroom's own date format, exactly as the previous plugin did.

use chrono::NaiveDateTime;
use serde::Serialize;

use crate::model::{Dive, Site};
use crate::profile::PhotoPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DepthUnit {
    #[default]
    Metres,
    Feet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TempUnit {
    #[default]
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Units {
    pub depth: DepthUnit,
    pub temp: TempUnit,
}

const FT_PER_M: f64 = 3.28084;

/// Values of the DiveLens fields for one dive. `None` clears the field: the
/// fields describe one dive, so a value left from another dive would be wrong.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiveTags {
    pub dive_id: Option<String>,
    /// Wall-clock, `YYYY-MM-DDTHH:MM:SS`; formatted by the plugin.
    pub start: String,
    pub end: String,
    pub duration: String,
    pub max_depth: Option<String>,
    pub temp_high: Option<String>,
    pub temp_low: Option<String>,
    pub rating: Option<String>,
    /// Dive application the data come from ("MacDive", "Subsurface").
    pub source: String,
}

/// Lightroom location fields. `None` leaves the photo's value untouched (a
/// site without a country must not erase one set by hand, and GPS from a
/// phone must survive a site without coordinates).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationTags {
    pub country: Option<String>,
    /// Lightroom's "City" holds the site's location (atoll, region…).
    pub city: Option<String>,
    /// Lightroom's "Location" holds the site name.
    pub location: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

/// Where "Tag Selection" puts the selected photos: the collection, inside
/// the collection set `parent` leads to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPlace {
    pub name: String,
    /// Names of the existing collection sets leading to where the collection
    /// goes, from the catalog's root; empty for the root.
    pub parent: Vec<String>,
    /// Collection set inside `parent` grouping the collections, created when
    /// needed; `None`: straight into `parent`.
    pub group: Option<String>,
}

/// Everything one "Tag Selection › Apply" writes. At least one of `dive`,
/// `location` and `collection` is present.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRequest {
    pub dive: Option<DiveTags>,
    pub location: Option<LocationTags>,
    /// Put the selected photos in this collection (made when missing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionPlace>,
    /// The selection the per-photo values were computed for, as the plugin
    /// numbered it when it described it; `None`: no per-photo values (no
    /// dive, or a dive without a profile). The plugin then writes only if
    /// the photos selected at that moment are exactly those.
    pub selection: Option<u64>,
    /// Per-photo values of the photos taken during the dive (spec F6, F8).
    /// The other photos of the selection are not listed: they get the dive
    /// data without per-photo values, nor a clock offset.
    pub photos: Vec<PhotoTags>,
}

/// Per-photo fields (spec F6, section 7). `None` clears the field: a photo
/// at the surface, or in a dive without a profile, has no depth.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoTags {
    /// Lightroom's photo identifier, as the plugin sent it.
    pub id: String,
    pub photo_depth: Option<String>,
    pub photo_temp: Option<String>,
    pub photo_elapsed: Option<String>,
    /// `±HH:MM`, always written.
    pub clock_offset: String,
    /// GPS altitude to write, in metres (spec F7: the depth, negative); the
    /// plugin records it in the photo's altitude marker. `None` writes no
    /// altitude: the photo keeps the one it has, unless the plugin finds it
    /// is one Dive Lens wrote from a depth now cleared or another dive's.
    pub gps_altitude: Option<f64>,
    /// Depth at the photo, in metres (for the altitude copy).
    #[serde(skip)]
    pub depth_m: Option<f64>,
}

/// One dive of a "Create Collections" write: its collection, its data and
/// location, and its photos.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionGroup {
    /// Collection name, inside the request's collection set; `None`: the
    /// diver unticked the collections, the photos are tagged only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    /// `None`: the diver unticked the dive details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dive: Option<DiveTags>,
    /// `None`: nothing of the place or GPS is written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationTags>,
    pub photos: Vec<PhotoTags>,
}

/// The parts of a write request the diver left ticked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kept {
    /// Dive details and per-photo values (depth, temperature, time, altitude).
    pub dive: bool,
    /// Country, city and location name.
    pub place: bool,
    /// Latitude and longitude.
    pub gps: bool,
    pub collections: bool,
}

impl Kept {
    pub const ALL: Kept = Kept {
        dive: true,
        place: true,
        gps: true,
        collections: true,
    };
}

impl LocationTags {
    /// Keeps the ticked half: the place names, the GPS position, or both.
    /// `None` when no value is left.
    pub fn keep(mut self, place: bool, gps: bool) -> Option<LocationTags> {
        if !place {
            self.country = None;
            self.city = None;
            self.location = None;
        }
        if !gps {
            self.lat = None;
            self.lon = None;
        }
        let empty = self.country.is_none()
            && self.city.is_none()
            && self.location.is_none()
            && self.lat.is_none()
            && self.lon.is_none();
        (!empty).then_some(self)
    }
}

impl PhotoTags {
    /// Drops the values that come from the dive (the id and clock offset stay).
    fn clear_dive_values(&mut self) {
        self.photo_depth = None;
        self.photo_temp = None;
        self.photo_elapsed = None;
        self.gps_altitude = None;
        self.depth_m = None;
    }
}

/// Everything one "Create Collections › Apply" writes, in one Undo step.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionsRequest {
    /// The photo search these photo identifiers come from.
    pub search: u64,
    /// Names of the existing collection sets leading to where the
    /// collections go, from the catalog's root; empty for the root.
    pub parent: Vec<String>,
    /// Collection set inside `parent` grouping the collections, created when
    /// needed; `None`: straight into `parent`.
    pub group: Option<String>,
    pub groups: Vec<CollectionGroup>,
}

impl CollectionsRequest {
    /// Removes what is not ticked. `other_site`, when the diver picked
    /// another site, replaces every group's location first (not the
    /// collection names). A group left with nothing is dropped.
    pub fn keep(&mut self, kept: Kept, other_site: Option<&LocationTags>) {
        for g in &mut self.groups {
            if let Some(site) = other_site {
                g.location = Some(site.clone());
            }
            g.location = g.location.take().and_then(|l| l.keep(kept.place, kept.gps));
            if !kept.dive {
                g.dive = None;
                g.photos.iter_mut().for_each(PhotoTags::clear_dive_values);
            }
            if !kept.collections {
                g.collection = None;
            }
        }
        self.groups
            .retain(|g| g.dive.is_some() || g.location.is_some() || g.collection.is_some());
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Copies each photo's depth to its GPS altitude, negative (spec F7).
    /// Photos without a depth get no altitude (see `PhotoTags::gps_altitude`).
    pub fn copy_depth_to_altitude(&mut self) {
        copy_depth_to_altitude(self.groups.iter_mut().flat_map(|g| &mut g.photos));
    }

    /// Photos that get a depth (none with the dive details unticked).
    pub fn with_depth(&self) -> usize {
        with_depth(self.groups.iter().flat_map(|g| &g.photos))
    }
}

impl TagRequest {
    /// Removes what is not ticked.
    pub fn keep(&mut self, kept: Kept) {
        if !kept.collections {
            self.collection = None;
        }
        self.location = self
            .location
            .take()
            .and_then(|l| l.keep(kept.place, kept.gps));
        if !kept.dive {
            self.dive = None;
            self.selection = None;
            self.photos.clear();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.dive.is_none() && self.location.is_none() && self.collection.is_none()
    }

    /// Copies the depth of each photo taken during the dive to its GPS
    /// altitude, negative (spec F7), as Create Collections does.
    pub fn copy_depth_to_altitude(&mut self) {
        copy_depth_to_altitude(&mut self.photos);
    }

    /// Photos that get a depth, as Create Collections counts them (none
    /// with the dive details unticked).
    pub fn with_depth(&self) -> usize {
        with_depth(&self.photos)
    }
}

fn copy_depth_to_altitude<'a>(photos: impl IntoIterator<Item = &'a mut PhotoTags>) {
    for photo in photos {
        photo.gps_altitude = photo.depth_m.map(altitude);
    }
}

fn with_depth<'a>(photos: impl IntoIterator<Item = &'a PhotoTags>) -> usize {
    photos
        .into_iter()
        .filter(|p| p.photo_depth.is_some())
        .count()
}

/// Altitude for a depth: negative, to the decimetre (never `-0`).
pub fn altitude(depth_m: f64) -> f64 {
    let a = -(depth_m * 10.0).round() / 10.0;
    if a == 0.0 { 0.0 } else { a }
}

fn wall_clock(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// `MM:SS`; minutes go past 99 when they must.
pub fn duration(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

/// `18.4 m` / `60 ft` (decimal point, never a comma: the value is searched
/// and sorted as text in Lightroom).
pub fn depth(m: f64, unit: DepthUnit) -> String {
    match unit {
        DepthUnit::Metres => format!("{m:.1} m"),
        DepthUnit::Feet => format!("{:.0} ft", m * FT_PER_M),
    }
}

/// `27.0°C` / `80.6°F`, zero and negative values included.
pub fn temperature(c: f64, unit: TempUnit) -> String {
    match unit {
        TempUnit::Celsius => format!("{c:.1}°C"),
        TempUnit::Fahrenheit => format!("{:.1}°F", c * 9.0 / 5.0 + 32.0),
    }
}

/// Clock offset as `+04:01`, `-00:30`, `+00:00`.
pub fn clock_offset(minutes: i32) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let m = minutes.unsigned_abs();
    format!("{sign}{:02}:{:02}", m / 60, m % 60)
}

/// One star per point; `None` when unrated.
pub fn rating(stars: Option<u8>) -> Option<String> {
    stars.filter(|n| *n > 0).map(|n| "★".repeat(n.into()))
}

impl DiveTags {
    /// Tags of `dive`, read from the dive application named `source`.
    pub fn new(dive: &Dive, units: Units, source: &str) -> Self {
        Self {
            dive_id: dive.number.map(|n| n.to_string()),
            start: wall_clock(dive.start),
            end: wall_clock(dive.end),
            duration: duration(dive.duration_s),
            max_depth: dive.max_depth_m.map(|m| depth(m, units.depth)),
            temp_high: dive.temp_high_c.map(|c| temperature(c, units.temp)),
            temp_low: dive.temp_low_c.map(|c| temperature(c, units.temp)),
            rating: rating(dive.rating),
            source: source.to_string(),
        }
    }
}

impl PhotoTags {
    pub fn new(id: String, point: Option<PhotoPoint>, offset_min: i32, units: Units) -> Self {
        Self {
            id,
            photo_depth: point.map(|p| depth(p.depth_m, units.depth)),
            photo_temp: point
                .and_then(|p| p.temp_c)
                .map(|c| temperature(c, units.temp)),
            photo_elapsed: point.map(|p| duration(p.elapsed_s)),
            clock_offset: clock_offset(offset_min),
            gps_altitude: None,
            depth_m: point.map(|p| p.depth_m),
        }
    }
}

impl LocationTags {
    /// The site's fields; `lat`/`lon` replace the site's coordinates when the
    /// user edited them.
    pub fn new(site: &Site, lat: Option<f64>, lon: Option<f64>) -> Self {
        let (lat, lon) = match (lat.or(site.lat), lon.or(site.lon)) {
            (Some(a), Some(b)) => (Some(a), Some(b)),
            _ => (None, None),
        };
        Self {
            country: site.country.clone(),
            city: site.location.clone(),
            location: site.name.clone(),
            lat,
            lon,
        }
    }
}

/// Valid GPS coordinates.
pub fn valid_gps(lat: f64, lon: f64) -> bool {
    lat.is_finite()
        && lon.is_finite()
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&lon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dive() -> Dive {
        let start = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(7, 14, 0)
            .unwrap();
        Dive {
            id: "UUID-A".into(),
            number: Some(700),
            start,
            end: start + chrono::Duration::seconds(6132),
            duration_s: 6132,
            max_depth_m: Some(31.25),
            temp_high_c: Some(28.0),
            temp_low_c: Some(-1.5),
            rating: Some(4),
            site: Site {
                id: Some("pk:1".into()),
                name: Some("Coral Garden".into()),
                location: Some("North Reef".into()),
                country: Some("Atlantis".into()),
                lat: Some(12.5),
                lon: Some(-45.25),
            },
            has_profile: true,
            invalid: false,
            notes: None,
        }
    }

    #[test]
    fn dive_fields_follow_the_previous_plugin() {
        let tags = DiveTags::new(&dive(), Units::default(), "MacDive");
        assert_eq!(
            tags,
            DiveTags {
                dive_id: Some("700".into()),
                start: "2025-12-31T07:14:00".into(),
                end: "2025-12-31T08:56:12".into(),
                duration: "102:12".into(),
                max_depth: Some("31.2 m".into()),
                temp_high: Some("28.0°C".into()),
                temp_low: Some("-1.5°C".into()),
                rating: Some("★★★★".into()),
                source: "MacDive".into(),
            }
        );
    }

    #[test]
    fn missing_values_clear_the_field() {
        let mut d = dive();
        d.number = None;
        d.max_depth_m = None;
        d.temp_high_c = None;
        d.temp_low_c = None;
        d.rating = None;
        d.duration_s = 0;
        let tags = DiveTags::new(&d, Units::default(), "MacDive");
        assert_eq!(tags.dive_id, None);
        assert_eq!(tags.duration, "00:00");
        assert_eq!(
            (tags.max_depth, tags.temp_high, tags.temp_low, tags.rating),
            (None, None, None, None)
        );
        assert_eq!(rating(Some(0)), None);
    }

    #[test]
    fn imperial_units() {
        let units = Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Fahrenheit,
        };
        let tags = DiveTags::new(&dive(), units, "MacDive");
        assert_eq!(tags.max_depth.as_deref(), Some("103 ft"));
        assert_eq!(tags.temp_high.as_deref(), Some("82.4°F"));
        assert_eq!(tags.temp_low.as_deref(), Some("29.3°F"));
    }

    #[test]
    fn location_maps_site_to_lightroom_fields() {
        let site = dive().site;
        let tags = LocationTags::new(&site, None, None);
        assert_eq!(
            tags,
            LocationTags {
                country: Some("Atlantis".into()),
                city: Some("North Reef".into()),
                location: Some("Coral Garden".into()),
                lat: Some(12.5),
                lon: Some(-45.25),
            }
        );
        let edited = LocationTags::new(&site, Some(12.75), Some(-45.0));
        assert_eq!((edited.lat, edited.lon), (Some(12.75), Some(-45.0)));
        let bare = LocationTags::new(&Site::default(), None, None);
        assert_eq!((bare.country, bare.lat), (None, None));
    }

    #[test]
    fn request_serializes_for_the_plugin() {
        let request = TagRequest {
            location: Some(LocationTags::new(&dive().site, None, None)),
            ..TagRequest::default()
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["dive"], serde_json::Value::Null);
        assert_eq!(json["location"]["city"], "North Reef");
        assert_eq!(json["location"]["lat"], 12.5);
        // No per-photo values: no selection named, no photo listed.
        assert_eq!(json["selection"], serde_json::Value::Null);
        assert_eq!(json["photos"], serde_json::json!([]));
    }

    #[test]
    fn tag_request_copies_depth_to_altitude_of_the_photos_listed() {
        let point = PhotoPoint {
            elapsed_s: 60,
            depth_m: 12.34,
            temp_c: Some(26.0),
        };
        let mut request = TagRequest {
            dive: Some(DiveTags::new(&dive(), Units::default(), "MacDive")),
            location: None,
            collection: None,
            selection: Some(7),
            photos: vec![PhotoTags::new(
                "1".into(),
                Some(point),
                240,
                Units::default(),
            )],
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["selection"], 7);
        assert_eq!(json["photos"][0]["photoDepth"], "12.3 m");
        assert_eq!(json["photos"][0]["clockOffset"], "+04:00");
        assert_eq!(json["photos"][0]["gpsAltitude"], serde_json::Value::Null);
        request.copy_depth_to_altitude();
        assert_eq!(request.photos[0].gps_altitude, Some(-12.3));
    }

    #[test]
    fn depth_copied_to_altitude_only_when_known() {
        let point = PhotoPoint {
            elapsed_s: 60,
            depth_m: 18.44,
            temp_c: None,
        };
        let mut request = CollectionsRequest {
            search: 1,
            parent: Vec::new(),
            group: None,
            groups: vec![CollectionGroup {
                collection: Some("700 - Coral Garden".into()),
                dive: Some(DiveTags::new(&dive(), Units::default(), "MacDive")),
                location: Some(LocationTags::new(&dive().site, None, None)),
                photos: vec![
                    PhotoTags::new("1".into(), Some(point), 0, Units::default()),
                    PhotoTags::new("2".into(), None, 0, Units::default()),
                ],
            }],
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json["groups"][0]["photos"][0]["gpsAltitude"],
            serde_json::Value::Null
        );
        assert!(json["groups"][0]["photos"][0].get("depthM").is_none());
        request.copy_depth_to_altitude();
        let photos = &request.groups[0].photos;
        assert_eq!(photos[0].gps_altitude, Some(-18.4));
        assert_eq!(
            photos[1].gps_altitude, None,
            "surface photo: no altitude written"
        );
        assert_eq!(altitude(0.02), 0.0);
        assert!(altitude(0.02).is_sign_positive());
    }

    #[test]
    fn gps_ranges() {
        assert!(valid_gps(-90.0, 180.0));
        assert!(!valid_gps(90.5, 0.0));
        assert!(!valid_gps(0.0, -180.1));
        assert!(!valid_gps(f64::NAN, 0.0));
    }

    fn full_location(site: &str) -> LocationTags {
        LocationTags {
            country: Some("Fictia".into()),
            city: Some("North Bay".into()),
            location: Some(site.into()),
            lat: Some(12.34),
            lon: Some(56.78),
        }
    }

    fn sample_photo(id: &str, depth: Option<f64>) -> PhotoTags {
        let point = depth.map(|d| PhotoPoint {
            elapsed_s: 60,
            depth_m: d,
            temp_c: Some(26.0),
        });
        let mut photo = PhotoTags::new(id.into(), point, 0, Units::default());
        photo.gps_altitude = depth.map(altitude);
        photo
    }

    fn sample_collections_request() -> CollectionsRequest {
        let group = |name: &str, site: &str| CollectionGroup {
            collection: Some(name.into()),
            dive: Some(DiveTags::new(&dive(), Units::default(), "MacDive")),
            location: Some(full_location(site)),
            photos: vec![sample_photo("1", Some(18.4)), sample_photo("2", None)],
        };
        CollectionsRequest {
            search: 1,
            parent: Vec::new(),
            group: None,
            groups: vec![
                group("1 - Three Arches Reef", "Three Arches Reef"),
                group("2 - Lantern Wall", "Lantern Wall"),
            ],
        }
    }

    fn sample_tag_request() -> TagRequest {
        TagRequest {
            dive: Some(DiveTags::new(&dive(), Units::default(), "MacDive")),
            location: Some(full_location("Three Arches Reef")),
            collection: None,
            selection: Some(7),
            photos: vec![sample_photo("1", Some(18.4))],
        }
    }

    #[test]
    fn location_keeps_only_the_ticked_half() {
        let l = full_location("Three Arches Reef");
        let place = l.clone().keep(true, false).unwrap();
        assert_eq!(
            (place.country.as_deref(), place.lat),
            (Some("Fictia"), None)
        );
        let gps = l.clone().keep(false, true).unwrap();
        assert_eq!(
            (gps.country, gps.city, gps.location, gps.lat),
            (None, None, None, Some(12.34))
        );
        assert!(l.clone().keep(false, false).is_none());
        let no_gps = LocationTags {
            lat: None,
            lon: None,
            ..l
        };
        assert!(no_gps.keep(false, true).is_none());
    }

    #[test]
    fn keep_all_changes_nothing() {
        let mut r = sample_collections_request();
        let before = serde_json::to_value(&r).unwrap();
        r.keep(Kept::ALL, None);
        assert_eq!(serde_json::to_value(&r).unwrap(), before);
    }

    #[test]
    fn every_combination_keeps_only_what_is_ticked() {
        for bits in 0u8..16 {
            let kept = Kept {
                dive: bits & 1 != 0,
                place: bits & 2 != 0,
                gps: bits & 4 != 0,
                collections: bits & 8 != 0,
            };
            let mut r = sample_collections_request();
            r.keep(kept, None);
            if bits == 0 {
                assert!(r.is_empty());
                continue;
            }
            assert_eq!(r.groups.len(), 2);
            for g in &r.groups {
                assert_eq!(g.dive.is_some(), kept.dive);
                assert_eq!(g.collection.is_some(), kept.collections);
                assert_eq!(
                    g.location.as_ref().is_some_and(|l| l.country.is_some()),
                    kept.place
                );
                assert_eq!(
                    g.location.as_ref().is_some_and(|l| l.lat.is_some()),
                    kept.gps
                );
                assert_eq!(
                    g.photos
                        .iter()
                        .any(|p| p.photo_depth.is_some() || p.gps_altitude.is_some()),
                    kept.dive
                );
            }
        }
    }

    #[test]
    fn a_group_left_with_nothing_goes_while_the_other_stays() {
        // The second dive's site has no field at all: with the site details
        // alone ticked, nothing is left to write on its group, which goes.
        let mut r = sample_collections_request();
        r.groups[1].location = Some(LocationTags {
            country: None,
            city: None,
            location: None,
            lat: None,
            lon: None,
        });
        let site_only = Kept {
            dive: false,
            place: true,
            gps: true,
            collections: false,
        };
        r.keep(site_only, None);
        // The first stays, with its location alone, and all its photos
        // without the dive's values.
        assert_eq!(r.groups.len(), 1);
        let g = &r.groups[0];
        assert_eq!(g.location, Some(full_location("Three Arches Reef")));
        assert_eq!((g.dive.as_ref(), g.collection.as_deref()), (None, None));
        let ids: Vec<_> = g.photos.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["1", "2"]);
        assert!(
            g.photos
                .iter()
                .all(|p| p.photo_depth.is_none() && p.gps_altitude.is_none())
        );
    }

    #[test]
    fn another_site_replaces_every_groups_location_not_its_collection() {
        let beach = LocationTags {
            country: None,
            city: None,
            location: Some("Plage".into()),
            lat: None,
            lon: None,
        };
        let mut r = sample_collections_request();
        let names: Vec<_> = r.groups.iter().map(|g| g.collection.clone()).collect();
        r.keep(Kept::ALL, Some(&beach));
        for g in &r.groups {
            assert_eq!(
                g.location.as_ref().unwrap().location.as_deref(),
                Some("Plage")
            );
        }
        let after: Vec<_> = r.groups.iter().map(|g| g.collection.clone()).collect();
        assert_eq!(names, after);
    }

    #[test]
    fn photos_with_a_depth_are_counted_until_the_dive_details_are_unticked() {
        // Each group: photo 1 at 18.4 m, photo 2 at the surface.
        let mut collections = sample_collections_request();
        assert_eq!(collections.with_depth(), 2);
        let mut tag = sample_tag_request();
        assert_eq!(tag.with_depth(), 1);
        let unticked = Kept {
            dive: false,
            ..Kept::ALL
        };
        collections.keep(unticked, None);
        tag.keep(unticked);
        assert_eq!((collections.with_depth(), tag.with_depth()), (0, 0));
    }

    #[test]
    fn tag_request_keep_all_changes_nothing() {
        let mut r = sample_tag_request();
        let before = serde_json::to_value(&r).unwrap();
        r.keep(Kept::ALL);
        assert_eq!(serde_json::to_value(&r).unwrap(), before);
    }

    #[test]
    fn tag_request_every_combination_keeps_only_what_is_ticked() {
        for bits in 0u8..8 {
            let kept = Kept {
                dive: bits & 1 != 0,
                place: bits & 2 != 0,
                gps: bits & 4 != 0,
                collections: true,
            };
            let mut r = sample_tag_request();
            r.keep(kept);
            if bits == 0 {
                assert!(r.is_empty());
                continue;
            }
            assert_eq!(r.dive.is_some(), kept.dive);
            assert_eq!(r.selection.is_some(), kept.dive);
            assert_eq!(!r.photos.is_empty(), kept.dive);
            assert_eq!(
                r.location.as_ref().is_some_and(|l| l.country.is_some()),
                kept.place
            );
            assert_eq!(
                r.location.as_ref().is_some_and(|l| l.lat.is_some()),
                kept.gps
            );
        }
    }

    fn place() -> CollectionPlace {
        CollectionPlace {
            name: "101 - Three Arches Reef".into(),
            parent: vec!["Voyages".into()],
            group: Some("Dive Lens".into()),
        }
    }

    #[test]
    fn tag_request_serialises_its_collection_only_when_present() {
        let without = serde_json::to_value(TagRequest::default()).unwrap();
        assert!(without.get("collection").is_none());
        let with = serde_json::to_value(TagRequest {
            collection: Some(place()),
            ..TagRequest::default()
        })
        .unwrap();
        assert_eq!(
            with["collection"],
            serde_json::json!({
                "name": "101 - Three Arches Reef",
                "parent": ["Voyages"],
                "group": "Dive Lens"
            })
        );
    }

    #[test]
    fn tag_request_collection_follows_the_collections_tick() {
        let mut r = sample_tag_request();
        r.collection = Some(place());
        r.keep(Kept::ALL);
        assert_eq!(r.collection, Some(place()));
        r.keep(Kept {
            collections: false,
            ..Kept::ALL
        });
        assert_eq!(r.collection, None);
        // A request with only a collection is not empty; without it, it is.
        let mut only = TagRequest {
            collection: Some(place()),
            ..TagRequest::default()
        };
        assert!(!only.is_empty());
        only.keep(Kept {
            collections: false,
            ..Kept::ALL
        });
        assert!(only.is_empty());
    }

    #[test]
    fn tag_request_is_empty_once_dive_place_and_gps_are_unticked() {
        // The sample names no collection: with the dive details, the place
        // and the GPS unticked, nothing is left, the collections tick or not.
        let mut r = sample_tag_request();
        r.keep(Kept {
            dive: false,
            place: false,
            gps: false,
            collections: true,
        });
        assert!(r.is_empty());
    }
}
