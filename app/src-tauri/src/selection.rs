//! Tag Selection's per-photo values (spec F8, F6): the photos selected in
//! Lightroom and their cameras, one clock offset each as in Create
//! Collections, the live counts and "Suggest" for the chosen dive, and the
//! values written on the photos taken during it. Nothing here talks to
//! Lightroom; `lib.rs` does.
//!
//! A photo is in the dive when its capture time corrected by its camera's
//! offset falls between the dive's start and end, exactly as in Create
//! Collections without a margin. Only those photos get per-photo values and
//! the clock offset; the others get the dive data alone.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDateTime;
use divelens_core::Dive;
use divelens_core::collections::{self as matching, Placement, Span};
use divelens_core::profile::Sample;
use divelens_core::tags::{PhotoTags, Units};
use serde::{Deserialize, Serialize};

use crate::collections::{Camera, CameraCount, Shots};
use crate::lightroom::FoundPhoto;
use crate::offsets::{Offsets, format_time, parse_time};

/// The photos selected in Lightroom, as the plugin described them, kept
/// between the live counts and "Apply".
pub struct Selection {
    /// The plugin's number for this description: "Apply" names it, and the
    /// plugin writes only if the photos selected then are exactly these.
    pub described: u64,
    shots: Shots,
}

/// What the screen shows of the selection.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPhotos {
    /// Number of the description (the screen's): the counts, "Suggest" and
    /// "Apply" name it.
    pub search: u64,
    /// Photos selected, with a capture time or not.
    pub photos: usize,
    /// Of which without a capture time: never in the dive.
    pub undated: usize,
    /// Cameras of the photos with a capture time, sorted by name.
    pub cameras: Vec<Camera>,
}

/// The chosen dive as the screen shows it: start and end, wall-clock
/// `YYYY-MM-DDTHH:MM:SS`.
#[derive(Debug, Clone, Deserialize)]
pub struct DiveSpan {
    pub start: String,
    pub end: String,
}

impl DiveSpan {
    pub fn span(&self) -> Option<Span> {
        Some(Span {
            start: parse_time(&self.start)?,
            end: parse_time(&self.end)?,
        })
    }
}

/// Live counts for the offsets shown.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagPreview {
    /// Photos selected.
    pub total: usize,
    /// Of which taken during the chosen dive (0 without one).
    pub in_dive: usize,
    /// By camera: photos in the dive, and the other photos with a capture
    /// time.
    pub cameras: Vec<CameraCount>,
    /// Earliest and latest capture times of the selection, each photo moved
    /// by its camera's offset: what the "covers the selection" hint checks.
    pub cover_from: Option<String>,
    pub cover_to: Option<String>,
}

/// A photo of the selection taken during the dive, with the offsets given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InDive {
    pub id: String,
    /// Its camera's key and offset (minutes).
    pub camera: String,
    pub offset: i32,
    /// Capture time corrected by the offset (dive computer time).
    pub corrected: NaiveDateTime,
}

fn inside(dive: Span, corrected: NaiveDateTime) -> bool {
    matches!(
        matching::place(&[dive], corrected, 0),
        Some(Placement::InDive(_))
    )
}

impl Selection {
    /// `offsets`: those remembered for the dive application of the log
    /// shown, pre-filled on each camera.
    pub fn new(described: u64, found: Vec<FoundPhoto>, offsets: &Offsets) -> Self {
        Self {
            described,
            shots: Shots::new(found, offsets),
        }
    }

    /// What the screen shows of description `search`.
    pub fn shown(&self, search: u64) -> SelectionPhotos {
        SelectionPhotos {
            search,
            photos: self.shots.photos.len() + self.shots.undated,
            undated: self.shots.undated,
            cameras: self.shots.cameras.clone(),
        }
    }

