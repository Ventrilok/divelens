mod lightroom;
mod settings;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use divelens_core::macdive::MacDive;
use divelens_core::profile::Sample;
use divelens_core::{Capabilities, Dive, DiveSource, Error};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use lightroom::{End, Event, LinkFile, Status, Timing};
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

/// Last known state of the Lightroom link, for screens that open later.
#[derive(Default)]
struct Lightroom(Mutex<Status>);

/// Current Lightroom link state; updates arrive as `lightroom-status` events.
#[tauri::command]
fn lightroom_status(state: State<Lightroom>) -> Status {
    state.0.lock().expect("lightroom state").clone()
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
fn lightroom_link(app: AppHandle) {
    let dir = config_dir(&app);
    let version = app.package_info().version.to_string();
    let mut last_end = None;
    loop {
        if let Some(link) = LinkFile::read(&dir) {
            let end =
                lightroom::run_session(
                    &link,
                    &version,
                    Timing::default(),
                    &mut |event| match event {
                        Event::Status(status) => {
                            *app.state::<Lightroom>().0.lock().expect("lightroom state") =
                                status.clone();
                            let _ = app.emit("lightroom-status", status);
                        }
                        Event::Show(screen) => show_screen(&app, screen),
                    },
                );
            // Log changes only: a stale link file fails every second.
            let text = format!("{end:?}");
            if last_end.as_ref() != Some(&text) {
                if end != End::Unreachable {
                    eprintln!("Lightroom link ended: {text}");
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
            versions,
            load_dives,
            dive_profile,
            set_source_path,
            lightroom_status,
            initial_screen
        ])
        .manage(Lightroom::default())
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("lightroom-link".into())
                .spawn(move || lightroom_link(handle))?;
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
