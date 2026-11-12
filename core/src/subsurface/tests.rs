//! Tests against fictional Subsurface logs (`core/fixtures/subsurface`). No
//! personal data.

use super::*;
use chrono::NaiveDate;
use tempfile::TempDir;

pub(super) fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/subsurface")
        .join(name)
}

fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(y, mo, d)
        .unwrap()
        .and_hms_opt(h, mi, s)
        .unwrap()
}

fn by_number(dives: &[Dive], n: u32) -> &Dive {
    dives.iter().find(|d| d.number == Some(n)).unwrap()
}

fn close(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-9)
}

/// Everything the fictional log must give, whatever its storage format.
pub(super) fn check_fictional(log: &Subsurface) {
    let dives = log.dives().unwrap();
    assert_eq!(dives.len(), 7);

    // 101: in a trip, one computer, site split from "Country / Location / Site".
    let d = by_number(&dives, 101);
    assert_eq!(d.id, "2025-06-14T09:30:00");
    assert_eq!(d.start, at(2025, 6, 14, 9, 30, 0));
    assert_eq!(
        d.duration_s,
        42 * 60 + 30,
        "Subsurface's duration, not the last sample"
    );
    assert_eq!(d.end, at(2025, 6, 14, 10, 12, 30));
    assert!(close(d.max_depth_m, 18.4));
    assert!(close(d.temp_low_c, 24.0), "water temperature");
    assert!(close(d.temp_high_c, 26.0), "warmest sample, never the air");
    assert_eq!(d.rating, Some(4));
    assert!(d.has_profile && !d.invalid);
    assert_eq!(
        d.site,
        Site {
            id: Some("c0ffee01".into()),
            name: Some("Coral Garden".into()),
            location: Some("North Reef".into()),
            country: Some("Atlantis".into()),
            lat: Some(12.5),
            lon: Some(-45.25),
        }
    );
    let p = log.profile(&d.id).unwrap();
    assert_eq!(p.len(), 10);
    assert_eq!(p[0].time_s, 10);
    assert!(close(Some(p[3].depth_m), 18.0));
    assert_eq!(
        p[3].temp_c,
        Some(24.0),
        "unchanged temperature carried over"
    );
    assert_eq!(p[9].time_s, 43 * 60);

    // 102: two computers, the first one counts; no water temperature, so the
    // coldest sample; site with Subsurface's own country and city.
    let d = by_number(&dives, 102);
    assert_eq!(d.duration_s, 35 * 60);
    assert!(close(d.max_depth_m, 12.0));
    assert!(close(d.temp_low_c, 22.0));
    assert!(close(d.temp_high_c, 23.0));
    assert_eq!(d.rating, None);
    let p = log.profile(&d.id).unwrap();
    assert_eq!(p.len(), 4, "first computer's samples");
    assert_eq!(d.site.name.as_deref(), Some("Glacier Wall"));
    assert_eq!(d.site.location.as_deref(), Some("Frostville"));
    assert_eq!(d.site.country.as_deref(), Some("Atlantis"));

    // 103: entered by hand in Subsurface: no profile; temperature of the dive.
    let d = by_number(&dives, 103);
    assert!(!d.has_profile);
    assert!(log.profile(&d.id).unwrap().is_empty());
    assert!(close(d.temp_low_c, 27.0) && close(d.temp_high_c, 27.0));
    assert!(close(d.max_depth_m, 20.0));
    assert_eq!(d.rating, Some(2));
    assert_eq!(
        d.site.name.as_deref(),
        Some("Shark Point / Bay"),
        "two parts: kept"
    );
    assert_eq!(
        (d.site.location.as_ref(), d.site.country.as_ref()),
        (None, None)
    );
    assert_eq!((d.site.lat, d.site.lon), (None, None));

    // 104: "No Computer" (imported from MacDive): no profile, no temperature;
    // three-part name kept whole because Subsurface has a country.
    let d = by_number(&dives, 104);
    assert!(!d.has_profile);
    assert_eq!((d.temp_low_c, d.temp_high_c), (None, None));
    assert_eq!(d.duration_s, 30 * 60);
    assert_eq!(
        d.site.name.as_deref(),
        Some("Lemuria / Sunken City / Temple")
    );
    assert_eq!(d.site.country.as_deref(), Some("Lemuria"));
    assert_eq!(d.site.location, None);

    // No number, invalid, no site, below zero.
    let d = dives.iter().find(|d| d.number.is_none()).unwrap();
    assert_eq!(d.id, "2025-06-16T07:00:00");
    assert!(d.invalid && d.has_profile);
    assert_eq!(d.site, Site::default());
    assert!(close(d.temp_low_c, -1.5) && close(d.temp_high_c, -1.0));

    // 106 and 107 start at the same second: ordered by number.
    assert_eq!(by_number(&dives, 106).id, "2025-06-17T10:00:00");
    assert_eq!(by_number(&dives, 107).id, "2025-06-17T10:00:00#2");
    assert_eq!(by_number(&dives, 107).rating, Some(5));
    let pearl = &by_number(&dives, 106).site;
    assert_eq!(
        pearl.location.as_deref(),
        Some("Oyster Town"),
        "town as city"
    );

    // Sites: every named one, including a site with no dive.
    let sites = log.sites().unwrap();
    assert_eq!(sites.len(), 6);
    let lagoon = sites
        .iter()
        .find(|s| s.name.as_deref() == Some("Blue Lagoon"))
        .unwrap();
    assert_eq!(
        lagoon.id.as_deref(),
        Some("000abcde"),
        "XML pads with spaces"
    );
    assert_eq!((lagoon.lat, lagoon.lon), (Some(1.25), Some(2.5)));

    assert!(matches!(log.profile("nope"), Err(Error::UnknownDive(_))));
}

