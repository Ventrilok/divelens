//! Checks a reader against a real dive log on this computer, for any
//! application Dive Lens reads: prints what Dive Lens reads from the log, to
//! compare with the dive application by hand.
//!
//! ```text
//! cargo run --release -p divelens-core --example verify_log -- [--app <application>] <log> [dive number]
//! ```
//!
//! - `<log>`: the MacDive database, a Subsurface XML file or a Subsurface git
//!   storage folder.
//! - `--app`: `MacDive` or `Subsurface`. Without it, a folder or an `.xml` or
//!   `.ssrf` file is read as Subsurface, anything else as MacDive.
//! - `[dive number]`: also prints that dive in full, with its profile.
//!
//! It prints the number of dives and sites, decodes every profile (listing
//! the dives whose profile fails), reads the log a second time to compare the
//! identifiers, and reports what breaks the rules every reader follows
//! (`core/src/conformance.rs`).
//!
//! **The output is personal data** (dates, places, positions, notes). It stays
//! on this computer: never put it, even in part, in the repository, an issue
//! or a pull request. Test data is always fictional
//! (`docs/contributing/reference-log.md`).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use divelens_core::macdive::MacDive;
use divelens_core::subsurface::Subsurface;
use divelens_core::{Dive, DiveSource, Result};

const HELP: &str = "\
Checks a reader against a real dive log on this computer.

Usage: cargo run --release -p divelens-core --example verify_log -- [--app <application>] <log> [dive number]

  <log>          the MacDive database, a Subsurface XML file or a Subsurface git storage folder
  --app          MacDive or Subsurface; by default a folder, .xml or .ssrf is Subsurface, anything else MacDive
  [dive number]  also prints that dive in full, with its profile

The output is personal data: keep it on this computer. Never put it, even in
part, in the repository, an issue or a pull request.";

/// The applications this tool reads.
const APPS: [&str; 2] = ["MacDive", "Subsurface"];

/// At most this many dives listed per problem.
const LISTED: usize = 20;

struct Args {
    app: &'static str,
    path: PathBuf,
    number: Option<u32>,
}

fn args() -> std::result::Result<Args, String> {
    let mut app = None;
    let mut path = None;
    let mut number = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "--app" => {
                let name = it.next().ok_or("--app needs an application")?;
                let known = APPS.iter().find(|n| n.eq_ignore_ascii_case(&name));
                app = Some(*known.ok_or(format!(
                    "unknown application {name:?}: {}",
                    APPS.join(" or ")
                ))?);
            }
            _ if path.is_none() => path = Some(PathBuf::from(a)),
            _ if number.is_none() => {
                number = Some(a.parse().map_err(|_| format!("not a dive number: {a}"))?);
            }
            _ => return Err(format!("unexpected argument {a:?}\n\n{HELP}")),
        }
    }
    let path = path.ok_or(HELP)?;
    Ok(Args {
        app: app.unwrap_or_else(|| guess(&path)),
        path,
        number,
    })
}

fn guess(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str());
    let xml = ext.is_some_and(|e| e.eq_ignore_ascii_case("xml") || e.eq_ignore_ascii_case("ssrf"));
    if path.is_dir() || xml {
        "Subsurface"
    } else {
        "MacDive"
    }
}

/// Opens the log, with a word on how it is stored when the reader tells.
fn open(app: &str, path: &Path) -> Result<(Box<dyn DiveSource>, String)> {
    Ok(match app {
        "MacDive" => (Box::new(MacDive::open(path)?), String::new()),
        _ => {
            let log = Subsurface::open(path)?;
            let storage = format!(" ({:?})", log.storage());
            (Box::new(log), storage)
        }
    })
}

fn label(d: &Dive) -> String {
    let number = d
        .number
        .map_or_else(|| "no number".into(), |n| n.to_string());
    format!("dive {number} ({})", d.start)
}

/// Prints `what` and the first dives it concerns.
fn report(what: &str, dives: &[String]) {
    if dives.is_empty() {
        return;
    }
    println!("  {what}: {}", dives.len());
    for d in dives.iter().take(LISTED) {
        println!("    {d}");
    }
    if dives.len() > LISTED {
        println!("    … and {} more", dives.len() - LISTED);
    }
}

fn run(args: &Args) -> Result<()> {
    eprintln!(
        "Personal data below: keep it on this computer (never in the repository, an issue or a pull request).\n"
    );
    let t = Instant::now();
    let (log, storage) = open(args.app, &args.path)?;
    let dives = log.dives()?;
    let sites = log.sites()?;
    println!(
        "{}{storage}: {} dives ({} with a profile, {} invalid, {} without number), {} sites, read in {:?}",
        args.app,
        dives.len(),
        dives.iter().filter(|d| d.has_profile).count(),
        dives.iter().filter(|d| d.invalid).count(),
        dives.iter().filter(|d| d.number.is_none()).count(),
        sites.len(),
        t.elapsed()
    );

    // Every profile, decoded.
    let t = Instant::now();
    let (mut decoded, mut failed, mut empty, mut unsorted, mut extra) =
        (0, Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for d in &dives {
        match log.profile(&d.id) {
            Ok(p) if !d.has_profile => {
                if !p.is_empty() {
                    extra.push(label(d));
                }
            }
            Ok(p) if p.is_empty() => empty.push(label(d)),
            Ok(p) => {
                decoded += 1;
                if p.windows(2).any(|w| w[0].time_s > w[1].time_s) {
                    unsorted.push(label(d));
                }
            }
            Err(e) => failed.push(format!("{}: {e}", label(d))),
        }
    }
    println!("profiles: {decoded} decoded in {:?}", t.elapsed());
    report("failed", &failed);
    report("flagged with a profile, but no samples", &empty);
    report("samples, but not flagged with a profile", &extra);
    report("samples not sorted by time", &unsorted);

    // The rules every reader follows that a real log can show.
    let mut seen = HashSet::new();
    let twice: Vec<_> = dives
        .iter()
        .filter(|d| !seen.insert(&d.id))
        .map(label)
        .collect();
    let end: Vec<_> = dives
        .iter()
        .filter(|d| (d.end - d.start).num_seconds() != i64::from(d.duration_s))
        .map(label)
        .collect();
    let ids = |dives: &[Dive], sites: &[divelens_core::Site]| {
        let mut d: Vec<_> = dives
            .iter()
            .map(|d| (d.id.clone(), d.site.id.clone()))
            .collect();
        d.sort();
        let mut s: Vec<_> = sites.iter().map(|s| s.id.clone()).collect();
        s.sort();
        (d, s)
    };
    let (again, _) = open(args.app, &args.path)?;
    let stable = ids(&dives, &sites) == ids(&again.dives()?, &again.sites()?);
    println!(
        "identifiers: {} on a second read",
        if stable { "the same" } else { "DIFFERENT" }
    );
    report("dives sharing an id with another", &twice);
    report("end not start + duration", &end);

    if let Some(n) = args.number {
        let Some(d) = dives.iter().find(|d| d.number == Some(n)) else {
            println!("no dive {n}");
            return Ok(());
        };
        println!("\n{d:#?}");
        let t = Instant::now();
        let p = log.profile(&d.id)?;
        println!(
            "profile: {} samples in {:?}; first {:?}; last {:?}",
            p.len(),
            t.elapsed(),
            p.first(),
            p.last()
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = match args() {
        Ok(a) => a,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} log {} not read: {e}", args.app, args.path.display());
            ExitCode::FAILURE
        }
    }
}
