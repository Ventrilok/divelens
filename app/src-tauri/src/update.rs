//! Updates of Dive Lens (spec F13).
//!
//! The app looks on GitHub Releases for a newer stable version (never a
//! pre-release) and, once the diver agrees, replaces the whole plug-in folder
//! it sits in: the app and the Lightroom plug-in together. The release
//! workflow publishes `latest.json` with each version, naming one file per
//! system and its signature, made with the Dive Lens update key; the
//! updater plugin downloads the file and refuses it unless the signature
//! matches the public key built into the app and names the version
//! announced.
//!
//! - macOS: the file is the plug-in folder zipped. It is unpacked next to
//!   the folder (same disk), checked, then swapped in by two renames; the
//!   app starts again from the new folder once this one has quit.
//! - Windows: a folder holding a running program cannot be renamed, so the
//!   file is the installer, run silently on the folder in place; it starts
//!   the new app when done. It also keeps the version shown in Windows'
//!   installed apps right.
//!
//! Nothing named `*.lrplugin` is ever left next to the folder: Lightroom
//! would load it as a second Dive Lens.

mod install;

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::dive_log::Writes;
use crate::files::config_dir;
use crate::settings::Settings;

pub(crate) use install::{AFTER_UPDATE, Copy, clean_up};

/// The project's releases page: the notes of each version, and where a copy
/// that cannot update itself is downloaded by hand.
pub const RELEASES_URL: &str = "https://github.com/Ventrilok/divelens/releases";
/// Replaces the address of `latest.json`, to try updates against a local
/// server. The signature is checked all the same.
pub const URL_VARIABLE: &str = "DIVELENS_UPDATE_URL";
/// The file keeping when updates were last looked for.
const STATE_FILE: &str = "update.json";
/// An automatic check happens at most this often.
pub const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// The name of this system in `latest.json`.
pub fn target() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("{os}-{}", std::env::consts::ARCH)
}

/// Whether `remote` is offered to a copy at `current`: newer, and a stable
/// version (a pre-release is never offered).
pub fn is_offered(current: &semver::Version, remote: &semver::Version) -> bool {
    remote.pre.is_empty() && remote > current
}

/// Where the notes of a version are.
pub fn notes_url(version: &str) -> String {
    format!("{RELEASES_URL}/tag/v{version}")
}

/// Seconds since 1970, as kept in the state file.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Whether an automatic check is due: never checked, checked a day ago or
/// more, or a last check in the future (the clock was changed).
pub fn due(last_check: Option<u64>, now: u64) -> bool {
    last_check.is_none_or(|last| last > now || now - last >= CHECK_EVERY.as_secs())
}

/// When updates were last looked for, and which version's notice the diver
/// closed: kept apart from the settings (never exported).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct State {
    /// Seconds since 1970.
    pub last_check: Option<u64>,
    /// The notice of this version was closed: it comes back for a newer one.
    pub dismissed: Option<String>,
}

impl State {
    pub fn load(dir: &Path) -> Self {
        crate::files::read_json(&dir.join(STATE_FILE))
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        crate::files::write_json(dir, STATE_FILE, self)
    }
}

/// Where the update stands, as Settings and the notice show it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Phase {
    /// Not looked for since the app started.
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available {
        version: String,
    },
    /// `percent`: unknown when the server does not give the size.
    Downloading {
        version: String,
        percent: Option<u8>,
    },
    /// Unpacking, checking and replacing the folder; the app then quits.
    Installing {
        version: String,
    },
    /// The last check or installation failed. `version`: the version that
    /// was being installed (none for a failed check).
    Failed {
        reason: Failure,
        version: Option<String>,
    },
}

/// Why a check or an installation failed (the details go to the log).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Failure {
    /// GitHub could not be reached, or answered something unusable.
    Network,
    /// The downloaded file does not carry the Dive Lens signature.
    Signature,
    /// The file is not a complete Dive Lens package.
    Package,
    /// The plug-in folder could not be replaced (the previous version was
    /// put back).
    Files,
    /// Lightroom is being written to: install once it is done.
    Writing,
    /// This copy cannot be updated in place.
    NotInstallable,
}

/// What the UI is told about updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub phase: Phase,
    /// The update can be installed here; otherwise Settings offers the
    /// releases page.
    pub installable: bool,
    /// Seconds since 1970.
    pub last_check: Option<u64>,
    pub dismissed: Option<String>,
    /// The notes of the version found (none found: `None`).
    pub notes_url: Option<String>,
    /// Where to download Dive Lens by hand.
    pub releases_url: &'static str,
}

/// Why the updater plugin's error happened, for the diver.
pub fn failure_of(error: &tauri_plugin_updater::Error) -> Failure {
    use tauri_plugin_updater::Error as E;
    match error {
        E::Minisign(_)
        | E::Base64(_)
        | E::SignatureUtf8(_)
        | E::SignedVersionMismatch { .. }
        | E::MissingSignedVersion => Failure::Signature,
        _ => Failure::Network,
    }
}

