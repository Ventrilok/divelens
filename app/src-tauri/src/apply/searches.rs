use std::collections::HashMap;
use std::sync::Mutex;

use chrono::NaiveDateTime;
use divelens_core::collections::{self as matching, Span};
use serde::Serialize;

use super::errors::CollectionsError;
use super::offsets::{Offsets, camera_key, parse_time};
use crate::dive_log::{LogId, SourceInfo};
use crate::lightroom::FoundPhoto;

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

/// A photo with a capture time, and the index of its camera.
pub(crate) struct Photo {
    pub id: String,
    pub time: NaiveDateTime,
    pub camera: usize,
}

/// Photos as Lightroom described them, with their cameras sorted by name
/// (spec F5), each with the offset last applied to it in `offsets`. Photos
/// without a capture time are only counted.
pub(crate) struct Shots {
    pub photos: Vec<Photo>,
    pub cameras: Vec<Camera>,
    pub undated: usize,
}

impl Shots {
    pub fn new(found: Vec<FoundPhoto>, offsets: &Offsets) -> Self {
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
            photos,
            cameras,
            undated,
        }
    }

    pub fn offset(&self, offsets: &HashMap<String, i32>, camera: usize) -> i32 {
        offset_of(&self.cameras, offsets, camera)
    }

    pub fn suggest(&self, spans: &[Span], keys: &[String]) -> HashMap<String, Option<i32>> {
        suggest_for(&self.photos, &self.cameras, spans, keys)
    }
}

/// Offset of camera `camera` in `offsets` (minutes by camera key; a missing
/// camera has none), within ±24 h.
pub(crate) fn offset_of(cameras: &[Camera], offsets: &HashMap<String, i32>, camera: usize) -> i32 {
    offsets
        .get(&cameras[camera].key)
        .copied()
        .unwrap_or(0)
        .clamp(-matching::MAX_OFFSET_MIN, matching::MAX_OFFSET_MIN)
}

/// Suggested offset of each camera `keys` names, for `spans` (spec F5
/// "Suggest"; `None`: no offset puts a photo of it in a dive).
pub(crate) fn suggest_for(
    photos: &[Photo],
    cameras: &[Camera],
    spans: &[Span],
    keys: &[String],
) -> HashMap<String, Option<i32>> {
    keys.iter()
        .map(|key| {
            let times: Vec<NaiveDateTime> = photos
                .iter()
                .filter(|p| &cameras[p.camera].key == key)
                .map(|p| p.time)
                .collect();
            (key.clone(), matching::suggest_offset(spans, &times))
        })
        .collect()
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

/// The photo searches of "Create Collections". The screen numbers each one,
/// in the order it asks for them; only the latest one keeps its session, and
/// every later step (live counts, "Suggest", "Apply") names the search it
/// works on. A slower, older search (more dives, more profiles to read) can
/// then never replace the newer one the screen shows.
///
/// Tag Selection keeps the photos of the selection the same way, in a
/// `Searches` of its own (`S` is then [`crate::tag::selection::Selection`]).
pub struct Searches<S> {
    started: u64,
    kept: Option<(u64, S)>,
}

impl<S> Default for Searches<S> {
    fn default() -> Self {
        Self {
            started: 0,
            kept: None,
        }
    }
}

impl<S> Searches<S> {
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
    pub fn finish(&mut self, number: u64, session: S) -> bool {
        if number != self.started {
            return false;
        }
        self.kept = Some((number, session));
        true
    }

    /// The session of search `number`; `None` when another one is kept.
    pub fn get(&self, number: u64) -> Option<&S> {
        match &self.kept {
            Some((kept, session)) if *kept == number => Some(session),
            _ => None,
        }
    }
}

/// Keeps the session of photo search `number` once it is done, unless the
/// configured log changed during it (its dives belong to the previous log)
/// or a newer search started since (the screen shows that one). Tag
/// Selection keeps its descriptions of the selection the same way.
pub(crate) fn keep_search<S>(
    searches: &Mutex<Searches<S>>,
    number: u64,
    log: &LogId,
    configured: &SourceInfo,
    session: S,
) -> Result<(), CollectionsError> {
    log.check(configured)?;
    if !searches
        .lock()
        .expect("collections")
        .finish(number, session)
    {
        return Err(CollectionsError::NoPhotos);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::plan::Session;
    use crate::tag::selection::Selection;
    use divelens_core::subsurface;

    #[test]
    fn a_search_is_kept_only_for_the_log_still_configured_and_if_newest() {
        let searches = Mutex::new(Searches::default());
        let at = |path: &str| SourceInfo {
            app: &subsurface::APP,
            path: path.into(),
            is_default_path: false,
        };
        let (a, b) = (at("/Logs/a.xml"), at("/Logs/b.xml"));
        let session = || Session::new(1, Vec::new(), Vec::new(), &Offsets::default());
        let next = std::cell::Cell::new(100);
        let start = || {
            next.set(next.get() + 1);
            assert!(searches.lock().unwrap().start(next.get()));
            next.get()
        };

        // The log changed in Settings while Lightroom searched.
        let n = start();
        let kept = keep_search(&searches, n, &a.id(), &b, session());
        assert!(matches!(kept, Err(CollectionsError::LogChanged)));
        assert!(searches.lock().unwrap().get(n).is_none());

        // A newer search started meanwhile: the older one is dropped.
        let older = start();
        let newer = start();
        let kept = keep_search(&searches, older, &a.id(), &a, session());
        assert!(matches!(kept, Err(CollectionsError::NoPhotos)));
        assert!(keep_search(&searches, newer, &a.id(), &a, session()).is_ok());
        assert!(searches.lock().unwrap().get(newer).is_some());
        assert!(searches.lock().unwrap().get(older).is_none());
    }

    #[test]
    fn a_selection_is_kept_only_for_the_log_still_configured_and_if_newest() {
        let selections = Mutex::new(Searches::<Selection>::default());
        let at = |path: &str| SourceInfo {
            app: &subsurface::APP,
            path: path.into(),
            is_default_path: false,
        };
        let (a, b) = (at("/Logs/a.xml"), at("/Logs/b.xml"));
        let described = || Selection::new(3, Vec::new(), &Offsets::default());
        // The log changed in Settings while Lightroom described the
        // selection: its remembered offsets belong to the previous log.
        assert!(selections.lock().unwrap().start(1));
        let kept = keep_search(&selections, 1, &a.id(), &b, described());
        assert!(matches!(kept, Err(CollectionsError::LogChanged)));
        assert!(selections.lock().unwrap().get(1).is_none());
        // The selection changed again meanwhile: only the newer one counts,
        // for the counts, "Suggest" and "Apply".
        assert!(selections.lock().unwrap().start(2));
        assert!(selections.lock().unwrap().start(3));
        let kept = keep_search(&selections, 2, &a.id(), &a, described());
        assert!(matches!(kept, Err(CollectionsError::NoPhotos)));
        assert!(keep_search(&selections, 3, &a.id(), &a, described()).is_ok());
        assert!(selections.lock().unwrap().get(2).is_none());
        assert_eq!(selections.lock().unwrap().get(3).unwrap().described, 3);
    }
}
