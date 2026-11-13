//! Subsurface git storage (the format of its cloud storage, whose local copy
//! is a folder). Like Subsurface, the log is read from the last commit of a
//! branch, not from the files in the folder (Subsurface does not always
//! check them out). Layout of the commit:
//!
//! ```text
//! 00-Subsurface                       "version 3", settings
//! 01-Divesites/Site-1a2b3c4d          name "…", gps 12.5 -45.25,
//!                                     geo cat 2 origin 0 "Country"
//! 2025/06/14-Sat-09=30=00/            one folder per dive
//!     Dive-101                        duration, rating, invalid,
//!                                     divesiteid, watertemp
//!     Divecomputer                    model "…", maxdepth, watertemp,
//!                                     then one line per sample:
//!                                       "  0:10 2.0m 26.0°C 200.0bar:0 …"
//! 2025/06/14-Atlantis~1a2b/           a trip: 00-Trip, then its dives
//! ```
//!
//! A dive folder is named `[[yyyy-]mm-]dd-Day-hh=mm=ss[~hex]`: the year and
//! month come from the path unless given. Several computers are stored as
//! `Divecomputer-001`, `Divecomputer-002`…; `Dive-<number>` carries the
//! number (plain `Dive` when there is none).

use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use gix::{ObjectId, Repository};

use super::{
    RawComputer, RawDive, RawLog, RawSite, depth, duration, gps, push_sample, set_geo, temperature,
    too_old, uuid,
};
use crate::{Error, Result};

const SETTINGS: &str = "00-Subsurface";
const SITES: &str = "01-Divesites";

fn db(e: impl std::fmt::Display) -> Error {
    Error::Database(format!("git: {e}"))
}

struct Entry {
    name: String,
    id: ObjectId,
    is_tree: bool,
}

fn entries(repo: &Repository, tree: ObjectId) -> Result<Vec<Entry>> {
    let tree = repo.find_tree(tree).map_err(db)?;
    let decoded = tree.decode().map_err(db)?;
    Ok(decoded
        .entries
        .iter()
        .map(|e| Entry {
            name: e.filename.to_string(),
            id: e.oid.to_owned(),
            is_tree: e.mode.is_tree(),
        })
        .collect())
}

fn blob(repo: &Repository, id: ObjectId) -> Result<String> {
    let blob = repo.find_blob(id).map_err(db)?;
    Ok(String::from_utf8_lossy(&blob.data).into_owned())
}

/// Root tree of a branch's last commit, when it holds a Subsurface log.
fn log_tree(repo: &Repository, commit: ObjectId) -> Option<ObjectId> {
    let tree = repo.find_commit(commit).ok()?.tree_id().ok()?.detach();
    entries(repo, tree)
        .ok()?
        .iter()
        .any(|e| e.name == SETTINGS && !e.is_tree)
        .then_some(tree)
}

/// The branch to read: the current one when it holds a log, else the only
/// branch that does.
fn pick(repo: &Repository, path: &Path) -> Result<(String, ObjectId)> {
    if let Ok(head) = repo.head()
        && let Some(id) = head.id()
        && let Some(tree) = log_tree(repo, id.detach())
    {
        let name = head
            .referent_name()
            .map_or_else(|| "HEAD".to_string(), |n| n.shorten().to_string());
        return Ok((name, tree));
    }
    let refs = repo.references().map_err(db)?;
    let mut logs = Vec::new();
    for r in refs.local_branches().map_err(db)?.flatten() {
        let Some(id) = r.try_id() else { continue };
        if let Some(tree) = log_tree(repo, id.detach()) {
            logs.push((r.name().shorten().to_string(), tree));
        }
    }
    match logs.len() {
        0 => Err(Error::NotADiveLog(path.to_path_buf())),
        1 => Ok(logs.remove(0)),
        _ => Err(Error::SeveralLogs(path.to_path_buf())),
    }
}

/// Branch and tree read; changes with every save in Subsurface.
pub(super) fn fingerprint(path: &Path) -> Option<String> {
    let repo = gix::open(path).ok()?;
    let (branch, tree) = pick(&repo, path).ok()?;
    Some(format!("{branch}/{tree}"))
}

