//! "Create Collections" (spec F5): the photos found in Lightroom for the
//! chosen dives, their cameras, live counts and the request to write.
//! Nothing here talks to Lightroom; `lib.rs` does.

use std::collections::{BTreeSet, HashMap};

use chrono::{Duration, NaiveDateTime};
use divelens_core::collections::{self as matching, Placement, Span};
use divelens_core::profile::Sample;
use divelens_core::tags::{
    CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
};
use divelens_core::{Dive, Error};
use serde::{Deserialize, Serialize};

use crate::dive_log::{LogChanged, LogId};
use crate::lightroom::FoundPhoto;
use crate::offsets::{Offsets, camera_key, parse_time};

/// Days Lightroom is asked for, around the dives: clock offsets go up to a
/// day and Lightroom searches by whole day.
pub const DAYS_AROUND: i64 = 1;

/// A camera found among the photos of the period.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    pub key: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    /// Photos of the period with a capture time.
    pub photos: usize,
    /// Offset last applied to this camera, in minutes.
    pub remembered: Option<i32>,
}

struct Photo {
    id: String,
    time: NaiveDateTime,
    camera: usize,
}

/// Photos of the chosen dives' period, kept between the live counts and
/// "Apply".
pub struct Session {
    pub search: u64,
    /// Chosen dives, by start.
    pub dives: Vec<Dive>,
    spans: Vec<Span>,
    photos: Vec<Photo>,
    pub cameras: Vec<Camera>,
    /// Photos of the period without a capture time (never placed).
    pub undated: usize,
    /// The dive log the dives were read from (`None` until known).
    log: Option<LogId>,
    /// Profiles of the dives that have one, read with the dives, by dive id.
    profiles: HashMap<String, Result<Vec<Sample>, Error>>,
}

/// What the screen shows once the photos are found.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    /// Number of the search (see [`Searches`]): the live counts, "Suggest"
    /// and "Apply" name it.
    pub search: u64,
    /// First and last day searched, `YYYY-MM-DD`.
    pub from: String,
    pub to: String,
    pub photos: usize,
    pub undated: usize,
    pub cameras: Vec<Camera>,
}

/// The user's choices on the screen.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choices {
    /// Minutes, by camera key; a missing camera has no offset.
    #[serde(default)]
    pub offsets: HashMap<String, i32>,
    #[serde(default)]
    pub ignored: BTreeSet<String>,
    #[serde(default)]
    pub margin_min: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiveCount {
    pub id: String,
    /// Photos going to this dive (in the dive or at the surface).
    pub photos: usize,
    /// Of which taken during the dive (they get per-photo values).
    pub in_dive: usize,
    /// Cameras contributing.
    pub cameras: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraCount {
    pub key: String,
    /// Photos going to one of the dives.
    pub in_dives: usize,
    /// Photos of the period that go nowhere.
    pub outside: usize,
}

/// Live counts.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub dives: Vec<DiveCount>,
    pub cameras: Vec<CameraCount>,
    /// Photos that will be written.
    pub total: usize,
    /// Photos of cameras taken into account that go to no dive.
    pub outside: usize,
    /// Photos of ignored cameras.
    pub ignored: usize,
}

/// The photo searches of "Create Collections". The screen numbers each one,
/// in the order it asks for them; only the latest one keeps its session, and
/// every later step (live counts, "Suggest", "Apply") names the search it
/// works on. A slower, older search (more dives, more profiles to read) can
/// then never replace the newer one the screen shows.
#[derive(Default)]
pub struct Searches {
    started: u64,
    kept: Option<(u64, Session)>,
}

impl Searches {
    /// Starts search `number`, unless a search numbered as high or higher
    /// started already (the screen asked for this one earlier).
    pub fn start(&mut self, number: u64) -> bool {
        if number <= self.started {
            return false;
        }
        self.started = number;
        true
    }

    /// No newer search started since search `number`.
    pub fn is_latest(&self, number: u64) -> bool {
        number == self.started
    }

