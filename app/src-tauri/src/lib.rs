mod applog;
mod collections;
mod lightroom;
mod offsets;
mod settings;
mod units;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use divelens_core::macdive::MacDive;
use divelens_core::profile::Sample;
use divelens_core::subsurface::{self, Storage, Subsurface};
use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units, valid_gps};
use divelens_core::{Capabilities, Dive, DiveSource, Error, Site};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use collections::{Choices, Period, Preview, Session};
use lightroom::{
    ApplyOutcome, CollectionSet, Command, End, Event, LinkFile, Status, Timing, WrittenCollection,
};
use offsets::Offsets;
use settings::{Settings, SourceKind};

/// Where the app writes its log (`None`: no home folder).
struct LogFile(Option<PathBuf>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: &'static str,
    os: &'static str,
    log_path: Option<PathBuf>,
}

/// Version, system and log file, for Settings.
#[tauri::command]
fn app_info(log: State<LogFile>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        log_path: log.0.clone(),
    }
}

/// Current settings.
#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    Settings::load(&config_dir(&app))
}

/// Changes the settings named in `patch` and returns them all; every window
/// hears of it through `settings-changed`.
#[tauri::command]
fn update_settings(app: AppHandle, patch: serde_json::Value) -> Result<Settings, String> {
    change_settings(&app, |s| s.patch(&patch))
}