/// Updates (spec F13): what this copy is, where the update stands, and the
/// update the last check found, ready to download.
pub(crate) struct Updates {
    pub(crate) copy: Copy,
    pub(crate) phase: Mutex<Phase>,
    pub(crate) found: Mutex<Option<tauri_plugin_updater::Update>>,
}

fn update_status_of(app: &AppHandle) -> Status {
    let updates = app.state::<Updates>();
    let state = State::load(&config_dir(app));
    Status {
        phase: updates.phase.lock().expect("update phase").clone(),
        installable: updates.copy.installable(),
        last_check: state.last_check,
        dismissed: state.dismissed,
        notes_url: updates
            .found
            .lock()
            .expect("update")
            .as_ref()
            .map(|u| notes_url(&u.version)),
        releases_url: RELEASES_URL,
    }
}

fn set_phase(app: &AppHandle, phase: Phase) {
    *app.state::<Updates>().phase.lock().expect("update phase") = phase;
    let _ = app.emit("update-status", update_status_of(app));
}

/// Where the update stands; changes arrive as `update-status` events.
#[tauri::command]
pub(crate) fn update_status(app: AppHandle) -> Status {
    update_status_of(&app)
}

/// Asks GitHub for the latest stable version (`latest.json`, or the address
/// in `DIVELENS_UPDATE_URL`), as offered to this system.
async fn find_update(
    app: &AppHandle,
) -> Result<Option<tauri_plugin_updater::Update>, tauri_plugin_updater::Error> {
    let mut builder = app
        .updater_builder()
        .target(target())
        .version_comparator(|current, remote| is_offered(&current, &remote.version));
    if let Ok(url) = std::env::var(URL_VARIABLE) {
        log::info!("update: looking at {url}");
        builder = builder.endpoints(vec![url.parse()?])?;
    }
    builder.build()?.check().await
}

/// Looks for a new version. `automatic` (at launch, or when the setting is
/// turned on): only when checks are on, this copy is an installed one, and
/// the last check is a day old; a failure then shows nothing.
#[tauri::command]
pub(crate) async fn check_update(app: AppHandle, automatic: bool) {
    let dir = config_dir(&app);
    if automatic {
        let installed = app.state::<Updates>().copy.folder().is_some();
        let due = due(State::load(&dir).last_check, now());
        if !Settings::load(&dir).check_updates || !installed || !due {
            return;
        }
    }
    {
        let updates = app.state::<Updates>();
        let mut phase = updates.phase.lock().expect("update phase");
        if matches!(
            *phase,
            Phase::Checking | Phase::Downloading { .. } | Phase::Installing { .. }
        ) {
            return;
        }
        *phase = Phase::Checking;
    }
    let _ = app.emit("update-status", update_status_of(&app));
    match find_update(&app).await {
        Ok(found) => {
            let mut state = State::load(&dir);
            state.last_check = Some(now());
            if let Err(e) = state.save(&dir) {
                log::warn!("update: last check not saved: {e}");
            }
            let phase = match &found {
                Some(u) => {
                    log::info!("update: Dive Lens {} available", u.version);
                    Phase::Available {
                        version: u.version.clone(),
                    }
                }
                None => {
                    log::info!("update: up to date");
                    Phase::UpToDate
                }
            };
            *app.state::<Updates>().found.lock().expect("update") = found;
            set_phase(&app, phase);
        }
        Err(e) => {
            log::warn!("update: check failed: {e}");
            set_phase(
                &app,
                if automatic {
                    Phase::Idle
                } else {
                    Phase::Failed {
                        reason: failure_of(&e),
                        version: None,
                    }
                },
            );
        }
    }
}

/// The diver closed the notice of `version`: it comes back for a newer one.
#[tauri::command]
pub(crate) fn dismiss_update(app: AppHandle, version: String) {
    let dir = config_dir(&app);
    let mut state = State::load(&dir);
    state.dismissed = Some(version);
    if let Err(e) = state.save(&dir) {
        log::warn!("update: closed notice not saved: {e}");
    }
    let _ = app.emit("update-status", update_status_of(&app));
}