    /// Keeps the session of search `number`, unless a newer search started
    /// since. Returns whether it was kept.
    pub fn finish(&mut self, number: u64, session: Session) -> bool {
        if number != self.started {
            return false;
        }
        self.kept = Some((number, session));
        true
    }

    /// The session of search `number`; `None` when another one is kept.
    pub fn get(&self, number: u64) -> Option<&Session> {
        match &self.kept {
            Some((kept, session)) if *kept == number => Some(session),
            _ => None,
        }
    }
}

/// First and last day to search for, `YYYY-MM-DD`.
pub fn search_days(dives: &[Dive]) -> Option<(String, String)> {
    let first = dives.iter().map(|d| d.start).min()?;
    let last = dives.iter().map(|d| d.end).max()?;
    let day = |t: NaiveDateTime| t.date().format("%Y-%m-%d").to_string();
    Some((
        day(first - Duration::days(DAYS_AROUND)),
        day(last + Duration::days(DAYS_AROUND)),
    ))
}

impl Session {
    pub fn new(
        search: u64,
        mut dives: Vec<Dive>,
        found: Vec<FoundPhoto>,
        offsets: &Offsets,
    ) -> Self {
        dives.sort_by_key(|d| d.start);
        let spans = dives.iter().map(Span::from).collect();
        let mut cameras: Vec<Camera> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut photos = Vec::new();
        let mut undated = 0;
        for p in found {
            let Some(time) = p.time.as_deref().and_then(parse_time) else {
                undated += 1;
                continue;
            };
            let key = camera_key(p.model.as_deref(), p.serial.as_deref());
            let camera = *index.entry(key.clone()).or_insert_with(|| {
                cameras.push(Camera {
                    remembered: offsets.get(&key),
                    key,
                    model: p.model.clone(),
                    serial: p.serial.clone(),
                    photos: 0,
                });
                cameras.len() - 1
            });
            cameras[camera].photos += 1;
            photos.push(Photo {
                id: p.id,
                time,
                camera,
            });
        }
        // Spec: cameras sorted by name. Photo indexes follow the new order.
        let mut order: Vec<usize> = (0..cameras.len()).collect();
        let name = |c: &Camera| {
            (
                c.model.clone().unwrap_or_default().to_lowercase(),
                c.serial.clone().unwrap_or_default(),
            )
        };
        order.sort_by_key(|&i| name(&cameras[i]));
        let mut new_index = vec![0; cameras.len()];
        for (new, &old) in order.iter().enumerate() {
            new_index[old] = new;
        }
        for p in &mut photos {
            p.camera = new_index[p.camera];
        }
        let cameras = order.into_iter().map(|i| cameras[i].clone()).collect();
        Self {
            search,
            dives,
            spans,
            photos,
            cameras,
            undated,
            log: None,
            profiles: HashMap::new(),
        }
    }

    /// Keeps the dive log the dives were read from and the profiles read
    /// with them: "Apply" writes these, never a later read of the log, and
    /// names the application they come from (spec F5).
    pub fn read_from(
        mut self,
        log: LogId,
        profiles: HashMap<String, Result<Vec<Sample>, Error>>,
    ) -> Self {
        self.log = Some(log);
        self.profiles = profiles;
        self
    }

    /// The write request for "Apply", from the dives and profiles kept, with
    /// their application's name. Refused unless they come from `log`, the
    /// one the screen shows and the settings still point at.
    pub fn write_request(
        &self,
        log: &LogId,
        choices: &Choices,
        units: Units,
    ) -> Result<Result<CollectionsRequest, Error>, LogChanged> {
        let kept = self.log.as_ref().ok_or(LogChanged)?;
        if kept != log {
            return Err(LogChanged);
        }
        Ok(self.request(choices, units, kept.kind.name(), |d| {
            self.profiles.get(&d.id).cloned().unwrap_or(Ok(Vec::new()))
        }))
    }