fn change_settings(
    app: &AppHandle,
    change: impl FnOnce(&mut Settings),
) -> Result<Settings, String> {
    let before = Settings::load(&config_dir(app));
    let after = Settings::update(&config_dir(app), change).map_err(|e| {
        log::error!("settings not saved: {e}");
        e.to_string()
    })?;
    if after != before {
        log::info!(
            "settings changed: {}",
            serde_json::to_string(&after).unwrap_or_default()
        );
        if after.log_level != before.log_level {
            applog::set_level(after.log_level);
            send_log_level(app, after.log_level);
        }
        let _ = app.emit("settings-changed", after.clone());
    }
    Ok(after)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceInfo {
    kind: SourceKind,
    path: PathBuf,
    is_default_path: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiveLog {
    source: SourceInfo,
    /// How a Subsurface log is stored (file or git); `None` for MacDive.
    storage: Option<Storage>,
    capabilities: Capabilities,
    dives: Vec<Dive>,
    /// Every named site of the log (Tag Selection › other site).
    sites: Vec<Site>,
    /// Units of the dive application's preferences (the settings may force
    /// others).
    source_units: units::UiUnits,
}

/// Error sent to the UI: the reader error plus the source it concerns.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadError {
    source: SourceInfo,
    error: Error,
}

fn config_dir(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().expect("no config folder")
}

/// Login name, which names Subsurface's standard file.
fn login_name() -> String {
    ["LOGNAME", "USER", "USERNAME"]
        .iter()
        .find_map(|v| std::env::var(v).ok().filter(|n| !n.is_empty()))
        .unwrap_or_else(|| "username".into())
}

/// Standard location of each application's log.
fn standard_path(app: &AppHandle, kind: SourceKind) -> PathBuf {
    match kind {
        SourceKind::MacDive => app
            .path()
            .home_dir()
            .map(|h| MacDive::default_path(&h))
            .unwrap_or_default(),
        // ~/Library/Application Support/Subsurface on macOS,
        // %APPDATA%\Subsurface on Windows.
        SourceKind::Subsurface => app
            .path()
            .data_dir()
            .map(|d| subsurface::default_path(&d.join("Subsurface"), &login_name()))
            .unwrap_or_default(),
    }
}

fn source_info(app: &AppHandle) -> SourceInfo {
    let settings = Settings::load(&config_dir(app));
    let kind = settings.source.kind();
    match settings.source.path(kind) {
        Some(path) => SourceInfo {
            kind,
            path: path.clone(),
            is_default_path: false,
        },
        None => SourceInfo {
            kind,
            path: standard_path(app, kind),
            is_default_path: true,
        },
    }
}

/// Units of the dive application the log belongs to, from its preferences.
fn source_units(app: &AppHandle, info: &SourceInfo) -> Units {
    let Ok(home) = app.path().home_dir() else {
        return Units::default();
    };
    match info.kind {
        SourceKind::MacDive => units::read_macdive(&home),
        SourceKind::Subsurface => units::read_subsurface(&home),
    }
}

/// Units in use (spec F2): the dive application's, unless forced in the
/// settings.
fn units_in_use(app: &AppHandle, info: &SourceInfo) -> Units {
    Settings::load(&config_dir(app))
        .units
        .resolve(source_units(app, info))
}

/// An opened dive log.
enum Source {
    MacDive(MacDive),
    Subsurface(Arc<Subsurface>),
}

impl Source {
    fn storage(&self) -> Option<Storage> {
        match self {
            Source::MacDive(_) => None,
            Source::Subsurface(s) => Some(s.storage().clone()),
        }
    }
}

impl DiveSource for Source {
    fn capabilities(&self) -> Capabilities {
        match self {
            Source::MacDive(s) => s.capabilities(),
            Source::Subsurface(s) => s.capabilities(),
        }
    }
    fn dives(&self) -> Result<Vec<Dive>, Error> {
        match self {
            Source::MacDive(s) => s.dives(),
            Source::Subsurface(s) => s.dives(),
        }
    }
    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>, Error> {
        match self {
            Source::MacDive(s) => s.profile(dive_id),
            Source::Subsurface(s) => s.profile(dive_id),
        }
    }
    fn sites(&self) -> Result<Vec<Site>, Error> {
        match self {
            Source::MacDive(s) => s.sites(),
            Source::Subsurface(s) => s.sites(),
        }
    }
}

/// The last Subsurface log read, kept while it does not change on disk:
/// Subsurface logs are read whole (profiles included), so reading one again
/// for every profile shown would be slow.
#[derive(Default)]
struct SubsurfaceCache(Mutex<Option<(PathBuf, String, Arc<Subsurface>)>>);

fn open_subsurface(app: &AppHandle, path: &Path) -> Result<Arc<Subsurface>, Error> {
    let state = app.state::<SubsurfaceCache>();
    let fingerprint = Subsurface::fingerprint(path);
    if let Some(f) = &fingerprint
        && let Some((p, cached, log)) = state.0.lock().expect("subsurface cache").as_ref()
        && p == path
        && cached == f
    {
        return Ok(log.clone());
    }
    let log = Arc::new(Subsurface::open(path)?);
    *state.0.lock().expect("subsurface cache") =
        fingerprint.map(|f| (path.to_path_buf(), f, log.clone()));
    Ok(log)
}

fn open(app: &AppHandle, info: &SourceInfo) -> Result<Source, Error> {
    match info.kind {
        SourceKind::MacDive => MacDive::open(&info.path).map(Source::MacDive),
        SourceKind::Subsurface => open_subsurface(app, &info.path).map(Source::Subsurface),
    }
}

/// Loads every dive of the configured dive log (off the UI thread: a dive log
/// stored in iCloud may take a while to download).
#[tauri::command]
async fn load_dives(app: AppHandle) -> Result<DiveLog, LoadError> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = source_info(&app);
        let result = open(&app, &source)
            .and_then(|s| Ok((s.storage(), s.capabilities(), s.dives()?, s.sites()?)));
        match &result {
            Ok((_, _, dives, sites)) => log::info!(
                "dive log {}: {} dives, {} sites",
                source.path.display(),
                dives.len(),
                sites.len()
            ),
            Err(e) => log::warn!("dive log {} not read: {e:?}", source.path.display()),
        }
        match result {
            Ok((storage, capabilities, dives, sites)) => Ok(DiveLog {
                source_units: source_units(&app, &source).into(),
                source,
                storage,
                capabilities,
                dives,
                sites,
            }),
            Err(error) => Err(LoadError { source, error }),
        }
    })
    .await
    .expect("dive loading task failed")
}

/// Profile of one dive; empty when the dive has none.
#[tauri::command]
async fn dive_profile(app: AppHandle, id: String) -> Result<Vec<Sample>, Error> {
    tauri::async_runtime::spawn_blocking(move || open(&app, &source_info(&app))?.profile(&id))
        .await
        .expect("profile task failed")
}

/// Points DiveLens at another dive-log file, chosen by the user.
#[tauri::command]
fn set_source_path(app: AppHandle, path: Option<PathBuf>) -> Result<Settings, String> {
    change_settings(&app, |s| {
        let kind = s.source.kind();
        s.source.set_path(kind, path);
    })
}

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
    Source {
        detail: Error,
    },
}

fn tag_request(app: &AppHandle, choice: &TagChoice) -> Result<TagRequest, TagError> {
    let info = source_info(app);
    let source = open(app, &info).map_err(|detail| TagError::Source { detail })?;
    let units = units_in_use(app, &info);
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
    Ok(TagRequest {
        dive: dive.map(|d| DiveTags::new(&d, units, info.kind.name())),
        location,
    })
}

