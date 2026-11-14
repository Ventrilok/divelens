//! Tests against fictional Subsurface logs (`core/fixtures/subsurface`): the
//! reference log in both storage formats. No personal data.

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

/// How Subsurface's reading rules apply to the fictional log, whatever its
/// storage format. The values every reader shares are checked by the
/// conformance tests (`crate::conformance`).
pub(super) fn check_fictional(log: &Subsurface) {
    let dives = log.dives().unwrap();
    assert_eq!(dives.len(), 10);

    // 100: a dive computer but no samples: no profile; the water temperature
    // is both the low and the high.
    let d = by_number(&dives, 100);
    assert!(!d.has_profile);
    assert!(close(d.temp_low_c, 21.0) && close(d.temp_high_c, 21.0));

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
        d.notes.as_deref(),
        Some("Turtle at the wall & a \"reef\" shark.\nSecond line"),
        "the dive's notes, not the trip's"
    );
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
    assert_eq!(d.notes, None);
    let p = log.profile(&d.id).unwrap();
    assert_eq!(p.len(), 4, "first computer's samples");
    assert_eq!(d.site.name.as_deref(), Some("Glacier Wall"));
    assert_eq!(d.site.location.as_deref(), Some("Frostville"));
    assert_eq!(d.site.country.as_deref(), Some("Atlantis"));

    // 103: entered by hand in Subsurface: no profile; temperature of the dive.
    let d = by_number(&dives, 103);
    assert_eq!(d.notes.as_deref(), Some("Entered\nby hand"));
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
    assert!(close(d.temp_low_c, -1.5) && close(d.temp_high_c, 0.0));

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

    // 108: accents and an escaped apostrophe, split from the name.
    let reef = &by_number(&dives, 108).site;
    assert_eq!(
        (
            reef.name.as_deref(),
            reef.location.as_deref(),
            reef.country.as_deref()
        ),
        (
            Some("Récif d'Émeraude"),
            Some("Île Verte"),
            Some("Atlantide")
        )
    );

    // 109: a site with GPS and no name.
    let gps_only = &by_number(&dives, 109).site;
    assert_eq!(gps_only.id.as_deref(), Some("c0ffee08"));
    assert_eq!(gps_only.name, None);
    assert_eq!((gps_only.lat, gps_only.lon), (Some(25.0), Some(-77.5)));

    // Sites: every named one, including a site with no dive.
    let sites = log.sites().unwrap();
    assert_eq!(sites.len(), 7, "the site with no name left out");
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

// ---------------------------------------------------------------------------
// Git storage. Repositories are built with the git command from
// `fixtures/subsurface/git-tree`, the fictional log in Subsurface's git
// layout.

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args([
            "-c",
            "init.defaultBranch=master",
            "-c",
            "core.autocrlf=false",
        ])
        // No housekeeping left running in the background once a command
        // returns: it would change the repository while the tests look at it.
        .args(["-c", "gc.auto=0", "-c", "maintenance.auto=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be installed to run these tests");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}

/// A repository holding the fictional log on `branch`, the current branch.
fn git_log(dir: &TempDir, branch: &str) -> PathBuf {
    git_log_in(dir.path(), branch)
}

fn git_log_in(dir: &Path, branch: &str) -> PathBuf {
    let work = dir.join("log");
    copy_tree(&fixture("git-tree"), &work);
    git(&work, &["init", "-q"]);
    git(&work, &["checkout", "-q", "-b", branch]);
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "Subsurface log"]);
    work
}

/// The reference log as an XML file, for the conformance tests.
pub(crate) fn reference_xml_log(dir: &Path) -> PathBuf {
    let path = dir.join("diver.xml");
    std::fs::copy(fixture("fictional.xml"), &path).unwrap();
    path
}

/// The reference log in git storage, for the conformance tests.
pub(crate) fn reference_git_log(dir: &Path) -> PathBuf {
    git_log_in(dir, "main")
}

/// Subsurface's runs of the common tests, one per storage format: its copies
/// of the reference log, opened as the app opens them (each time anew).
pub(crate) fn conformance() -> Vec<crate::conformance::Run> {
    let run = |write_log: fn(&Path) -> PathBuf| crate::conformance::Run {
        write_log,
        open: |path| Ok((APP.open)(path, &crate::apps::Kept::default())?.log),
        capabilities: Capabilities {
            temperature: true,
            rating: true,
            gps: true,
            profile: true,
        },
        marks_invalid: true,
    };
    vec![run(reference_xml_log), run(reference_git_log)]
}