    /// What the screen shows of search `search`.
    pub fn period(&self, search: u64, from: String, to: String) -> Period {
        Period {
            search,
            from,
            to,
            photos: self.photos.len() + self.undated,
            undated: self.undated,
            cameras: self.cameras.clone(),
        }
    }

    fn offset(&self, choices: &Choices, camera: usize) -> i32 {
        choices
            .offsets
            .get(&self.cameras[camera].key)
            .copied()
            .unwrap_or(0)
            .clamp(-matching::MAX_OFFSET_MIN, matching::MAX_OFFSET_MIN)
    }

    fn ignored(&self, choices: &Choices, camera: usize) -> bool {
        choices.ignored.contains(&self.cameras[camera].key)
    }

    /// Each photo taken into account with its corrected time and placement.
    fn placements(&self, choices: &Choices) -> Vec<(&Photo, NaiveDateTime, Option<Placement>)> {
        self.photos
            .iter()
            .filter(|p| !self.ignored(choices, p.camera))
            .map(|p| {
                let t = matching::corrected(p.time, self.offset(choices, p.camera));
                (p, t, matching::place(&self.spans, t, choices.margin_min))
            })
            .collect()
    }

    pub fn preview(&self, choices: &Choices) -> Preview {
        let mut dives: Vec<DiveCount> = self
            .dives
            .iter()
            .map(|d| DiveCount {
                id: d.id.clone(),
                photos: 0,
                in_dive: 0,
                cameras: 0,
            })
            .collect();
        let mut dive_cameras = vec![BTreeSet::new(); self.dives.len()];
        let mut cameras: Vec<CameraCount> = self
            .cameras
            .iter()
            .map(|c| CameraCount {
                key: c.key.clone(),
                in_dives: 0,
                outside: 0,
            })
            .collect();
        let mut total = 0;
        let mut outside = 0;
        for (p, _, placement) in self.placements(choices) {
            match placement {
                Some(placement) => {
                    let d = placement.dive();
                    dives[d].photos += 1;
                    if matches!(placement, Placement::InDive(_)) {
                        dives[d].in_dive += 1;
                    }
                    dive_cameras[d].insert(p.camera);
                    cameras[p.camera].in_dives += 1;
                    total += 1;
                }
                None => {
                    cameras[p.camera].outside += 1;
                    outside += 1;
                }
            }
        }
        for (count, set) in dives.iter_mut().zip(dive_cameras) {
            count.cameras = set.len();
        }
        let ignored = self
            .photos
            .iter()
            .filter(|p| self.ignored(choices, p.camera))
            .count();
        Preview {
            dives,
            cameras,
            total,
            outside,
            ignored,
        }
    }

    /// Suggested offset of each camera (spec F5 "Suggest").
    pub fn suggest(&self, keys: &[String]) -> HashMap<String, Option<i32>> {
        keys.iter()
            .map(|key| {
                let times: Vec<NaiveDateTime> = self
                    .photos
                    .iter()
                    .filter(|p| &self.cameras[p.camera].key == key)
                    .map(|p| p.time)
                    .collect();
                (key.clone(), matching::suggest_offset(&self.spans, &times))
            })
            .collect()
    }

    /// Dives that will receive photos, as indexes, and whether any of their
    /// photos was taken during the dive (its profile is then needed).
    pub fn dives_with_photos(&self, choices: &Choices) -> Vec<(usize, bool)> {
        let mut used: Vec<Option<bool>> = vec![None; self.dives.len()];
        for (_, _, placement) in self.placements(choices) {
            if let Some(placement) = placement {
                let in_dive = matches!(placement, Placement::InDive(_));
                let slot = &mut used[placement.dive()];
                *slot = Some(slot.unwrap_or(false) || in_dive);
            }
        }
        used.into_iter()
            .enumerate()
            .filter_map(|(i, u)| u.map(|in_dive| (i, in_dive)))
            .collect()
    }

