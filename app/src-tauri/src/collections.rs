//! "Create Collections" (spec F5): the commands the screen calls. The plan
//! (photos found, live counts, what to write) is in `plan`.

pub(crate) mod plan;

use std::sync::{Mutex, mpsc};

use divelens_core::Dive;
use divelens_core::tags::{CollectionsRequest, LocationTags};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::apply::errors::ApplyError;
use crate::apply::offsets::Offsets;
use crate::apply::options::WriteOptions;
use crate::apply::searches::{Searches, keep_search};
use crate::apply::{ask, send, write_and_remember};
use crate::dive_log::{LogId, Writes, open, source_info};
use crate::files::config_dir;
use crate::lightroom::{CollectionSet, Command, WrittenCollection};
use crate::settings::{Settings, units_in_use};
use crate::tag::{LocationChoice, rules};
use plan::{Choices, Period, Preview, Session};

/// Photos found for "Create Collections", kept until the next search.
#[derive(Default)]
pub(crate) struct CollectionsState {
    searches: Mutex<Searches<Session>>,
}

/// Finds in Lightroom every photo of the chosen dives' period (whole
/// catalog) and their cameras. The dives and their profiles are read now,
/// from `log`, the dive log the screen shows, and kept for "Apply".
/// `search` numbers the search, higher than any the screen asked for before.
#[tauri::command]
pub(crate) async fn find_collection_photos(
    app: AppHandle,
    log: LogId,
    search: u64,
    dive_ids: Vec<String>,
) -> Result<Period, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let number = search;
        if !state.searches.lock().expect("collections").start(number) {
            return Err(ApplyError::NoPhotos);
        }
        // A search a newer one replaced stops at once.
        let replaced = || {
            !state
                .searches
                .lock()
                .expect("collections")
                .is_latest(number)
        };
        let info = source_info(&app);
        log.check(&info)?;
        let source = open(&app, &info)
            .map_err(|detail| ApplyError::Source { detail })?
            .log;
        let dives: Vec<Dive> = source
            .dives()
            .map_err(|detail| ApplyError::Source { detail })?
            .into_iter()
            .filter(|d| dive_ids.contains(&d.id))
            .collect();
        let (from, to) = plan::search_days(&dives).ok_or(ApplyError::NothingChosen)?;
        // Which dives get photos depends on the offsets, not known yet: every
        // profile is read. One that does not read fails "Apply" only if it
        // is needed, as before.
        let mut profiles = std::collections::HashMap::new();
        for dive in dives.iter().filter(|d| d.has_profile) {
            if replaced() {
                return Err(ApplyError::NoPhotos);
            }
            profiles.insert(dive.id.clone(), source.profile(&dive.id));
        }
        // Lightroom keeps the photos of its last search only. Checked and
        // sent under the search lock: a newer search cannot start in between
        // and reach Lightroom first.
        let (reply, answer) = mpsc::channel();
        {
            let searches = state.searches.lock().expect("collections");
            if !searches.is_latest(number) {
                return Err(ApplyError::NoPhotos);
            }
            send(
                &app,
                Command::FindPhotos {
                    from: from.clone(),
                    to: to.clone(),
                    reply,
                },
            )?;
        }
        let found = answer.recv().map_err(|_| ApplyError::LinkLost)??;
        let offsets = Offsets::load(&config_dir(&app), info.app);
        let session = Session::new(found.search, dives, found.photos, &offsets)
            .read_from(log.clone(), profiles);
        let period = session.period(number, from, to);
        // Checked again once done: the log may have changed, or the user may
        // have gone back and started a newer search, during this one.
        keep_search(&state.searches, number, &log, &source_info(&app), session)?;
        Ok(period)
    })
    .await
    .expect("photo search task failed")
}

/// Live counts of photo search `search` for the current choices.
#[tauri::command]
pub(crate) fn collections_preview(
    state: State<CollectionsState>,
    search: u64,
    choices: Choices,
) -> Result<Preview, ApplyError> {
    let searches = state.searches.lock().expect("collections");
    let session = searches.get(search).ok_or(ApplyError::NoPhotos)?;
    Ok(session.preview(&choices))
}