    /// Each photo with a capture time, its camera's offset and corrected time.
    fn corrected<'a>(
        &'a self,
        offsets: &'a HashMap<String, i32>,
    ) -> impl Iterator<Item = (usize, &'a str, i32, NaiveDateTime)> + 'a {
        self.shots.photos.iter().map(move |p| {
            let offset = self.shots.offset(offsets, p.camera);
            (
                p.camera,
                p.id.as_str(),
                offset,
                matching::corrected(p.time, offset),
            )
        })
    }

    pub fn preview(&self, dive: Option<Span>, offsets: &HashMap<String, i32>) -> TagPreview {
        let mut cameras: Vec<CameraCount> = self
            .shots
            .cameras
            .iter()
            .map(|c| CameraCount {
                key: c.key.clone(),
                in_dives: 0,
                outside: 0,
            })
            .collect();
        let mut in_dive = 0;
        let mut cover: Option<(NaiveDateTime, NaiveDateTime)> = None;
        for (camera, _, _, t) in self.corrected(offsets) {
            if dive.is_some_and(|d| inside(d, t)) {
                cameras[camera].in_dives += 1;
                in_dive += 1;
            } else {
                cameras[camera].outside += 1;
            }
            cover = Some(match cover {
                None => (t, t),
                Some((a, b)) => (a.min(t), b.max(t)),
            });
        }
        TagPreview {
            total: self.shots.photos.len() + self.shots.undated,
            in_dive,
            cameras,
            cover_from: cover.map(|(a, _)| format_time(a)),
            cover_to: cover.map(|(_, b)| format_time(b)),
        }
    }

    /// Suggested offset of each camera `keys` names: the one that puts the
    /// most of its selected photos in `dive` (same search as Create
    /// Collections, spec F5).
    pub fn suggest(&self, dive: Span, keys: &[String]) -> HashMap<String, Option<i32>> {
        self.shots.suggest(&[dive], keys)
    }

    /// The photos taken during `dive` with these offsets.
    pub fn in_dive(&self, dive: Span, offsets: &HashMap<String, i32>) -> Vec<InDive> {
        self.corrected(offsets)
            .filter(|&(_, _, _, t)| inside(dive, t))
            .map(|(camera, id, offset, corrected)| InDive {
                id: id.to_string(),
                camera: self.shots.cameras[camera].key.clone(),
                offset,
                corrected,
            })
            .collect()
    }
}

/// Per-photo values (spec F6) of `photos`, taken during `dive`, from its
/// profile: depth, temperature and elapsed time interpolated as Create
/// Collections does, and the clock offset applied.
pub fn photo_tags(
    dive: &Dive,
    profile: &[Sample],
    photos: &[InDive],
    units: Units,
) -> Vec<PhotoTags> {
    photos
        .iter()
        .map(|p| {
            matching::photo_tags(
                p.id.clone(),
                dive,
                profile,
                p.corrected,
                Placement::InDive(0),
                p.offset,
                units,
            )
        })
        .collect()
}

/// The offset applied to each camera with a photo taken during the dive:
/// remembered once written, shared with Create Collections (spec F5). A
/// camera without one had no offset applied.
pub fn applied_offsets(photos: &[InDive]) -> BTreeMap<String, i32> {
    photos
        .iter()
        .map(|p| (p.camera.clone(), p.offset))
        .collect()
}

