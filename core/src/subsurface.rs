//! Subsurface reader (macOS and Windows), for both storage formats:
//!
//! - an XML file (`.xml` or `.ssrf`), see [`xml`];
//! - git storage (a folder, used by Subsurface's cloud storage), see [`git`].
//!
//! Both parsers produce the same raw log; the rules of the spec (section 8,
//! "Règles de lecture") are applied once, here:
//!
//! - values are taken as Subsurface shows them;
//! - with several dive computers, the first one (the one Subsurface shows)
//!   gives the profile, maximum depth and temperatures;
//! - a dive without a dive computer ("manually added dive", or "No Computer"
//!   when imported from MacDive) has no profile: Subsurface only stores the
//!   few points it draws for it;
//! - temperatures: low = water temperature, else the coldest sample; high =
//!   the warmest sample, else the water temperature; never the air;
//! - a site named "Country / Location / Site" is split only when Subsurface's
//!   own country and city are empty and the name has exactly three parts.
//!
//! Subsurface stores everything in metric units and wall-clock times with no
//! time zone, like the common format. The whole log is read on open (profiles
//! included): there is no index to read a single dive from.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use serde::Serialize;

use crate::model::{Capabilities, Dive, DiveSource, Site, clean_notes};
use crate::profile::Sample;
use crate::{Error, Result};

mod app;
mod git;
mod xml;

pub use app::APP;

/// Subsurface's standard folder, relative to the home folder (macOS).
pub const MAC_RELATIVE_FOLDER: &str = "Library/Application Support/Subsurface";

/// Dive computer names that mean "no dive computer".
const MANUAL_MODELS: [&str; 2] = ["manually added dive", "No Computer"];

/// Oldest data format read (version 3 introduced the separate site list).
const MIN_FORMAT_VERSION: u32 = 3;

/// How the log is stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Storage {
    /// An XML file.
    Xml,
    /// Git storage; the branch holding the log.
    Git { branch: String },
}

/// Standard log file: `<user>.xml` in Subsurface's folder, the file Subsurface
/// creates on first launch. `folder` is `~/Library/Application Support/
/// Subsurface` on macOS and `%APPDATA%\Subsurface` on Windows.
pub fn default_path(folder: &Path, user: &str) -> PathBuf {
    folder.join(format!("{user}.xml"))
}

// ---------------------------------------------------------------------------
// Raw log, as both formats store it.

#[derive(Debug, Default, Clone, PartialEq)]
struct RawSite {
    uuid: u32,
    name: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    /// Subsurface's own country (taxonomy "Country").
    country: Option<String>,
    /// Subsurface's own city (taxonomy "City", else "Town").
    city: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct RawComputer {
    model: Option<String>,
    duration_s: Option<u32>,
    max_depth_m: Option<f64>,
    water_c: Option<f64>,
    /// Sorted by time, with Subsurface's "unchanged since the previous
    /// sample" already filled in.
    samples: Vec<Sample>,
}

#[derive(Debug, Clone, PartialEq)]
struct RawDive {
    number: Option<u32>,
    start: NaiveDateTime,
    /// The dive's duration as Subsurface shows it (its first computer's).
    duration_s: Option<u32>,
    rating: Option<u8>,
    invalid: bool,
    notes: Option<String>,
    site: Option<u32>,
    /// Water temperature entered on the dive itself, which Subsurface shows
    /// instead of the computers'.
    water_c: Option<f64>,
    computers: Vec<RawComputer>,
}

#[derive(Debug, Default)]
struct RawLog {
    sites: Vec<RawSite>,
    dives: Vec<RawDive>,
}

// ---------------------------------------------------------------------------

pub struct Subsurface {
    storage: Storage,
    dives: Vec<Dive>,
    sites: Vec<Site>,
    profiles: HashMap<String, Vec<Sample>>,
}

impl Subsurface {
    /// Reads the log at `path`: a folder is git storage, a file is XML.
    /// Nothing is ever written.
    pub fn open(path: &Path) -> Result<Self> {
        if path.is_dir() {
            let (branch, raw) = git::read(path)?;
            Ok(Self::build(Storage::Git { branch }, raw))
        } else if path.is_file() {
            Ok(Self::build(Storage::Xml, xml::read(path)?))
        } else {
            Err(Error::NotFound(path.to_path_buf()))
        }
    }

