//! "Create Collections" (spec F5): the commands the screen calls. The plan
//! (photos found, live counts, what to write) is in `plan`.

pub(crate) mod plan;

use std::sync::{Mutex, mpsc};

use divelens_core::Dive;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::apply::errors::ApplyError;
use crate::apply::offsets::Offsets;
use crate::apply::searches::{Searches, keep_search};
use crate::apply::{ask, send, write_and_remember};
use crate::dive_log::{LogId, Writes, open, source_info};
use crate::files::config_dir;
use crate::lightroom::{CollectionSet, Command, WrittenCollection};
use crate::settings::{Settings, units_in_use};
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
#[tauri::command]
pub(crate) async fn apply_collections(
    app: AppHandle,
    log: LogId,
    search: u64,
    choices: Choices,
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
            if settings.depth_to_altitude {
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
        if request.groups.is_empty() {
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
            if matches!(e, ApplyError::NotConnected | ApplyError::LinkLost) {
                log::warn!("collections not written: {e:?}");
            }
        })?;
        log::info!(
            "collections written: {} collections, {} photos ({with_depth} with depth)",
            written.len(),
            written.iter().map(|c| c.count).sum::<usize>()
        );
        Ok(CollectionsResult {
            photos: written.iter().map(|c| c.count).sum(),
            collections: written,
            with_depth,
            empty_dives,
        })
    })
    .await
    .expect("collections task failed")
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