#[test]
fn reads_the_fictional_xml_log() {
    let log = Subsurface::open(&fixture("fictional.xml")).unwrap();
    assert_eq!(log.storage(), &Storage::Xml);
    check_fictional(&log);
}

fn write(dir: &TempDir, name: &str, text: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn reports_missing_foreign_and_old_files() {
    let dir = TempDir::new().unwrap();
    assert!(matches!(
        Subsurface::open(&dir.path().join("none.xml")),
        Err(Error::NotFound(_))
    ));
    let other = write(&dir, "other.xml", "<dives><dive/></dives>");
    assert!(matches!(
        Subsurface::open(&other),
        Err(Error::NotADiveLog(_))
    ));
    let binary = write(&dir, "binary.ssrf", "\u{0}\u{1}not xml at all");
    assert!(matches!(
        Subsurface::open(&binary),
        Err(Error::NotADiveLog(_))
    ));
    let old = write(
        &dir,
        "old.xml",
        "<divelog program='subsurface' version='2'><dives/></divelog>",
    );
    assert!(matches!(Subsurface::open(&old), Err(Error::OldFormat(_))));
    let cut = write(
        &dir,
        "cut.ssrf",
        "<divelog program='subsurface' version='3'><dives><dive date='2025-01-01' time='10:00:00'><divecomputer",
    );
    assert!(matches!(Subsurface::open(&cut), Err(Error::Database(_))));
}

#[test]
fn reads_an_empty_log() {
    let dir = TempDir::new().unwrap();
    let path = write(
        &dir,
        "empty.ssrf",
        "<divelog program='subsurface' version='3'>\n<settings>\n</settings>\n<dives>\n</dives>\n</divelog>\n",
    );
    let log = Subsurface::open(&path).unwrap();
    assert!(log.dives().unwrap().is_empty());
    assert!(log.sites().unwrap().is_empty());
}

#[test]
fn splits_site_names_only_when_subsurface_has_no_place() {
    let raw = |name: &str, country: Option<&str>, city: Option<&str>| RawSite {
        uuid: 1,
        name: Some(name.into()),
        country: country.map(Into::into),
        city: city.map(Into::into),
        ..RawSite::default()
    };
    let s = site(&raw("A / B / C", None, None));
    assert_eq!(
        (
            s.country.as_deref(),
            s.location.as_deref(),
            s.name.as_deref()
        ),
        (Some("A"), Some("B"), Some("C"))
    );
    let s = site(&raw("A / B / C / D", None, None));
    assert_eq!(s.name.as_deref(), Some("A / B / C / D"));
    assert_eq!(s.country, None);
    let s = site(&raw("A/B/C", None, None));
    assert_eq!(s.name.as_deref(), Some("A/B/C"), "separator is ' / '");
    let s = site(&raw("A / B / C", None, Some("Town")));
    assert_eq!(s.name.as_deref(), Some("A / B / C"));
    assert_eq!(s.location.as_deref(), Some("Town"));
    let s = site(&raw("  ", None, None));
    assert_eq!(s.name, None);
}

#[test]
fn parses_subsurface_values() {
    assert_eq!(duration("45:00 min"), Some(2700));
    assert_eq!(duration("0:10 min"), Some(10));
    assert_eq!(duration("1:05:00"), Some(3900));
    assert_eq!(duration("45 min"), Some(2700));
    assert_eq!(duration("x"), None);
    assert_eq!(depth("18.4 m"), Some(18.4));
    assert_eq!(depth("18.4m"), Some(18.4));
    assert!((depth("10 ft").unwrap() - 3.048).abs() < 1e-9);
    assert_eq!(temperature("24.0 C"), Some(24.0));
    assert_eq!(temperature("-1.5°C"), Some(-1.5));
    assert!((temperature("77 F").unwrap() - 25.0).abs() < 1e-9);
    assert_eq!(gps("12.500000 -45.250000"), Some((12.5, -45.25)));
    assert_eq!(gps("0.000000 0.000000"), None);
    assert_eq!(uuid("   abcde"), Some(0xabcde));
    assert_eq!(uuid("c0ffee01"), Some(0xc0ffee01));
    assert_eq!(uuid(""), None);
}

#[test]
fn default_path_is_the_login_name() {
    assert_eq!(
        default_path(
            Path::new("/Users/diver/Library/Application Support/Subsurface"),
            "diver"
        ),
        Path::new("/Users/diver/Library/Application Support/Subsurface/diver.xml")
    );
}
