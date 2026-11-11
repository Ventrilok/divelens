//! Tests against a fictional MacDive database built on the fly with the real
//! MacDive table definitions. No personal data.

use super::*;
use chrono::NaiveDate;
use tempfile::TempDir;

const SCHEMA: &str = "
CREATE TABLE ZDIVE ( Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER, Z_OPT INTEGER, ZDECOMPRESSION INTEGER, ZDIVENUMBER INTEGER, ZHASAIR INTEGER, ZHASNDT INTEGER, ZHASPPO2 INTEGER, ZHASTEMP INTEGER, ZPARSERVERSION INTEGER, ZREPETITIVEDIVENUMBER INTEGER, ZRGBMPERCENT INTEGER, ZSYNCPHOTOS INTEGER, ZRELATIONSHIPCERTIFICATION INTEGER, ZRELATIONSHIPDIVESITE INTEGER, ZRELATIONSHIPDIVER INTEGER, ZAIRTEMP FLOAT, ZAVERAGEDEPTH FLOAT, ZCNS FLOAT, ZDATEADDED TIMESTAMP, ZMAXDEPTH FLOAT, ZMODIFIED TIMESTAMP, ZRATING FLOAT, ZRAWDATE TIMESTAMP, ZSAMPLEINTERVAL FLOAT, ZSETPOINTHIGH FLOAT, ZSETPOINTLOW FLOAT, ZSURFACEINTERVAL FLOAT, ZTEMPHIGH FLOAT, ZTEMPLOW FLOAT, ZTOTALDURATION FLOAT, ZALTITUDEMODE VARCHAR, ZBOATCAPTAIN VARCHAR, ZBOATNAME VARCHAR, ZCOMPUTER VARCHAR, ZCOMPUTERSERIAL VARCHAR, ZCURRENT VARCHAR, ZDATASOURCE VARCHAR, ZDECOMODEL VARCHAR, ZDIVEMASTER VARCHAR, ZDIVEOPERATOR VARCHAR, ZDIVELOGUUID VARCHAR, ZENTRYTYPE VARCHAR, ZGASMODEL VARCHAR, ZIDENTIFIER VARCHAR, ZNOTES VARCHAR, ZPARSERTYPE VARCHAR, ZPERSONALMODE VARCHAR, ZSCRUBBERTIMEAFTER VARCHAR, ZSCRUBBERTIMEBEFORE VARCHAR, ZSCRUBBERTYPE VARCHAR, ZSIGNATURE VARCHAR, ZSURFACECONDITIONS VARCHAR, ZUUID VARCHAR, ZVISIBILITY VARCHAR, ZWEATHER VARCHAR, ZWEIGHT VARCHAR, ZRAWDATA BLOB, ZSAMPLES BLOB, ZTIMEZONE BLOB , ZSAMPLES_JSON TEXT);
CREATE TABLE ZDIVESITE ( Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER, Z_OPT INTEGER, ZALTITUDE FLOAT, ZGPSLAT FLOAT, ZGPSLON FLOAT, ZMODIFIED TIMESTAMP, ZBODYOFWATER VARCHAR, ZCOUNTRY VARCHAR, ZDIFFICULTY VARCHAR, ZDIVELOGUUID VARCHAR, ZFLAG VARCHAR, ZIMAGE VARCHAR, ZLASTDIVELOGIMAGEHASH VARCHAR, ZLOCATION VARCHAR, ZNAME VARCHAR, ZNOTES VARCHAR, ZUUID VARCHAR, ZWATERTYPE VARCHAR, ZZOOM VARCHAR );
";

// Fictional sites and dives.
const DATA: &str = r#"
INSERT INTO ZDIVESITE (Z_PK, ZNAME, ZLOCATION, ZCOUNTRY, ZGPSLAT, ZGPSLON)
  VALUES (1, 'Coral Garden', 'North Reef', 'Atlantis', 12.5, -45.25),
         (2, 'Glacier Wall', NULL, 'Égypte', 0, 0),
         (3, '  ', '', NULL, NULL, NULL);

-- 1: full dive with profile, rated 4, GPS.
INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZMAXDEPTH, ZTEMPHIGH, ZTEMPLOW, ZRATING, ZRELATIONSHIPDIVESITE, ZSAMPLES_JSON)
  VALUES (1, 'UUID-A', 12, 771586200, 2130.0, 18.4, 26.0, 24.0, 4.0, 1,
  '[{"t":0,"d":1.40,"T":26.0,"air":3200.0,"ppo2":0.23},{"t":10,"d":3.20,"T":25.0},{"t":20,"d":5.00}]');

