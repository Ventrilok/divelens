//! Example reader: the starting point for a reader of a new dive
//! application. It reads the log of "Example", an application invented for
//! this purpose, kept in a small SQLite database that no real application
//! writes. It is built only with the tests, where it passes the common tests
//! (`core/src/conformance.rs`) on macOS and Windows, so that a copy starts
//! from code that works; Dive Lens never offers it.
//!
//! The Example database (the tests build one, in `skeleton/tests.rs`):
//!
//! - `sites`: `id` (text, never changes), `name`, `place`, `country`,
//!   `latitude` and `longitude` (decimal degrees);
//! - `dives`: `id` (text, never changes), `number`, `start` (wall clock,
//!   `YYYY-MM-DD HH:MM:SS`), `duration_s`, `max_depth_m`, `temp_min_c`,
//!   `temp_max_c`, `rating` (1 to 5, 0 when unrated), `notes`, `site_id`;
//! - `samples`: `dive_id`, `time_s`, `depth_m`, `temp_c`, in no particular
//!   order. A dive without samples has no profile.
//!
//! Missing values are NULL or blank text, except the rating (0). Example has
//! no notion of an invalid dive.
//!
//! To read another application, copy this reader; each place to change is
//! marked `ADAPT:`. The steps, detailed in
//! `docs/contributing/reference-log.md` ("Adding a reader"):
//!
//! 1. Copy this file and `skeleton/tests.rs` to `<application>.rs` and
//!    `<application>/tests.rs`, rename `Example`, and declare the module in
//!    `lib.rs` as `pub mod <application>;` (this one is `#[cfg(test)]`).
//! 2. Read the application's own tables and translate its values: stable
//!    ids, wall-clock times, metres, °C, its way of storing missing values.
//!    Declare what it provides in `capabilities`.
//! 3. In the tests, write the reference log as the application stores it.
//! 4. Add the application's run at the bottom of `core/src/conformance.rs`,
//!    then run `cargo test -p divelens-core` until it passes.
//! 5. Register the application in `core/examples/verify_log.rs` and check
//!    the reader against a real log on your own computer.
//!
//! The app side (the application's descriptor, which offers it in Settings)
//! is added separately. A log that is not a SQLite database: see the
//! Subsurface reader for files and folders.

use std::path::Path;

use chrono::NaiveDateTime;
use rusqlite::types::Type;
use rusqlite::{Connection, Row};

use crate::model::{Capabilities, Dive, DiveSource, Site, clean_notes};
use crate::profile::Sample;
use crate::sqlite;
use crate::{Error, Result};

/// How Example writes times: the wall clock, as the dive computer showed it.
// ADAPT: the application's own format.
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// A log of the Example application, opened read-only.
pub struct Example {
    conn: Connection,
}

impl Example {
    /// Opens the database read-only, creating no file next to it; the
    /// application may stay open meanwhile.
    pub fn open(path: &Path) -> Result<Self> {
        // ADAPT: tables every log of the application has. A file without
        // them is reported as not a dive log.
        let conn = sqlite::open_read_only(path, &["dives", "sites", "samples"])?;
        Ok(Self { conn })
    }
}

/// The `start` column as the wall clock.
// ADAPT: the time as the dive computer showed it, never shifted. An
// application that stores an instant (UTC) records the dive's own offset:
// add it. Never use the computer's time zone: not chrono's `Local`, nor
// SQLite's `'localtime'` modifier (`datetime(x, 'localtime')`).
fn wall_clock(r: &Row) -> rusqlite::Result<NaiveDateTime> {
    let i = r.as_ref().column_index("start")?;
    let text: String = r.get(i)?;
    NaiveDateTime::parse_from_str(text.trim(), TIME_FORMAT)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(i, Type::Text, e.into()))
}

