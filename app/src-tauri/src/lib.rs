mod applog;
mod apply;
mod collections;
mod dive_log;
mod files;
mod lightroom;
mod selection;
mod settings;
mod update;

use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use divelens_core::Error;
use divelens_core::apps::Kept;
use divelens_core::collections::Span;
use divelens_core::macdive;
use divelens_core::tags::{DiveTags, LocationTags, TagRequest, valid_gps};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use apply::errors::CollectionsError;
use apply::offsets::{self, Offsets};
use apply::searches::{Searches, keep_search};
use apply::send;
use dive_log::{LogChanged, LogId, SourceInfo, Writes, open, source_info, standard_path};
use files::config_dir;
use lightroom::{ApplyOutcome, Command, End, Event, LinkFile, Status, Timing};
use selection::{DiveSpan, PerPhoto, Selection, SelectionPhotos, TagPreview};
use settings::{LogFile, Settings, change_settings, units_in_use};

/// Tells the plugin what to write in its log (dropped when not connected:
/// the level is sent again on connection).
fn send_log_level(app: &AppHandle, level: settings::LogLevel) {
    if let Some(state) = app.try_state::<Lightroom>() {
        let _ = state
            .commands
            .lock()
            .expect("lightroom commands")
            .send(Command::SetLogLevel(level));
    }
}

/// Last known state of the Lightroom link, and the way to send it requests.
struct Lightroom {
    status: Mutex<Status>,
    commands: Mutex<Sender<Command>>,
    /// Thumbnails of the current selection, as data URLs, by position.
    preview: Mutex<SelectionPreview>,
}

#[derive(Debug, Clone, Default, Serialize)]
struct SelectionPreview {
    version: u64,
    thumbnails: Vec<Option<String>>,
}

/// Thumbnails received so far for the current selection; new ones arrive as
/// `selection-preview` events.
#[tauri::command]
fn selection_preview(state: State<Lightroom>) -> SelectionPreview {
    state.preview.lock().expect("preview").clone()
}

/// Current Lightroom link state; updates arrive as `lightroom-status` events.
#[tauri::command]
fn lightroom_status(state: State<Lightroom>) -> Status {
    state.status.lock().expect("lightroom state").clone()
}

/// Whether Lightroom is being written to (the dive log cannot change);
/// changes arrive as `lightroom-writing` events.
#[tauri::command]
fn lightroom_writing(writes: State<Writes>) -> bool {
    writes.in_progress()
}

