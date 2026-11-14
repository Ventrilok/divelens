//! Tests against a fictional MacDive database built on the fly with the real
//! MacDive table definitions: the reference log, plus a few MacDive-only
//! cases. No personal data.

use super::*;
use crate::conformance::reference;
use chrono::NaiveDate;
use tempfile::TempDir;

const SCHEMA: &str = "
CREATE TABLE ZDIVE ( Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER, Z_OPT INTEGER, ZDECOMPRESSION INTEGER, ZDIVENUMBER INTEGER, ZHASAIR INTEGER, ZHASNDT INTEGER, ZHASPPO2 INTEGER, ZHASTEMP INTEGER, ZPARSERVERSION INTEGER, ZREPETITIVEDIVENUMBER INTEGER, ZRGBMPERCENT INTEGER, ZSYNCPHOTOS INTEGER, ZRELATIONSHIPCERTIFICATION INTEGER, ZRELATIONSHIPDIVESITE INTEGER, ZRELATIONSHIPDIVER INTEGER, ZAIRTEMP FLOAT, ZAVERAGEDEPTH FLOAT, ZCNS FLOAT, ZDATEADDED TIMESTAMP, ZMAXDEPTH FLOAT, ZMODIFIED TIMESTAMP, ZRATING FLOAT, ZRAWDATE TIMESTAMP, ZSAMPLEINTERVAL FLOAT, ZSETPOINTHIGH FLOAT, ZSETPOINTLOW FLOAT, ZSURFACEINTERVAL FLOAT, ZTEMPHIGH FLOAT, ZTEMPLOW FLOAT, ZTOTALDURATION FLOAT, ZALTITUDEMODE VARCHAR, ZBOATCAPTAIN VARCHAR, ZBOATNAME VARCHAR, ZCOMPUTER VARCHAR, ZCOMPUTERSERIAL VARCHAR, ZCURRENT VARCHAR, ZDATASOURCE VARCHAR, ZDECOMODEL VARCHAR, ZDIVEMASTER VARCHAR, ZDIVEOPERATOR VARCHAR, ZDIVELOGUUID VARCHAR, ZENTRYTYPE VARCHAR, ZGASMODEL VARCHAR, ZIDENTIFIER VARCHAR, ZNOTES VARCHAR, ZPARSERTYPE VARCHAR, ZPERSONALMODE VARCHAR, ZSCRUBBERTIMEAFTER VARCHAR, ZSCRUBBERTIMEBEFORE VARCHAR, ZSCRUBBERTYPE VARCHAR, ZSIGNATURE VARCHAR, ZSURFACECONDITIONS VARCHAR, ZUUID VARCHAR, ZVISIBILITY VARCHAR, ZWEATHER VARCHAR, ZWEIGHT VARCHAR, ZRAWDATA BLOB, ZSAMPLES BLOB, ZTIMEZONE BLOB );
CREATE TABLE ZDIVESITE ( Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER, Z_OPT INTEGER, ZALTITUDE FLOAT, ZGPSLAT FLOAT, ZGPSLON FLOAT, ZMODIFIED TIMESTAMP, ZBODYOFWATER VARCHAR, ZCOUNTRY VARCHAR, ZDIFFICULTY VARCHAR, ZDIVELOGUUID VARCHAR, ZFLAG VARCHAR, ZIMAGE VARCHAR, ZLASTDIVELOGIMAGEHASH VARCHAR, ZLOCATION VARCHAR, ZNAME VARCHAR, ZNOTES VARCHAR, ZUUID VARCHAR, ZWATERTYPE VARCHAR, ZZOOM VARCHAR );
";