-- 2: cold dive, negative temperature, GPS 0/0 = missing, unrated.
INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZMAXDEPTH, ZTEMPHIGH, ZTEMPLOW, ZRATING, ZRELATIONSHIPDIVESITE, ZSAMPLES_JSON)
  VALUES (2, 'UUID-B', 13, 771602700, 600.0, 9.0, 0.0, -1.5, 0.0, 2,
  '[{"t":10,"d":4.0,"T":-1.0},{"t":0,"d":0.5,"T":0.0}]');

-- 3: manual dive, no profile, no temperature, no site, no number, no UUID.
INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZMAXDEPTH, ZTEMPHIGH, ZTEMPLOW, ZRATING, ZRELATIONSHIPDIVESITE, ZSAMPLES_JSON)
  VALUES (3, NULL, NULL, 757381200, NULL, NULL, NULL, NULL, NULL, NULL, NULL);

-- 4: blank site fields are treated as missing.
INSERT INTO ZDIVE (Z_PK, ZUUID, ZDIVENUMBER, ZRAWDATE, ZTOTALDURATION, ZRELATIONSHIPDIVESITE)
  VALUES (4, 'UUID-D', 14, 757381200, 60.0, 3);
"#;

fn fixture() -> (TempDir, MacDive) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("Mac Dive ~ test.sqlite");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn.execute_batch(DATA).unwrap();
    drop(conn);
    let md = MacDive::open(&path).unwrap();
    (dir, md)
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

#[test]
fn reads_a_full_dive() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    assert_eq!(dives.len(), 4);
    let d = by_id(&dives, "UUID-A");
    assert_eq!(d.number, Some(12));
    assert_eq!(d.start, at(2025, 6, 14, 9, 30, 0));
    assert_eq!(d.end, at(2025, 6, 14, 10, 5, 30));
    assert_eq!(d.duration_s, 2130);
    assert_eq!(d.max_depth_m, Some(18.4));
    assert_eq!((d.temp_low_c, d.temp_high_c), (Some(24.0), Some(26.0)));
    assert_eq!(d.rating, Some(4));
    assert_eq!(d.site.name.as_deref(), Some("Coral Garden"));
    assert_eq!(d.site.location.as_deref(), Some("North Reef"));
    assert_eq!(d.site.country.as_deref(), Some("Atlantis"));
    assert_eq!((d.site.lat, d.site.lon), (Some(12.5), Some(-45.25)));
    assert!(d.has_profile);
}

#[test]
fn keeps_zero_and_negative_temperatures_and_drops_gps_zero() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    let d = by_id(&dives, "UUID-B");
    assert_eq!((d.temp_low_c, d.temp_high_c), (Some(-1.5), Some(0.0)));
    assert_eq!(d.rating, None);
    assert_eq!((d.site.lat, d.site.lon), (None, None));
    assert_eq!(d.site.location, None);
}

#[test]
fn manual_dive_without_anything() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    let d = by_id(&dives, "pk:3");
    assert_eq!(d.number, None);
    assert_eq!(d.start, at(2024, 12, 31, 23, 40, 0));
    assert_eq!(d.duration_s, 0);
    assert_eq!(d.max_depth_m, None);
    assert_eq!((d.temp_low_c, d.temp_high_c), (None, None));
    assert_eq!(d.site, Site::default());
    assert!(!d.has_profile);
    assert!(md.profile("pk:3").unwrap().is_empty());
}

#[test]
fn blank_site_fields_are_missing() {
    let (_dir, md) = fixture();
    let dives = md.dives().unwrap();
    assert_eq!(by_id(&dives, "UUID-D").site, Site::default());
}

#[test]
fn reads_profiles_sorted_with_optional_temperature() {
    let (_dir, md) = fixture();
    let p = md.profile("UUID-A").unwrap();
    assert_eq!(p.len(), 3);
    assert_eq!(
        p[0],
        Sample {
            time_s: 0,
            depth_m: 1.4,
            temp_c: Some(26.0)
        }
    );
    assert_eq!(
        p[2],
        Sample {
            time_s: 20,
            depth_m: 5.0,
            temp_c: None
        }
    );
    let p = md.profile("UUID-B").unwrap();
    assert_eq!(p[0].time_s, 0);
    assert_eq!(p[1].temp_c, Some(-1.0));
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

#[test]
fn bad_profile_json_is_an_error() {
    assert!(matches!(
        parse_profile("not json"),
        Err(Error::BadProfile(_))
    ));
}
