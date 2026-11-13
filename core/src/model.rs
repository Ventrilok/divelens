//! Common dive format. Every reader translates its source into these types;
//! the UI and the Lightroom plugin only know this format.

use chrono::NaiveDateTime;
use serde::Serialize;

use crate::Result;
use crate::profile::Sample;

/// A dive site. Every field is optional: sources do not all provide them.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    /// Stable identifier of the site inside the source; `None` when the dive
    /// has no site.
    pub id: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

/// One dive, without its profile (loaded on demand with
/// [`DiveSource::profile`]).
///
/// Dates are **wall-clock times as shown by the dive computer**, without a
/// time zone, exactly like photo capture times in Lightroom. Never convert
/// them to an instant.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dive {
    /// Stable identifier inside the source (never shown to the user).
    pub id: String,
    /// Dive number from the log. This is what the `DiveId` field holds.
    pub number: Option<u32>,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub duration_s: u32,
    pub max_depth_m: Option<f64>,
    pub temp_high_c: Option<f64>,
    pub temp_low_c: Option<f64>,
    /// 1 to 5 stars; `None` when unrated.
    pub rating: Option<u8>,
    pub site: Site,
    /// The dive has a recorded profile (so per-photo values are possible).
    pub has_profile: bool,
    /// The dive log marks the dive as invalid (Subsurface); it is still
    /// listed, with a badge.
    pub invalid: bool,
    /// The diver's own notes on the dive, line breaks kept; `None` when the
    /// log has none.
    pub notes: Option<String>,
}

/// Notes as shown: line breaks as `\n`, blank notes as `None`.
pub(crate) fn clean_notes(notes: Option<String>) -> Option<String> {
    let notes = notes?.replace("\r\n", "\n").replace('\r', "\n");
    let notes = notes.trim();
    (!notes.is_empty()).then(|| notes.to_string())
}

/// What a source can provide; the UI hides what is missing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub temperature: bool,
    pub rating: bool,
    pub gps: bool,
    pub profile: bool,
}

/// A dive-log reader. Sources are always opened read-only.
pub trait DiveSource {
    fn capabilities(&self) -> Capabilities;
    /// All dives, in no particular order.
    fn dives(&self) -> Result<Vec<Dive>>;
    /// Profile of one dive, sorted by time; empty when the dive has none.
    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>>;
    /// Every named site of the log, including sites with no dive yet, in no
    /// particular order.
    fn sites(&self) -> Result<Vec<Site>>;
}