    /// The write request. `profile` gives a dive's profile (only asked for
    /// dives with photos taken during the dive); `source` names the dive
    /// application.
    pub fn request(
        &self,
        choices: &Choices,
        units: Units,
        source: &str,
        mut profile: impl FnMut(&Dive) -> Result<Vec<Sample>, Error>,
    ) -> Result<CollectionsRequest, Error> {
        let mut profiles: HashMap<usize, Vec<Sample>> = HashMap::new();
        for (i, in_dive) in self.dives_with_photos(choices) {
            if in_dive && self.dives[i].has_profile {
                profiles.insert(i, profile(&self.dives[i])?);
            }
        }
        let mut photos: Vec<Vec<PhotoTags>> = vec![Vec::new(); self.dives.len()];
        for (p, t, placement) in self.placements(choices) {
            let Some(placement) = placement else { continue };
            let d = placement.dive();
            let samples = profiles.get(&d).map(Vec::as_slice).unwrap_or(&[]);
            photos[d].push(matching::photo_tags(
                p.id.clone(),
                &self.dives[d],
                samples,
                t,
                placement,
                self.offset(choices, p.camera),
                units,
            ));
        }
        let groups = self
            .dives
            .iter()
            .zip(photos)
            .filter(|(_, photos)| !photos.is_empty())
            .map(|(dive, photos)| CollectionGroup {
                collection: matching::collection_name(dive),
                dive: DiveTags::new(dive, units, source),
                location: LocationTags::new(&dive.site, None, None),
                photos,
            })
            .collect();
        Ok(CollectionsRequest {
            search: self.search,
            parent: Vec::new(),
            group: None,
            groups,
        })
    }