// The reference log (`crate::conformance::reference`) as MacDive stores it.
// Dates are seconds since 2001-01-01 00:00 of the wall clock: 771586200 is
// 2025-06-14 09:30:00. Profiles come from raw downloads, added by
// `reference_log`.
const REFERENCE: &str = r#"
INSERT INTO ZDIVESITE (Z_PK, ZUUID, ZNAME, ZLOCATION, ZCOUNTRY, ZGPSLAT, ZGPSLON) VALUES
  (1, NULL, 'Coral Garden', 'North Reef', 'Atlantis', 12.5, -45.25),
  (2, NULL, 'Glacier Wall', 'Frostville', 'Atlantis', -64.125, -60.5),
  (3, NULL, 'Shark Point / Bay', NULL, NULL, NULL, NULL),
  (4, NULL, 'Lemuria / Sunken City / Temple', '', 'Lemuria', -8.25, 115.5),
  -- GPS 0/0 means none.
  (5, NULL, 'Pearl Bank', 'Oyster Town', NULL, 0, 0),
  -- No dive yet.
  (6, 'SITE-6', 'Blue Lagoon', NULL, NULL, 1.25, 2.5),
  (7, 'SITE-7', 'Récif d''Émeraude', 'Île Verte', 'Atlantide', -20.5, 57.25),
  -- GPS only.
  (8, NULL, NULL, NULL, NULL, 25.0, -77.5),
  -- Blank fields only: no site.
  (9, NULL, '  ', '', NULL, NULL, NULL);

INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZMAXDEPTH, ZTEMPHIGH, ZTEMPLOW, ZRATING, ZRELATIONSHIPDIVESITE, ZCOMPUTER, ZNOTES) VALUES
  -- A computer libdivecomputer knows, but no raw download kept.
  (1, 'UUID-100', 100, 757381200, 1920.0, 14.2, 21.0, 21.0, 3.0, 1, 'Suunto Vyper', 'Night dive, lobsters everywhere'),
  -- Notes with Windows line breaks and spaces around.
  (2, 'UUID-101', 101, 771586200, 2550.0, 18.4, 26.0, 24.0, 4.0, 1, 'Sporasub SP2',
   '  Turtle at the wall & a "reef" shark.' || char(13, 10) || 'Second line  '),
  -- Rating 0 means unrated; blank notes.
  (3, 'UUID-102', 102, 771602400, 2100.0, 12.0, 23.0, 22.0, 0.0, 2, 'Sporasub SP2', '  '),
  -- Entered by hand: no computer, no raw download.
  (4, 'UUID-103', 103, 771667200, 3000.0, 20.0, 27.0, 27.0, 2.0, 3, NULL, 'Entered' || char(10) || 'by hand'),
  -- Raw download from a computer libdivecomputer does not know.
  (5, 'UUID-104', 104, 771678000, 1800.0, 15.0, NULL, NULL, NULL, 4, 'Acme Wonder 3000', NULL),
  -- No UUID, no number; linked to the blank site.
  (6, NULL, NULL, 771750000, 720.0, 9.0, 0.0, -1.5, NULL, 9, 'Sporasub SP2', NULL),
  (7, 'UUID-106', 106, 771847200, 1500.0, 8.0, 28.5, 28.0, NULL, 5, 'Sporasub SP2', NULL),
  (8, 'UUID-107', 107, 771847200, 1200.0, 7.0, 28.0, 28.0, 5.0, 5, 'Sporasub SP2', NULL),
  (9, 'UUID-108', 108, 771930000, 3840.0, 31.2, 27.0, 25.0, 3.0, 7, 'Sporasub SP2', 'Manta ray at 30 m — café afterwards.'),
  (10, 'UUID-109', 109, 771953400, 1440.0, 16.0, 26.0, 25.0, NULL, 8, 'Sporasub SP2', NULL);
UPDATE ZDIVE SET ZRAWDATA = x'00112233' WHERE Z_PK = 5;
"#;

// MacDive-only cases, kept out of the reference log.
const EXTRA: &str = r#"
-- Manual dive with nothing but its date (the same as dive 100's).
INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZMAXDEPTH, ZTEMPHIGH, ZTEMPLOW, ZRATING, ZRELATIONSHIPDIVESITE, ZCOMPUTER, ZRAWDATA)
  VALUES (11, NULL, NULL, 757381200, NULL, NULL, NULL, NULL, NULL, NULL, 'No Computer', NULL);
-- Raw download libdivecomputer cannot decode.
INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZCOMPUTER, ZRAWDATA)
  VALUES (12, 'UUID-BAD', 110, 771960000, 600.0, 'Mares Genius', x'00112233');
"#;