pub(super) fn read(path: &Path) -> Result<(String, RawLog)> {
    let repo = gix::open(path).map_err(|_| Error::NotADiveLog(path.to_path_buf()))?;
    let (branch, root) = pick(&repo, path)?;
    let mut log = RawLog::default();
    for e in entries(&repo, root)? {
        match (e.name.as_str(), e.is_tree) {
            (SETTINGS, false) => {
                let version = lines(&blob(&repo, e.id)?)
                    .iter()
                    .find_map(|l| l.strip_prefix("version ")?.trim().parse().ok());
                too_old(path, version)?;
            }
            (SITES, true) => {
                for s in entries(&repo, e.id)? {
                    if let Some(id) = s.name.strip_prefix("Site-").and_then(uuid)
                        && !s.is_tree
                    {
                        log.sites.push(parse_site(id, &blob(&repo, s.id)?));
                    }
                }
            }
            (name, true) if is_number(name, 4) => {
                let year = name.parse().unwrap_or(0);
                for m in entries(&repo, e.id)? {
                    if m.is_tree && is_number(&m.name, 2) {
                        let month = m.name.parse().unwrap_or(0);
                        walk(&repo, m.id, year, month, &mut log)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok((branch, log))
}

fn is_number(s: &str, digits: usize) -> bool {
    s.len() == digits && s.bytes().all(|b| b.is_ascii_digit())
}

/// Dives of a month folder, or of a trip folder inside it.
fn walk(repo: &Repository, tree: ObjectId, year: i32, month: u32, log: &mut RawLog) -> Result<()> {
    for e in entries(repo, tree)? {
        if !e.is_tree {
            continue;
        }
        if let Some(start) = dive_start(&e.name, year, month) {
            log.dives.push(parse_dive(repo, e.id, start)?);
        } else if e.name.len() > 3 && is_number(&e.name[..2], 2) && e.name.as_bytes()[2] == b'-' {
            walk(repo, e.id, year, month, log)?;
        }
    }
    Ok(())
}

/// Start of a dive from its folder name `[[yyyy-]mm-]dd-Day-hh=mm=ss[~hex]`.
fn dive_start(name: &str, year: i32, month: u32) -> Option<NaiveDateTime> {
    let name = name.split('~').next()?;
    let parts: Vec<&str> = name.split('-').collect();
    let (time, rest) = parts.split_last()?;
    let (day_name, date) = rest.split_last()?;
    if day_name.len() != 3 || !day_name.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    let hms: Vec<u32> = time
        .split(['=', ':'])
        .map(|p| p.parse().ok())
        .collect::<Option<_>>()?;
    let [h, mi, s] = hms[..] else { return None };
    let (y, mo, d) = match date {
        [d] => (year, month, d.parse().ok()?),
        [mo, d] => (year, mo.parse().ok()?, d.parse().ok()?),
        [y, mo, d] => (y.parse().ok()?, mo.parse().ok()?, d.parse().ok()?),
        _ => return None,
    };
    Some(NaiveDate::from_ymd_opt(y, mo, d)?.and_time(NaiveTime::from_hms_opt(h, mi, s)?))
}

fn parse_dive(repo: &Repository, tree: ObjectId, start: NaiveDateTime) -> Result<RawDive> {
    let mut dive = RawDive {
        number: None,
        start,
        duration_s: None,
        rating: None,
        invalid: false,
        notes: None,
        site: None,
        water_c: None,
        computers: Vec::new(),
    };
    let mut files = entries(repo, tree)?;
    files.retain(|f| !f.is_tree);
    files.sort_by(|a, b| a.name.cmp(&b.name));
    for f in files {
        if f.name.starts_with("Divecomputer") {
            dive.computers.push(parse_computer(&blob(repo, f.id)?));
        } else if let Some(suffix) = f.name.strip_prefix("Dive") {
            dive.number = suffix
                .strip_prefix('-')
                .and_then(|n| n.split('~').next()?.parse().ok());
            for line in lines(&blob(repo, f.id)?) {
                let (key, value) = split(&line);
                match key {
                    "duration" => dive.duration_s = duration(value),
                    "rating" => dive.rating = value.trim().parse().ok(),
                    "invalid" => dive.invalid = true,
                    "notes" => dive.notes = quoted(value),
                    "divesiteid" => dive.site = uuid(value),
                    "watertemp" => dive.water_c = temperature(value),
                    _ => {}
                }
            }
        }
    }
    Ok(dive)
}

fn parse_computer(text: &str) -> RawComputer {
    let mut c = RawComputer::default();
    for line in lines(text) {
        if line.starts_with(|ch: char| ch.is_ascii_digit() || ch == ' ') {
            parse_sample(&line, &mut c);
            continue;
        }
        let (key, value) = split(&line);
        match key {
            "model" => c.model = quoted(value),
            "duration" => c.duration_s = duration(value),
            "maxdepth" => c.max_depth_m = depth(value),
            "watertemp" => c.water_c = temperature(value),
            _ => {}
        }
    }
    c.samples.sort_by_key(|s| s.time_s);
    c
}

/// "  0:10 2.0m 26.0°C 200.0bar:0 ndl=…": time, then values with their unit;
/// named values (`key=value`) are not used.
fn parse_sample(line: &str, c: &mut RawComputer) {
    let mut words = line.split_whitespace();
    let Some(time) = words.next().and_then(duration) else {
        return;
    };
    let (mut d, mut t) = (None, None);
    for w in words {
        if w.starts_with(|ch: char| ch.is_ascii_lowercase()) {
            continue;
        }
        // Like Subsurface: "m" is a depth, "bar" a pressure, anything else a
        // temperature.
        let unit = w.trim_start_matches(|ch: char| ch.is_ascii_digit() || "+-.".contains(ch));
        match unit.chars().next() {
            Some('m' | 'f') => d = depth(w),
            Some('b') => {}
            _ => t = temperature(w),
        }
    }
    push_sample(&mut c.samples, time, d, t);
}

fn parse_site(id: u32, text: &str) -> RawSite {
    let mut site = RawSite {
        uuid: id,
        ..RawSite::default()
    };
    for line in lines(text) {
        let (key, value) = split(&line);
        match key {
            "name" => site.name = quoted(value),
            "gps" => (site.lat, site.lon) = gps(value).unzip(),
            "geo" => {
                // geo cat 2 origin 0 "Country"
                let mut w = value.split_whitespace();
                if w.next() == Some("cat")
                    && let Some(cat) = w.next().and_then(|c| c.parse().ok())
                {
                    set_geo(&mut site, cat, &quoted(value).unwrap_or_default());
                }
            }
            _ => {}
        }
    }
    site
}

/// Lines of a file; a line starting with a tab continues the previous one
/// (multi-line text).
fn lines(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        match (line.strip_prefix('\t'), out.last_mut()) {
            (Some(rest), Some(prev)) => {
                prev.push('\n');
                prev.push_str(rest);
            }
            _ => out.push(line.to_string()),
        }
    }
    out
}

/// "keyword rest of line".
fn split(line: &str) -> (&str, &str) {
    line.split_once(' ').unwrap_or((line, ""))
}

/// First quoted string of `s`, with Subsurface's `\"` and `\\` escapes.
fn quoted(s: &str) -> Option<String> {
    let mut chars = s[s.find('"')? + 1..].chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => out.extend(chars.next()),
            c => out.push(c),
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dive_folder_names() {
        let at = |y, mo, d, h, mi, s| {
            NaiveDate::from_ymd_opt(y, mo, d)
                .unwrap()
                .and_hms_opt(h, mi, s)
                .unwrap()
        };
        assert_eq!(
            dive_start("14-Sat-09=30=00", 2025, 6),
            Some(at(2025, 6, 14, 9, 30, 0))
        );
        assert_eq!(
            dive_start("14-Sat-09:30:00~1a2b", 2025, 6),
            Some(at(2025, 6, 14, 9, 30, 0))
        );
        assert_eq!(
            dive_start("07-01-Tue-08=00=00", 2025, 6),
            Some(at(2025, 7, 1, 8, 0, 0))
        );
        assert_eq!(
            dive_start("2026-01-02-Fri-08=00=00", 2025, 12),
            Some(at(2026, 1, 2, 8, 0, 0))
        );
        assert_eq!(dive_start("14-Atlantis~1a2b", 2025, 6), None, "a trip");
        assert_eq!(dive_start("31-Mon-09=30=00", 2025, 6), None, "no 31 June");
    }

    #[test]
    fn strings_and_samples() {
        assert_eq!(
            quoted(r#""Say \"hi\" \\ there" x"#).as_deref(),
            Some(r#"Say "hi" \ there"#)
        );
        assert_eq!(quoted("none"), None);
        assert_eq!(
            lines("notes \"a\n\tb\"\nrating 3\n"),
            vec!["notes \"a\nb\"", "rating 3"]
        );
        let mut c = RawComputer::default();
        parse_sample("  0:10 2.0m 26.0°C 200.0bar:0 ndl=99:00", &mut c);
        parse_sample("  1:00 5.5m 198.0bar:0", &mut c);
        parse_sample("  2:00 -0.5m -1.5°C", &mut c);
        assert_eq!(c.samples.len(), 3);
        assert_eq!(
            (
                c.samples[0].time_s,
                c.samples[0].depth_m,
                c.samples[0].temp_c
            ),
            (10, 2.0, Some(26.0))
        );
        assert_eq!(
            (c.samples[1].depth_m, c.samples[1].temp_c),
            (5.5, Some(26.0))
        );
        assert_eq!(
            (c.samples[2].depth_m, c.samples[2].temp_c),
            (-0.5, Some(-1.5))
        );
    }
}
