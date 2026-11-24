//! "Tag Selection" (spec F8): the commands the screen calls. The photos
//! selected in Lightroom and their per-photo values are in `selection`; what
//! a choice writes, and which refusal wins, in `rules`.

mod rules;
pub(crate) mod selection;

use std::sync::Mutex;
use std::sync::mpsc;

use divelens_core::collections::collection_name;
use divelens_core::tags::{CollectionPlace, Kept, TagRequest};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::apply::errors::ApplyError;
use crate::apply::offsets::Offsets;
use crate::apply::options::WriteOptions;
use crate::apply::searches::{Searches, keep_search};
use crate::apply::{send, write_and_remember};
use crate::dive_log::{LogId, SourceInfo, Writes, open, source_info};
use crate::files::config_dir;
use crate::lightroom::{Command, WrittenCollection};
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
    /// What the diver ticked. `None` (the current screens): everything the
    /// choice names is written, the altitude follows the setting. With
    /// options, `dive_id` is the chosen dive even when its details are
    /// unticked (it gives the site and the collection's name); `None` there
    /// is "no dive".
    #[serde(default)]
    options: Option<WriteOptions>,
}

/// What "Tag Selection › Apply" wrote: the number of photos tagged, and the
/// collection the selection was put in, if one was written.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagApplied {
    count: usize,
    collection: Option<WrittenCollection>,
}

impl From<usize> for TagApplied {
    fn from(count: usize) -> Self {
        Self {
            count,
            collection: None,
        }
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
) -> Result<(TagRequest, std::collections::BTreeMap<String, i32>), ApplyError> {
    let source = open(app, info)
        .map_err(|detail| ApplyError::Source { detail })?
        .log;
    let units = units_in_use(app, info);
    let dive = match &choice.dive_id {
        Some(id) => {
            let dives = source
                .dives()
                .map_err(|detail| ApplyError::Source { detail })?;
            Some(rules::find_dive(dives, id)?)
        }
        None => None,
    };
    let location = rules::location(choice.location.as_ref(), dive.as_ref(), || source.sites())?;
    let dive_id = dive.as_ref().map(|d| d.id.clone());
    let (kept, collection, altitude_set) = match &choice.options {
        None => (Kept::ALL, None, None),
        Some(options) => {
            let collection = match (&dive, options.collections) {
                (Some(dive), true) => {
                    let settings = Settings::load(&config_dir(app));
                    Some(CollectionPlace {
                        name: collection_name(dive),
                        parent: settings.collections.parent.clone(),
                        group: settings.collections.group_name(),
                    })
                }
                _ => None,
            };
            (options.kept(), collection, Some(*options))
        }
    };
    let built = rules::build_request(
        dive,
        location,
        units,
        info.app.name,
        |dive| {
            rules::described(
                &app.state::<TagState>().selections,
                choice.search,
                dive,
                &choice.offsets,
            )
        },
        |id| source.profile(id),
        || {
            let setting = || Settings::load(&config_dir(app)).depth_to_altitude;
            match altitude_set {
                None => setting(),
                Some(options) => options.copies_altitude(setting),
            }
        },
        kept,
        collection,
    )?;
    if let Some(e) = built.unreadable {
        log::warn!(
            "profile of dive {} not read, dive data without per-photo values: {e:?}",
            dive_id.unwrap_or_default()
        );
    }
    Ok((built.request, built.applied))
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
) -> Result<TagApplied, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let writes = app.state::<Writes>();
        let (_writing, info) = writes.begin(&log, || source_info(&app))?;
        let (request, applied) = tag_request(&app, &info, &choice)?;
        let with_values = request.photos.len();
        let written = write_and_remember(
            &app,
            &info,
            |reply| Command::Apply {
                request: Box::new(request),
                reply,
            },
            &applied,
            |outcome| log::warn!("selection not tagged: {outcome:?}"),
        )?;
        let count = written.count;
        log::info!("selection tagged: {count} photos, per-photo values on {with_values}");
        Ok(TagApplied {
            count,
            collection: written.collection,
        })
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
) -> Result<SelectionPhotos, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<TagState>();
        if !state.selections.lock().expect("selections").start(search) {
            return Err(ApplyError::NoPhotos);
        }
        let info = source_info(&app);
        log.check(&info)?;
        // The plugin keeps the photos of its last description only: checked
        // and sent under the lock, so a newer one cannot reach it first.
        let (reply, answer) = mpsc::channel();
        {
            let selections = state.selections.lock().expect("selections");
            if !selections.is_latest(search) {
                return Err(ApplyError::NoPhotos);
            }
            send(&app, Command::DescribeSelection { reply })?;
        }
        let found = answer.recv().map_err(|_| ApplyError::LinkLost)??;
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
) -> Result<TagPreview, ApplyError> {
    let selections = state.selections.lock().expect("selections");
    let selection = selections.get(search).ok_or(ApplyError::NoPhotos)?;
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
) -> Result<std::collections::HashMap<String, Option<i32>>, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<TagState>();
        let selections = state.selections.lock().expect("selections");
        let selection = selections.get(search).ok_or(ApplyError::NoPhotos)?;
        let span = dive.span().ok_or(ApplyError::NothingChosen)?;
        Ok(selection.suggest(span, &cameras))
    })
    .await
    .expect("suggestion task failed")
}
