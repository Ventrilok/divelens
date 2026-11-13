//! Clock offset last applied to each camera by "Create Collections" (spec F5):
//! pre-filled the next time, and used by Tag Selection's "covers the
//! selection" hint (F8). One small JSON file per dive application in the app
//! config folder: the same dive may start at another time in another
//! application's log (seen after importing a MacDive log into Subsurface).

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::settings::SourceKind;

/// File of each application (MacDive's is the one from before Subsurface).
pub fn file_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::MacDive => "clock-offsets.json",
        SourceKind::Subsurface => "clock-offsets-subsurface.json",
    }
}
const CURRENT_VERSION: u32 = 1;

/// A camera: model and serial number when the photos carry one, model alone
/// otherwise.
pub fn camera_key(model: Option<&str>, serial: Option<&str>) -> String {
    format!("{}|{}", model.unwrap_or(""), serial.unwrap_or(""))
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Offsets {
    pub version: u32,
    /// Minutes, by camera key.
    pub cameras: BTreeMap<String, i32>,
}

impl Offsets {
    /// Loads the offsets of `kind`; a missing or unreadable file gives none.
    pub fn load(dir: &Path, kind: SourceKind) -> Self {
        std::fs::read_to_string(dir.join(file_name(kind)))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, dir: &Path, kind: SourceKind) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let name = file_name(kind);
        let tmp = dir.join(format!("{name}.tmp"));
        let mut file = self.clone();
        file.version = CURRENT_VERSION;
        std::fs::write(&tmp, serde_json::to_string_pretty(&file)?)?;
        std::fs::rename(tmp, dir.join(name))
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        self.cameras.get(key).copied()
    }
}

/// A wall-clock time `YYYY-MM-DDTHH:MM:SS`.
pub fn parse_time(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
}

pub fn format_time(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Earliest and latest capture times of a selection, each camera's range
/// moved by its remembered offset (none: unchanged). `ranges`: (camera key,
/// from, to), wall-clock.
pub fn corrected_range(
    offsets: &Offsets,
    ranges: &[(String, String, String)],
) -> Option<(String, String)> {
    let mut range: Option<(NaiveDateTime, NaiveDateTime)> = None;
    for (key, from, to) in ranges {
        let (Some(from), Some(to)) = (parse_time(from), parse_time(to)) else {
            continue;
        };
        let shift = Duration::minutes(offsets.get(key).unwrap_or(0).into());
        let (from, to) = (from + shift, to + shift);
        range = Some(match range {
            None => (from, to),
            Some((a, b)) => (a.min(from), b.max(to)),
        });
    }
    range.map(|(a, b)| (format_time(a), format_time(b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_keys() {
        assert_eq!(camera_key(Some("TG-7"), Some("A1")), "TG-7|A1");
        assert_eq!(camera_key(Some("TG-7"), None), "TG-7|");
        assert_eq!(camera_key(None, None), "|");
    }

    #[test]
    fn saves_and_loads() {
        let dir = std::env::temp_dir().join(format!("divelens-offsets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let md = SourceKind::MacDive;
        assert_eq!(Offsets::load(&dir, md), Offsets::default());
        let mut offsets = Offsets::default();
        offsets.cameras.insert("TG-7|A1".into(), 241);
        offsets.save(&dir, md).unwrap();
        let loaded = Offsets::load(&dir, md);
        assert_eq!(loaded.get("TG-7|A1"), Some(241));
        assert_eq!(loaded.version, 1);
        assert!(
            dir.join("clock-offsets.json").is_file(),
            "MacDive keeps its file"
        );
        // Each application has its own offsets.
        assert_eq!(
            Offsets::load(&dir, SourceKind::Subsurface),
            Offsets::default()
        );
        std::fs::write(dir.join(file_name(md)), "{broken").unwrap();
        assert_eq!(Offsets::load(&dir, md), Offsets::default());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn selection_range_uses_each_cameras_offset() {
        let mut offsets = Offsets::default();
        offsets.cameras.insert("TG-7|A1".into(), 240);
        let ranges = [
            (
                "TG-7|A1".to_string(),
                "2025-12-30T05:02:00".to_string(),
                "2025-12-30T05:40:00".to_string(),
            ),
            (
                "GoPro|".to_string(),
                "2025-12-30T09:10:00".to_string(),
                "2025-12-30T09:20:00".to_string(),
            ),
        ];
        assert_eq!(
            corrected_range(&offsets, &ranges),
            Some(("2025-12-30T09:02:00".into(), "2025-12-30T09:40:00".into()))
        );
        assert_eq!(corrected_range(&offsets, &[]), None);
    }
}
