//! Prints what the MacDive reader sees, to compare with MacDive by hand.
//! Usage: cargo run -p divelens-core --example macdive_dump -- <MacDive.sqlite> [dive number]
//! Output may contain personal data: never paste it into tests or commits.

use std::path::PathBuf;
use std::time::Instant;

use divelens_core::DiveSource;
use divelens_core::macdive::MacDive;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(args.next().expect("path to MacDive.sqlite"));
    let number: Option<u32> = args.next().and_then(|n| n.parse().ok());

    let t = Instant::now();
    let md = MacDive::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let dives = md.dives().unwrap_or_else(|e| panic!("{e}"));
    let with_profile = dives.iter().filter(|d| d.has_profile).count();
    println!(
        "{} dives, {} with profile, read in {:?}",
        dives.len(),
        with_profile,
        t.elapsed()
    );

    if let Some(n) = number {
        let d = dives
            .iter()
            .find(|d| d.number == Some(n))
            .expect("no such dive");
        println!("{d:#?}");
        let t = Instant::now();
        let p = md.profile(&d.id).unwrap_or_else(|e| panic!("{e}"));
        println!(
            "profile: {} samples in {:?}; first {:?}; last {:?}",
            p.len(),
            t.elapsed(),
            p.first(),
            p.last()
        );
    }
}