/// Suggested clock offset of the given cameras of photo search `search`
/// (`None`: no offset fits).
#[tauri::command]
pub(crate) async fn suggest_offsets(
    app: AppHandle,
    search: u64,
    cameras: Vec<String>,
) -> Result<std::collections::HashMap<String, Option<i32>>, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let searches = state.searches.lock().expect("collections");
        let session = searches.get(search).ok_or(ApplyError::NoPhotos)?;
        Ok(session.suggest(&cameras))
    })
    .await
    .expect("suggestion task failed")
}

/// What "Create Collections › Apply" wrote.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CollectionsResult {
    /// The collections written: the groups that have a name.
    collections: Vec<WrittenCollection>,
    photos: usize,
    with_depth: usize,
    /// Chosen dives that got no photo (no collection).
    empty_dives: Vec<String>,
}

/// Writes the collections, dive data and per-photo values in one Lightroom
/// Undo step, then remembers each camera's offset (also when only the GPS
/// and altitude could not be written: the offsets are on the photos).
/// Writes the dives and profiles read with photo search `search`, the one
/// the screen shows; refused when it is no longer kept, or when `log`, the
/// dive log the screen shows, is not theirs or no longer the configured one.
/// `options`, what the diver ticked, and `other_site`, a site of the log for
/// every photo instead of each dive's own, are both absent for today's
/// screen: everything is written and the altitude follows the setting.
#[tauri::command]
pub(crate) async fn apply_collections(
    app: AppHandle,
    log: LogId,
    search: u64,
    choices: Choices,
    options: Option<WriteOptions>,
    other_site: Option<LocationChoice>,
) -> Result<CollectionsResult, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let writes = app.state::<Writes>();
        let (_writing, info) = writes.begin(&log, || source_info(&app))?;
        let units = units_in_use(&app, &info);
        let (request, empty_dives, applied) = {
            let state = app.state::<CollectionsState>();
            let searches = state.searches.lock().expect("collections");
            let session = searches.get(search).ok_or(ApplyError::NoPhotos)?;
            let mut request = session
                .write_request(&log, &choices, units)?
                .map_err(|detail| ApplyError::Source { detail })?;
            let settings = Settings::load(&config_dir(&app));
            request.parent = settings.collections.parent.clone();
            request.group = settings.collections.group_name();
            // With options, the altitude is decided below, with the rest.
            if options.is_none() && settings.depth_to_altitude {
                request.copy_depth_to_altitude();
            }
            let preview = session.preview(&choices);
            let empty: Vec<String> = preview
                .dives
                .iter()
                .filter(|d| d.photos == 0)
                .map(|d| d.id.clone())
                .collect();
            (request, empty, session.applied(&choices))
        };
        let mut request = request;
        if let Some(options) = &options {
            let other = match &other_site {
                Some(LocationChoice::Site { site_id, lat, lon }) => {
                    rules::check_gps(*lat, *lon)?;
                    let sites = open(&app, &info)
                        .and_then(|opened| opened.log.sites())
                        .map_err(|detail| ApplyError::Source { detail })?;
                    let site = rules::find_site(sites, site_id)?;
                    Some(LocationTags::new(&site, *lat, *lon))
                }
                // Each dive's own site is already on its group.
                Some(LocationChoice::DiveSite) | None => None,
            };
            with_options(&mut request, options, other.as_ref(), || {
                Settings::load(&config_dir(&app)).depth_to_altitude
            })?;
        }
        if request.is_empty() {
            return Err(ApplyError::NothingToWrite);
        }
        let with_depth = request
            .groups
            .iter()
            .flat_map(|g| &g.photos)
            .filter(|p| p.photo_depth.is_some())
            .count();
        let written = write_and_remember(
            &app,
            &info,
            |reply| Command::ApplyCollections { request, reply },
            &applied,
            |outcome| log::warn!("collections refused by Lightroom: {outcome:?}"),
        )
        .inspect_err(|e| {
            // `ask` (inside) fails only with these two; a Lightroom
            // refusal is logged by `on_refused` above.
            if matches!(e, ApplyError::NotConnected | ApplyError::LinkLost) {
                log::warn!("collections not written: {e:?}");
            }
        })?;
        let photos: usize = written.iter().map(|c| c.count).sum();
        let collections: Vec<WrittenCollection> = written
            .into_iter()
            .filter(|c| c.collection.is_some())
            .collect();
        log::info!(
            "collections written: {} collections, {photos} photos ({with_depth} with depth)",
            collections.len(),
        );
        Ok(CollectionsResult {
            photos,
            collections,
            with_depth,
            empty_dives,
        })
    })
    .await
    .expect("collections task failed")
}

