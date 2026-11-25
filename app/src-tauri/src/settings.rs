//! The settings: the file ([`mod@file`]), the units the dive application shows
//! ([`units`]), and the commands the Settings screen calls, the remembered
//! clock offsets' among them (their file is the journey's, `apply::offsets`).

mod file;
mod units;

#[cfg(test)]
pub(crate) use file::DepthChoice;
pub(crate) use file::{ImportError, Legacy, LogLevel, Settings};
pub(crate) use units::UiUnits;

use std::path::{Path, PathBuf};

use divelens_core::apps::{self, DiveApp, Os};
use divelens_core::tags::Units;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::applog;
use crate::apply::offsets::{self, Offsets, Remembered};
use crate::dive_log::{
    LogId, SourceInfo, WriteInProgress, Writes, guarded_change, source_info, source_info_of,
};
use crate::files::config_dir;
use crate::lightroom::commands::send_log_level;
use crate::lightroom::events::Lightroom;
use crate::update::check_update;

/// Where the app writes its log (`None`: no home folder).
pub(crate) struct LogFile(pub(crate) Option<PathBuf>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppInfo {
    version: String,
    os: &'static str,
    log_path: Option<PathBuf>,
}

/// The dive applications of this system, in the order Settings lists
/// them, with what the screens show of each.
#[tauri::command]
pub(crate) fn dive_apps() -> Vec<&'static DiveApp> {
    apps::available(Os::current()).collect()
}

/// Version (app/package.json, through tauri.conf.json), system and log file,
/// for Settings.
#[tauri::command]
pub(crate) fn app_info(app: AppHandle, log: State<LogFile>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS,
        log_path: log.0.clone(),
    }
}

/// Current settings.
#[tauri::command]
pub(crate) fn get_settings(app: AppHandle) -> Settings {
    Settings::load(&config_dir(&app))
}

/// Changes the settings named in `patch` and returns them all; every window
/// hears of it through `settings-changed`.
#[tauri::command]
pub(crate) fn update_settings(
    app: AppHandle,
    patch: serde_json::Value,
) -> Result<Settings, ChangeError> {
    change_settings(&app, |s| s.patch(&patch))
}

/// Writes the current settings to the file the user chose (spec F11).
#[tauri::command]
pub(crate) fn export_settings(app: AppHandle, path: PathBuf) -> Result<(), String> {
    Settings::load(&config_dir(&app))
        .export(&path)
        .map_err(|e| {
            log::error!("settings not exported to {}: {e}", path.display());
            e.to_string()
        })?;
    log::info!("settings exported to {}", path.display());
    Ok(())
}

/// The clock offsets remembered for the dive application in use, the one
/// the journey pre-fills them from (spec F2, F5). A file that does not read
/// holds none, as on the journey.
#[tauri::command]
pub(crate) fn remembered_offsets(app: AppHandle) -> Vec<Remembered> {
    Offsets::load(&config_dir(&app), source_info(&app).app).remembered()
}

/// Forgets the clock offset remembered for `camera` (its key) for the dive
/// application in use, and returns the offsets left. Nothing in Lightroom
/// changes: the offset already written on photos stays. The selection's
/// cover range, computed from the remembered offsets, is computed again and
/// told to the screens when it moved.
#[tauri::command]
pub(crate) fn forget_offset(app: AppHandle, camera: String) -> Result<Vec<Remembered>, String> {
    let dive_app = source_info(&app).app;
    let left = offsets::forget(&config_dir(&app), dive_app, &camera).map_err(|e| {
        log::error!("clock offset of {camera} not forgotten: {e}");
        e.to_string()
    })?;
    log::info!("clock offset of {camera} forgotten ({})", dive_app.name);
    let moved = {
        let lightroom = app.state::<Lightroom>();
        let mut status = lightroom.status.lock().expect("lightroom state");
        offsets::set_cover(&mut status, &left).then(|| status.clone())
    };
    if let Some(status) = moved {
        let _ = app.emit("lightroom-status", status);
    }
    Ok(left.remembered())
}

/// A settings file's dive log: the application it means on this computer,
/// where the log is, and whether it is there.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportCheck {
    source: SourceInfo,
    found: bool,
}

pub(crate) fn import_check(app: &AppHandle, settings: &Settings) -> ImportCheck {
    let source = source_info_of(app, settings);
    ImportCheck {
        found: source.path.exists(),
        source,
    }
}

/// Reads a settings file before importing it, so the user can confirm
/// knowing whether its dive log is on this computer.
#[tauri::command]
pub(crate) fn check_settings_file(
    app: AppHandle,
    path: PathBuf,
) -> Result<ImportCheck, ImportError> {
    let settings = Settings::read_export(&path).inspect_err(|e| {
        log::warn!("settings file {} not usable: {e:?}", path.display());
    })?;
    Ok(import_check(&app, &settings))
}

/// Replaces every setting with those of the file, once the user confirmed.
/// A dive log missing on this computer is kept and reported: the user then
/// chooses where it is (never a silent switch to another log).
#[tauri::command]
pub(crate) fn import_settings(app: AppHandle, path: PathBuf) -> Result<ImportCheck, ImportError> {
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
pub(crate) enum ChangeError {
    /// The settings file could not be written.
    NotSaved(String),
    /// It would switch the dive log while Lightroom is being written to.
    WriteInProgress,
}

/// Every change of settings goes through here, whichever way it comes (the
/// Settings screen, the dive-log error screen, an import, the previous
/// plugin's settings): none switches the dive log during a write.
pub(crate) fn change_settings(
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
        if after.check_updates && !before.check_updates {
            let app = app.clone();
            tauri::async_runtime::spawn(async move { check_update(app, true).await });
        }
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

/// Units of the dive application the log belongs to, from its preferences.
pub(crate) fn source_units(app: &AppHandle, info: &SourceInfo) -> Units {
    let Ok(home) = app.path().home_dir() else {
        return Units::default();
    };
    units::read(&info.app.preferences, &home)
}

/// Units in use (spec F2): the dive application's, unless forced in the
/// settings.
pub(crate) fn units_in_use(app: &AppHandle, info: &SourceInfo) -> Units {
    Settings::load(&config_dir(app))
        .units
        .resolve(source_units(app, info))
}

#[cfg(test)]
mod tests {
    use super::{ChangeError, DepthChoice, Settings, change_unless_writing};
    use crate::dive_log::{self, Writes};
    use divelens_core::apps::{DiveApp, Os};
    use divelens_core::subsurface;
    use std::path::{Path, PathBuf};

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
}
