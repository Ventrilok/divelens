//! Subsurface XML file (`.xml` or `.ssrf`), data format version 3:
//!
//! ```text
//! <divelog program='subsurface' version='3'>
//!   <divesites>
//!     <site uuid='1a2b3c4d' name='…' gps='12.5 -45.25'>
//!       <geo cat='2' origin='0' value='Country' />
//!     </site>
//!   </divesites>
//!   <dives>
//!     <trip …> <notes>…</notes> <dive …> … </dive> </trip>
//!     <dive number='12' rating='4' invalid='1' divesiteid='1a2b3c4d'
//!           date='2025-06-14' time='09:30:00' duration='45:00 min'>
//!       <notes>Free text, line breaks kept</notes>
//!       <divetemperature water='24.0 C' />
//!       <divecomputer model='…' duration='…'>
//!         <depth max='18.4 m' />
//!         <temperature water='24.0 C' air='…' />
//!         <sample time='0:10 min' depth='1.2 m' temp='26.0 C' />
//!       </divecomputer>
//!     </dive>
//!   </dives>
//! </divelog>
//! ```

use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use super::{
    RawComputer, RawDive, RawLog, RawSite, depth, duration, gps, push_sample, set_geo, temperature,
    too_old, uuid,
};
use crate::{Error, Result};

/// Attribute `name` of `e`, unescaped.
fn attr(e: &BytesStart, name: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == name)
        .and_then(|a| a.normalized_value(XmlVersion::Implicit1_0).ok())
        .map(|v| v.into_owned())
}

fn start_of(e: &BytesStart) -> Option<NaiveDateTime> {
    let date = NaiveDate::parse_from_str(attr(e, "date")?.trim(), "%Y-%m-%d").ok()?;
    let time = attr(e, "time")
        .and_then(|t| NaiveTime::parse_from_str(t.trim(), "%H:%M:%S").ok())
        .unwrap_or_default();
    Some(date.and_time(time))
}

fn new_dive(e: &BytesStart) -> Option<RawDive> {
    Some(RawDive {
        number: attr(e, "number").and_then(|n| n.trim().parse().ok()),
        start: start_of(e)?,
        duration_s: attr(e, "duration").and_then(|d| duration(&d)),
        rating: attr(e, "rating").and_then(|r| r.trim().parse().ok()),
        invalid: attr(e, "invalid").is_some_and(|v| v.trim() == "1"),
        notes: None,
        site: attr(e, "divesiteid").and_then(|u| uuid(&u)),
        water_c: None,
        computers: Vec::new(),
    })
}

pub(super) fn read(path: &Path) -> Result<RawLog> {
    let bytes = std::fs::read(path).map_err(|e| Error::Database(e.to_string()))?;
    let text = String::from_utf8_lossy(&bytes);
    let mut reader = Reader::from_str(&text);
    let mut log = RawLog::default();
    let mut root_seen = false;
    let mut site: Option<RawSite> = None;
    let mut dive: Option<RawDive> = None;
    let mut computer: Option<RawComputer> = None;
    // Inside the dive's own <notes> (not a trip's, not a computer's).
    let mut in_notes = false;

    loop {
        let event = reader.read_event().map_err(|e| {
            if root_seen {
                Error::Database(format!("XML: {e}"))
            } else {
                Error::NotADiveLog(path.to_path_buf())
            }
        })?;
        // Notes text arrives in pieces: text, then each `&…;` reference.
        if in_notes {
            let piece = match &event {
                Event::Text(t) => Some(t.xml_content(XmlVersion::Implicit1_0).into_owned()),
                Event::CData(t) => Some(t.xml_content(XmlVersion::Implicit1_0).into_owned()),
                Event::GeneralRef(r) => match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c.to_string()),
                    _ => resolve_predefined_entity(&r.xml_content(XmlVersion::Implicit1_0))
                        .map(str::to_string),
                },
                _ => None,
            };
            if let (Some(piece), Some(d)) = (piece, dive.as_mut()) {
                d.notes.get_or_insert_default().push_str(&piece);
                continue;
            }
        }
        let (e, empty) = match &event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(e) => {
                match e.name().as_ref() {
                    "site" => log.sites.extend(site.take()),
                    "divecomputer" => {
                        if let (Some(d), Some(c)) = (dive.as_mut(), computer.take()) {
                            d.computers.push(c);
                        }
                    }
                    "dive" => log.dives.extend(dive.take()),
                    "notes" => in_notes = false,
                    _ => {}
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        let name = e.name();
        if !root_seen {
            if name.as_ref() != "divelog" {
                return Err(Error::NotADiveLog(path.to_path_buf()));
            }
            root_seen = true;
            too_old(path, attr(e, "version").and_then(|v| v.trim().parse().ok()))?;
            continue;
        }
        match name.as_ref() {
            "site" => {
                let (lat, lon) = attr(e, "gps").and_then(|g| gps(&g)).unzip();
                let s = RawSite {
                    uuid: attr(e, "uuid").and_then(|u| uuid(&u)).unwrap_or(0),
                    name: attr(e, "name"),
                    lat,
                    lon,
                    ..RawSite::default()
                };
                if empty {
                    log.sites.push(s);
                } else {
                    site = Some(s);
                }
            }
            "geo" => {
                if let (Some(s), Some(cat)) = (site.as_mut(), attr(e, "cat")) {
                    let cat = cat.trim().parse().unwrap_or(0);
                    set_geo(s, cat, &attr(e, "value").unwrap_or_default());
                }
            }
            "dive" => {
                // A dive without a date cannot be placed in time: skipped.
                let d = new_dive(e);
                if empty {
                    log.dives.extend(d);
                } else {
                    dive = d;
                }
            }
            "notes" => in_notes = !empty && dive.is_some() && computer.is_none(),
            "divetemperature" => {
                if let Some(d) = dive.as_mut() {
                    d.water_c = attr(e, "water").and_then(|t| temperature(&t));
                }
            }
            "divecomputer" => {
                let c = RawComputer {
                    model: attr(e, "model"),
                    duration_s: attr(e, "duration").and_then(|d| duration(&d)),
                    ..RawComputer::default()
                };
                if empty {
                    if let Some(d) = dive.as_mut() {
                        d.computers.push(c);
                    }
                } else {
                    computer = Some(c);
                }
            }
            "depth" => {
                if let Some(c) = computer.as_mut() {
                    c.max_depth_m = attr(e, "max").and_then(|m| depth(&m));
                }
            }
            "temperature" => {
                if let Some(c) = computer.as_mut() {
                    c.water_c = attr(e, "water").and_then(|t| temperature(&t));
                }
            }
            "sample" => {
                if let (Some(c), Some(t)) = (
                    computer.as_mut(),
                    attr(e, "time").and_then(|t| duration(&t)),
                ) {
                    push_sample(
                        &mut c.samples,
                        t,
                        attr(e, "depth").and_then(|d| depth(&d)),
                        attr(e, "temp").and_then(|t| temperature(&t)),
                    );
                }
            }
            _ => {}
        }
    }
    if !root_seen {
        return Err(Error::NotADiveLog(path.to_path_buf()));
    }
    for d in &mut log.dives {
        for c in &mut d.computers {
            c.samples.sort_by_key(|s| s.time_s);
        }
    }
    Ok(log)
}
