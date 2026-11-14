//! Common tests every dive-log reader must pass, written once and run
//! against each reader's copy of the reference log ([`reference`], described
//! for contributors in `docs/contributing/reference-log.md`).
//!
//! Whatever the application, a reader:
//! - gives wall-clock times exactly as the log holds them, never shifted;
//! - gives the same identifiers, for dives and sites, every time the log is
//!   read;
//! - gives depths in metres and temperatures in °C, as the application
//!   shows them;
//! - gives profiles sorted by time;
//! - reports a dive without a profile as having none, with no samples;
//! - never modifies the log: no file changed, created or removed next to it,
//!   whether looked at while the log is open or after;
//! - declares the [`Capabilities`] its run states, and leaves out what they
//!   do not cover.
//!
//! The common reader interface is [`DiveSource`] plus the reader's own
//! `open(path)`. To add a reader: write the reference log in the
//! application's format (a function of the reader's tests that writes it into
//! a folder), then give the application's description (`crate::apps`) its
//! runs of [`check`]: every registered application is run at the bottom of
//! this file.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::ptr;

use tempfile::TempDir;

use crate::profile::{self, Sample};
use crate::{Capabilities, Dive, DiveSource, Error, Result, Site};

pub(crate) mod reference;

/// Values may differ from the reference by less than half of the 0.1 m and
/// 0.1 °C dive logs show, so that a log storing feet or Fahrenheit passes;
/// a unit mistake does not.
const DEPTH_M: f64 = 0.05;
const TEMP_C: f64 = 0.05;
/// About a metre.
const GPS_DEG: f64 = 1e-5;

/// One reader's run of the common tests.
pub(crate) struct Run {
    /// Writes the reference log, in the application's own format, into an
    /// empty folder; returns the path a diver would choose in Dive Lens.
    pub write_log: fn(&Path) -> PathBuf,
    /// Opens a log read-only: the reader's `open`.
    pub open: fn(&Path) -> Result<Box<dyn DiveSource>>,
    /// What the reader must declare it provides. Values it does not provide
    /// must be absent; the reader cannot lower the bar by declaring less.
    pub capabilities: Capabilities,
    /// The application can mark a dive invalid (Subsurface).
    pub marks_invalid: bool,
}

/// Runs every common test on `run`'s reader; panics with the list of what
/// differs from the reference log.
pub(crate) fn check(run: &Run) {
    let dir = TempDir::new().unwrap();
    let path = (run.write_log)(dir.path());
    let before = snapshot(dir.path(), true);
    let mut problems = Problems::default();
    {
        let open = || (run.open)(&path).unwrap_or_else(|e| panic!("reference log not read: {e}"));
        let log = open();
        let again = open();
        problems.same("capabilities", log.capabilities(), run.capabilities);
        check_dives(log.as_ref(), run, &mut problems);
        check_sites(log.as_ref(), run.capabilities, &mut problems);
        check_ids(log.as_ref(), again.as_ref(), &mut problems);
        match log.profile("no such dive") {
            Err(Error::UnknownDive(_)) => {}
            other => problems.add(format!(
                "profile of an unknown dive: {other:?}, expected Error::UnknownDive"
            )),
        }
        // While the log is open: a reader that opens it for writing may
        // create files that disappear once it is closed (SQLite's -wal and
        // -shm). Names only: Windows keeps open files busy.
        for name in snapshot(dir.path(), false).keys() {
            if !before.contains_key(name) {
                problems.add(format!("log modified: {name} created while reading"));
            }
        }
    }
    let after = snapshot(dir.path(), true);
    for (name, contents) in &after {
        match before.get(name) {
            None => problems.add(format!("log modified: {name} created")),
            Some(old) if old != contents => problems.add(format!("log modified: {name} changed")),
            _ => {}
        }
    }
    for name in before.keys().filter(|n| !after.contains_key(*n)) {
        problems.add(format!("log modified: {name} removed"));
    }
    assert!(
        problems.0.is_empty(),
        "{} difference(s) with the reference log:\n- {}",
        problems.0.len(),
        problems.0.join("\n- ")
    );
}

#[derive(Default)]
struct Problems(Vec<String>);

impl Problems {
    fn add(&mut self, problem: String) {
        self.0.push(problem);
    }

