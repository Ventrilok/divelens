mod applog;
mod collections;
mod dive_log;
mod lightroom;
mod offsets;
mod settings;
mod units;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use divelens_core::apps::{self, DiveApp, Kept, Opened, Os, Places};
use divelens_core::macdive;
use divelens_core::profile::Sample;
use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units, valid_gps};
use divelens_core::{Capabilities, Dive, Error, Site};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use collections::{Choices, Period, Preview, Searches, Session};
use dive_log::{LogChanged, LogId, SourceInfo, WriteInProgress, Writes, guarded_change};
use lightroom::{
    ApplyOutcome, CollectionSet, Command, End, Event, LinkFile, Status, Timing, WrittenCollection,
};
use offsets::Offsets;
use settings::{ImportError, Settings};

/// Where the app writes its log (`None`: no home folder).
struct LogFile(Option<PathBuf>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    os: &'static str,
    log_path: Option<PathBuf>,
}

/// The dive applications of this system, in the order Settings lists
/// them, with what the screens show of each.
#[tauri::command]
fn dive_apps() -> Vec<&'static DiveApp> {
    apps::available(Os::current()).collect()
}

/// Version (app/package.json, through tauri.conf.json), system and log file,
/// for Settings.
#[tauri::command]
fn app_info(app: AppHandle, log: State<LogFile>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
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
fn update_settings(app: AppHandle, patch: serde_json::Value) -> Result<Settings, ChangeError> {
    change_settings(&app, |s| s.patch(&patch))
}

/// Writes the current settings to the file the user chose (spec F11).
#[tauri::command]
fn export_settings(app: AppHandle, path: PathBuf) -> Result<(), String> {
    Settings::load(&config_dir(&app))
        .export(&path)
        .map_err(|e| {
            log::error!("settings not exported to {}: {e}", path.display());
            e.to_string()
        })?;
    log::info!("settings exported to {}", path.display());
    Ok(())
}

/// A settings file's dive log: the application it means on this computer,
/// where the log is, and whether it is there.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportCheck {
    source: SourceInfo,
    found: bool,
}

fn import_check(app: &AppHandle, settings: &Settings) -> ImportCheck {
    let source = source_info_of(app, settings);
    ImportCheck {
        found: source.path.exists(),
        source,
    }
}

/// Reads a settings file before importing it, so the user can confirm
/// knowing whether its dive log is on this computer.
#[tauri::command]
fn check_settings_file(app: AppHandle, path: PathBuf) -> Result<ImportCheck, ImportError> {
    let settings = Settings::read_export(&path).inspect_err(|e| {
        log::warn!("settings file {} not usable: {e:?}", path.display());
    })?;
    Ok(import_check(&app, &settings))
}

/// Replaces every setting with those of the file, once the user confirmed.
/// A dive log missing on this computer is kept and reported: the user then
/// chooses where it is (never a silent switch to another log).
#[tauri::command]
fn import_settings(app: AppHandle, path: PathBuf) -> Result<ImportCheck, ImportError> {
    let imported = Settings::read_export(&path)?;
    let after = change_settings(&app, |s| s.replace_with(imported)).map_err(|e| match e {
        ChangeError::NotSaved(detail) => ImportError::NotSaved(detail),
        ChangeError::WriteInProgress => ImportError::WriteInProgress,
    })?;
    log::info!("settings imported from {}", path.display());
    Ok(import_check(&app, &after))
}

/// Why a change of settings was not made.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
enum ChangeError {
    /// The settings file could not be written.
    NotSaved(String),
    /// It would switch the dive log while Lightroom is being written to.
    WriteInProgress,
}