/// Downloads the update found, checks its signature, replaces the plug-in
/// folder and starts the new version; this app then quits. Never while
/// Lightroom is being written to, and no write starts meanwhile.
#[tauri::command]
pub(crate) async fn install_update(app: AppHandle) {
    let updates = app.state::<Updates>();
    let found = updates.found.lock().expect("update").clone();
    let Some(found) = found else {
        return;
    };
    let version = found.version.clone();
    let fail = |reason, detail: String| {
        log::error!("update: Dive Lens {version} not installed: {detail}");
        set_phase(
            &app,
            Phase::Failed {
                reason,
                version: Some(version.clone()),
            },
        );
    };
    let Some(folder) = updates.copy.folder().map(Path::to_path_buf) else {
        return fail(Failure::NotInstallable, format!("{:?}", updates.copy));
    };
    if let Err(e) = install::check_installable(&folder) {
        return fail(e.reason, e.detail);
    }
    if app.state::<Writes>().in_progress() {
        return fail(Failure::Writing, "Lightroom is being written to".into());
    }
    {
        let mut phase = updates.phase.lock().expect("update phase");
        if matches!(*phase, Phase::Downloading { .. } | Phase::Installing { .. }) {
            return;
        }
        *phase = Phase::Downloading {
            version: version.clone(),
            percent: None,
        };
    }
    let _ = app.emit("update-status", update_status_of(&app));
    log::info!(
        "update: downloading Dive Lens {version} from {}",
        found.download_url
    );
    let mut done: u64 = 0;
    let mut shown = None;
    let bytes = found
        .download(
            |chunk, total| {
                done += chunk as u64;
                let percent = total
                    .filter(|t| *t > 0)
                    .map(|t| (done * 100 / t).min(100) as u8);
                if percent != shown {
                    shown = percent;
                    set_phase(
                        &app,
                        Phase::Downloading {
                            version: version.clone(),
                            percent,
                        },
                    );
                }
            },
            || {},
        )
        .await;
    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(e) => return fail(failure_of(&e), e.to_string()),
    };
    set_phase(
        &app,
        Phase::Installing {
            version: version.clone(),
        },
    );
    let handle = app.clone();
    let installed = tauri::async_runtime::spawn_blocking(move || {
        let writes = handle.state::<Writes>();
        writes.exclusive(|writing| {
            if writing {
                return Err(install::InstallError {
                    reason: Failure::Writing,
                    detail: "Lightroom is being written to".into(),
                });
            }
            let new_app = install::install(&folder, &bytes, &found.version)?;
            log::info!(
                "update: Dive Lens {} installed in {}",
                found.version,
                folder.display()
            );
            if let Err(e) = install::relaunch(&new_app) {
                log::error!("update: new version not started: {e}");
            }
            handle.exit(0);
            // Holds off any write to Lightroom until this app has quit.
            std::thread::sleep(Duration::from_secs(30));
            Ok(())
        })
    })
    .await
    .expect("update task failed");
    if let Err(e) = installed {
        fail(e.reason, e.detail);
    }
}

#[cfg(test)]
mod test_support {
    use std::path::{Path, PathBuf};

    pub(super) fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("divelens-update-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A plug-in folder as the release workflow makes it, at `version`.
    pub(super) fn plugin(dir: &Path, name: &str, version: &str) -> PathBuf {
        let folder = dir.join(name);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("Info.lua"),
            format!(
                "return {{\n    LrToolkitIdentifier = \"com.querton.divelens\",\n    \
                 VERSION = {{ major = 2, minor = 1, revision = 0, display = \"{version}\" }},\n}}\n"
            ),
        )
        .unwrap();
        std::fs::write(folder.join("Dive Lens.exe"), version).unwrap();
        folder
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::temp_dir;
    use super::*;

    fn v(s: &str) -> semver::Version {
        semver::Version::parse(s).unwrap()
    }

    #[test]
    fn only_newer_stable_versions_are_offered() {
        assert!(is_offered(&v("2.0.0"), &v("2.0.1")));
        assert!(is_offered(&v("2.0.0"), &v("2.1.0")));
        assert!(is_offered(&v("0.1.0"), &v("2.0.0")));
        assert!(!is_offered(&v("2.1.0"), &v("2.1.0")));
        assert!(!is_offered(&v("2.1.0"), &v("2.0.9")));
        // Pre-releases are never offered, even to a pre-release.
        assert!(!is_offered(&v("2.0.0"), &v("2.1.0-beta.1")));
        assert!(!is_offered(&v("2.1.0-beta.1"), &v("2.1.0-beta.2")));
        // The stable version is offered to its own pre-releases.
        assert!(is_offered(&v("2.1.0-beta.2"), &v("2.1.0")));
        // Numbers compare as numbers.
        assert!(is_offered(&v("2.9.0"), &v("2.10.0")));
    }

    #[test]
    fn a_check_is_due_once_a_day() {
        let day = CHECK_EVERY.as_secs();
        assert!(due(None, 1_000_000));
        assert!(!due(Some(1_000_000), 1_000_000));
        assert!(!due(Some(1_000_000), 1_000_000 + day - 1));
        assert!(due(Some(1_000_000), 1_000_000 + day));
        // A clock set back: checked again rather than waiting.
        assert!(due(Some(1_000_000), 999_000));
    }

    #[test]
    fn names_this_system_as_latest_json_does() {
        let t = target();
        if cfg!(target_os = "macos") {
            assert!(t == "darwin-aarch64" || t == "darwin-x86_64", "{t}");
        } else if cfg!(windows) {
            assert!(t.starts_with("windows-"), "{t}");
        }
        assert_eq!(
            notes_url("2.1.0"),
            "https://github.com/Ventrilok/divelens/releases/tag/v2.1.0"
        );
    }

    #[test]
    fn keeps_the_last_check_and_the_closed_notice() {
        let dir = temp_dir("state");
        assert_eq!(State::load(&dir), State::default());
        let state = State {
            last_check: Some(1_700_000_000),
            dismissed: Some("2.1.0".into()),
        };
        state.save(&dir).unwrap();
        assert_eq!(State::load(&dir), state);
        std::fs::write(dir.join(STATE_FILE), "not json").unwrap();
        assert_eq!(State::load(&dir), State::default());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