    /// Records a difference between `got` and `expected`.
    fn same<T: PartialEq + std::fmt::Debug>(&mut self, what: &str, got: T, expected: T) {
        if got != expected {
            self.add(format!("{what}: {got:?}, expected {expected:?}"));
        }
    }

    /// Records a difference larger than `tolerance`.
    fn close(&mut self, what: &str, got: Option<f64>, expected: Option<f64>, tolerance: f64) {
        let ok = match (got, expected) {
            (Some(a), Some(b)) => (a - b).abs() < tolerance,
            (a, b) => a.is_none() && b.is_none(),
        };
        if !ok {
            self.add(format!("{what}: {got:?}, expected {expected:?}"));
        }
    }
}

fn label(number: Option<u32>) -> String {
    number.map_or_else(|| "dive without number".into(), |n| format!("dive {n}"))
}

/// What the reader must give for `value` given its capability.
fn declared<T>(capable: bool, value: Option<T>) -> Option<T> {
    if capable { value } else { None }
}

fn check_dives(log: &dyn DiveSource, run: &Run, p: &mut Problems) {
    let caps = run.capabilities;
    let dives = log
        .dives()
        .unwrap_or_else(|e| panic!("dives not read: {e}"));
    p.same("number of dives", dives.len(), reference::DIVES.len());
    for r in &reference::DIVES {
        let found: Vec<&Dive> = dives.iter().filter(|d| d.number == r.number).collect();
        match found[..] {
            [d] => check_dive(log, d, r, caps, run, p),
            _ => p.add(format!("{}: found {} times", label(r.number), found.len())),
        }
    }
}

fn check_dive(
    log: &dyn DiveSource,
    d: &Dive,
    r: &reference::Dive,
    caps: Capabilities,
    run: &Run,
    p: &mut Problems,
) {
    let what = |field: &str| format!("{} {field}", label(r.number));
    // Wall clock: what the app receives is the time written in the log, with
    // no time zone, whatever the computer's own.
    let json = serde_json::to_value(d).unwrap();
    p.same(&what("start"), json["start"].as_str(), Some(r.start));
    p.same(&what("end"), json["end"].as_str(), Some(r.end));
    p.same(
        &what("end minus start"),
        (d.end - d.start).num_seconds(),
        i64::from(d.duration_s),
    );
    p.same(&what("duration (s)"), d.duration_s, r.duration_s);
    p.close(
        &what("max depth (m)"),
        d.max_depth_m,
        r.max_depth_m,
        DEPTH_M,
    );
    let temp = |v| declared(caps.temperature, v);
    p.close(
        &what("low temperature (°C)"),
        d.temp_low_c,
        temp(r.temp_low_c),
        TEMP_C,
    );
    p.close(
        &what("high temperature (°C)"),
        d.temp_high_c,
        temp(r.temp_high_c),
        TEMP_C,
    );
    p.same(&what("rating"), d.rating, declared(caps.rating, r.rating));
    p.same(&what("notes"), d.notes.as_deref(), r.notes);
    p.same(&what("invalid"), d.invalid, r.invalid && run.marks_invalid);
    match r.site {
        None => p.same(&what("site"), &d.site, &Site::default()),
        Some(s) => {
            if d.site.id.as_deref().is_none_or(str::is_empty) {
                p.add(format!("{}: no site id", what("site")));
            }
            check_site(&what("site"), &d.site, s, caps, p);
        }
    }

    let expected = !r.profile.is_empty() && caps.profile;
    p.same(&what("has a profile"), d.has_profile, expected);
    match log.profile(&d.id) {
        Err(e) => p.add(format!("{}: {e}", what("profile"))),
        Ok(samples) if !expected => p.same(&what("profile samples"), samples.len(), 0),
        Ok(samples) => check_profile(&what("profile"), &samples, r.profile, caps, p),
    }
}

fn check_site(what: &str, s: &Site, r: &reference::Site, caps: Capabilities, p: &mut Problems) {
    p.same(&format!("{what} name"), s.name.as_deref(), r.name);
    p.same(
        &format!("{what} location"),
        s.location.as_deref(),
        r.location,
    );
    p.same(&format!("{what} country"), s.country.as_deref(), r.country);
    let gps = declared(caps.gps, r.gps);
    p.close(
        &format!("{what} latitude"),
        s.lat,
        gps.map(|g| g.0),
        GPS_DEG,
    );
    p.close(
        &format!("{what} longitude"),
        s.lon,
        gps.map(|g| g.1),
        GPS_DEG,
    );
}