#[test]
fn reads_the_fictional_git_log() {
    let dir = TempDir::new().unwrap();
    let path = git_log(&dir, "main");
    let log = Subsurface::open(&path).unwrap();
    assert_eq!(
        log.storage(),
        &Storage::Git {
            branch: "main".into()
        }
    );
    check_fictional(&log);
}

#[test]
fn git_and_xml_give_the_same_log() {
    let dir = TempDir::new().unwrap();
    let git = Subsurface::open(&git_log(&dir, "main")).unwrap();
    let xml = Subsurface::open(&fixture("fictional.xml")).unwrap();
    assert_eq!(git.dives().unwrap(), xml.dives().unwrap());
    let mut a = git.sites().unwrap();
    let mut b = xml.sites().unwrap();
    a.sort_by(|x, y| x.id.cmp(&y.id));
    b.sort_by(|x, y| x.id.cmp(&y.id));
    assert_eq!(a, b);
    for d in xml.dives().unwrap() {
        assert_eq!(git.profile(&d.id).unwrap(), xml.profile(&d.id).unwrap());
    }
}

#[test]
fn reads_the_commit_not_the_files() {
    let dir = TempDir::new().unwrap();
    let work = git_log(&dir, "main");
    // Packed objects, and no files at all: a bare copy.
    git(&work, &["gc", "-q"]);
    git(dir.path(), &["clone", "-q", "--bare", "log", "bare"]);
    check_fictional(&Subsurface::open(&dir.path().join("bare")).unwrap());
    // Files deleted from the folder, commit untouched.
    std::fs::remove_dir_all(work.join("2025")).unwrap();
    check_fictional(&Subsurface::open(&work).unwrap());
}

#[test]
fn picks_the_only_log_when_head_has_none() {
    let dir = TempDir::new().unwrap();
    let work = git_log(&dir, "diver@example.invalid");
    // As after Subsurface's "Save as" into a new repository: the current
    // branch has no commit, the log is on another branch.
    git(&work, &["symbolic-ref", "HEAD", "refs/heads/master"]);
    let log = Subsurface::open(&work).unwrap();
    assert_eq!(
        log.storage(),
        &Storage::Git {
            branch: "diver@example.invalid".into()
        }
    );
    assert_eq!(log.dives().unwrap().len(), 10);
}

#[test]
fn several_logs_without_a_current_one() {
    let dir = TempDir::new().unwrap();
    let work = git_log(&dir, "first");
    git(&work, &["branch", "second"]);
    // The current branch holds a log: it wins.
    assert_eq!(
        Subsurface::open(&work).unwrap().storage(),
        &Storage::Git {
            branch: "first".into()
        }
    );
    git(&work, &["symbolic-ref", "HEAD", "refs/heads/none"]);
    assert!(matches!(
        Subsurface::open(&work),
        Err(Error::SeveralLogs(_))
    ));
}

#[test]
fn folders_that_are_not_subsurface_logs() {
    let dir = TempDir::new().unwrap();
    let plain = dir.path().join("plain");
    std::fs::create_dir(&plain).unwrap();
    assert!(matches!(
        Subsurface::open(&plain),
        Err(Error::NotADiveLog(_))
    ));
    git(&plain, &["init", "-q"]);
    assert!(
        matches!(Subsurface::open(&plain), Err(Error::NotADiveLog(_))),
        "empty repository"
    );
    std::fs::write(plain.join("README"), "not a dive log").unwrap();
    git(&plain, &["add", "-A"]);
    git(&plain, &["commit", "-q", "-m", "other"]);
    assert!(matches!(
        Subsurface::open(&plain),
        Err(Error::NotADiveLog(_))
    ));

    let old = git_log(&dir, "main");
    std::fs::write(old.join("00-Subsurface"), "version 2\n").unwrap();
    git(&old, &["commit", "-q", "-am", "old"]);
    assert!(matches!(Subsurface::open(&old), Err(Error::OldFormat(_))));
}