    /// Changes whenever the log at `path` changes (file date and size, or
    /// the git commit read); `None` when it cannot be told. Lets the app keep
    /// an opened log instead of reading it again for every request.
    pub fn fingerprint(path: &Path) -> Option<String> {
        if path.is_dir() {
            git::fingerprint(path)
        } else {
            let m = std::fs::metadata(path).ok()?;
            Some(format!("{:?}/{}", m.modified().ok()?, m.len()))
        }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    fn build(storage: Storage, raw: RawLog) -> Self {
        let raw_sites: HashMap<u32, &RawSite> = raw.sites.iter().map(|s| (s.uuid, s)).collect();
        let mut dives = raw.dives;
        // Stable order so that two dives starting at the same second get the
        // same identifiers every time.
        dives.sort_by_key(|d| (d.start, d.number));
        let mut seen: HashMap<NaiveDateTime, u32> = HashMap::new();
        let mut profiles = HashMap::new();
        let mut out = Vec::with_capacity(dives.len());
        for d in dives {
            let n = seen.entry(d.start).or_insert(0);
            *n += 1;
            let mut id = d.start.format("%Y-%m-%dT%H:%M:%S").to_string();
            if *n > 1 {
                id.push_str(&format!("#{n}"));
            }
            let site = d
                .site
                .and_then(|u| raw_sites.get(&u))
                .map(|s| site(s))
                .unwrap_or_default();
            let (dive, samples) = dive(id, d, site);
            if dive.has_profile {
                profiles.insert(dive.id.clone(), samples);
            }
            out.push(dive);
        }
        Self {
            storage,
            dives: out,
            sites: raw
                .sites
                .iter()
                .map(site)
                .filter(|s| s.name.is_some())
                .collect(),
            profiles,
        }
    }
}

fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn site(s: &RawSite) -> Site {
    let name = non_empty(s.name.as_deref());
    let country = non_empty(s.country.as_deref());
    let city = non_empty(s.city.as_deref());
    let (name, location, country) = match (&name, &country, &city) {
        (Some(n), None, None) => match n.split(" / ").collect::<Vec<_>>()[..] {
            [c, l, s] => (non_empty(Some(s)), non_empty(Some(l)), non_empty(Some(c))),
            _ => (name, None, None),
        },
        _ => (name, city, country),
    };
    Site {
        id: Some(format!("{:08x}", s.uuid)),
        name,
        location,
        country,
        lat: s.lat,
        lon: s.lon,
    }
}

fn dive(id: String, d: RawDive, site: Site) -> (Dive, Vec<Sample>) {
    let first = d.computers.into_iter().next().unwrap_or_default();
    let manual = first.model.as_deref().is_some_and(|m| {
        MANUAL_MODELS
            .iter()
            .any(|x| x.eq_ignore_ascii_case(m.trim()))
    });
    let samples = first.samples;
    let duration_s = d
        .duration_s
        .or(first.duration_s)
        .or(samples.last().map(|s| s.time_s))
        .unwrap_or(0);
    let max_depth_m = first
        .max_depth_m
        .or_else(|| samples.iter().map(|s| s.depth_m).reduce(f64::max))
        .filter(|m| *m > 0.0);
    let temps = || samples.iter().filter_map(|s| s.temp_c);
    let water = d.water_c.or(first.water_c);
    let temp_low_c = water.or_else(|| temps().reduce(f64::min));
    let temp_high_c = temps()
        .reduce(f64::max)
        .or(water)
        .map(|h| temp_low_c.map_or(h, |l| h.max(l)));
    let has_profile = !manual && !samples.is_empty();
    let dive = Dive {
        id,
        number: d.number.filter(|n| *n > 0),
        start: d.start,
        end: d.start + chrono::Duration::seconds(i64::from(duration_s)),
        duration_s,
        max_depth_m,
        temp_high_c,
        temp_low_c,
        rating: d.rating.filter(|r| (1..=5).contains(r)),
        site,
        has_profile,
        invalid: d.invalid,
        notes: clean_notes(d.notes),
    };
    (dive, if has_profile { samples } else { Vec::new() })
}

impl DiveSource for Subsurface {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            temperature: true,
            rating: true,
            gps: true,
            profile: true,
        }
    }

