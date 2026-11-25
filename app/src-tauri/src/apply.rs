//! What "Create Collections" and "Tag Selection" share when they write to
//! Lightroom: the clock offsets, the kept photo searches, the errors, and the
//! requests to the plugin link.

pub(crate) mod errors;
pub(crate) mod offsets;
pub(crate) mod options;
pub(crate) mod searches;
pub(crate) mod site;

use std::collections::BTreeMap;
use std::sync::mpsc::{self, RecvError};

use tauri::{AppHandle, Manager};

use crate::dive_log::SourceInfo;
use crate::files::config_dir;
use crate::lightroom::events::Lightroom;
use crate::lightroom::{ApplyOutcome, Command};
use errors::ApplyError;

/// Hands a request to the plugin link (never waits); its answer comes on the
/// request's own reply channel.
pub(crate) fn send(app: &AppHandle, command: Command) -> Result<(), ApplyError> {
    send_to(&app.state::<Lightroom>(), command)
}

/// `send`, on the link state itself.
pub(crate) fn send_to(state: &Lightroom, command: Command) -> Result<(), ApplyError> {
    if !state.status.lock().expect("lightroom state").connected {
        return Err(ApplyError::NotConnected);
    }
    state
        .commands
        .lock()
        .expect("lightroom commands")
        .send(command)
        .map_err(|_| ApplyError::NotConnected)
}

/// Sends a request to the plugin and waits for its answer.
pub(crate) fn ask<T>(
    app: &AppHandle,
    command: impl FnOnce(mpsc::Sender<T>) -> Command,
) -> Result<T, ApplyError> {
    let (reply, answer) = mpsc::channel();
    send(app, command(reply))?;
    heard(answer.recv())
}

/// The answer of the plugin link, or "link lost" when its reply channel
/// closed without one (the link dropped while Lightroom was writing).
pub(crate) fn heard<T>(received: Result<T, RecvError>) -> Result<T, ApplyError> {
    received.map_err(|_| ApplyError::LinkLost)
}

/// Sends a write to Lightroom, waits for its answer and, when Lightroom
/// wrote (see [`offsets::should_remember`]), remembers the clock offsets
/// `applied` for the application of `info`. `on_refused` logs a refusal
/// (the feature's own line), after the offsets are saved. The logs of what
/// was written stay with the caller.
pub(crate) fn write_and_remember<T>(
    app: &AppHandle,
    info: &SourceInfo,
    command: impl FnOnce(mpsc::Sender<Result<T, ApplyOutcome>>) -> Command,
    applied: &BTreeMap<String, i32>,
    on_refused: impl FnOnce(&ApplyOutcome),
) -> Result<T, ApplyError> {
    let answer = ask(app, command)?;
    offsets::remember_applied(&config_dir(app), info.app, &answer, applied);
    answered(answer, on_refused)
}

/// What Lightroom answered: the written value, or a refusal as the screen
/// reads it (`on_refused` sees the outcome first, once).
pub(crate) fn answered<T>(
    answer: Result<T, ApplyOutcome>,
    on_refused: impl FnOnce(&ApplyOutcome),
) -> Result<T, ApplyError> {
    answer.map_err(|outcome| {
        on_refused(&outcome);
        ApplyError::from(outcome)
    })
}
