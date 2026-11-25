//! The Lightroom link as the screens use it: commands they invoke and the
//! screen asked for on the command line.

use tauri::{AppHandle, Manager, State};

use super::events::{Lightroom, SelectionPreview};
use super::{Command, Status};
use crate::apply::errors::ApplyError;
use crate::apply::send_to;
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

/// Asks Lightroom for the number of photos in the catalog again. Never
/// waits: the request is handed over and the new number arrives in the
/// Lightroom status (`catalogPhotos`) whenever the plugin answers, if it
/// does. `NotConnected` without a link.
#[tauri::command]
pub(crate) fn refresh_catalog_size(state: State<Lightroom>) -> Result<(), ApplyError> {
    refresh_size(&state)
}

fn refresh_size(state: &Lightroom) -> Result<(), ApplyError> {
    let (reply, _) = std::sync::mpsc::channel();
    send_to(state, Command::CountPhotos { reply })
}

/// Screen asked for on the command line (`--screen journey`), when Lightroom
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
    use std::sync::Mutex;
    use std::sync::mpsc::{Receiver, channel};

    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /// The link state, connected or not, and what it hands to the plugin.
    fn link(connected: bool) -> (Lightroom, Receiver<Command>) {
        let (commands, inbox) = channel();
        let state = Lightroom {
            status: Mutex::new(Status {
                connected,
                ..Status::default()
            }),
            commands: Mutex::new(commands),
            preview: Mutex::new(SelectionPreview::default()),
        };
        (state, inbox)
    }

    #[test]
    fn refreshing_the_size_hands_one_count_over_without_waiting() {
        // No link: refused, nothing handed over.
        let (state, inbox) = link(false);
        assert!(matches!(
            refresh_size(&state),
            Err(ApplyError::NotConnected)
        ));
        assert!(inbox.try_recv().is_err());
        // Linked: one count request handed over, and the call is back at
        // once, nobody waiting for its answer.
        let (state, inbox) = link(true);
        assert!(refresh_size(&state).is_ok());
        let Ok(Command::CountPhotos { reply }) = inbox.try_recv() else {
            panic!("a count request");
        };
        assert!(reply.send(Ok(1)).is_err(), "nobody waits for the answer");
        assert!(inbox.try_recv().is_err(), "one request only");
        // The link's thread gone: not connected either.
        let (state, inbox) = link(true);
        drop(inbox);
        assert!(matches!(
            refresh_size(&state),
            Err(ApplyError::NotConnected)
        ));
    }

    #[test]
    fn reads_screen_argument() {
        assert_eq!(
            screen_arg(args(&["DiveLens", "--screen", "journey"])),
            Some("journey".into())
        );
        assert_eq!(screen_arg(args(&["DiveLens"])), None);
        assert_eq!(screen_arg(args(&["DiveLens", "--screen"])), None);
        // macOS may add its own arguments before ours.
        assert_eq!(
            screen_arg(args(&["DiveLens", "-psn_0_1234", "--screen", "settings"])),
            Some("settings".into())
        );
    }
}
