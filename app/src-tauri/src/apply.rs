//! What "Create Collections" and "Tag Selection" share when they write to
//! Lightroom: the clock offsets, the kept photo searches, the errors, and the
//! requests to the plugin link.

pub(crate) mod errors;
pub(crate) mod offsets;
pub(crate) mod searches;

use std::sync::mpsc;

use tauri::{AppHandle, Manager};

use crate::lightroom::Command;
use crate::lightroom::events::Lightroom;
use errors::CollectionsError;

/// Hands a request to the plugin link (never waits); its answer comes on the
/// request's own reply channel.
pub(crate) fn send(app: &AppHandle, command: Command) -> Result<(), CollectionsError> {
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
pub(crate) fn ask<T>(
    app: &AppHandle,
    command: impl FnOnce(mpsc::Sender<T>) -> Command,
) -> Result<T, CollectionsError> {
    let (reply, answer) = mpsc::channel();
    send(app, command(reply))?;
    answer.recv().map_err(|_| CollectionsError::LinkLost)
}
