//! "Tag Selection" (spec F8): the commands the screen calls. The photos
//! selected in Lightroom and their per-photo values are in `selection`; what
//! a choice writes, and which refusal wins, in `rules`.

mod rules;
pub(crate) mod selection;

use std::sync::Mutex;
use std::sync::mpsc;

use divelens_core::Error;
use divelens_core::tags::TagRequest;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::apply::errors::CollectionsError;
use crate::apply::offsets::Offsets;
use crate::apply::searches::{Searches, keep_search};
use crate::apply::send;
use crate::dive_log::{LogChanged, LogId, SourceInfo, Writes, open, source_info};
use crate::files::config_dir;
use crate::lightroom::events::Lightroom;
use crate::lightroom::{ApplyOutcome, Command};
use crate::settings::{Settings, units_in_use};
use selection::{DiveSpan, Selection, SelectionPhotos, TagPreview};

/// Where the location written by Tag Selection comes from.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum LocationChoice {
    /// The chosen dive's site.
    DiveSite,
    /// Another site of the log, with coordinates possibly edited.
    #[serde(rename_all = "camelCase")]
    Site {
        site_id: String,
        lat: Option<f64>,
        lon: Option<f64>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagChoice {
    /// Write this dive's data; `None`: dive data unticked.
    dive_id: Option<String>,
    /// Write a location; `None`: location unticked.
    location: Option<LocationChoice>,
    /// The description of the selection the screen shows
    /// (`find_selection_photos`): needed when the dive has a profile, for
    /// the per-photo values. `None` for such a dive: the screen could not
    /// read its profile, the dive data are written alone.
    #[serde(default)]
    search: Option<u64>,
    /// The clock offsets shown, in minutes by camera key.
    #[serde(default)]
    offsets: std::collections::HashMap<String, i32>,
}

/// Why Tag Selection wrote nothing before reaching Lightroom, or lost track.
#[derive(Debug, Serialize)]
#[serde(tag = "error", rename_all = "camelCase")]
pub(crate) enum TagError {
    NothingChosen,
    NotConnected,
    /// The link dropped while Lightroom was writing: the result is unknown.
    LinkLost,
    BadGps,
    UnknownDive,
    UnknownSite,
    /// The screen shows another dive log than the configured one.
    LogChanged,
    /// The dive has a profile but the selection the screen shows is no
    /// longer kept (a newer description replaced it, the link restarted):
    /// it must be described again.
    NoPhotos,
    Source {
        detail: Error,
    },
}

impl From<LogChanged> for TagError {
    fn from(_: LogChanged) -> Self {
        Self::LogChanged
    }
}

/// Tag Selection's descriptions of the Lightroom selection: the latest one
/// only is kept (its own, never Create Collections' photo search).
#[derive(Default)]
pub(crate) struct TagState {
    selections: Mutex<Searches<Selection>>,
}

/// What to write, from `info`, the dive log checked for this write, and the
/// offset applied to each camera, to remember once written.
fn tag_request(
    app: &AppHandle,
    info: &SourceInfo,
    choice: &TagChoice,
) -> Result<(TagRequest, std::collections::BTreeMap<String, i32>), TagError> {
    let source = open(app, info)
        .map_err(|detail| TagError::Source { detail })?
        .log;
    let units = units_in_use(app, info);
    let dive = match &choice.dive_id {
        Some(id) => {
            let dives = source
                .dives()
                .map_err(|detail| TagError::Source { detail })?;
            Some(rules::find_dive(dives, id)?)
        }
        None => None,
    };
    let location = rules::location(choice.location.as_ref(), dive.as_ref(), || source.sites())?;
    let mut request = rules::base_request(dive.as_ref(), location, units, info.app.name)?;
    let mut applied = std::collections::BTreeMap::new();
    // Per-photo values (spec F8): when the dive has a profile, on the photos
    // of the selection shown taken during the dive, with the offsets shown.
    // A profile that cannot be read leaves the dive data alone.
    if let Some(dive) = dive.filter(|d| d.has_profile) {
        let described = rules::described(
            &app.state::<TagState>().selections,
            choice.search,
            &dive,
            &choice.offsets,
        )?;
        // The profile of the configured log, read now; only when needed.
        let outcome = selection::per_photo(&dive, described, units, || source.profile(&dive.id));
        let (offsets, unreadable) = rules::apply_per_photo(&mut request, outcome, || {
            Settings::load(&config_dir(app)).depth_to_altitude
        })?;
        if let Some(e) = unreadable {
            log::warn!(
                "profile of dive {} not read, dive data without per-photo values: {e:?}",
                dive.id
            );
        }
        applied = offsets;
    }
    Ok((request, applied))
}

/// Tags the photos selected in Lightroom (at the moment Lightroom gets the
/// request) with a dive's data and/or a location, in one undoable step.
/// `log` is the dive log the screen shows: refused when it changed. When
/// the dive has a profile, the photos taken during it also get their
/// per-photo values and clock offset, computed for the selection the
/// screen shows; the plugin refuses if the selection changed since. The
/// offsets applied are then remembered, as Create Collections does (also
/// when only the GPS and altitude could not be written).
#[tauri::command]
pub(crate) async fn tag_selection(
    app: AppHandle,
    log: LogId,
    choice: TagChoice,
) -> Result<ApplyOutcome, TagError> {
    tauri::async_runtime::spawn_blocking(move || {
        let writes = app.state::<Writes>();
        let (_writing, info) = writes.begin(&log, || source_info(&app))?;
        let (request, applied) = tag_request(&app, &info, &choice)?;
        let with_values = request.photos.len();
        let state = app.state::<Lightroom>();
        if !state.status.lock().expect("lightroom state").connected {
            return Err(TagError::NotConnected);
        }
        let (reply, outcome) = mpsc::channel();
        let sent = state
            .commands
            .lock()
            .expect("lightroom commands")
            .send(Command::Apply {
                request: Box::new(request),
                reply,
            });
        if sent.is_err() {
            return Err(TagError::NotConnected);
        }
        let outcome = outcome.recv().map_err(|_| TagError::LinkLost)?;
        if rules::remember_offsets(&outcome, &applied) {
            let dir = config_dir(&app);
            let mut offsets = Offsets::load(&dir, info.app);
            selection::remember(&applied, &mut offsets);
            if let Err(e) = offsets.save(&dir, info.app) {
                log::error!("clock offsets not saved: {e}");
            }
        }
        match &outcome {
            ApplyOutcome::Applied { count } => {
                log::info!("selection tagged: {count} photos, per-photo values on {with_values}")
            }
            ApplyOutcome::Failed { .. } => log::warn!("selection not tagged: {outcome:?}"),
        }
        Ok(outcome)
    })
    .await
    .expect("tagging task failed")
}

/// Describes the photos selected in Lightroom now, for Tag Selection's
/// per-photo values: their cameras, each with the offset last applied to it
/// for the application of `log`, the dive log the screen shows. `search`
/// numbers the description, higher than any the screen asked for before:
/// only the latest one is kept.
#[tauri::command]
pub(crate) async fn find_selection_photos(
    app: AppHandle,
    log: LogId,
    search: u64,
) -> Result<SelectionPhotos, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<TagState>();
        if !state.selections.lock().expect("selections").start(search) {
            return Err(CollectionsError::NoPhotos);
        }
        let info = source_info(&app);
        log.check(&info)?;
        // The plugin keeps the photos of its last description only: checked
        // and sent under the lock, so a newer one cannot reach it first.
        let (reply, answer) = mpsc::channel();
        {
            let selections = state.selections.lock().expect("selections");
            if !selections.is_latest(search) {
                return Err(CollectionsError::NoPhotos);
            }
            send(&app, Command::DescribeSelection { reply })?;
        }
        let found = answer.recv().map_err(|_| CollectionsError::LinkLost)??;
        let offsets = Offsets::load(&config_dir(&app), info.app);
        let selection = Selection::new(found.search, found.photos, &offsets);
        let shown = selection.shown(search);
        keep_search(
            &state.selections,
            search,
            &log,
            &source_info(&app),
            selection,
        )?;
        Ok(shown)
    })
    .await
    .expect("selection task failed")
}

