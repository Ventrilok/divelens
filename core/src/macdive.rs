//! MacDive reader (SQLite database, macOS).
//!
//! - Dates are Core Data timestamps: seconds since 2001-01-01, holding the
//!   dive computer's wall-clock time.
//! - Profile: decoded from the dive computer's raw download (`ZRAWDATA`) with
//!   libdivecomputer, using the computer name MacDive records (`ZCOMPUTER`).
//!   MacDive's own encoded copy (`ZSAMPLES`) is proprietary and not used, so
//!   dives without a raw download (manual entries, some very old computers)
//!   have no profile.
//! - Missing values: NULL temperature, rating 0, GPS 0/0.

use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row};

use crate::divecomputer::Catalog;
use crate::model::{Capabilities, Dive, DiveSource, Site};
use crate::profile::Sample;
use crate::{Error, Result};

/// Seconds between 1970-01-01 and 2001-01-01 (Core Data reference date).
const CORE_DATA_EPOCH: i64 = 978_307_200;

/// Standard location of the MacDive database, relative to the home folder.
pub const DEFAULT_RELATIVE_PATH: &str = "Library/Application Support/MacDive/MacDive.sqlite";

pub struct MacDive {
    conn: Connection,
    computers: Catalog,
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
        Ok(Self {
            conn,
            computers: Catalog::new()?,
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

/// Site from the `site_id, site, location, country, lat, lon` columns. A site
/// with nothing usable in it counts as no site.
fn site_from_row(r: &Row) -> rusqlite::Result<Site> {
    let lat: Option<f64> = r.get("lat")?;
    let lon: Option<f64> = r.get("lon")?;
    let (lat, lon) = match (lat, lon) {
        (Some(a), Some(b)) if !(a == 0.0 && b == 0.0) => (Some(a), Some(b)),
        _ => (None, None),
    };
    let site = Site {
        id: r.get("site_id")?,
        name: non_empty(r.get("site")?),
        location: non_empty(r.get("location")?),
        country: non_empty(r.get("country")?),
        lat,
        lon,
    };
    let empty = site.name.is_none()
        && site.location.is_none()
        && site.country.is_none()
        && site.lat.is_none();
    Ok(if empty { Site::default() } else { site })
}

fn dive_from_row(r: &Row, computers: &Catalog) -> rusqlite::Result<Dive> {
    let start_raw: f64 = r.get("start")?;
    let duration_s = r
        .get::<_, Option<f64>>("duration")?
        .unwrap_or(0.0)
        .max(0.0)
        .round() as u32;
    let start = wall_clock(start_raw);
    let rating = r
        .get::<_, Option<f64>>("rating")?
        .map(|v| v.round() as i64)
        .filter(|v| (1..=5).contains(v))
        .map(|v| v as u8);
    let has_raw: bool = r.get("has_raw")?;
    let computer: Option<String> = r.get("computer")?;
    let has_profile = has_raw && computer.is_some_and(|c| computers.find(&c).is_some());
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
        site: site_from_row(r)?,
        has_profile,
    })
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
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(D.ZUUID, 'pk:' || D.Z_PK) AS id,
                    D.ZDIVENUMBER AS number,
                    D.ZRAWDATE AS start,
                    D.ZTOTALDURATION AS duration,
                    D.ZMAXDEPTH AS max_depth,
                    D.ZTEMPHIGH AS temp_high,
                    D.ZTEMPLOW AS temp_low,
                    D.ZRATING AS rating,
                    D.ZCOMPUTER AS computer,
                    D.ZRAWDATA IS NOT NULL AND length(D.ZRAWDATA) > 0 AS has_raw,
                    CASE WHEN S.Z_PK IS NULL THEN NULL
                         ELSE COALESCE(S.ZUUID, 'pk:' || S.Z_PK) END AS site_id,
                    S.ZNAME AS site, S.ZLOCATION AS location, S.ZCOUNTRY AS country,
                    S.ZGPSLAT AS lat, S.ZGPSLON AS lon
             FROM ZDIVE D
             LEFT JOIN ZDIVESITE S ON D.ZRELATIONSHIPDIVESITE = S.Z_PK
             WHERE D.ZRAWDATE IS NOT NULL",
        )?;
        let dives = stmt
            .query_map([], |r| dive_from_row(r, &self.computers))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(dives)
    }

    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>> {
        let row: Option<(Option<String>, Option<Vec<u8>>)> = self
            .conn
            .query_row(
                "SELECT ZCOMPUTER, ZRAWDATA FROM ZDIVE
                 WHERE COALESCE(ZUUID, 'pk:' || Z_PK) = ?1",
                [dive_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let (computer, raw) = row.ok_or_else(|| Error::UnknownDive(dive_id.to_string()))?;
        match (computer.and_then(|c| self.computers.find(&c)), raw) {
            (Some(model), Some(raw)) if !raw.is_empty() => self.computers.profile(model, &raw),
            _ => Ok(Vec::new()),
        }
    }

    fn sites(&self) -> Result<Vec<Site>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(ZUUID, 'pk:' || Z_PK) AS site_id,
                    ZNAME AS site, ZLOCATION AS location, ZCOUNTRY AS country,
                    ZGPSLAT AS lat, ZGPSLON AS lon
             FROM ZDIVESITE",
        )?;
        let sites = stmt
            .query_map([], site_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sites.into_iter().filter(|s| s.name.is_some()).collect())
    }
}

#[cfg(test)]
mod tests;