fn check_profile(
    what: &str,
    samples: &[Sample],
    points: &[(u32, f64, f64)],
    caps: Capabilities,
    p: &mut Problems,
) {
    if samples.is_empty() {
        return p.add(format!("{what}: no samples"));
    }
    if let Some(w) = samples.windows(2).find(|w| w[0].time_s > w[1].time_s) {
        p.add(format!(
            "{what}: not sorted by time ({} s before {} s)",
            w[0].time_s, w[1].time_s
        ));
        return;
    }
    for &(time_s, depth_m, temp_c) in points {
        let at = profile::at(samples, time_s).unwrap();
        let what = format!("{what} at {time_s} s");
        p.close(
            &format!("{what}, depth (m)"),
            Some(at.depth_m),
            Some(depth_m),
            DEPTH_M,
        );
        let temp = declared(caps.temperature, Some(temp_c));
        p.close(
            &format!("{what}, temperature (°C)"),
            at.temp_c,
            temp,
            TEMP_C,
        );
    }
}

fn check_sites(log: &dyn DiveSource, caps: Capabilities, p: &mut Problems) {
    let sites = log
        .sites()
        .unwrap_or_else(|e| panic!("sites not read: {e}"));
    p.same("number of sites", sites.len(), reference::SITES.len());
    for r in reference::SITES {
        let found: Vec<&Site> = sites
            .iter()
            .filter(|s| s.name.as_deref() == r.name)
            .collect();
        let what = format!("site {:?}", r.name.unwrap_or_default());
        match found[..] {
            [s] => {
                if s.id.as_deref().is_none_or(str::is_empty) {
                    p.add(format!("{what}: no id"));
                }
                check_site(&what, s, r, caps, p);
            }
            _ => p.add(format!("{what}: listed {} times", found.len())),
        }
    }
    // The site of a dive is the one listed, with the same id.
    let dives = log
        .dives()
        .unwrap_or_else(|e| panic!("dives not read: {e}"));
    for d in dives.iter().filter(|d| d.site.name.is_some()) {
        if !sites.iter().any(|s| s.id == d.site.id) {
            p.add(format!(
                "{} site: id {:?} not in the list of sites",
                label(d.number),
                d.site.id
            ));
        }
    }
}

/// Number, id and site id of every dive.
type DiveIds = Vec<(Option<u32>, String, Option<String>)>;
/// Name and id of every listed site.
type SiteIds = Vec<(Option<String>, Option<String>)>;

/// Dive and site ids of a log, sorted.
fn ids(log: &dyn DiveSource) -> (DiveIds, SiteIds) {
    let mut dives: Vec<_> = log
        .dives()
        .unwrap_or_else(|e| panic!("dives not read: {e}"))
        .into_iter()
        .map(|d| (d.number, d.id, d.site.id))
        .collect();
    dives.sort();
    let mut sites: Vec<_> = log
        .sites()
        .unwrap_or_else(|e| panic!("sites not read: {e}"))
        .into_iter()
        .map(|s| (s.name, s.id))
        .collect();
    sites.sort();
    (dives, sites)
}

fn check_ids(log: &dyn DiveSource, again: &dyn DiveSource, p: &mut Problems) {
    let (dives, sites) = ids(log);
    let mut seen = HashMap::new();
    for (number, id, _) in &dives {
        if id.is_empty() {
            p.add(format!("{}: empty id", label(*number)));
        }
        if let Some(other) = seen.insert(id.clone(), *number) {
            p.add(format!(
                "{} and {}: same id {id:?}",
                label(other),
                label(*number)
            ));
        }
    }
    // Dives at the same reference site share its id; other sites differ.
    let site_id = |number| {
        dives
            .iter()
            .find(|d| d.0 == number)
            .and_then(|d| d.2.clone())
    };
    for a in &reference::DIVES {
        for b in &reference::DIVES {
            let (Some(sa), Some(sb)) = (a.site, b.site) else {
                continue;
            };
            let same = ptr::eq(sa, sb);
            let (ia, ib) = (site_id(a.number), site_id(b.number));
            if ia.is_some() && (ia == ib) != same {
                p.add(format!(
                    "{} and {}: site ids {ia:?} and {ib:?}",
                    label(a.number),
                    label(b.number)
                ));
            }
        }
    }
    let ids_of_sites: Vec<_> = sites.iter().map(|s| &s.1).collect();
    for (i, id) in ids_of_sites.iter().enumerate() {
        if ids_of_sites[..i].contains(id) {
            p.add(format!("two listed sites with id {id:?}"));
        }
    }
    // Stable: read again, and opened again.
    p.same(
        "ids on a second read",
        ids(log),
        (dives.clone(), sites.clone()),
    );
    p.same("ids once opened again", ids(again), (dives, sites));
}