/// Tags the photos selected in Lightroom (at the moment Lightroom gets the
/// request) with a dive's data and/or a location, in one undoable step.
#[tauri::command]
async fn tag_selection(app: AppHandle, choice: TagChoice) -> Result<ApplyOutcome, TagError> {
    tauri::async_runtime::spawn_blocking(move || {
        let request = tag_request(&app, &choice)?;
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
        outcome.recv().map_err(|_| TagError::LinkLost)
    })
    .await
    .expect("tagging task failed")
}

/// Photos found for "Create Collections", kept until the next search.
#[derive(Default)]
struct CollectionsState {
    session: Mutex<Option<Session>>,
}

/// Why "Create Collections" could not go on.
#[derive(Debug, Serialize)]
#[serde(tag = "error", rename_all = "camelCase")]
enum CollectionsError {
    NothingChosen,
    NotConnected,
    LinkLost,
    /// No photo search to work on (link reconnected, app restarted).
    NoPhotos,
    /// No photo goes to any dive.
    NothingToWrite,
    Source {
        detail: Error,
    },
    /// Lightroom refused or failed; `reason` as in [`ApplyOutcome::Failed`].
    #[serde(rename_all = "camelCase")]
    Lightroom {
        reason: String,
        detail: Option<String>,
    },
}

impl From<ApplyOutcome> for CollectionsError {
    fn from(outcome: ApplyOutcome) -> Self {
        match outcome {
            ApplyOutcome::Failed { reason, detail } => Self::Lightroom { reason, detail },
            ApplyOutcome::Applied { .. } => Self::Lightroom {
                reason: "error".into(),
                detail: Some("unexpected answer".into()),
            },
        }
    }
}

/// Sends a request to the plugin and waits for its answer.
fn ask<T>(
    app: &AppHandle,
    command: impl FnOnce(mpsc::Sender<T>) -> Command,
) -> Result<T, CollectionsError> {
    let state = app.state::<Lightroom>();
    if !state.status.lock().expect("lightroom state").connected {
        return Err(CollectionsError::NotConnected);
    }
    let (reply, answer) = mpsc::channel();
    let sent = state
        .commands
        .lock()
        .expect("lightroom commands")
        .send(command(reply));
    if sent.is_err() {
        return Err(CollectionsError::NotConnected);
    }
    answer.recv().map_err(|_| CollectionsError::LinkLost)
}

/// Finds in Lightroom every photo of the chosen dives' period (whole
/// catalog) and their cameras.
#[tauri::command]
async fn find_collection_photos(
    app: AppHandle,
    dive_ids: Vec<String>,
) -> Result<Period, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let info = source_info(&app);
        let source = open(&app, &info).map_err(|detail| CollectionsError::Source { detail })?;
        let dives: Vec<Dive> = source
            .dives()
            .map_err(|detail| CollectionsError::Source { detail })?
            .into_iter()
            .filter(|d| dive_ids.contains(&d.id))
            .collect();
        let (from, to) = collections::search_days(&dives).ok_or(CollectionsError::NothingChosen)?;
        let (f, t) = (from.clone(), to.clone());
        let found = ask(&app, |reply| Command::FindPhotos {
            from: f,
            to: t,
            reply,
        })??;
        let offsets = Offsets::load(&config_dir(&app), info.kind);
        let session = Session::new(found.search, dives, found.photos, &offsets);
        let period = session.period(from, to);
        *app.state::<CollectionsState>()
            .session
            .lock()
            .expect("collections") = Some(session);
        Ok(period)
    })
    .await
    .expect("photo search task failed")
}

/// Live counts for the current choices.
#[tauri::command]
fn collections_preview(
    state: State<CollectionsState>,
    choices: Choices,
) -> Result<Preview, CollectionsError> {
    let session = state.session.lock().expect("collections");
    let session = session.as_ref().ok_or(CollectionsError::NoPhotos)?;
    Ok(session.preview(&choices))
}

/// Suggested clock offset of the given cameras (`None`: no offset fits).
#[tauri::command]
async fn suggest_offsets(
    app: AppHandle,
    cameras: Vec<String>,
) -> Result<std::collections::HashMap<String, Option<i32>>, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let session = state.session.lock().expect("collections");
        let session = session.as_ref().ok_or(CollectionsError::NoPhotos)?;
        Ok(session.suggest(&cameras))
    })
    .await
    .expect("suggestion task failed")
}