    /// Offsets to remember after a successful write: those of the cameras
    /// with at least one photo written (the offset was applied to them).
    pub fn remember(&self, choices: &Choices, offsets: &mut Offsets) {
        let mut written = vec![false; self.cameras.len()];
        for (p, _, placement) in self.placements(choices) {
            if placement.is_some() {
                written[p.camera] = true;
            }
        }
        for (i, camera) in self.cameras.iter().enumerate() {
            if written[i] {
                offsets
                    .cameras
                    .insert(camera.key.clone(), self.offset(choices, i));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use divelens_core::Site;

    fn at(d: u32, h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2025, 12, d)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    fn dive(id: &str, number: u32, start: NaiveDateTime, minutes: i64, has_profile: bool) -> Dive {
        Dive {
            id: id.into(),
            number: Some(number),
            start,
            end: start + Duration::minutes(minutes),
            duration_s: (minutes * 60) as u32,
            max_depth_m: Some(20.0),
            temp_high_c: Some(28.0),
            temp_low_c: Some(27.0),
            rating: None,
            site: Site {
                name: Some(format!("Site {number}")),
                ..Site::default()
            },
            has_profile,
            invalid: false,
            notes: None,
        }
    }

    fn photo(id: &str, time: &str, model: Option<&str>, serial: Option<&str>) -> FoundPhoto {
        FoundPhoto {
            id: id.into(),
            time: Some(time.into()),
            model: model.map(Into::into),
            serial: serial.map(Into::into),
        }
    }

    /// Two dives in fictional waters; a TG-7 on French time (4 h behind), a
    /// GoPro on dive time, a phone, and a photo without capture time.
    fn session(offsets: &Offsets) -> Session {
        let dives = vec![
            dive("B", 702, at(31, 8, 30), 50, false),
            dive("A", 701, at(30, 9, 0), 50, true),
        ];
        let found = vec![
            photo("1", "2025-12-30T05:05:00", Some("TG-7"), Some("A1")),
            photo("2", "2025-12-30T05:40:00", Some("TG-7"), Some("A1")),
            photo("3", "2025-12-31T04:40:00", Some("TG-7"), Some("A1")),
            photo("4", "2025-12-30T09:20:00", Some("GoPro"), None),
            photo("5", "2025-12-30T08:55:00", Some("GoPro"), None),
            photo("6", "2025-12-30T12:00:00", Some("iPhone"), Some("Z9")),
            FoundPhoto {
                id: "7".into(),
                time: None,
                model: Some("TG-7".into()),
                serial: Some("A1".into()),
            },
        ];
        Session::new(9, dives, found, offsets)
    }

    fn choices(margin_min: u32) -> Choices {
        Choices {
            offsets: HashMap::from([("TG-7|A1".into(), 240)]),
            ignored: BTreeSet::from(["iPhone|Z9".to_string()]),
            margin_min,
        }
    }

    #[test]
    fn cameras_are_sorted_by_name_and_remembered() {
        let mut offsets = Offsets::default();
        offsets.cameras.insert("TG-7|A1".into(), 241);
        let s = session(&offsets);
        let keys: Vec<_> = s.cameras.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["GoPro|", "iPhone|Z9", "TG-7|A1"]);
        assert_eq!(s.cameras[2].photos, 3);
        assert_eq!(s.cameras[2].remembered, Some(241));
        assert_eq!(s.undated, 1);
        assert_eq!(s.dives[0].id, "A", "dives by start");
    }

    #[test]
    fn live_counts_follow_offsets_margin_and_ignored_cameras() {
        let s = session(&Offsets::default());
        let p = s.preview(&choices(0));
        // Dive A: TG-7 1 and 2 (+4 h), GoPro 4; GoPro 5 is 5 min early.
        assert_eq!(
            p.dives[0],
            DiveCount {
                id: "A".into(),
                photos: 3,
                in_dive: 3,
                cameras: 2
            }
        );
        assert_eq!(
            p.dives[1],
            DiveCount {
                id: "B".into(),
                photos: 1,
                in_dive: 1,
                cameras: 1
            }
        );
        assert_eq!((p.total, p.outside, p.ignored), (4, 1, 1));
        assert_eq!(
            p.cameras[0],
            CameraCount {
                key: "GoPro|".into(),
                in_dives: 1,
                outside: 1
            }
        );

        // A 10-minute margin brings the early GoPro photo in, at the surface.
        let p = s.preview(&choices(10));
        assert_eq!((p.dives[0].photos, p.dives[0].in_dive), (4, 3));
        assert_eq!((p.total, p.outside), (5, 0));

        // Without the TG-7 offset, its photos go nowhere.
        let mut c = choices(0);
        c.offsets.clear();
        assert_eq!(s.preview(&c).total, 1);
    }

    #[test]
    fn suggests_each_cameras_offset() {
        let s = session(&Offsets::default());
        let suggested = s.suggest(&["TG-7|A1".into(), "Nikon|".into()]);
        assert_eq!(suggested["TG-7|A1"], Some(240));
        assert_eq!(suggested["Nikon|"], None, "no photo");
    }

    #[test]
    fn request_groups_photos_by_dive_with_per_photo_values() {
        let s = session(&Offsets::default());
        let mut asked = Vec::new();
        let request = s
            .request(&choices(10), Units::default(), "Subsurface", |d| {
                asked.push(d.id.clone());
                Ok(vec![
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
                ])
            })
            .unwrap();
        assert_eq!(
            asked,
            ["A"],
            "only dives with a profile and photos in the dive"
        );
        assert_eq!(request.search, 9);
        let names: Vec<_> = request
            .groups
            .iter()
            .map(|g| g.collection.as_str())
            .collect();
        assert_eq!(names, ["701 - Site 701", "702 - Site 702"]);
        assert!(request.groups.iter().all(|g| g.dive.source == "Subsurface"));
        let a = &request.groups[0];
        let by_id = |id: &str| a.photos.iter().find(|p| p.id == id).unwrap();
        // TG-7 photo 1: 05:05 + 4 h = 09:05, 300 s into the dive.
        assert_eq!(by_id("1").photo_depth.as_deref(), Some("3.0 m"));
        assert_eq!(by_id("1").photo_elapsed.as_deref(), Some("05:00"));
        assert_eq!(by_id("1").clock_offset, "+04:00");
        // GoPro photo 5, at the surface: no per-photo values.
        assert_eq!(by_id("5").photo_depth, None);
        assert_eq!(by_id("5").clock_offset, "+00:00");
        // Dive B has no profile: photo there, fields empty.
        assert_eq!(request.groups[1].photos[0].photo_depth, None);
        assert!(!a.photos.iter().any(|p| p.id == "6"), "ignored camera");
    }

    fn log(kind: crate::settings::SourceKind, path: &str) -> LogId {
        LogId {
            kind,
            path: path.into(),
        }
    }

    fn ramp(depth_m: f64) -> Vec<Sample> {
        vec![
            Sample {
                time_s: 0,
                depth_m: 0.0,
                temp_c: None,
            },
            Sample {
                time_s: 3000,
                depth_m,
                temp_c: None,
            },
        ]
    }

    #[test]
    fn apply_writes_the_profiles_and_application_read_with_the_dives() {
        use crate::settings::SourceKind;
        let read = log(SourceKind::Subsurface, "/Logs/diver.xml");
        let s = session(&Offsets::default()).read_from(
            read.clone(),
            HashMap::from([("A".to_string(), Ok(ramp(30.0)))]),
        );
        let request = s
            .write_request(&read, &choices(10), Units::default())
            .expect("same log")
            .unwrap();
        // The profile read with the dives: photo 1, 300 s in, 3.0 m.
        let a = &request.groups[0];
        let photo = a.photos.iter().find(|p| p.id == "1").unwrap();
        assert_eq!(photo.photo_depth.as_deref(), Some("3.0 m"));
        assert!(
            request.groups.iter().all(|g| g.dive.source == "Subsurface"),
            "the application the dives were read from"
        );
    }

    #[test]
    fn apply_is_refused_when_the_log_changed_since_the_dives_were_read() {
        use crate::settings::SourceKind;
        let read = log(SourceKind::Subsurface, "/Logs/diver.xml");
        let s = session(&Offsets::default()).read_from(
            read.clone(),
            HashMap::from([("A".to_string(), Ok(ramp(30.0)))]),
        );
        let elsewhere = log(SourceKind::Subsurface, "/Logs/other.xml");
        assert!(matches!(
            s.write_request(&elsewhere, &choices(10), Units::default()),
            Err(LogChanged)
        ));
        let other_app = log(SourceKind::MacDive, "/Logs/diver.xml");
        assert!(matches!(
            s.write_request(&other_app, &choices(10), Units::default()),
            Err(LogChanged)
        ));
        // Dives of unknown origin are never written.
        let unknown = session(&Offsets::default());
        assert!(matches!(
            unknown.write_request(&read, &choices(10), Units::default()),
            Err(LogChanged)
        ));
    }

    #[test]
    fn a_profile_that_did_not_read_fails_apply_only_when_needed() {
        use crate::settings::SourceKind;
        let read = log(SourceKind::MacDive, "/Dives/MacDive.sqlite");
        let s = session(&Offsets::default()).read_from(
            read.clone(),
            HashMap::from([("A".to_string(), Err(Error::BadProfile("garbled".into())))]),
        );
        // Dive A gets photos taken during the dive: its profile is needed.
        assert!(matches!(
            s.write_request(&read, &choices(0), Units::default()),
            Ok(Err(Error::BadProfile(_)))
        ));
        // Only a photo at the surface before dive A (GoPro 09:20 − 25 min,
        // in a 10-minute margin): its profile is not needed.
        let c = Choices {
            offsets: HashMap::from([("GoPro|".into(), -25)]),
            ignored: BTreeSet::from(["iPhone|Z9".to_string(), "TG-7|A1".to_string()]),
            margin_min: 10,
        };
        let request = s
            .write_request(&read, &c, Units::default())
            .unwrap()
            .unwrap();
        assert_eq!(request.groups.len(), 1);
        assert_eq!(request.groups[0].photos[0].id, "4");
        assert_eq!(request.groups[0].photos[0].photo_depth, None);
        assert_eq!(request.groups[0].dive.source, "MacDive");
    }

    #[test]
    fn remembers_offsets_of_cameras_with_photos_written() {
        let s = session(&Offsets::default());
        let mut offsets = Offsets::default();
        offsets.cameras.insert("Nikon|".into(), 60);
        let mut c = choices(0);
        c.offsets.insert("GoPro|".into(), 600); // puts no GoPro photo in a dive
        s.remember(&c, &mut offsets);
        assert_eq!(offsets.get("TG-7|A1"), Some(240));
        assert_eq!(offsets.get("GoPro|"), None, "nothing written");
        assert_eq!(offsets.get("iPhone|Z9"), None, "ignored");
        assert_eq!(offsets.get("Nikon|"), Some(60), "other cameras kept");
    }

    #[test]
    fn a_slower_older_search_never_replaces_the_newer_one() {
        use crate::settings::SourceKind;
        let read = log(SourceKind::Subsurface, "/Logs/diver.xml");
        let mut searches = Searches::default();
        // Many dives checked, Next: this search reads every profile, slowly.
        let older = 1_700_000_000_001;
        assert!(searches.start(older));
        // Back to step 1, only dive B kept, Next: a quicker search.
        let newer = older + 1;
        assert!(searches.start(newer));
        // Done reading its profiles, the older one does not even ask
        // Lightroom: the plug-in keeps the photos of the newer one only.
        assert!(!searches.is_latest(older));
        assert!(searches.is_latest(newer));
        let only_b = |s: Session| {
            let dives = s.dives.iter().filter(|d| d.id == "B").cloned().collect();
            let found = vec![photo("3", "2025-12-31T04:40:00", Some("TG-7"), Some("A1"))];
            Session::new(10, dives, found, &Offsets::default())
                .read_from(read.clone(), HashMap::new())
        };
        assert!(searches.finish(newer, only_b(session(&Offsets::default()))));
        // The older one finishes last: dropped, the newer one stays.
        let all = session(&Offsets::default()).read_from(read.clone(), HashMap::new());
        assert!(!searches.finish(older, all));
        assert!(
            searches.get(older).is_none(),
            "its counts and Apply are refused"
        );
        let kept = searches.get(newer).expect("the screen's search");
        let request = kept
            .write_request(&read, &choices(0), Units::default())
            .unwrap()
            .unwrap();
        let written: Vec<_> = request
            .groups
            .iter()
            .map(|g| g.collection.as_str())
            .collect();
        assert_eq!(
            written,
            ["702 - Site 702"],
            "never the dives unchecked since"
        );
    }

    #[test]
    fn a_newer_search_replaces_the_kept_one() {
        let mut searches = Searches::default();
        let first = 7;
        assert!(searches.start(first));
        assert!(searches.finish(first, session(&Offsets::default())));
        let second = 8;
        assert!(searches.start(second));
        assert!(searches.get(first).is_some(), "until the next one finishes");
        assert!(searches.finish(second, session(&Offsets::default())));
        assert!(searches.get(first).is_none());
        assert!(searches.get(second).is_some());
    }

    #[test]
    fn the_screen_sets_the_order_of_its_searches() {
        let mut searches = Searches::default();
        // The dev window runs each effect twice: the abandoned first search
        // may reach the back end after the live one, and is refused.
        assert!(searches.start(11));
        assert!(!searches.start(10), "asked for earlier");
        assert!(!searches.start(11), "the same search twice");
        assert!(searches.is_latest(11));
        assert!(!searches.finish(10, session(&Offsets::default())));
        assert!(searches.get(10).is_none());
    }

    #[test]
    fn searches_a_day_around_the_dives() {
        let dives = vec![
            dive("A", 1, at(30, 23, 0), 50, false),
            dive("B", 2, at(31, 8, 0), 50, false),
        ];
        assert_eq!(
            search_days(&dives),
            Some(("2025-12-29".into(), "2026-01-01".into()))
        );
        assert_eq!(search_days(&[]), None);
    }
}
