//! Decodes every profile of a MacDive database and reports how many work.
//! When the database also has a readable `ZSAMPLES_JSON` column (test
//! databases prepared by an external tool), compares both.
//! Usage: cargo run --release -p divelens-core --example macdive_profiles -- <MacDive.sqlite>
//! Output may contain personal data: never paste it into tests or commits.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use divelens_core::DiveSource;
use divelens_core::macdive::MacDive;
use rusqlite::{Connection, OpenFlags};

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).expect("path to MacDive.sqlite"));
    let md = MacDive::open(&path).unwrap_or_else(|e| panic!("{e}"));
    let dives = md.dives().unwrap();

    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let mut reference: HashMap<String, Vec<(u32, f64, Option<f64>)>> = HashMap::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT ZUUID, ZSAMPLES_JSON FROM ZDIVE WHERE ZSAMPLES_JSON IS NOT NULL")
    {
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .unwrap();
        for (id, json) in rows.flatten() {
            let v: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap_or_default();
            let pts = v
                .iter()
                .filter_map(|s| Some((s["t"].as_f64()? as u32, s["d"].as_f64()?, s["T"].as_f64())))
                .collect();
            reference.insert(id, pts);
        }
    }

    let t = Instant::now();
    let (mut ok, mut empty, mut failed, mut compared, mut worst_depth, mut worst_temp) =
        (0, 0, 0, 0, 0f64, 0f64);
    let mut by_computer: HashMap<String, (u32, u32)> = HashMap::new();
    let computer_of: HashMap<String, String> = conn
        .prepare("SELECT COALESCE(ZUUID, 'pk:' || Z_PK), COALESCE(ZCOMPUTER, '') FROM ZDIVE")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .flatten()
        .collect();
    for d in dives.iter().filter(|d| d.has_profile) {
        let entry = by_computer.entry(computer_of[&d.id].clone()).or_default();
        entry.0 += 1;
        match md.profile(&d.id) {
            Ok(p) if p.len() >= 2 => {
                ok += 1;
                entry.1 += 1;
                if let Some(r) = reference.get(&d.id) {
                    compared += 1;
                    let by_time: HashMap<u32, &divelens_core::profile::Sample> =
                        p.iter().map(|s| (s.time_s, s)).collect();
                    for (time, depth, temp) in r {
                        if let Some(s) = by_time.get(time) {
                            worst_depth = worst_depth.max((s.depth_m - depth).abs());
                            if let (Some(a), Some(b)) = (s.temp_c, temp) {
                                worst_temp = worst_temp.max((a - b).abs());
                            }
                        }
                    }
                }
            }
            Ok(_) => empty += 1,
            Err(e) => {
                failed += 1;
                eprintln!("dive {:?}: {e}", d.number);
            }
        }
    }
    println!(
        "{} dives, {} flagged with profile: {ok} decoded, {empty} empty, {failed} failed, in {:?}",
        dives.len(),
        dives.iter().filter(|d| d.has_profile).count(),
        t.elapsed()
    );
    for (c, (n, k)) in &by_computer {
        println!("  {c}: {k}/{n}");
    }
    if compared > 0 {
        println!(
            "compared with ZSAMPLES_JSON on {compared} dives: worst depth gap {worst_depth:.2} m, worst temperature gap {worst_temp:.2} °C"
        );
    }
}
