//! The reference log as Example stores it, built on the fly (fictional data
//! only), plus what the common tests do not cover: a case the reference log
//! leaves out, and the errors of `open`.

use std::path::PathBuf;

use super::*;
use crate::conformance::reference;
use tempfile::TempDir;

// ADAPT: the application's own table definitions, as it creates them (taken
// from an empty log it made, never from one with dives in it).
const SCHEMA: &str = "
CREATE TABLE sites ( id TEXT PRIMARY KEY, name TEXT, place TEXT, country TEXT, latitude REAL, longitude REAL );
CREATE TABLE dives ( id TEXT PRIMARY KEY, number INTEGER, start TEXT NOT NULL, duration_s INTEGER NOT NULL, max_depth_m REAL, temp_min_c REAL, temp_max_c REAL, rating INTEGER NOT NULL DEFAULT 0, notes TEXT, site_id TEXT REFERENCES sites (id) );
CREATE TABLE samples ( dive_id TEXT NOT NULL REFERENCES dives (id), time_s INTEGER NOT NULL, depth_m REAL NOT NULL, temp_c REAL );
CREATE INDEX samples_dive ON samples (dive_id);
";

// ADAPT: the reference log (`crate::conformance::reference`, described in
// `docs/contributing/reference-log.md`) as the application stores it, with
// its own ways of storing missing values. Profiles are added by
// `reference_log`.
const REFERENCE: &str = r#"
INSERT INTO sites (id, name, place, country, latitude, longitude) VALUES
  ('site-1', 'Coral Garden', 'North Reef', 'Atlantis', 12.5, -45.25),
  ('site-2', 'Glacier Wall', 'Frostville', 'Atlantis', -64.125, -60.5),
  ('site-3', 'Shark Point / Bay', NULL, NULL, NULL, NULL),
  -- A blank place is no place.
  ('site-4', 'Lemuria / Sunken City / Temple', '', 'Lemuria', -8.25, 115.5),
  ('site-5', 'Pearl Bank', 'Oyster Town', NULL, NULL, NULL),
  -- No dive yet.
  ('site-6', 'Blue Lagoon', NULL, NULL, 1.25, 2.5),
  ('site-7', 'Récif d''Émeraude', 'Île Verte', 'Atlantide', -20.5, 57.25),
  -- GPS only: on its dive, not in the list of sites.
  ('site-8', NULL, NULL, NULL, 25.0, -77.5);

INSERT INTO dives (id, number, start, duration_s, max_depth_m, temp_min_c, temp_max_c, rating, notes, site_id) VALUES
  -- Across midnight and the new year; no samples kept.
  ('dive-100', 100, '2024-12-31 23:40:00', 1920, 14.2, 21.0, 21.0, 3, 'Night dive, lobsters everywhere', 'site-1'),
  -- Notes with Windows line breaks and spaces around.
  ('dive-101', 101, '2025-06-14 09:30:00', 2550, 18.4, 24.0, 26.0, 4,
   '  Turtle at the wall & a "reef" shark.' || char(13, 10) || 'Second line  ', 'site-1'),
  -- Rating 0 means unrated; blank notes.
  ('dive-102', 102, '2025-06-14 14:00:00', 2100, 12.0, 22.0, 23.0, 0, '  ', 'site-2'),
  -- Entered by hand: no samples.
  ('dive-103', 103, '2025-06-15 08:00:00', 3000, 20.0, 27.0, 27.0, 2, 'Entered' || char(10) || 'by hand', 'site-3'),
  -- No samples, no temperature.
  ('dive-104', 104, '2025-06-15 11:00:00', 1800, 15.0, NULL, NULL, 0, NULL, 'site-4'),
  -- No number, no site; 0 °C is a temperature.
  ('dive-unnumbered', NULL, '2025-06-16 07:00:00', 720, 9.0, -1.5, 0.0, 0, NULL, NULL),
  -- 106 and 107 start at the same second.
  ('dive-106', 106, '2025-06-17 10:00:00', 1500, 8.0, 28.0, 28.5, 0, NULL, 'site-5'),
  ('dive-107', 107, '2025-06-17 10:00:00', 1200, 7.0, 28.0, 28.0, 5, NULL, 'site-5'),
  ('dive-108', 108, '2025-06-18 09:00:00', 3840, 31.2, 25.0, 27.0, 3, 'Manta ray at 30 m — café afterwards.', 'site-7'),
  ('dive-109', 109, '2025-06-18 15:30:00', 1440, 16.0, 25.0, 26.0, 0, NULL, 'site-8');
"#;

// ADAPT: the application's own cases, kept out of the reference log because
// another application cannot store them.
const EXTRA: &str = "
-- A latitude without a longitude: no position.
INSERT INTO sites (id, name, latitude) VALUES ('site-9', 'Half Mapped Reef', 33.75);
";

/// A database holding the reference log, then `sql`. Its name holds what a
/// `file:` URI must escape.
fn database(dir: &Path, sql: &str) -> PathBuf {
    let path = dir.join("Example log #1 été.sqlite");
    let conn = Connection::open(&path).unwrap();
    // ADAPT: the journal mode the application uses. A log in WAL mode, closed,
    // must be read leaving no -wal or -shm file behind.
    conn.execute_batch("PRAGMA journal_mode = WAL;").unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn.execute_batch(REFERENCE).unwrap();
    // ADAPT: the profiles as the application stores them. Here the points of
    // the reference profiles, last first: the reader sorts them.
    for d in &reference::DIVES {
        for &(time_s, depth_m, temp_c) in d.profile.iter().rev() {
            let added = conn
                .execute(
                    "INSERT INTO samples (dive_id, time_s, depth_m, temp_c)
                     SELECT id, ?2, ?3, ?4 FROM dives WHERE number IS ?1",
                    (d.number, time_s, depth_m, temp_c),
                )
                .unwrap();
            assert_eq!(added, 1, "dive {:?}", d.number);
        }
    }
    conn.execute_batch(sql).unwrap();
    drop(conn);
    path
}

/// The reference log, for the conformance tests.
pub(crate) fn reference_log(dir: &Path) -> PathBuf {
    database(dir, "")
}

#[test]
fn a_single_coordinate_is_no_position() {
    let dir = TempDir::new().unwrap();
    let log = Example::open(&database(dir.path(), EXTRA)).unwrap();
    let sites = log.sites().unwrap();
    let reef = sites
        .iter()
        .find(|s| s.id.as_deref() == Some("site-9"))
        .unwrap();
    assert_eq!(reef.name.as_deref(), Some("Half Mapped Reef"));
    assert_eq!((reef.lat, reef.lon), (None, None));
}

#[test]
fn missing_file_and_wrong_file_are_reported() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("none.sqlite");
    assert!(matches!(Example::open(&missing), Err(Error::NotFound(_))));

    // Another application's database.
    let other = dir.path().join("other.sqlite");
    Connection::open(&other)
        .unwrap()
        .execute_batch("CREATE TABLE dives (x); CREATE TABLE sites (x);")
        .unwrap();
    assert!(matches!(Example::open(&other), Err(Error::NotADiveLog(_))));

    let text = dir.path().join("notes.txt");
    std::fs::write(&text, "hello").unwrap();
    assert!(matches!(Example::open(&text), Err(Error::NotADiveLog(_))));
}