    fn dives(&self) -> Result<Vec<Dive>> {
        Ok(self.dives.clone())
    }

    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>> {
        if let Some(p) = self.profiles.get(dive_id) {
            return Ok(p.clone());
        }
        if self.dives.iter().any(|d| d.id == dive_id) {
            Ok(Vec::new())
        } else {
            Err(Error::UnknownDive(dive_id.to_string()))
        }
    }

    fn sites(&self) -> Result<Vec<Site>> {
        Ok(self.sites.clone())
    }
}

// ---------------------------------------------------------------------------
// Value parsing shared by both formats. Subsurface writes metric values with
// their unit ("18.4 m", "24.0 C" in XML, "18.4m", "24.0°C" in git); other
// units are converted in case a hand-edited file holds them.

/// Leading decimal number of `s` and the rest.
fn number(s: &str) -> Option<(f64, &str)> {
    let s = s.trim_start();
    let end = s
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || ((c == '-' || c == '+') && i == 0)))
        .map_or(s.len(), |(i, _)| i);
    let v = s[..end].parse().ok()?;
    Some((v, s[end..].trim_start()))
}

/// Depth in metres: "18.4 m", "18.4m", "60 ft".
fn depth(s: &str) -> Option<f64> {
    let (v, unit) = number(s)?;
    Some(if unit.starts_with("ft") {
        v * 0.3048
    } else {
        v
    })
}

/// Temperature in °C: "24.0 C", "24.0°C", "75 F", "297.15 K".
fn temperature(s: &str) -> Option<f64> {
    let (v, unit) = number(s)?;
    let unit = unit.trim_start_matches('°');
    Some(match unit.chars().next() {
        Some('F') => (v - 32.0) * 5.0 / 9.0,
        Some('K') => v - 273.15,
        _ => v,
    })
}

/// Duration in seconds: "45:00 min", "1:05:00", "45 min", "0:10".
fn duration(s: &str) -> Option<u32> {
    let s = s.split_whitespace().next()?;
    let mut total = 0u32;
    for part in s.split(':') {
        total = total.checked_mul(60)?.checked_add(part.parse().ok()?)?;
    }
    if s.contains(':') {
        Some(total)
    } else {
        // A bare number is in minutes.
        total.checked_mul(60)
    }
}

/// GPS "lat lon" in decimal degrees; 0/0 means none.
fn gps(s: &str) -> Option<(f64, f64)> {
    let mut it = s.split_whitespace();
    let lat: f64 = it.next()?.parse().ok()?;
    let lon: f64 = it.next()?.parse().ok()?;
    (!(lat == 0.0 && lon == 0.0)).then_some((lat, lon))
}

/// Subsurface's site identifier, hexadecimal (XML pads it with spaces).
fn uuid(s: &str) -> Option<u32> {
    u32::from_str_radix(s.trim(), 16).ok().filter(|u| *u != 0)
}

/// Taxonomy categories Subsurface uses for a site's country and city.
const TAXONOMY_COUNTRY: u32 = 2;
const TAXONOMY_TOWN: u32 = 5;
const TAXONOMY_CITY: u32 = 6;

/// Records a taxonomy entry ("geo") of a site.
fn set_geo(site: &mut RawSite, category: u32, value: &str) {
    let value = non_empty(Some(value));
    match category {
        TAXONOMY_COUNTRY => site.country = value,
        TAXONOMY_CITY => site.city = value.or(site.city.take()),
        TAXONOMY_TOWN if site.city.is_none() => site.city = value,
        _ => {}
    }
}

/// Fills in what Subsurface leaves out of a sample when it has not changed
/// since the previous one (temperature, sometimes depth).
fn push_sample(samples: &mut Vec<Sample>, time_s: u32, depth_m: Option<f64>, temp_c: Option<f64>) {
    let prev = samples.last();
    let Some(depth_m) = depth_m.or(prev.map(|p| p.depth_m)) else {
        return;
    };
    let temp_c = temp_c.or(prev.and_then(|p| p.temp_c));
    samples.push(Sample {
        time_s,
        depth_m,
        temp_c,
    });
}

fn too_old(path: &Path, version: Option<u32>) -> Result<()> {
    match version {
        Some(v) if v >= MIN_FORMAT_VERSION => Ok(()),
        _ => Err(Error::OldFormat(path.to_path_buf())),
    }
}

#[cfg(test)]
pub(crate) mod tests;
