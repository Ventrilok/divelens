//! Prints what the Subsurface reader sees, to compare with Subsurface by hand.
//! Usage: cargo run -p divelens-core --example subsurface_dump -- <log.xml | git folder> [dive number]
//! Output may contain personal data: never paste it into tests or commits.

use std::path::PathBuf;
use std::time::Instant;

use divelens_core::DiveSource;
use divelens_core::subsurface::Subsurface;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(args.next().expect("path to a Subsurface log"));
    let number: Option<u32> = args.next().and_then(|n| n.parse().ok());

    let t = Instant::now();
    let log = Subsurface::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let dives = log.dives().unwrap_or_else(|e| panic!("{e}"));
    let sites = log.sites().unwrap_or_else(|e| panic!("{e}"));
    println!(
        "{:?}: {} dives ({} with profile, {} invalid, {} without number), {} sites, read in {:?}",
        log.storage(),
        dives.len(),
        dives.iter().filter(|d| d.has_profile).count(),
        dives.iter().filter(|d| d.invalid).count(),
        dives.iter().filter(|d| d.number.is_none()).count(),
        sites.len(),
        t.elapsed()
    );

    if let Some(n) = number {
        let d = dives
            .iter()
            .find(|d| d.number == Some(n))
            .expect("no such dive");
        println!("{d:#?}");
        let p = log.profile(&d.id).unwrap_or_else(|e| panic!("{e}"));
        println!(
            "profile: {} samples; first {:?}; last {:?}",
            p.len(),
            p.first(),
            p.last()
        );
    }
}