/// Live counts of selection description `search` for the dive shown (none:
/// not chosen yet) and the offsets shown.
#[tauri::command]
pub(crate) fn tag_preview(
    state: State<TagState>,
    search: u64,
    dive: Option<DiveSpan>,
    offsets: std::collections::HashMap<String, i32>,
) -> Result<TagPreview, CollectionsError> {
    let selections = state.selections.lock().expect("selections");
    let selection = selections.get(search).ok_or(CollectionsError::NoPhotos)?;
    let span = dive.as_ref().and_then(DiveSpan::span);
    Ok(selection.preview(span, &offsets))
}

/// Suggested clock offset of the given cameras of selection description
/// `search`, for the dive shown (`None`: no offset fits).
#[tauri::command]
pub(crate) async fn suggest_tag_offsets(
    app: AppHandle,
    search: u64,
    dive: DiveSpan,
    cameras: Vec<String>,
) -> Result<std::collections::HashMap<String, Option<i32>>, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<TagState>();
        let selections = state.selections.lock().expect("selections");
        let selection = selections.get(search).ok_or(CollectionsError::NoPhotos)?;
        let span = dive.span().ok_or(CollectionsError::NothingChosen)?;
        Ok(selection.suggest(span, &cameras))
    })
    .await
    .expect("suggestion task failed")
}
