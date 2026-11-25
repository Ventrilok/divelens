//! Clock offset last applied to each camera on the catalog path (spec F5) or
//! the selection path (F8): pre-filled the next time on both, and used by the
//! selection path's "covers the selection" hint until the selection's cameras are
//! shown with their offsets. One small JSON file per dive application in the app
//! config folder: the same dive may start at another time in another
//! application's log (seen after importing a MacDive log into Subsurface).

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{Duration, NaiveDateTime};
use divelens_core::apps::DiveApp;
use serde::{Deserialize, Serialize};

use crate::lightroom::ApplyOutcome;

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
    /// Loads the offsets of `app` (its own file); a missing or unreadable
    /// file gives none.
    pub fn load(dir: &Path, app: &DiveApp) -> Self {
        crate::files::read_json(&dir.join(app.offsets_file))
    }

    pub fn save(&self, dir: &Path, app: &DiveApp) -> std::io::Result<()> {
        let mut file = self.clone();
        file.version = CURRENT_VERSION;
        crate::files::write_json(dir, app.offsets_file, &file)
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        self.cameras.get(key).copied()
    }
}

/// Whether the offsets `applied` are remembered once Lightroom gave `answer`: when it wrote, even if only the GPS and altitude were left out
/// (`gpsNotWritten`: the offsets are on the photos), and there are some.
pub(crate) fn should_remember<T>(
    answer: &Result<T, ApplyOutcome>,
    applied: &BTreeMap<String, i32>,
) -> bool {
    answer.as_ref().err().is_none_or(ApplyOutcome::half_written) && !applied.is_empty()
}

/// Remembers `applied` in the offsets file of `app` in `dir` when
/// [`should_remember`] says so, leaving the other cameras as they are; a
/// file that cannot be saved is logged, never an error (the write is done).
pub(crate) fn remember_applied<T>(
    dir: &Path,
    app: &DiveApp,
    answer: &Result<T, ApplyOutcome>,
    applied: &BTreeMap<String, i32>,
) {
    if !should_remember(answer, applied) {
        return;
    }
    let mut offsets = Offsets::load(dir, app);
    offsets.cameras.extend(
        applied
            .iter()
            .map(|(camera, offset)| (camera.clone(), *offset)),
    );
    if let Err(e) = offsets.save(dir, app) {
        log::error!("clock offsets not saved: {e}");
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
        let md = &divelens_core::macdive::APP;
        let ss = &divelens_core::subsurface::APP;
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
        assert_eq!(Offsets::load(&dir, ss), Offsets::default());
        offsets.save(&dir, ss).unwrap();
        assert!(dir.join("clock-offsets-subsurface.json").is_file());
        std::fs::write(dir.join(md.offsets_file), "{broken").unwrap();
        assert_eq!(Offsets::load(&dir, md), Offsets::default());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn offsets_are_remembered_only_when_something_was_written() {
        let applied = BTreeMap::from([("TG-7|A1".to_string(), 240)]);
        let none = BTreeMap::new();
        let written = Ok::<usize, ApplyOutcome>(3);
        let refused = Err::<usize, _>(ApplyOutcome::Failed {
            reason: "busy".into(),
            detail: None,
        });
        let gps_not_written = Err::<usize, _>(ApplyOutcome::Failed {
            reason: "gpsNotWritten".into(),
            detail: None,
        });
        assert!(should_remember(&written, &applied));
        assert!(!should_remember(&written, &none));
        assert!(!should_remember(&refused, &applied));
        assert!(should_remember(&gps_not_written, &applied));
    }

    #[test]
    fn offsets_are_saved_only_for_a_write_that_happened() {
        let dir = std::env::temp_dir().join(format!("divelens-remember-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let app = &divelens_core::macdive::APP;
        let file = dir.join(app.offsets_file);
        let applied = BTreeMap::from([("TG-7|A1".to_string(), 240)]);
        let refused = Err::<usize, _>(ApplyOutcome::Failed {
            reason: "busy".into(),
            detail: None,
        });
        // Refused: the file is not touched (not even created).
        remember_applied(&dir, app, &refused, &applied);
        assert!(!file.exists());
        // Nothing applied: no file written.
        remember_applied(&dir, app, &Ok::<usize, ApplyOutcome>(3), &BTreeMap::new());
        assert!(!file.exists());
        // Written: saved, the other cameras kept.
        let mut before = Offsets::default();
        before.cameras.insert("Nikon|".into(), 60);
        before.cameras.insert("TG-7|A1".into(), 120);
        before.save(&dir, app).unwrap();
        remember_applied(&dir, app, &Ok::<usize, ApplyOutcome>(3), &applied);
        let saved = Offsets::load(&dir, app);
        assert_eq!(saved.get("TG-7|A1"), Some(240));
        assert_eq!(saved.get("Nikon|"), Some(60));
        // Refused after a saved write: the saved values stay as they are.
        remember_applied(
            &dir,
            app,
            &refused,
            &BTreeMap::from([("Nikon|".to_string(), 999)]),
        );
        assert_eq!(Offsets::load(&dir, app), saved);
        // The GPS and altitude missing: the offsets are on the photos.
        let gps = Err::<usize, _>(ApplyOutcome::Failed {
            reason: "gpsNotWritten".into(),
            detail: None,
        });
        remember_applied(
            &dir,
            app,
            &gps,
            &BTreeMap::from([("Nikon|".to_string(), 5)]),
        );
        assert_eq!(Offsets::load(&dir, app).get("Nikon|"), Some(5));
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