/// A Sporasub SP2 download whose profile goes through `points` (time s,
/// depth m, temperature °C): the simplest format libdivecomputer decodes
/// in 1 cm and 0.1 °C steps. One sample every 10 s from 10 s, values linear
/// between the points.
fn sp2_download(points: &[(u32, f64, f64)]) -> Vec<u8> {
    let through: Vec<Sample> = points
        .iter()
        .map(|&(time_s, depth_m, temp_c)| {
            assert_eq!(time_s % 10, 0, "SP2 samples every 10 s");
            Sample {
                time_s,
                depth_m,
                temp_c: Some(temp_c),
            }
        })
        .collect();
    let count = through.last().unwrap().time_s / 10;
    let mut data = vec![0u8; 0x20];
    data[..2].copy_from_slice(&u16::try_from(count).unwrap().to_le_bytes());
    data[0x1A] = 3; // one sample every 10 s
    for i in 1..=count {
        let p = crate::profile::at(&through, i * 10).unwrap();
        let depth = (p.depth_m * 100.0).round() as u32;
        let temp = ((p.temp_c.unwrap() + 20.0) * 10.0).round() as u32;
        assert!(depth < 1 << 14 && temp < 1 << 10);
        data.extend(((temp << 14) | depth).to_le_bytes());
    }
    data
}

/// A database holding `sql` (after the reference log), in WAL mode like every
/// Core Data store, closed. Its name holds what a `file:` URI must escape.
fn database(dir: &Path, sql: &str) -> PathBuf {
    let path = dir.join("Mac Dive ~ #1 100% été.sqlite");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA journal_mode = WAL;").unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn.execute_batch(REFERENCE).unwrap();
    for d in reference::DIVES.iter().filter(|d| !d.profile.is_empty()) {
        let changed = conn
            .execute(
                "UPDATE ZDIVE SET ZRAWDATA = ?1
                 WHERE ZDIVENUMBER IS ?2 AND ZCOMPUTER = 'Sporasub SP2'",
                (sp2_download(d.profile), d.number),
            )
            .unwrap();
        assert_eq!(changed, 1, "dive {:?}", d.number);
    }
    conn.execute_batch(sql).unwrap();
    drop(conn);
    path
}

/// The reference log, for the conformance tests.
pub(crate) fn reference_log(dir: &Path) -> PathBuf {
    database(dir, "")
}

/// The reference log and MacDive's own cases.
fn fixture() -> (TempDir, MacDive) {
    let dir = TempDir::new().unwrap();
    let md = MacDive::open(&database(dir.path(), EXTRA)).unwrap();
    (dir, md)
}

/// Names of the files in `dir`.
fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn reading_creates_no_file_next_to_the_database() {
    let dir = TempDir::new().unwrap();
    let path = database(dir.path(), EXTRA);
    let before = files(dir.path());
    assert_eq!(before.len(), 1, "closed WAL database: no -wal, no -shm");
    let md = MacDive::open(&path).unwrap();
    assert_eq!(md.dives().unwrap().len(), 12);
    md.sites().unwrap();
    md.profile("UUID-101").unwrap();
    drop(md);
    assert_eq!(files(dir.path()), before);
}

/// Saves a change and closes the database the way Apple's SQLite does for
/// MacDive: an empty `-wal` file and the `-shm` index stay next to it.
fn leave_wal_files(path: &Path) {
    let conn = Connection::open(path).unwrap();
    let mut persist: std::ffi::c_int = 1;
    // SAFETY: a valid connection handle and an int, as the file control
    // expects.
    let rc = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            conn.handle(),
            c"main".as_ptr(),
            rusqlite::ffi::SQLITE_FCNTL_PERSIST_WAL,
            (&raw mut persist).cast(),
        )
    };
    assert_eq!(rc, rusqlite::ffi::SQLITE_OK);
    // A change saved, copied into the database on close; the -wal file is
    // then emptied, not deleted.
    conn.execute_batch(
        "PRAGMA journal_size_limit = 0;
         UPDATE ZDIVE SET ZNOTES = ZNOTES WHERE Z_PK = 1;",
    )
    .unwrap();
    drop(conn);
    // An index another SQLite left in its own state (a reader slot marked
    // 0), which a plain read-only open would rewrite.
    let mut shm = path.as_os_str().to_owned();
    shm.push("-shm");
    let mut index = std::fs::read(&shm).unwrap();
    index[104..108].copy_from_slice(&[0; 4]);
    std::fs::write(&shm, &index).unwrap();
}