/// Blank text is a missing value.
fn non_empty(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Site from the `site_id, site, place, country, lat, lon` columns; no site
/// when `site_id` is NULL.
// ADAPT: the application's site fields. Latitude and longitude in decimal
// degrees, both or neither.
fn site_from_row(r: &Row) -> rusqlite::Result<Site> {
    let Some(id) = r.get::<_, Option<String>>("site_id")? else {
        return Ok(Site::default());
    };
    let (lat, lon) = match (r.get("lat")?, r.get("lon")?) {
        (Some(lat), Some(lon)) => (Some(lat), Some(lon)),
        _ => (None, None),
    };
    Ok(Site {
        id: Some(id),
        name: non_empty(r.get("site")?),
        location: non_empty(r.get("place")?),
        country: non_empty(r.get("country")?),
        lat,
        lon,
    })
}

fn dive_from_row(r: &Row) -> rusqlite::Result<Dive> {
    let start = wall_clock(r)?;
    // ADAPT: the duration the application shows, in seconds; the end follows
    // from it.
    let duration_s = r
        .get::<_, Option<i64>>("duration_s")?
        .and_then(|s| u32::try_from(s).ok())
        .unwrap_or(0);
    Ok(Dive {
        // ADAPT: an id the application gives the dive and never changes. Not
        // the start time when the application has one: two dives can start
        // at the same second.
        id: r.get("id")?,
        number: r
            .get::<_, Option<i64>>("number")?
            .and_then(|n| u32::try_from(n).ok()),
        start,
        end: start + chrono::Duration::seconds(i64::from(duration_s)),
        duration_s,
        // ADAPT: metres and °C, converted here if the application stores
        // feet, Fahrenheit or anything else.
        max_depth_m: r.get("max_depth_m")?,
        temp_high_c: r.get("temp_max_c")?,
        temp_low_c: r.get("temp_min_c")?,
        // ADAPT: how the application stores "unrated" (here 0).
        rating: r
            .get::<_, Option<i64>>("rating")?
            .and_then(|v| u8::try_from(v).ok())
            .filter(|v| (1..=5).contains(v)),
        site: site_from_row(r)?,
        has_profile: r.get("has_profile")?,
        // ADAPT: the application's own "invalid" mark, if it has one (then
        // `marks_invalid: true` in its conformance run).
        invalid: false,
        notes: clean_notes(r.get("notes")?),
    })
}

impl DiveSource for Example {
    // ADAPT: what the application stores. For what it does not, declare
    // `false` and give `None`; without profiles, every dive has
    // `has_profile: false` and `profile` gives no samples.
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            temperature: true,
            rating: true,
            gps: true,
            profile: true,
        }
    }

    fn dives(&self) -> Result<Vec<Dive>> {
        // ADAPT: the application's tables. A dive has a profile when it has
        // samples, which `profile` gives.
        let mut stmt = self.conn.prepare(
            "SELECT D.id, D.number, D.start, D.duration_s, D.max_depth_m,
                    D.temp_min_c, D.temp_max_c, D.rating, D.notes,
                    EXISTS (SELECT 1 FROM samples P WHERE P.dive_id = D.id) AS has_profile,
                    S.id AS site_id, S.name AS site, S.place, S.country,
                    S.latitude AS lat, S.longitude AS lon
             FROM dives D
             LEFT JOIN sites S ON S.id = D.site_id",
        )?;
        let dives = stmt
            .query_map([], dive_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(dives)
    }

    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>> {
        let known: bool = self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM dives WHERE id = ?1)",
            [dive_id],
            |r| r.get(0),
        )?;
        if !known {
            return Err(Error::UnknownDive(dive_id.to_string()));
        }
        // ADAPT: the application's samples, in seconds since the start,
        // metres and °C, sorted by time. A dive without a profile gives no
        // samples at all.
        let mut stmt = self.conn.prepare(
            "SELECT time_s, depth_m, temp_c FROM samples
             WHERE dive_id = ?1 ORDER BY time_s",
        )?;
        let samples = stmt
            .query_map([dive_id], |r| {
                Ok(Sample {
                    time_s: r.get("time_s")?,
                    depth_m: r.get("depth_m")?,
                    temp_c: r.get("temp_c")?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(samples)
    }

    fn sites(&self) -> Result<Vec<Site>> {
        // ADAPT: every named site, those without a dive yet included, with
        // the same id as on its dives.
        let mut stmt = self.conn.prepare(
            "SELECT id AS site_id, name AS site, place, country,
                    latitude AS lat, longitude AS lon
             FROM sites",
        )?;
        let sites = stmt
            .query_map([], site_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sites.into_iter().filter(|s| s.name.is_some()).collect())
    }
}

#[cfg(test)]
pub(crate) mod tests;