/// Remembers `applied` in `offsets`, leaving the other cameras as they are.
pub fn remember(applied: &BTreeMap<String, i32>, offsets: &mut Offsets) {
    for (camera, offset) in applied {
        offsets.cameras.insert(camera.clone(), *offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate};
    use divelens_core::Site;
    use divelens_core::tags::{DiveTags, TagRequest, Units};

    fn at(d: u32, h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2025, 12, d)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    /// A dive in fictional waters, 50 minutes from 09:00 on the 30th.
    fn dive() -> Dive {
        let start = at(30, 9, 0);
        Dive {
            id: "A".into(),
            number: Some(701),
            start,
            end: start + Duration::minutes(50),
            duration_s: 3000,
            max_depth_m: Some(30.0),
            temp_high_c: Some(28.0),
            temp_low_c: Some(26.0),
            rating: None,
            site: Site {
                name: Some("Coral Garden".into()),
                ..Site::default()
            },
            has_profile: true,
            invalid: false,
            notes: None,
        }
    }

    /// 0 m at the start, 30 m at the end, 28 °C to 26 °C.
    fn profile() -> Vec<Sample> {
        vec![
            Sample {
                time_s: 0,
                depth_m: 0.0,
                temp_c: Some(28.0),
            },
            Sample {
                time_s: 3000,
                depth_m: 30.0,
                temp_c: Some(26.0),
            },
        ]
    }

    fn photo(id: &str, time: Option<&str>, model: &str, serial: Option<&str>) -> FoundPhoto {
        FoundPhoto {
            id: id.into(),
            time: time.map(Into::into),
            model: Some(model.into()),
            serial: serial.map(Into::into),
        }
    }

    /// A TG-7 still on French time (4 h behind the dive computer), a GoPro
    /// on dive time with one photo on the boat, and a photo without a
    /// capture time.
    fn selection(remembered: &Offsets) -> Selection {
        let found = vec![
            photo("1", Some("2025-12-30T05:05:00"), "TG-7", Some("A1")),
            photo("2", Some("2025-12-30T05:40:00"), "TG-7", Some("A1")),
            photo("3", Some("2025-12-30T05:45:00"), "TG-7", Some("A1")),
            photo("4", Some("2025-12-30T09:20:00"), "GoPro", None),
            photo("5", Some("2025-12-30T08:00:00"), "GoPro", None),
            photo("6", None, "TG-7", Some("A1")),
        ];
        Selection::new(42, found, remembered)
    }

    fn offsets(list: &[(&str, i32)]) -> HashMap<String, i32> {
        list.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn span() -> Span {
        Span::from(&dive())
    }

    #[test]
    fn cameras_by_name_with_their_remembered_offset() {
        let mut remembered = Offsets::default();
        remembered.cameras.insert("TG-7|A1".into(), 240);
        let shown = selection(&remembered).shown(7);
        assert_eq!(shown.search, 7);
        assert_eq!((shown.photos, shown.undated), (6, 1));
        let cams: Vec<_> = shown
            .cameras
            .iter()
            .map(|c| (c.key.as_str(), c.photos, c.remembered))
            .collect();
        assert_eq!(cams, [("GoPro|", 2, None), ("TG-7|A1", 3, Some(240))]);
    }

    #[test]
    fn counts_and_cover_follow_the_offsets_shown() {
        let s = selection(&Offsets::default());
        // No offset: only the GoPro photo at 09:20 is in the dive.
        let p = s.preview(Some(span()), &HashMap::new());
        assert_eq!((p.total, p.in_dive), (6, 1));
        assert_eq!(
            p.cameras,
            [
                CameraCount {
                    key: "GoPro|".into(),
                    in_dives: 1,
                    outside: 1
                },
                CameraCount {
                    key: "TG-7|A1".into(),
                    in_dives: 0,
                    outside: 3
                },
            ]
        );
        assert_eq!(p.cover_from.as_deref(), Some("2025-12-30T05:05:00"));
        assert_eq!(p.cover_to.as_deref(), Some("2025-12-30T09:20:00"));
        // The TG-7 put on dive time: its three photos join the dive, and the
        // range the "covers the selection" hint checks moves with it.
        let p = s.preview(Some(span()), &offsets(&[("TG-7|A1", 240)]));
        assert_eq!(p.in_dive, 4);
        assert_eq!(p.cameras[1].in_dives, 3);
        assert_eq!(p.cover_from.as_deref(), Some("2025-12-30T08:00:00"));
        assert_eq!(p.cover_to.as_deref(), Some("2025-12-30T09:45:00"));
        // No dive chosen yet: no photo in a dive, the range all the same.
        let p = s.preview(None, &offsets(&[("TG-7|A1", 240)]));
        assert_eq!(p.in_dive, 0);
        assert_eq!(p.cover_to.as_deref(), Some("2025-12-30T09:45:00"));
        // A selection without capture times has no range.
        let undated = Selection::new(1, vec![photo("9", None, "TG-7", None)], &Offsets::default());
        let p = undated.preview(Some(span()), &HashMap::new());
        assert_eq!((p.total, p.cover_from), (1, None));
    }

    #[test]
    fn suggests_the_time_zone_difference() {
        // Spec F5, same criterion: photos on French time, the dive in the
        // Maldives (+4 h).
        let s = selection(&Offsets::default());
        let found = s.suggest(
            span(),
            &["TG-7|A1".into(), "GoPro|".into(), "Nikon|".into()],
        );
        assert_eq!(found["TG-7|A1"], Some(240));
        assert_eq!(found["GoPro|"], Some(0), "already on dive time");
        assert_eq!(found["Nikon|"], None, "not in the selection");
    }

    #[test]
    fn per_photo_values_only_for_photos_in_the_dive() {
        let s = selection(&Offsets::default());
        let inside = s.in_dive(span(), &offsets(&[("TG-7|A1", 240)]));
        let ids: Vec<_> = inside.iter().map(|p| p.id.as_str()).collect();
        // The boat photo (GoPro 08:00) and the undated one are not listed.
        assert_eq!(ids, ["1", "2", "3", "4"]);
        let tags = photo_tags(&dive(), &profile(), &inside, Units::default());
        let by_id = |id: &str| tags.iter().find(|t| t.id == id).unwrap();
        // TG-7 photo 1: 05:05 + 4 h = 09:05, 300 s into the dive.
        assert_eq!(by_id("1").photo_depth.as_deref(), Some("3.0 m"));
        assert_eq!(by_id("1").photo_temp.as_deref(), Some("27.8°C"));
        assert_eq!(by_id("1").photo_elapsed.as_deref(), Some("05:00"));
        assert_eq!(by_id("1").clock_offset, "+04:00");
        // GoPro photo 4: 09:20, no offset.
        assert_eq!(by_id("4").photo_depth.as_deref(), Some("12.0 m"));
        assert_eq!(by_id("4").clock_offset, "+00:00");
        // Without the TG-7's offset, only the GoPro photo is in the dive.
        let inside = s.in_dive(span(), &HashMap::new());
        assert_eq!(inside.len(), 1);
        assert_eq!(inside[0].id, "4");
    }

    #[test]
    fn the_depth_goes_to_the_altitude_only_with_the_option() {
        let s = selection(&Offsets::default());
        let inside = s.in_dive(span(), &offsets(&[("TG-7|A1", 240)]));
        let request = |copy: bool| {
            let mut r = TagRequest {
                dive: Some(DiveTags::new(&dive(), Units::default(), "MacDive")),
                selection: Some(s.described),
                photos: photo_tags(&dive(), &profile(), &inside, Units::default()),
                ..TagRequest::default()
            };
            if copy {
                r.copy_depth_to_altitude();
            }
            r
        };
        assert!(
            request(false)
                .photos
                .iter()
                .all(|p| p.gps_altitude.is_none())
        );
        let on = request(true);
        assert_eq!(on.selection, Some(42));
        assert_eq!(on.photos[0].gps_altitude, Some(-3.0));
        assert_eq!(on.photos.len(), 4, "only the photos in the dive");
    }

    #[test]
    fn offsets_applied_are_remembered_for_the_application_only() {
        let s = selection(&Offsets::default());
        // The GoPro set to an offset that puts none of its photos in the dive:
        // nothing applied to it, nothing remembered.
        let inside = s.in_dive(span(), &offsets(&[("TG-7|A1", 240), ("GoPro|", 600)]));
        let applied = applied_offsets(&inside);
        assert_eq!(applied, BTreeMap::from([("TG-7|A1".to_string(), 240)]));
        let mut remembered = Offsets::default();
        remembered.cameras.insert("Nikon|".into(), 60);
        remembered.cameras.insert("TG-7|A1".into(), 120);
        remember(&applied, &mut remembered);
        assert_eq!(remembered.get("TG-7|A1"), Some(240));
        assert_eq!(remembered.get("GoPro|"), None);
        assert_eq!(remembered.get("Nikon|"), Some(60), "other cameras kept");

        // Saved in the application's own file, the one Create Collections
        // reads: another application does not see it.
        let dir = std::env::temp_dir().join(format!("divelens-selection-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ss = &divelens_core::subsurface::APP;
        let md = &divelens_core::macdive::APP;
        remembered.save(&dir, ss).unwrap();
        assert_eq!(Offsets::load(&dir, ss).get("TG-7|A1"), Some(240));
        assert_eq!(Offsets::load(&dir, md).get("TG-7|A1"), None);
        let again = selection(&Offsets::load(&dir, ss)).shown(1);
        assert_eq!(
            again.cameras[1].remembered,
            Some(240),
            "pre-filled next time"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reads_the_dive_shown() {
        let span = DiveSpan {
            start: "2025-12-30T09:00:00".into(),
            end: "2025-12-30T09:50:00".into(),
        };
        assert_eq!(span.span(), Some(super::Span::from(&dive())));
        let bad = DiveSpan {
            start: "30/12/2025".into(),
            end: "2025-12-30T09:50:00".into(),
        };
        assert_eq!(bad.span(), None);
    }
}