/// Every change of settings goes through here, whichever way it comes (the
/// Settings screen, the dive-log error screen, an import, the previous
/// plugin's settings): none switches the dive log during a write.
fn change_settings(
    app: &AppHandle,
    change: impl FnOnce(&mut Settings),
) -> Result<Settings, ChangeError> {
    let log_of = |s: &Settings| source_info_of(app, s).id();
    let (before, after) =
        change_unless_writing(&app.state::<Writes>(), &config_dir(app), log_of, change)?;
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

/// Saves `change` to the settings in `dir`, unless it switches the dive log
/// (`log_of` tells which log some settings point at) while a write is under
/// way: then nothing is saved. Returns the settings before and after.
fn change_unless_writing(
    writes: &Writes,
    dir: &Path,
    log_of: impl Fn(&Settings) -> LogId,
    change: impl FnOnce(&mut Settings),
) -> Result<(Settings, Settings), ChangeError> {
    let mut before = Settings::default();
    let mut refused = None;
    let after = writes
        .exclusive(|writing| {
            Settings::update(dir, |s| {
                before = s.clone();
                refused = guarded_change(writing, s, change, log_of).err();
            })
        })
        .map_err(|e| {
            log::error!("settings not saved: {e}");
            ChangeError::NotSaved(e.to_string())
        })?;
    if let Some(WriteInProgress) = refused {
        log::warn!("dive log not changed: Lightroom is being written to");
        return Err(ChangeError::WriteInProgress);
    }
    Ok((before, after))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiveLog {
    source: SourceInfo,
    /// How the log is stored, as its reader tells: what Settings says once
    /// it is read (a key of the application's texts).
    format: &'static str,
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

/// Standard location of an application's log on this computer.
fn standard_path(app: &AppHandle, dive_app: &DiveApp) -> PathBuf {
    let home = app.path().home_dir().ok();
    // ~/Library/Application Support on macOS, %APPDATA% on Windows.
    let data = app.path().data_dir().ok();
    let user = login_name();
    let places = Places {
        os: Os::current(),
        home: home.as_deref(),
        data: data.as_deref(),
        user: &user,
    };
    (dive_app.standard_location)(&places).unwrap_or_default()
}

/// The configured dive log, as every action checks it.
fn source_info(app: &AppHandle) -> SourceInfo {
    dive_log::configured(&config_dir(app), Os::current(), |dive_app| {
        standard_path(app, dive_app)
    })
}

/// The dive log `settings` point at: the application in use on this
/// computer and its location.
fn source_info_of(app: &AppHandle, settings: &Settings) -> SourceInfo {
    SourceInfo::of(settings, Os::current(), |dive_app| {
        standard_path(app, dive_app)
    })
}

/// Units of the dive application the log belongs to, from its preferences.
fn source_units(app: &AppHandle, info: &SourceInfo) -> Units {
    let Ok(home) = app.path().home_dir() else {
        return Units::default();
    };
    units::read(&info.app.preferences, &home)
}

/// Units in use (spec F2): the dive application's, unless forced in the
/// settings.
fn units_in_use(app: &AppHandle, info: &SourceInfo) -> Units {
    Settings::load(&config_dir(app))
        .units
        .resolve(source_units(app, info))
}

/// Opens the dive log `info` names with its application's reader. A log
/// its reader reads whole is kept between requests while it does not change
/// on disk (the last one only).
fn open(app: &AppHandle, info: &SourceInfo) -> Result<Opened, Error> {
    (info.app.open)(&info.path, &app.state::<Kept>())
}

/// Loads every dive of the configured dive log (off the UI thread: a dive log
/// stored in iCloud may take a while to download).
#[tauri::command]
async fn load_dives(app: AppHandle) -> Result<DiveLog, LoadError> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = source_info(&app);
        let result = open(&app, &source).and_then(|o| {
            let log = &o.log;
            Ok((o.format, log.capabilities(), log.dives()?, log.sites()?))
        });
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
            Ok((format, capabilities, dives, sites)) => Ok(DiveLog {
                source_units: source_units(&app, &source).into(),
                source,
                format,
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

/// Why a dive profile was not read.
#[derive(Debug, Serialize)]
#[serde(tag = "error", rename_all = "camelCase")]
enum ProfileError {
    /// The screen shows another dive log than the configured one.
    LogChanged,
    Source {
        detail: Error,
    },
}

impl From<LogChanged> for ProfileError {
    fn from(_: LogChanged) -> Self {
        Self::LogChanged
    }
}

/// Profile of one dive of `log`, the dive log the screen shows; empty when
/// the dive has none.
#[tauri::command]
async fn dive_profile(app: AppHandle, log: LogId, id: String) -> Result<Vec<Sample>, ProfileError> {
    tauri::async_runtime::spawn_blocking(move || {
        let info = source_info(&app);
        log.check(&info)?;
        open(&app, &info)
            .and_then(|o| o.log.profile(&id))
            .map_err(|detail| ProfileError::Source { detail })
    })
    .await
    .expect("profile task failed")
}

/// Points DiveLens at another dive-log file, chosen by the user.
#[tauri::command]
fn set_source_path(app: AppHandle, path: Option<PathBuf>) -> Result<Settings, ChangeError> {
    change_settings(&app, |s| {
        let dive_app = s.source.app(Os::current());
        s.source.set_location(dive_app, path);
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
    Source {
        detail: Error,
    },
}

impl From<LogChanged> for TagError {
    fn from(_: LogChanged) -> Self {
        Self::LogChanged
    }
}

/// What to write, from `info`, the dive log checked for this write.
fn tag_request(
    app: &AppHandle,
    info: &SourceInfo,
    choice: &TagChoice,
) -> Result<TagRequest, TagError> {
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
    Ok(TagRequest {
        dive: dive.map(|d| DiveTags::new(&d, units, info.app.name)),
        location,
    })
}

/// Tags the photos selected in Lightroom (at the moment Lightroom gets the
/// request) with a dive's data and/or a location, in one undoable step.
/// `log` is the dive log the screen shows: refused when it changed.
#[tauri::command]
async fn tag_selection(
    app: AppHandle,
    log: LogId,
    choice: TagChoice,
) -> Result<ApplyOutcome, TagError> {
    tauri::async_runtime::spawn_blocking(move || {
        let writes = app.state::<Writes>();
        let (_writing, info) = writes.begin(&log, || source_info(&app))?;
        let request = tag_request(&app, &info, &choice)?;
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
    searches: Mutex<Searches>,
}

/// Keeps the session of photo search `number` once it is done, unless the
/// configured log changed during it (its dives belong to the previous log)
/// or a newer search started since (the screen shows that one).
fn keep_search(
    searches: &Mutex<Searches>,
    number: u64,
    log: &LogId,
    configured: &SourceInfo,
    session: Session,
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

/// Why "Create Collections" could not go on.
#[derive(Debug, Serialize)]
#[serde(tag = "error", rename_all = "camelCase")]
enum CollectionsError {
    NothingChosen,
    NotConnected,
    LinkLost,
    /// No photo search to work on (link reconnected, app restarted, or a
    /// newer search replaced it).
    NoPhotos,
    /// No photo goes to any dive.
    NothingToWrite,
    /// The screen shows another dive log than the configured one, or the
    /// dives were read from another one.
    LogChanged,
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

impl From<LogChanged> for CollectionsError {
    fn from(_: LogChanged) -> Self {
        Self::LogChanged
    }
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

/// Hands a request to the plugin link (never waits); its answer comes on the
/// request's own reply channel.
fn send(app: &AppHandle, command: Command) -> Result<(), CollectionsError> {
    let state = app.state::<Lightroom>();
    if !state.status.lock().expect("lightroom state").connected {
        return Err(CollectionsError::NotConnected);
    }
    state
        .commands
        .lock()
        .expect("lightroom commands")
        .send(command)
        .map_err(|_| CollectionsError::NotConnected)
}

/// Sends a request to the plugin and waits for its answer.
fn ask<T>(
    app: &AppHandle,
    command: impl FnOnce(mpsc::Sender<T>) -> Command,
) -> Result<T, CollectionsError> {
    let (reply, answer) = mpsc::channel();
    send(app, command(reply))?;
    answer.recv().map_err(|_| CollectionsError::LinkLost)
}

/// Finds in Lightroom every photo of the chosen dives' period (whole
/// catalog) and their cameras. The dives and their profiles are read now,
/// from `log`, the dive log the screen shows, and kept for "Apply".
/// `search` numbers the search, higher than any the screen asked for before.
#[tauri::command]
async fn find_collection_photos(
    app: AppHandle,
    log: LogId,
    search: u64,
    dive_ids: Vec<String>,
) -> Result<Period, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let number = search;
        if !state.searches.lock().expect("collections").start(number) {
            return Err(CollectionsError::NoPhotos);
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
            .map_err(|detail| CollectionsError::Source { detail })?
            .log;
        let dives: Vec<Dive> = source
            .dives()
            .map_err(|detail| CollectionsError::Source { detail })?
            .into_iter()
            .filter(|d| dive_ids.contains(&d.id))
            .collect();
        let (from, to) = collections::search_days(&dives).ok_or(CollectionsError::NothingChosen)?;
        // Which dives get photos depends on the offsets, not known yet: every
        // profile is read. One that does not read fails "Apply" only if it
        // is needed, as before.
        let mut profiles = std::collections::HashMap::new();
        for dive in dives.iter().filter(|d| d.has_profile) {
            if replaced() {
                return Err(CollectionsError::NoPhotos);
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
                return Err(CollectionsError::NoPhotos);
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
        let found = answer.recv().map_err(|_| CollectionsError::LinkLost)??;
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
fn collections_preview(
    state: State<CollectionsState>,
    search: u64,
    choices: Choices,
) -> Result<Preview, CollectionsError> {
    let searches = state.searches.lock().expect("collections");
    let session = searches.get(search).ok_or(CollectionsError::NoPhotos)?;
    Ok(session.preview(&choices))
}

/// Suggested clock offset of the given cameras of photo search `search`
/// (`None`: no offset fits).
#[tauri::command]
async fn suggest_offsets(
    app: AppHandle,
    search: u64,
    cameras: Vec<String>,
) -> Result<std::collections::HashMap<String, Option<i32>>, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let searches = state.searches.lock().expect("collections");
        let session = searches.get(search).ok_or(CollectionsError::NoPhotos)?;
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
/// step, then remembers each camera's offset. Writes the dives and profiles
/// read with photo search `search`, the one the screen shows; refused when
/// it is no longer kept, or when `log`, the dive log the screen shows, is
/// not theirs or no longer the configured one.
#[tauri::command]
async fn apply_collections(
    app: AppHandle,
    log: LogId,
    search: u64,
    choices: Choices,
) -> Result<CollectionsResult, CollectionsError> {
    tauri::async_runtime::spawn_blocking(move || {
        let writes = app.state::<Writes>();
        let (_writing, info) = writes.begin(&log, || source_info(&app))?;
        let units = units_in_use(&app, &info);
        let (request, empty_dives) = {
            let state = app.state::<CollectionsState>();
            let searches = state.searches.lock().expect("collections");
            let session = searches.get(search).ok_or(CollectionsError::NoPhotos)?;
            let mut request = session
                .write_request(&log, &choices, units)?
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
        let mut offsets = Offsets::load(&dir, log.app);
        if let Some(session) = app
            .state::<CollectionsState>()
            .searches
            .lock()
            .expect("collections")
            .get(search)
        {
            session.remember(&choices, &mut offsets);
        }
        if let Err(e) = offsets.save(&dir, log.app) {
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
            dive_apps,
            app_info,
            get_settings,
            update_settings,
            export_settings,
            check_settings_file,
            import_settings,
            load_dives,
            dive_profile,
            set_source_path,
            lightroom_status,
            lightroom_writing,
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
            app.manage(CollectionsState::default());
            app.manage(Kept::default());
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
    use super::{
        ChangeError, CollectionsError, DiveApp, Offsets, Os, Searches, Session, Settings, Writes,
        change_unless_writing, dive_log, keep_search,
    };
    use crate::settings::DepthChoice;
    use divelens_core::subsurface;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    #[test]
    fn a_search_is_kept_only_for_the_log_still_configured_and_if_newest() {
        let searches = Mutex::new(Searches::default());
        let at = |path: &str| dive_log::SourceInfo {
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

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("divelens-lib-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The system the tests run as: both applications are available.
    const OS: Os = Os::MacOs;

    fn standard(app: &DiveApp) -> PathBuf {
        match app.id {
            "macDive" => "/Users/diver/MacDive.sqlite".into(),
            _ => "/Users/diver/diver.xml".into(),
        }
    }

    fn log_of(s: &Settings) -> dive_log::LogId {
        dive_log::SourceInfo::of(s, OS, standard).id()
    }

    fn on_file(s: &mut Settings, path: &Path) {
        s.source.kind = subsurface::APP.id.into();
        s.source
            .set_location(&subsurface::APP, Some(path.to_path_buf()));
    }

    #[test]
    fn settings_cannot_switch_the_dive_log_during_a_write() {
        let dir = temp_dir("gate");
        let (a, b) = (dir.join("a.xml"), dir.join("b.xml"));
        let writes = Writes::new(|_| {});
        change_unless_writing(&writes, &dir, log_of, |s| on_file(s, &a)).unwrap();
        let shown = dive_log::configured(&dir, OS, standard).id();
        let (writing, _) = writes
            .begin(&shown, || dive_log::configured(&dir, OS, standard))
            .unwrap();

        let refused = change_unless_writing(&writes, &dir, log_of, |s| on_file(s, &b));
        assert!(matches!(refused, Err(ChangeError::WriteInProgress)));
        let location = |dir: &Path| {
            Settings::load(dir)
                .source
                .location(&subsurface::APP)
                .cloned()
        };
        assert_eq!(location(&dir), Some(a.clone()));
        // An import naming another log is refused whole: units untouched.
        let mut imported = Settings::default();
        on_file(&mut imported, &b);
        imported.units.depth = DepthChoice::Feet;
        let refused = change_unless_writing(&writes, &dir, log_of, |s| s.replace_with(imported));
        assert!(matches!(refused, Err(ChangeError::WriteInProgress)));
        assert_eq!(Settings::load(&dir).units.depth, DepthChoice::Source);
        // Other settings may change meanwhile.
        change_unless_writing(&writes, &dir, log_of, |s| s.units.depth = DepthChoice::Feet)
            .unwrap();
        assert_eq!(Settings::load(&dir).units.depth, DepthChoice::Feet);

        drop(writing);
        change_unless_writing(&writes, &dir, log_of, |s| on_file(s, &b)).unwrap();
        assert_eq!(location(&dir), Some(b));
        std::fs::remove_dir_all(dir).unwrap();
    }

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