/// Every file and folder under `root`, by path relative to it with `/`
/// separators: the contents of files when `contents`, nothing for folders.
fn snapshot(root: &Path, contents: bool) -> BTreeMap<String, Option<Vec<u8>>> {
    fn walk(root: &Path, dir: &Path, contents: bool, out: &mut BTreeMap<String, Option<Vec<u8>>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let path = e.unwrap().path();
            let name = path
                .strip_prefix(root)
                .unwrap()
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if path.is_dir() {
                out.insert(format!("{name}/"), None);
                walk(root, &path, contents, out);
            } else {
                out.insert(name, contents.then(|| std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, contents, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Every registered application (`crate::apps::ALL`), each of its runs: one
// per storage format it reads.

/// Every run of `app`'s description; what failed, by run.
fn check_app(app: &crate::apps::DiveApp) -> Vec<String> {
    let mut failures = Vec::new();
    let runs = (app.conformance)();
    if runs.is_empty() {
        failures.push(format!("{}: no run of the common tests", app.name));
    }
    for (i, run) in runs.iter().enumerate() {
        let what = format!("{}, run {} of {}", app.name, i + 1, runs.len());
        if let Err(panic) = std::panic::catch_unwind(|| check(run)) {
            let message = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("panicked");
            failures.push(format!("{what}: {message}"));
        }
        // What Settings says once the log is read.
        let dir = TempDir::new().unwrap();
        let path = (run.write_log)(dir.path());
        match (app.open)(&path, &crate::apps::Kept::default()) {
            Ok(opened) if !app.texts.en.read.iter().any(|(f, _)| *f == opened.format) => {
                failures.push(format!("{what}: no text for format {:?}", opened.format));
            }
            Ok(_) => {}
            Err(e) => failures.push(format!("{what}: not opened: {e}")),
        }
    }
    failures
}

#[test]
fn every_registered_application() {
    let failures: Vec<String> = crate::apps::ALL.iter().flat_map(|a| check_app(a)).collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

/// The example reader every new reader starts from (`crate::skeleton`), run
/// through its description like a registered one: not offered in the app,
/// kept passing so that a copy starts from code that works.
#[test]
fn skeleton() {
    let failures = check_app(&crate::skeleton::APP);
    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

/// The tests above again, in a child process whose time zone is 14 hours
/// ahead of UTC: a reader turning wall-clock times into instants would shift
/// every dive there. Unix only, as Windows ignores `TZ`; the comparison with
/// the literal reference times holds everywhere. The child first lists the
/// tests it runs, so that every run above must pass there.
#[test]
#[cfg(unix)]
fn in_another_time_zone() {
    const CHILD: &str = "DIVELENS_CONFORMANCE_CHILD";
    if std::env::var_os(CHILD).is_some() {
        return;
    }
    let child = |args: &[&str]| {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("conformance::")
            .args(args)
            .env("TZ", "Pacific/Kiritimati")
            .env(CHILD, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(out.status.success(), "in UTC+14:\n{stdout}");
        stdout
    };
    // Every test of this file: the runs above, and this one, which returns
    // at once in the child.
    let runs = child(&["--list"])
        .lines()
        .filter(|l| l.ends_with(": test"))
        .count();
    assert!(runs > 1, "no run listed in UTC+14");
    let stdout = child(&[]);
    assert!(
        stdout.contains(&format!("test result: ok. {runs} passed;")),
        "in UTC+14, {runs} tests expected to pass:\n{stdout}"
    );
}
