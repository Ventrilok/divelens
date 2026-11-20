//! The Lightroom link as the screens use it: commands they invoke and the
//! screen asked for on the command line.

use tauri::{AppHandle, Manager, State};

use super::events::{Lightroom, SelectionPreview};
use super::{Command, Status};
use crate::dive_log::Writes;
use crate::settings;

/// Tells the plugin what to write in its log (dropped when not connected:
/// the level is sent again on connection).
pub(crate) fn send_log_level(app: &AppHandle, level: settings::LogLevel) {
    if let Some(state) = app.try_state::<Lightroom>() {
        let _ = state
            .commands
            .lock()
            .expect("lightroom commands")
            .send(Command::SetLogLevel(level));
    }
}

/// Thumbnails received so far for the current selection; new ones arrive as
/// `selection-preview` events.
#[tauri::command]
pub(crate) fn selection_preview(state: State<Lightroom>) -> SelectionPreview {
    state.preview.lock().expect("preview").clone()
}

/// Current Lightroom link state; updates arrive as `lightroom-status` events.
#[tauri::command]
pub(crate) fn lightroom_status(state: State<Lightroom>) -> Status {
    state.status.lock().expect("lightroom state").clone()
}

/// Whether Lightroom is being written to (the dive log cannot change);
/// changes arrive as `lightroom-writing` events.
#[tauri::command]
pub(crate) fn lightroom_writing(writes: State<Writes>) -> bool {
    writes.in_progress()
}

/// Screen asked for on the command line (`--screen tag`), when Lightroom
/// launched the app from one of its menus.
#[tauri::command]
pub(crate) fn initial_screen() -> Option<String> {
    screen_arg(std::env::args())
}

pub(crate) fn screen_arg(args: impl IntoIterator<Item = String>) -> Option<String> {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--screen" {
            return args.next();
        }
    }
    None
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
