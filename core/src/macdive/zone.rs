//! The time zone MacDive keeps with each dive.
//!
//! MacDive stores a dive's start as an instant, seconds since 2001-01-01
//! 00:00 UTC (`ZRAWDATE`), and beside it the time zone it shows the dive in
//! (`ZTIMEZONE`): an `NSTimeZone` archived by `NSKeyedArchiver`, a binary
//! property list holding the zone's name (`NS.name`) and its rules
//! (`NS.data`). The dive computer's wall clock is the instant in that zone.
//! Many dives carry "GMT" (downloaded from a Shearwater computer, for
//! instance): their wall clock is the instant itself. Others carry a place's
//! zone, such as "Europe/Zurich": read as GMT, they would start one or two
//! hours early.
//!
//! The zone is looked up by name in the time zone database built into Dive
//! Lens, the same on every system; the rules archived with it are not used.

use chrono::{DateTime, FixedOffset, NaiveDateTime, Offset, TimeZone, Utc};
use plist::Value;

/// Wall-clock time of `instant` in the zone archived in `archive`. Without a
/// zone, or with one Dive Lens cannot read, the instant itself, as for GMT.
pub(super) fn wall_clock(instant: DateTime<Utc>, archive: Option<&[u8]>) -> NaiveDateTime {
    match archive.and_then(name).and_then(|n| offset(&n, instant)) {
        Some(offset) => instant.with_timezone(&offset).naive_local(),
        None => instant.naive_utc(),
    }
}

/// Name of the archived zone: `Europe/Zurich`, `GMT`, `GMT+0200`.
fn name(archive: &[u8]) -> Option<String> {
    let archive = Value::from_reader(std::io::Cursor::new(archive)).ok()?;
    let archive = archive.as_dictionary()?;
    let objects = archive.get("$objects")?.as_array()?;
    // Archived objects point to each other by their index in `$objects`.
    let object = |v: &Value| objects.get(usize::try_from(v.as_uid()?.get()).ok()?);
    let root = object(archive.get("$top")?.as_dictionary()?.get("root")?)?;
    let name = object(root.as_dictionary()?.get("NS.name")?)?;
    match name {
        Value::String(s) => Some(s.clone()),
        // A name archived as a string object of its own.
        Value::Dictionary(d) => d.get("NS.string")?.as_string().map(str::to_string),
        _ => None,
    }
}

/// Offset from UTC of the zone `name` at `instant`: a zone of the time zone
/// database (`Europe/Zurich`, `GMT`), or a fixed offset as Apple names them
/// (`GMT+0200`, `GMT-05:30`, `GMT+2`).
fn offset(name: &str, instant: DateTime<Utc>) -> Option<FixedOffset> {
    match name.parse::<chrono_tz::Tz>() {
        Ok(tz) => Some(tz.offset_from_utc_datetime(&instant.naive_utc()).fix()),
        Err(_) => fixed(name),
    }
}

/// `GMT+0200` and the like; `None` for anything else.
fn fixed(name: &str) -> Option<FixedOffset> {
    let rest = name
        .strip_prefix("GMT")
        .or_else(|| name.strip_prefix("UTC"))?;
    let (sign, rest) = if let Some(r) = rest.strip_prefix('+') {
        (1, r)
    } else {
        (-1, rest.strip_prefix('-')?)
    };
    let digits = rest.replacen(':', "", 1);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (hours, minutes) = match digits.len() {
        1 | 2 => (digits.as_str(), "0"),
        3 | 4 => digits.split_at(digits.len() - 2),
        _ => return None,
    };
    let (hours, minutes): (i32, i32) = (hours.parse().ok()?, minutes.parse().ok()?);
    if hours > 18 || minutes > 59 {
        return None;
    }
    FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))
}