/// What the diver ticked, applied to the request built as today: another
/// site on every group, what is not ticked removed, then the depth copied to
/// the altitude (only with the dive details; `setting` decides when the
/// options do not say). Refused when nothing is left to write.
fn with_options(
    request: &mut CollectionsRequest,
    options: &WriteOptions,
    other: Option<&LocationTags>,
    setting: impl FnOnce() -> bool,
) -> Result<(), ApplyError> {
    request.keep(options.kept(), other);
    if request.is_empty() {
        return Err(ApplyError::NothingToWrite);
    }
    if options.copies_altitude(setting) {
        request.copy_depth_to_altitude();
    }
    Ok(())
}

/// The catalog's collection sets (Settings › Collections).
#[tauri::command]
pub(crate) async fn collection_sets(app: AppHandle) -> Result<Vec<CollectionSet>, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(ask(&app, |reply| Command::ListSets { reply })??)
    })
    .await
    .expect("collection sets task failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::options::WriteOptions;
    use divelens_core::profile::PhotoPoint;
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    use divelens_core::{Dive, Site};

    fn dive(id: &str, site: &str) -> Dive {
        let start = chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(7, 14, 0)
            .unwrap();
        Dive {
            id: id.into(),
            number: Some(700),
            start,
            end: start + chrono::Duration::seconds(2100),
            duration_s: 2100,
            max_depth_m: Some(31.2),
            temp_high_c: Some(28.0),
            temp_low_c: None,
            rating: None,
            site: Site {
                id: Some(site.into()),
                name: Some(site.into()),
                location: None,
                country: Some("Atlantis".into()),
                lat: Some(12.5),
                lon: Some(-45.25),
            },
            has_profile: true,
            invalid: false,
            notes: None,
        }
    }

    /// Two groups, each with a photo taken at 12 m.
    fn request() -> CollectionsRequest {
        let point = PhotoPoint {
            elapsed_s: 160,
            depth_m: 12.0,
            temp_c: Some(25.0),
        };
        let group = |id: &str, site: &str| {
            let dive = dive(id, site);
            CollectionGroup {
                collection: Some(format!("700 - {site}")),
                dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
                location: Some(LocationTags::new(&dive.site, None, None)),
                photos: vec![PhotoTags::new(
                    format!("photo {id}"),
                    Some(point),
                    0,
                    Units::default(),
                )],
            }
        };
        CollectionsRequest {
            search: 3,
            parent: Vec::new(),
            group: None,
            groups: vec![group("A", "Coral Garden"), group("B", "Blue Hole")],
        }
    }

    fn options(
        dive: bool,
        altitude: Option<bool>,
        place: bool,
        gps: bool,
        collections: bool,
    ) -> WriteOptions {
        WriteOptions {
            dive,
            altitude,
            place,
            gps,
            collections,
        }
    }

    #[test]
    fn all_options_and_the_setting_give_todays_request() {
        let mut expected = request();
        expected.copy_depth_to_altitude();
        let mut got = request();
        with_options(
            &mut got,
            &options(true, None, true, true, true),
            None,
            || true,
        )
        .unwrap();
        assert_eq!(got, expected);
        // Setting off: no altitude, as today.
        let mut got = request();
        with_options(
            &mut got,
            &options(true, None, true, true, true),
            None,
            || false,
        )
        .unwrap();
        assert_eq!(got, request());
    }

    #[test]
    fn another_site_goes_on_every_group() {
        let other = LocationTags::new(&dive("C", "Shark Reef").site, None, None);
        let mut got = request();
        with_options(
            &mut got,
            &options(true, Some(false), true, true, true),
            Some(&other),
            || panic!("setting read"),
        )
        .unwrap();
        assert!(
            got.groups
                .iter()
                .all(|g| g.location.as_ref() == Some(&other))
        );
        // The collection names stay those of each dive.
        assert_eq!(
            got.groups[0].collection.as_deref(),
            Some("700 - Coral Garden")
        );
        assert_eq!(got.groups[1].collection.as_deref(), Some("700 - Blue Hole"));
    }

    #[test]
    fn nothing_ticked_is_refused() {
        let mut got = request();
        let none = options(false, Some(true), false, false, false);
        let refused = with_options(&mut got, &none, None, || panic!("setting read"));
        assert!(matches!(refused, Err(ApplyError::NothingToWrite)));
    }
}