/// Every file in `dir` with its contents.
fn contents(dir: &Path) -> Vec<(String, Vec<u8>)> {
    files(dir)
        .into_iter()
        .map(|name| {
            let bytes = std::fs::read(dir.join(&name)).unwrap();
            (name, bytes)
        })
        .collect()
}

#[test]
fn reading_changes_no_file_left_by_macdive() {
    let dir = TempDir::new().unwrap();
    let path = database(dir.path(), EXTRA);
    leave_wal_files(&path);
    let before = contents(dir.path());
    let names: Vec<_> = before.iter().map(|f| f.0.as_str()).collect();
    assert_eq!(names.len(), 3, "database, -shm and -wal: {names:?}");
    assert!(
        before
            .iter()
            .any(|(n, b)| n.ends_with("-wal") && b.is_empty())
    );
    let md = MacDive::open(&path).unwrap();
    assert_eq!(md.dives().unwrap().len(), 12);
    md.profile("UUID-101").unwrap();
    drop(md);
    assert!(contents(dir.path()) == before, "a file was changed");

    // The -shm index gone, the empty -wal left.
    let mut shm = path.as_os_str().to_owned();
    shm.push("-shm");
    std::fs::remove_file(&shm).unwrap();
    let before = contents(dir.path());
    let md = MacDive::open(&path).unwrap();
    md.dives().unwrap();
    drop(md);
    assert!(contents(dir.path()) == before, "-shm created");
}

#[test]
fn reads_dives_macdive_has_not_copied_into_the_database_yet() {
    let dir = TempDir::new().unwrap();
    let path = database(dir.path(), EXTRA);
    // MacDive open: its last dive is still in the -wal file only.
    let macdive = Connection::open(&path).unwrap();
    macdive
        .execute_batch(
            "PRAGMA wal_autocheckpoint = 0;
             INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE) VALUES (99, 'UUID-NEW', 111, 771970000);",
        )
        .unwrap();
    let md = MacDive::open(&path).unwrap();
    let dives = md.dives().unwrap();
    assert_eq!(dives.len(), 13);
    assert_eq!(by_id(&dives, "UUID-NEW").number, Some(111));
    drop(md);
    drop(macdive);
}

fn by_id<'a>(dives: &'a [Dive], id: &str) -> &'a Dive {
    dives.iter().find(|d| d.id == id).unwrap()
}

fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(y, mo, d)
        .unwrap()
        .and_hms_opt(h, mi, s)
        .unwrap()
}

// The values every reader shares are checked by the conformance tests
// (`crate::conformance`); below, how MacDive stores them.

#[test]
fn reads_a_full_dive() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    assert_eq!(dives.len(), 12);
    let d = by_id(&dives, "UUID-101");
    assert_eq!(d.number, Some(101));
    assert_eq!(
        d.notes.as_deref(),
        Some("Turtle at the wall & a \"reef\" shark.\nSecond line")
    );
    assert_eq!(d.start, at(2025, 6, 14, 9, 30, 0));
    assert_eq!(d.end, at(2025, 6, 14, 10, 12, 30));
    assert_eq!(d.duration_s, 2550);
    assert_eq!(d.max_depth_m, Some(18.4));
    assert_eq!((d.temp_low_c, d.temp_high_c), (Some(24.0), Some(26.0)));
    assert_eq!(d.rating, Some(4));
    assert_eq!(d.site.id.as_deref(), Some("pk:1"));
    assert_eq!(d.site.name.as_deref(), Some("Coral Garden"));
    assert_eq!(d.site.location.as_deref(), Some("North Reef"));
    assert_eq!(d.site.country.as_deref(), Some("Atlantis"));
    assert_eq!((d.site.lat, d.site.lon), (Some(12.5), Some(-45.25)));
    assert!(d.has_profile, "raw data from a known computer");
}