/// An `NSTimeZone` named `name`, archived as MacDive archives it, without
/// the zone's rules (Dive Lens does not read them).
#[cfg(test)]
pub(crate) fn archive(name: &str) -> Vec<u8> {
    use plist::{Dictionary, Uid};
    let dict = |pairs: Vec<(&str, Value)>| {
        Value::Dictionary(
            pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<Dictionary>(),
        )
    };
    let archive = dict(vec![
        ("$version", Value::Integer(100_000.into())),
        ("$archiver", Value::String("NSKeyedArchiver".into())),
        ("$top", dict(vec![("root", Value::Uid(Uid::new(1)))])),
        (
            "$objects",
            Value::Array(vec![
                Value::String("$null".into()),
                dict(vec![
                    ("NS.name", Value::Uid(Uid::new(2))),
                    ("$class", Value::Uid(Uid::new(3))),
                ]),
                Value::String(name.into()),
                dict(vec![
                    ("$classname", Value::String("NSTimeZone".into())),
                    (
                        "$classes",
                        Value::Array(vec![
                            Value::String("NSTimeZone".into()),
                            Value::String("NSObject".into()),
                        ]),
                    ),
                ]),
            ]),
        ),
    ]);
    let mut bytes = Vec::new();
    archive.to_writer_binary(&mut bytes).unwrap();
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    /// Wall clock of the UTC time `utc` in the zone `name`, archived.
    fn clock(name: &str, utc: NaiveDateTime) -> NaiveDateTime {
        wall_clock(utc.and_utc(), Some(&archive(name)))
    }

    #[test]
    fn reads_a_place_zone_with_its_summer_time() {
        assert_eq!(
            clock("Europe/Zurich", at(2025, 1, 10, 9, 0)),
            at(2025, 1, 10, 10, 0)
        );
        assert_eq!(
            clock("Europe/Zurich", at(2025, 7, 10, 9, 0)),
            at(2025, 7, 10, 11, 0)
        );
        assert_eq!(
            clock("America/Kralendijk", at(2025, 2, 3, 14, 10)),
            at(2025, 2, 3, 10, 10)
        );
        assert_eq!(
            clock("Asia/Kolkata", at(2025, 6, 17, 4, 30)),
            at(2025, 6, 17, 10, 0)
        );
    }

    #[test]
    fn the_wall_clock_can_fall_on_another_day() {
        assert_eq!(
            clock("Pacific/Auckland", at(2025, 6, 14, 23, 0)),
            at(2025, 6, 15, 11, 0)
        );
        assert_eq!(
            clock("America/Kralendijk", at(2025, 6, 16, 2, 0)),
            at(2025, 6, 15, 22, 0)
        );
    }

    #[test]
    fn gmt_and_fixed_offsets() {
        let t = at(2025, 6, 14, 9, 30);
        assert_eq!(clock("GMT", t), t);
        assert_eq!(clock("UTC", t), t);
        assert_eq!(clock("GMT+0200", t), at(2025, 6, 14, 11, 30));
        assert_eq!(clock("GMT-05:30", t), at(2025, 6, 14, 4, 0));
        assert_eq!(clock("GMT+2", t), at(2025, 6, 14, 11, 30));
        assert_eq!(clock("UTC-0330", t), at(2025, 6, 14, 6, 0));
    }

    #[test]
    fn without_a_readable_zone_the_instant_is_the_wall_clock() {
        let t = at(2025, 6, 14, 9, 30);
        assert_eq!(wall_clock(t.and_utc(), None), t);
        assert_eq!(wall_clock(t.and_utc(), Some(b"not a plist")), t);
        assert_eq!(clock("Atlantis/Lost City", t), t);
        assert_eq!(clock("GMT+2x", t), t);
        assert_eq!(clock("GMT+99", t), t);
        assert_eq!(clock("", t), t);
    }

    #[test]
    fn reads_a_name_archived_as_a_string_object() {
        let mut archive = Value::from_reader(std::io::Cursor::new(archive("GMT"))).unwrap();
        let objects = archive
            .as_dictionary_mut()
            .unwrap()
            .get_mut("$objects")
            .unwrap()
            .as_array_mut()
            .unwrap();
        let mut string = plist::Dictionary::new();
        string.insert("NS.string".into(), Value::String("Europe/Zurich".into()));
        objects[2] = Value::Dictionary(string);
        let mut bytes = Vec::new();
        archive.to_writer_binary(&mut bytes).unwrap();
        let t = at(2025, 1, 10, 9, 0);
        assert_eq!(
            wall_clock(t.and_utc(), Some(&bytes)),
            at(2025, 1, 10, 10, 0)
        );
    }
}
