mod settings;

use std::path::PathBuf;

use divelens_core::macdive::MacDive;
use divelens_core::profile::Sample;
use divelens_core::{Capabilities, Dive, DiveSource, Error};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use settings::{Settings, SourceKind};

/// Versions of the app and of the core library, shown in Settings › About.
#[tauri::command]
fn versions() -> (String, String) {
    (
        env!("CARGO_PKG_VERSION").to_string(),
        divelens_core::VERSION.to_string(),
    )
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
    capabilities: Capabilities,
    dives: Vec<Dive>,
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

fn source_info(app: &AppHandle) -> SourceInfo {
    let settings = Settings::load(&config_dir(app));
    let default = app
        .path()
        .home_dir()
        .map(|h| MacDive::default_path(&h))
        .unwrap_or_default();
    match settings.source.path {
        Some(path) => SourceInfo {
            kind: settings.source.kind,
            path,
            is_default_path: false,
        },
        None => SourceInfo {
            kind: settings.source.kind,
            path: default,
            is_default_path: true,
        },
    }
}

fn open(info: &SourceInfo) -> Result<MacDive, Error> {
    match info.kind {
        SourceKind::MacDive => MacDive::open(&info.path),
    }
}

/// Loads every dive of the configured dive log (off the UI thread: a dive log
/// stored in iCloud may take a while to download).
#[tauri::command]
async fn load_dives(app: AppHandle) -> Result<DiveLog, LoadError> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = source_info(&app);
        let result = open(&source).and_then(|s| Ok((s.capabilities(), s.dives()?)));
        match result {
            Ok((capabilities, dives)) => Ok(DiveLog {
                source,
                capabilities,
                dives,
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
    tauri::async_runtime::spawn_blocking(move || open(&source_info(&app))?.profile(&id))
        .await
        .expect("profile task failed")
}

/// Points DiveLens at another dive-log file, chosen by the user.
#[tauri::command]
fn set_source_path(app: AppHandle, path: PathBuf) -> Result<(), String> {
    let dir = config_dir(&app);
    let mut settings = Settings::load(&dir);
    settings.source.path = Some(path);
    settings.save(&dir).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            versions,
            load_dives,
            dive_profile,
            set_source_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running DiveLens");
}
