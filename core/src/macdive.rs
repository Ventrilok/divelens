//! MacDive reader (SQLite database, macOS).
//!
//! - Dates are Core Data timestamps: seconds since 2001-01-01, holding the
//!   dive computer's wall-clock time.
//! - Profile: real MacDive databases store it only in encoded form
//!   (`ZSAMPLES`, proprietary) and as the dive computer's raw download
//!   (`ZRAWDATA`). Decoding is not implemented yet. Some prepared test
//!   databases carry an extra readable `ZSAMPLES_JSON` column (added by an
//!   external tool, not by MacDive); it is used when present.
//! - Missing values: NULL temperature, rating 0, GPS 0/0.

use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row};
use serde::Deserialize;

use crate::model::{Capabilities, Dive, DiveSource, Site};
use crate::profile::Sample;
use crate::{Error, Result};

/// Seconds between 1970-01-01 and 2001-01-01 (Core Data reference date).
const CORE_DATA_EPOCH: i64 = 978_307_200;

/// Standard location of the MacDive database, relative to the home folder.
pub const DEFAULT_RELATIVE_PATH: &str = "Library/Application Support/MacDive/MacDive.sqlite";

pub struct MacDive {
    conn: Connection,
    /// The database has the extra readable profile column (see module doc).
    has_json_profile: bool,
}

impl MacDive {
    /// Opens the database read-only. MacDive may stay open meanwhile.
    pub fn open(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Err(Error::NotFound(path.to_path_buf()));
        }
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(path, flags)
            .map_err(|_| Error::NotADiveLog(path.to_path_buf()))?;
        let is_macdive: bool = conn
            .query_row(
                "SELECT count(*) = 2 FROM sqlite_master
                 WHERE type = 'table' AND name IN ('ZDIVE', 'ZDIVESITE')",
                [],
                |r| r.get(0),
            )
            .map_err(|_| Error::NotADiveLog(path.to_path_buf()))?;
        if !is_macdive {
            return Err(Error::NotADiveLog(path.to_path_buf()));
        }
        let has_json_profile: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_table_info('ZDIVE') WHERE name = 'ZSAMPLES_JSON'",
                [],
                |r| r.get(0),
            )
            .map_err(Error::from)?;
        Ok(Self {
            conn,
            has_json_profile,
        })
    }

    /// Standard database location for the given home folder.
    pub fn default_path(home: &Path) -> PathBuf {
        home.join(DEFAULT_RELATIVE_PATH)
    }
}

fn wall_clock(core_data_s: f64) -> NaiveDateTime {
    let secs = core_data_s.round() as i64 + CORE_DATA_EPOCH;
    DateTime::from_timestamp(secs, 0)
        .unwrap_or_default()
        .naive_utc()
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn dive_from_row(r: &Row) -> rusqlite::Result<Dive> {
    let start_raw: f64 = r.get("start")?;
    let duration_s = r
        .get::<_, Option<f64>>("duration")?
        .unwrap_or(0.0)
        .max(0.0)
        .round() as u32;
    let start = wall_clock(start_raw);
    let lat: Option<f64> = r.get("lat")?;
    let lon: Option<f64> = r.get("lon")?;
    let (lat, lon) = match (lat, lon) {
        (Some(a), Some(b)) if !(a == 0.0 && b == 0.0) => (Some(a), Some(b)),
        _ => (None, None),
    };
    let rating = r
        .get::<_, Option<f64>>("rating")?
        .map(|v| v.round() as i64)
        .filter(|v| (1..=5).contains(v))
        .map(|v| v as u8);
    Ok(Dive {
        id: r.get("id")?,
        number: r
            .get::<_, Option<i64>>("number")?
            .and_then(|n| u32::try_from(n).ok()),
        start,
        end: start + chrono::Duration::seconds(i64::from(duration_s)),
        duration_s,
        max_depth_m: r.get::<_, Option<f64>>("max_depth")?.filter(|d| *d > 0.0),
        temp_high_c: r.get("temp_high")?,
        temp_low_c: r.get("temp_low")?,
        rating,
        site: Site {
            name: non_empty(r.get("site")?),
            location: non_empty(r.get("location")?),
            country: non_empty(r.get("country")?),
            lat,
            lon,
        },
        has_profile: r.get("has_profile")?,
    })
}

#[derive(Deserialize)]
struct RawSample {
    t: f64,
    d: Option<f64>,
    #[serde(rename = "T")]
    temp: Option<f64>,
}

/// Decodes MacDive's readable profile (`ZSAMPLES_JSON`).
pub fn parse_profile(json: &str) -> Result<Vec<Sample>> {
    let raw: Vec<RawSample> =
        serde_json::from_str(json).map_err(|e| Error::BadProfile(e.to_string()))?;
    let mut samples: Vec<Sample> = raw
        .into_iter()
        .filter(|s| s.t >= 0.0)
        .filter_map(|s| {
            Some(Sample {
                time_s: s.t.round() as u32,
                depth_m: s.d?,
                temp_c: s.temp,
            })
        })
        .collect();
    samples.sort_by_key(|s| s.time_s);
    Ok(samples)
}

impl DiveSource for MacDive {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            temperature: true,
            rating: true,
            gps: true,
            profile: true,
        }
    }

    fn dives(&self) -> Result<Vec<Dive>> {
        let has_profile = if self.has_json_profile {
            "D.ZSAMPLES_JSON IS NOT NULL AND length(D.ZSAMPLES_JSON) > 2"
        } else {
            "0"
        };
        let mut stmt = self.conn.prepare(&format!(
            "SELECT COALESCE(D.ZUUID, 'pk:' || D.Z_PK) AS id,
                    D.ZDIVENUMBER AS number,
                    D.ZRAWDATE AS start,
                    D.ZTOTALDURATION AS duration,
                    D.ZMAXDEPTH AS max_depth,
                    D.ZTEMPHIGH AS temp_high,
                    D.ZTEMPLOW AS temp_low,
                    D.ZRATING AS rating,
                    S.ZNAME AS site, S.ZLOCATION AS location, S.ZCOUNTRY AS country,
                    S.ZGPSLAT AS lat, S.ZGPSLON AS lon,
                    {has_profile} AS has_profile
             FROM ZDIVE D
             LEFT JOIN ZDIVESITE S ON D.ZRELATIONSHIPDIVESITE = S.Z_PK
             WHERE D.ZRAWDATE IS NOT NULL"
        ))?;
        let dives = stmt
            .query_map([], dive_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(dives)
    }

    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>> {
        if !self.has_json_profile {
            let exists: bool = self.conn.query_row(
                "SELECT count(*) > 0 FROM ZDIVE WHERE COALESCE(ZUUID, 'pk:' || Z_PK) = ?1",
                [dive_id],
                |r| r.get(0),
            )?;
            return if exists {
                Ok(Vec::new())
            } else {
                Err(Error::UnknownDive(dive_id.to_string()))
            };
        }
        let json: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT ZSAMPLES_JSON FROM ZDIVE
                 WHERE COALESCE(ZUUID, 'pk:' || Z_PK) = ?1",
                [dive_id],
                |r| r.get(0),
            )
            .optional()?;
        match json {
            None => Err(Error::UnknownDive(dive_id.to_string())),
            Some(None) => Ok(Vec::new()),
            Some(Some(j)) => parse_profile(&j),
        }
    }
}

#[cfg(test)]
mod tests;