#[test]
fn decodes_the_profile_from_the_raw_download() {
    let (_dir, md) = fixture();
    let p = md.profile("UUID-101").unwrap();
    assert_eq!(p.len(), 258, "every 10 s up to 43:00");
    assert_eq!(
        p[0],
        Sample {
            time_s: 10,
            depth_m: 2.0,
            temp_c: Some(26.0)
        }
    );
    assert_eq!(
        (p[29].time_s, p[29].depth_m, p[29].temp_c),
        (300, 18.4, Some(24.0))
    );
}

#[test]
fn keeps_zero_and_negative_temperatures_and_drops_gps_zero() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    let d = by_id(&dives, "pk:6");
    assert_eq!((d.temp_low_c, d.temp_high_c), (Some(-1.5), Some(0.0)));
    let d = by_id(&dives, "UUID-102");
    assert_eq!(d.rating, None, "rating 0");
    let d = by_id(&dives, "UUID-106");
    assert_eq!((d.site.lat, d.site.lon), (None, None));
    assert_eq!(d.site.location.as_deref(), Some("Oyster Town"));
    let d = by_id(&dives, "UUID-104");
    assert_eq!(d.site.location, None, "blank location");
    assert!(!d.has_profile, "unknown computer");
    assert!(md.profile("UUID-104").unwrap().is_empty());
}

#[test]
fn manual_dive_without_anything() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    let d = by_id(&dives, "pk:11");
    assert_eq!(d.number, None);
    assert_eq!(d.start, at(2024, 12, 31, 23, 40, 0));
    assert_eq!(d.duration_s, 0);
    assert_eq!(d.max_depth_m, None);
    assert_eq!((d.temp_low_c, d.temp_high_c), (None, None));
    assert_eq!(d.site, Site::default());
    assert!(!d.has_profile);
    assert!(md.profile("pk:11").unwrap().is_empty());
}

#[test]
fn blank_site_fields_are_missing() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    assert_eq!(by_id(&dives, "pk:6").site, Site::default());
    let d = by_id(&dives, "UUID-100");
    assert!(!d.has_profile, "known computer, no raw data");
    assert_eq!(by_id(&dives, "UUID-102").notes, None, "blank notes");
}

#[test]
fn lists_named_sites_even_without_dives() {
    let (_dir, md) = fixture();
    let mut sites = md.sites().unwrap();
    sites.sort_by(|a, b| a.id.cmp(&b.id));
    let ids: Vec<_> = sites.iter().map(|s| s.id.as_deref().unwrap()).collect();
    assert_eq!(
        ids,
        ["SITE-6", "SITE-7", "pk:1", "pk:2", "pk:3", "pk:4", "pk:5"],
        "unnamed site 8 and blank site 9 left out"
    );
    let lagoon = &sites[0];
    assert_eq!(lagoon.name.as_deref(), Some("Blue Lagoon"));
    assert_eq!((lagoon.lat, lagoon.lon), (Some(1.25), Some(2.5)));
    assert_eq!(sites[1].country.as_deref(), Some("Atlantide"));
    assert_eq!(
        (sites[6].lat, sites[6].lon),
        (None, None),
        "GPS 0/0 is missing"
    );
}

#[test]
fn undecodable_raw_data_is_an_error() {
    let (_dir, md) = fixture();
    assert!(matches!(md.profile("UUID-BAD"), Err(Error::Decoder(_))));
}

#[test]
fn unknown_dive_is_an_error() {
    let (_dir, md) = fixture();
    assert!(matches!(md.profile("nope"), Err(Error::UnknownDive(_))));
}

#[test]
fn missing_file_and_wrong_file_are_reported() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("none.sqlite");
    assert!(matches!(MacDive::open(&missing), Err(Error::NotFound(_))));

    let other = dir.path().join("other.sqlite");
    Connection::open(&other)
        .unwrap()
        .execute_batch("CREATE TABLE T (x);")
        .unwrap();
    assert!(matches!(MacDive::open(&other), Err(Error::NotADiveLog(_))));

    let text = dir.path().join("notes.txt");
    std::fs::write(&text, "hello").unwrap();
    assert!(matches!(MacDive::open(&text), Err(Error::NotADiveLog(_))));
}
