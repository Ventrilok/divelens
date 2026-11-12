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

/// Everything one "Apply" writes. At least one part is present.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRequest {
    pub dive: Option<DiveTags>,
    pub location: Option<LocationTags>,
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
}

/// One dive of a "Create Collections" write: its collection, its data and
/// location, and its photos.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionGroup {
    /// Collection name inside the "Dive Lens" set.
    pub collection: String,
    pub dive: DiveTags,
    pub location: LocationTags,
    pub photos: Vec<PhotoTags>,
}

/// Everything one "Create Collections › Apply" writes, in one catalog step.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionsRequest {
    /// The photo search these photo identifiers come from.
    pub search: u64,
    pub groups: Vec<CollectionGroup>,
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
    pub fn new(dive: &Dive, units: Units) -> Self {
        Self {
            dive_id: dive.number.map(|n| n.to_string()),
            start: wall_clock(dive.start),
            end: wall_clock(dive.end),
            duration: duration(dive.duration_s),
            max_depth: dive.max_depth_m.map(|m| depth(m, units.depth)),
            temp_high: dive.temp_high_c.map(|c| temperature(c, units.temp)),
            temp_low: dive.temp_low_c.map(|c| temperature(c, units.temp)),
            rating: rating(dive.rating),
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
        }
    }

    #[test]
    fn dive_fields_follow_the_previous_plugin() {
        let tags = DiveTags::new(&dive(), Units::default());
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
        let tags = DiveTags::new(&d, Units::default());
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
        let tags = DiveTags::new(&dive(), units);
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
            dive: None,
            location: Some(LocationTags::new(&dive().site, None, None)),
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["dive"], serde_json::Value::Null);
        assert_eq!(json["location"]["city"], "North Reef");
        assert_eq!(json["location"]["lat"], 12.5);
    }

    #[test]
    fn gps_ranges() {
        assert!(valid_gps(-90.0, 180.0));
        assert!(!valid_gps(90.5, 0.0));
        assert!(!valid_gps(0.0, -180.1));
        assert!(!valid_gps(f64::NAN, 0.0));
    }
}
