mod applog;
mod apply;
mod collections;
mod dive_log;
mod files;
mod lightroom;
mod settings;
mod tag;
mod update;

use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use divelens_core::apps::Kept;
use divelens_core::macdive;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use apply::offsets::{self, Offsets};
use dive_log::{Writes, source_info, standard_path};
use files::config_dir;
use lightroom::{Command, End, Event, LinkFile, Status, Timing};
use settings::{LogFile, Settings, change_settings};

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
            tag::tag_selection,
            tag::find_selection_photos,
            tag::tag_preview,
            tag::suggest_tag_offsets,
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
            app.manage(tag::TagState::default());
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
