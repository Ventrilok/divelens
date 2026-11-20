//! The app side of the Lightroom link: the state the screens read, and the
//! thread that keeps the link up and turns what the plugin says into events.

use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use divelens_core::macdive;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::commands::send_log_level;
use super::{Command, End, Event, LinkFile, Status, Timing};
use crate::apply::offsets::{self, Offsets};
use crate::dive_log::{source_info, standard_path};
use crate::files::config_dir;
use crate::settings::{Settings, change_settings};

/// Last known state of the Lightroom link, and the way to send it requests.
pub(crate) struct Lightroom {
    pub(crate) status: Mutex<Status>,
    pub(crate) commands: Mutex<Sender<Command>>,
    /// Thumbnails of the current selection, as data URLs, by position.
    pub(crate) preview: Mutex<SelectionPreview>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct SelectionPreview {
    pub(crate) version: u64,
    pub(crate) thumbnails: Vec<Option<String>>,
}

pub(crate) fn bring_to_front(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub(crate) fn show_screen(app: &AppHandle, screen: String) {
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
pub(crate) fn lightroom_link(app: AppHandle, commands: mpsc::Receiver<Command>) {
    let dir = config_dir(&app);
    let version = app.package_info().version.to_string();
    let mut last_end = None;
    loop {
        // Requests made while there is no link are dropped, so whoever sent
        // one stops waiting (and a write in progress ends) even when
        // Lightroom quit and its link file is gone.
        super::drop_requests(&commands);
        if let Some(link) = LinkFile::read(&dir) {
            let end = super::run_session(
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