/// What "Create Collections › Apply" wrote.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CollectionsResult {
    collections: Vec<WrittenCollection>,
    photos: usize,
    with_depth: usize,
    /// Chosen dives that got no photo (no collection).
    empty_dives: Vec<String>,
}

/// Writes the collections, dive data and per-photo values in one Lightroom
/// step, then remembers each camera's offset.
#[tauri::command]
async fn apply_collections(
    app: AppHandle,
    choices: Choices,
) -> Result<CollectionsResult, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let info = source_info(&app);
        let source = open(&app, &info).map_err(|detail| CollectionsError::Source { detail })?;
        let units = units_in_use(&app, &info);
        let (request, empty_dives) = {
            let state = app.state::<CollectionsState>();
            let session = state.session.lock().expect("collections");
            let session = session.as_ref().ok_or(CollectionsError::NoPhotos)?;
            let mut request = session
                .request(&choices, units, info.kind.name(), |d| source.profile(&d.id))
                .map_err(|detail| CollectionsError::Source { detail })?;
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
            (request, empty)
        };
        if request.groups.is_empty() {
            return Err(CollectionsError::NothingToWrite);
        }
        let with_depth = request
            .groups
            .iter()
            .flat_map(|g| &g.photos)
            .filter(|p| p.photo_depth.is_some())
            .count();
        let written = ask(&app, |reply| Command::ApplyCollections { request, reply })
            .inspect_err(|e| log::warn!("collections not written: {e:?}"))?
            .map_err(|outcome| {
                log::warn!("collections refused by Lightroom: {outcome:?}");
                CollectionsError::from(outcome)
            })?;
        log::info!(
            "collections written: {} collections, {} photos ({with_depth} with depth)",
            written.len(),
            written.iter().map(|c| c.count).sum::<usize>()
        );
        let dir = config_dir(&app);
        let mut offsets = Offsets::load(&dir, info.kind);
        if let Some(session) = app
            .state::<CollectionsState>()
            .session
            .lock()
            .expect("collections")
            .as_ref()
        {
            session.remember(&choices, &mut offsets);
        }
        if let Err(e) = offsets.save(&dir, info.kind) {
            log::error!("clock offsets not saved: {e}");
        }
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
async fn collection_sets(app: AppHandle) -> Result<Vec<CollectionSet>, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(ask(&app, |reply| Command::ListSets { reply })??)
    })
    .await
    .expect("collection sets task failed")
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

/// Keeps trying to reach the plugin for as long as the app runs: whichever of
/// Lightroom and the app starts first, they find each other.
fn lightroom_link(app: AppHandle, commands: mpsc::Receiver<Command>) {
    let dir = config_dir(&app);
    let version = app.package_info().version.to_string();
    let mut last_end = None;
    loop {
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
                        let kind = source_info(&app).kind;
                        let cover = offsets::corrected_range(&Offsets::load(&dir, kind), &ranges);
                        (status.cover_from, status.cover_to) = cover.unzip();
                        *app.state::<Lightroom>()
                            .status
                            .lock()
                            .expect("lightroom state") = status.clone();
                        let _ = app.emit("lightroom-status", status);
                    }
                    Event::Show(screen) => show_screen(&app, screen),
                    Event::Legacy(legacy) => {
                        if let Ok(home) = app.path().home_dir() {
                            let standard = MacDive::default_path(&home);
                            let taken =
                                change_settings(&app, |s| s.take_over(&legacy, &home, &standard));
                            if let Err(e) = taken {
                                log::error!("previous plugin's settings not taken over: {e}");
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
            // Log changes only: a stale link file fails every second.
            let text = format!("{end:?}");
            if last_end.as_ref() != Some(&text) {
                if end != End::Unreachable {
                    log::info!("Lightroom link ended: {text}");
                }
                last_end = Some(text);
            }
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
        .invoke_handler(tauri::generate_handler![
            app_info,
            get_settings,
            update_settings,
            load_dives,
            dive_profile,
            set_source_path,
            lightroom_status,
            initial_screen,
            tag_selection,
            selection_preview,
            find_collection_photos,
            collections_preview,
            suggest_offsets,
            apply_collections,
            collection_sets
        ])
        .setup(|app| {
            let settings = Settings::load(&config_dir(app.handle()));
            let log_path = app.path().home_dir().ok().map(|home| {
                let path = applog::start(applog::folder(&home), settings.log_level);
                log::info!(
                    "DiveLens {} on {}, log {}",
                    env!("CARGO_PKG_VERSION"),
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
            app.manage(CollectionsState::default());
            app.manage(SubsurfaceCache::default());
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