/// Where the location written by Tag Selection comes from.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum LocationChoice {
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
struct TagChoice {
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
enum TagError {
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
struct TagState {
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
        Some(id) => Some(
            source
                .dives()
                .map_err(|detail| TagError::Source { detail })?
                .into_iter()
                .find(|d| &d.id == id)
                .ok_or(TagError::UnknownDive)?,
        ),
        None => None,
    };
    let location = match &choice.location {
        None => None,
        Some(LocationChoice::DiveSite) => {
            let dive = dive.as_ref().ok_or(TagError::NothingChosen)?;
            Some(LocationTags::new(&dive.site, None, None))
        }
        Some(LocationChoice::Site { site_id, lat, lon }) => {
            match (lat, lon) {
                (Some(a), Some(b)) if !valid_gps(*a, *b) => return Err(TagError::BadGps),
                (Some(_), None) | (None, Some(_)) => return Err(TagError::BadGps),
                _ => {}
            }
            let site = source
                .sites()
                .map_err(|detail| TagError::Source { detail })?
                .into_iter()
                .find(|s| s.id.as_deref() == Some(site_id.as_str()))
                .ok_or(TagError::UnknownSite)?;
            Some(LocationTags::new(&site, *lat, *lon))
        }
    };
    if dive.is_none() && location.is_none() {
        return Err(TagError::NothingChosen);
    }
    let mut request = TagRequest {
        dive: dive
            .as_ref()
            .map(|d| DiveTags::new(d, units, info.app.name)),
        location,
        ..TagRequest::default()
    };
    let mut applied = std::collections::BTreeMap::new();
    // Per-photo values (spec F8): when the dive has a profile, on the photos
    // of the selection shown taken during the dive, with the offsets shown.
    // A profile that cannot be read leaves the dive data alone.
    if let Some(dive) = dive.filter(|d| d.has_profile) {
        let described = match choice.search {
            None => None,
            Some(n) => {
                let state = app.state::<TagState>();
                let selections = state.selections.lock().expect("selections");
                let selection = selections.get(n).ok_or(TagError::NoPhotos)?;
                Some((
                    selection.described,
                    selection.in_dive(Span::from(&dive), &choice.offsets),
                ))
            }
        };
        // The profile of the configured log, read now; only when needed.
        match selection::per_photo(&dive, described, units, || source.profile(&dive.id)) {
            Ok(PerPhoto::Values {
                described,
                photos,
                applied: offsets,
            }) => {
                request.selection = Some(described);
                request.photos = photos;
                if Settings::load(&config_dir(app)).depth_to_altitude {
                    request.copy_depth_to_altitude();
                }
                applied = offsets;
            }
            Ok(PerPhoto::Unreadable(e)) => log::warn!(
                "profile of dive {} not read, dive data without per-photo values: {e:?}",
                dive.id
            ),
            Err(selection::Refusal::Describe) => return Err(TagError::NoPhotos),
            Err(selection::Refusal::Profile(detail)) => return Err(TagError::Source { detail }),
        }
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
async fn tag_selection(
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
        let written = matches!(outcome, ApplyOutcome::Applied { .. }) || outcome.half_written();
        if written && !applied.is_empty() {
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
async fn find_selection_photos(
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
fn tag_preview(
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
async fn suggest_tag_offsets(
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

/// Screen asked for on the command line (`--screen tag`), when Lightroom
/// launched the app from one of its menus.
#[tauri::command]
fn initial_screen() -> Option<String> {
    screen_arg(std::env::args())
}

fn screen_arg(args: impl IntoIterator<Item = String>) -> Option<String> {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--screen" {
            return args.next();
        }
    }
    None
}

fn bring_to_front(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn show_screen(app: &AppHandle, screen: String) {
    bring_to_front(app);
    let _ = app.emit("show-screen", screen);
}

/// Shows `status` when it differs from the one shown.
fn show_status(app: &AppHandle, status: Status) {
    let state = app.state::<Lightroom>();
    let mut shown = state.status.lock().expect("lightroom state");
    if *shown != status {
        *shown = status.clone();
        drop(shown);
        let _ = app.emit("lightroom-status", status);
    }
}

/// Keeps trying to reach the plugin for as long as the app runs: whichever of
/// Lightroom and the app starts first, they find each other.
fn lightroom_link(app: AppHandle, commands: mpsc::Receiver<Command>) {
    let dir = config_dir(&app);
    let version = app.package_info().version.to_string();
    let mut last_end = None;
    loop {
        // Requests made while there is no link are dropped, so whoever sent
        // one stops waiting (and a write in progress ends) even when
        // Lightroom quit and its link file is gone.
        lightroom::drop_requests(&commands);
        if let Some(link) = LinkFile::read(&dir) {
            let end = lightroom::run_session(
                &link,
                &version,
                Timing::default(),
                &commands,
                &mut |event| match event {
                    Event::Status(mut status) => {
                        let was_connected = app
                            .state::<Lightroom>()
                            .status
                            .lock()
                            .expect("lightroom state")
                            .connected;
                        if status.connected && !was_connected {
                            log::info!(
                                "Lightroom {} connected, plugin {}, catalog {}",
                                status.lightroom_version.as_deref().unwrap_or("?"),
                                status.plugin_version.as_deref().unwrap_or("?"),
                                status.catalog.as_deref().unwrap_or("?")
                            );
                            send_log_level(&app, Settings::load(&dir).log_level);
                        }
                        let ranges: Vec<_> = status
                            .selection_cameras
                            .iter()
                            .map(|c| {
                                let key =
                                    offsets::camera_key(c.model.as_deref(), c.serial.as_deref());
                                (key, c.from.clone(), c.to.clone())
                            })
                            .collect();
                        status.restart_lightroom = status.other_version(&version);
                        let dive_app = source_info(&app).app;
                        let cover =
                            offsets::corrected_range(&Offsets::load(&dir, dive_app), &ranges);
                        (status.cover_from, status.cover_to) = cover.unzip();
                        *app.state::<Lightroom>()
                            .status
                            .lock()
                            .expect("lightroom state") = status.clone();
                        let _ = app.emit("lightroom-status", status);
                    }
                    Event::Show(screen) => show_screen(&app, screen),
                    Event::Legacy(legacy) => {
                        // Refused during a write when it would change the
                        // dive log: nothing is saved, not even that it was
                        // looked at, so it is tried again at the next
                        // connection.
                        // The previous plugin only read MacDive.
                        if let Ok(home) = app.path().home_dir() {
                            let standard = standard_path(&app, &macdive::APP);
                            let taken =
                                change_settings(&app, |s| s.take_over(&legacy, &home, &standard));
                            if let Err(e) = taken {
                                log::error!("previous plugin's settings not taken over: {e:?}");
                            }
                        }
                    }
                    Event::Progress { done, total } => {
                        let _ = app.emit("apply-progress", (done, total));
                    }
                    Event::Thumbnail {
                        version,
                        index,
                        jpeg,
                    } => {
                        let state = app.state::<Lightroom>();
                        let mut preview = state.preview.lock().expect("preview");
                        if preview.version != version {
                            *preview = SelectionPreview {
                                version,
                                thumbnails: Vec::new(),
                            };
                        }
                        let slot = index.saturating_sub(1).min(31);
                        if preview.thumbnails.len() <= slot {
                            preview.thumbnails.resize(slot + 1, None);
                        }
                        preview.thumbnails[slot] = Some(format!("data:image/jpeg;base64,{jpeg}"));
                        let _ = app.emit("selection-preview", preview.clone());
                    }
                },
            );
            // A plug-in from before an update, still running in Lightroom.
            if let End::Protocol(_) = end {
                show_status(&app, Status::other_protocol(&link));
            }
            // Log changes only: a stale link file fails every second.
            let text = format!("{end:?}");
            if last_end.as_ref() != Some(&text) {
                if end != End::Unreachable {
                    log::info!("Lightroom link ended: {text}");
                }
                last_end = Some(text);
            }
        } else {
            // Lightroom quit: nothing left to restart.
            show_status(&app, Status::default());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // First: a second launch (Lightroom menu while the app is open but not
        // yet linked) brings this window forward on the requested screen.
        .plugin(tauri_plugin_single_instance::init(
            |app, args, _cwd| match screen_arg(args) {
                Some(screen) => show_screen(app, screen),
                None => bring_to_front(app),
            },
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings::dive_apps,
            settings::app_info,
            settings::get_settings,
            settings::update_settings,
            settings::export_settings,
            settings::check_settings_file,
            settings::import_settings,
            dive_log::load_dives,
            dive_log::dive_profile,
            dive_log::set_source_path,
            lightroom_status,
            lightroom_writing,
            initial_screen,
            tag_selection,
            find_selection_photos,
            tag_preview,
            suggest_tag_offsets,
            selection_preview,
            collections::find_collection_photos,
            collections::collections_preview,
            collections::suggest_offsets,
            collections::apply_collections,
            collections::collection_sets,
            update::update_status,
            update::check_update,
            update::dismiss_update,
            update::install_update
        ])
        .setup(|app| {
            let settings = Settings::load(&config_dir(app.handle()));
            let version = app.package_info().version.to_string();
            let log_path = app.path().home_dir().ok().map(|home| {
                let path = applog::start(applog::folder(&home), settings.log_level);
                log::info!(
                    "Dive Lens {} on {}, log {}",
                    version,
                    std::env::consts::OS,
                    path.display()
                );
                path
            });
            app.manage(LogFile(log_path));
            let (commands, inbox) = mpsc::channel();
            app.manage(Lightroom {
                status: Mutex::new(Status::default()),
                commands: Mutex::new(commands),
                preview: Mutex::new(SelectionPreview::default()),
            });
            app.manage(collections::CollectionsState::default());
            app.manage(TagState::default());
            app.manage(Kept::default());
            let copy = std::env::current_exe()
                .map(|exe| update::Copy::locate(&exe))
                .unwrap_or(update::Copy::Development);
            log::info!("update: this copy is {copy:?}");
            if let Some(folder) = copy.folder()
                && std::env::args().any(|a| a == update::AFTER_UPDATE)
            {
                update::clean_up(folder);
            }
            app.manage(update::Updates {
                copy,
                phase: Mutex::new(update::Phase::default()),
                found: Mutex::new(None),
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move { update::check_update(handle, true).await });
            // Writes are told to the UI from a thread of their own, in order:
            // never from under the lock that settings changes wait on.
            let (writing, told) = mpsc::channel::<bool>();
            app.manage(Writes::new(move |on| {
                let _ = writing.send(on);
            }));
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("lightroom-writing".into())
                .spawn(move || {
                    for on in told {
                        let _ = handle.emit("lightroom-writing", on);
                    }
                })?;
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("lightroom-link".into())
                .spawn(move || lightroom_link(handle, inbox))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running DiveLens");
}

#[cfg(test)]
mod tests {
    use super::screen_arg;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reads_screen_argument() {
        assert_eq!(
            screen_arg(args(&["DiveLens", "--screen", "tag"])),
            Some("tag".into())
        );
        assert_eq!(screen_arg(args(&["DiveLens"])), None);
        assert_eq!(screen_arg(args(&["DiveLens", "--screen"])), None);
        // macOS may add its own arguments before ours.
        assert_eq!(
            screen_arg(args(&[
                "DiveLens",
                "-psn_0_1234",
                "--screen",
                "collections"
            ])),
            Some("collections".into())
        );
    }
}
