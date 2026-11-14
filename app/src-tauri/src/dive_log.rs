//! Which dive log the app works on: the one configured in Settings, one at a
//! time (spec F2). Every action names the log its screen shows and is
//! refused when that is no longer the configured one; the log cannot change
//! while a write to Lightroom is under way.
//!
//! A log is identified by its application and location only, never by its
//! content or modification time: MacDive saves its database often, and a
//! finer check would refuse actions on the very log the screen shows.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use divelens_core::apps::{self, DiveApp, Os};
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

/// The dive log some settings point at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    /// The application in use, written as its id.
    #[serde(rename = "kind", with = "apps::id")]
    pub app: &'static DiveApp,
    pub path: PathBuf,
    pub is_default_path: bool,
}

impl SourceInfo {
    /// The log `settings` point at on `os`: the application in use there and
    /// its location (`standard` gives an application's standard one).
    pub fn of(
        settings: &Settings,
        os: Os,
        standard: impl FnOnce(&'static DiveApp) -> PathBuf,
    ) -> Self {
        let app = settings.source.app(os);
        match settings.source.location(app) {
            Some(path) => SourceInfo {
                app,
                path: path.clone(),
                is_default_path: false,
            },
            None => SourceInfo {
                app,
                path: standard(app),
                is_default_path: true,
            },
        }
    }

    pub fn id(&self) -> LogId {
        LogId {
            app: self.app,
            path: self.path.clone(),
        }
    }
}

/// The dive log configured in the settings folder `dir`, as every action
/// checks it on `os`. Only the settings are read, never the log itself.
pub fn configured(
    dir: &Path,
    os: Os,
    standard: impl FnOnce(&'static DiveApp) -> PathBuf,
) -> SourceInfo {
    SourceInfo::of(&Settings::load(dir), os, standard)
}

/// A dive log by application and location, as a screen names the log it
/// shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogId {
    /// The application, written as its id.
    #[serde(rename = "kind", with = "apps::id")]
    pub app: &'static DiveApp,
    pub path: PathBuf,
}

/// The dive log a screen showed is no longer the configured one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogChanged;

impl LogId {
    /// Refuses unless this is the configured log.
    pub fn check(&self, configured: &SourceInfo) -> Result<(), LogChanged> {
        if self.app == configured.app && self.path == configured.path {
            Ok(())
        } else {
            Err(LogChanged)
        }
    }
}

/// A change of dive log asked for while Lightroom is being written to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteInProgress;

/// Applies `change` to `settings`, unless it switches the dive log while a
/// write is under way (`writing`): the settings then stay exactly as they
/// were (an import is refused whole, never half applied). `log_of` tells
/// which log some settings point at.
pub fn guarded_change(
    writing: bool,
    settings: &mut Settings,
    change: impl FnOnce(&mut Settings),
    log_of: impl Fn(&Settings) -> LogId,
) -> Result<(), WriteInProgress> {
    let before = settings.clone();
    change(settings);
    if writing && log_of(&before) != log_of(settings) {
        *settings = before;
        return Err(WriteInProgress);
    }
    Ok(())
}

/// Writes to Lightroom under way. Starting a write and changing the dive log
/// exclude each other, so a write always works on the log its screen showed.
pub struct Writes {
    count: Mutex<usize>,
    /// Told when the first write starts (`true`) and the last one ends.
    notify: Box<dyn Fn(bool) + Send + Sync>,
}

/// A write in progress; it ends when dropped, whatever happened.
pub struct Writing<'a>(&'a Writes);

impl Writes {
    pub fn new(notify: impl Fn(bool) + Send + Sync + 'static) -> Self {
        Self {
            count: Mutex::new(0),
            notify: Box::new(notify),
        }
    }

    /// Starts a write on `named`, unless it is no longer the configured log.
    /// `configured` is read once no change of log can happen any more, and
    /// returned: the write works on that one.
    pub fn begin(
        &self,
        named: &LogId,
        configured: impl FnOnce() -> SourceInfo,
    ) -> Result<(Writing<'_>, SourceInfo), LogChanged> {
        let mut count = self.count.lock().unwrap_or_else(|e| e.into_inner());
        let info = configured();
        named.check(&info)?;
        *count += 1;
        if *count == 1 {
            (self.notify)(true);
        }
        Ok((Writing(self), info))
    }

    pub fn in_progress(&self) -> bool {
        *self.count.lock().unwrap_or_else(|e| e.into_inner()) > 0
    }

    /// Runs a settings change knowing whether a write is under way; no write
    /// starts until it is done.
    pub fn exclusive<T>(&self, change: impl FnOnce(bool) -> T) -> T {
        let count = self.count.lock().unwrap_or_else(|e| e.into_inner());
        change(*count > 0)
    }
}

impl Drop for Writing<'_> {
    fn drop(&mut self) {
        let mut count = self.0.count.lock().unwrap_or_else(|e| e.into_inner());
        *count = count.saturating_sub(1);
        if *count == 0 {
            (self.0.notify)(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::DepthChoice;
    use divelens_core::{macdive, subsurface};
    use std::path::Path;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    /// The system the tests run as: both applications are available.
    const OS: Os = Os::MacOs;

    fn standard(app: &DiveApp) -> PathBuf {
        match app.id {
            "macDive" => "/Users/diver/Library/MacDive/MacDive.sqlite".into(),
            _ => "/Users/diver/Subsurface/diver.xml".into(),
        }
    }

    fn log_of(s: &Settings) -> LogId {
        SourceInfo::of(s, OS, standard).id()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("divelens-divelog-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn on(app: &DiveApp, path: &Path) -> Settings {
        let mut s = Settings::default();
        s.source.kind = app.id.into();
        s.source.set_location(app, Some(path.to_path_buf()));
        s
    }

    #[test]
    fn standard_location_when_none_is_chosen() {
        let mut s = Settings::default();
        s.source.kind = "subsurface".into();
        let info = SourceInfo::of(&s, OS, standard);
        assert_eq!(info.app, &subsurface::APP);
        assert_eq!(info.path, standard(&subsurface::APP));
        assert!(info.is_default_path);
        // The standard file chosen by hand is the same log.
        s.source
            .set_location(&subsurface::APP, Some(standard(&subsurface::APP)));
        assert!(info.id().check(&SourceInfo::of(&s, OS, standard)).is_ok());
    }

    /// What the actions do: a read (profile, photo search) checks the log
    /// named against the configured one, a write starts through `begin`.
    fn accepted(writes: &Writes, shown: &LogId, settings_dir: &Path) -> bool {
        let read = shown.check(&configured(settings_dir, OS, standard)).is_ok();
        let write = writes
            .begin(shown, || configured(settings_dir, OS, standard))
            .is_ok();
        assert_eq!(read, write);
        read
    }

    #[test]
    fn the_log_shown_stays_accepted_when_its_file_is_saved_again() {
        let dir = temp_dir("same");
        let settings_dir = dir.join("settings");
        let file = dir.join("diver.xml");
        std::fs::write(&file, "<divelog/>").unwrap();
        Settings::update(&settings_dir, |s| {
            *s = on(&subsurface::APP, &file);
        })
        .unwrap();
        let writes = Writes::new(|_| {});
        let shown = configured(&settings_dir, OS, standard).id();
        assert!(accepted(&writes, &shown, &settings_dir));

        // The dive application saves the log again: new content, new
        // modification time. Only application and location count.
        std::fs::write(&file, "<divelog><dives/></divelog>").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(SystemTime::now() + Duration::from_secs(3600))
            .unwrap();
        assert!(accepted(&writes, &shown, &settings_dir));

        // Another file chosen in Settings: refused.
        Settings::update(&settings_dir, |s| {
            s.source
                .set_location(&subsurface::APP, Some(dir.join("other.xml")));
        })
        .unwrap();
        assert!(!accepted(&writes, &shown, &settings_dir));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn another_log_or_application_is_refused() {
        let dir = temp_dir("other");
        let a = dir.join("a.xml");
        let b = dir.join("b.xml");
        let shown = SourceInfo::of(&on(&subsurface::APP, &a), OS, standard).id();
        let elsewhere = SourceInfo::of(&on(&subsurface::APP, &b), OS, standard);
        assert_eq!(shown.check(&elsewhere), Err(LogChanged));
        // Same location, other application (MacDive exists on macOS only).
        let mut other_app = on(&macdive::APP, &a);
        assert_eq!(
            shown.check(&SourceInfo::of(&other_app, Os::MacOs, standard)),
            Err(LogChanged)
        );
        // On Windows, the same settings read Subsurface's location.
        other_app
            .source
            .set_location(&subsurface::APP, Some(a.clone()));
        assert!(
            shown
                .check(&SourceInfo::of(&other_app, Os::Windows, standard))
                .is_ok()
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_log_cannot_change_during_a_write() {
        let start = on(&subsurface::APP, Path::new("/Logs/a.xml"));
        let to_b = |s: &mut Settings| {
            s.source
                .set_location(&subsurface::APP, Some("/Logs/b.xml".into()))
        };
        let to_feet = |s: &mut Settings| s.units.depth = DepthChoice::Feet;

        let mut s = start.clone();
        assert_eq!(
            guarded_change(true, &mut s, to_b, log_of),
            Err(WriteInProgress)
        );
        assert_eq!(s, start, "nothing changed");
        // Other settings may change during a write.
        assert_eq!(guarded_change(true, &mut s, to_feet, log_of), Ok(()));
        assert_eq!(s.units.depth, DepthChoice::Feet);
        // Without a write, the log changes.
        assert_eq!(guarded_change(false, &mut s, to_b, log_of), Ok(()));
        assert_eq!(
            s.source.location(&subsurface::APP),
            Some(&PathBuf::from("/Logs/b.xml"))
        );
    }

    #[test]
    fn an_import_naming_another_log_is_refused_whole_during_a_write() {
        let start = on(&subsurface::APP, Path::new("/Logs/a.xml"));
        let mut imported = on(&subsurface::APP, Path::new("/Logs/b.xml"));
        imported.units.depth = DepthChoice::Feet;
        let mut s = start.clone();
        let file = imported.clone();
        assert_eq!(
            guarded_change(true, &mut s, |s| s.replace_with(file), log_of),
            Err(WriteInProgress)
        );
        assert_eq!(s, start, "units untouched too");
        // An import keeping the same log goes through.
        let mut same = start.clone();
        same.units.depth = DepthChoice::Feet;
        assert_eq!(
            guarded_change(true, &mut s, |s| s.replace_with(same), log_of),
            Ok(())
        );
        assert_eq!(s.units.depth, DepthChoice::Feet);
    }

    #[test]
    fn a_write_starts_on_the_configured_log_only_and_blocks_changes() {
        let told = Arc::new(Mutex::new(Vec::new()));
        let heard = told.clone();
        let writes = Writes::new(move |on| heard.lock().unwrap().push(on));
        let a = SourceInfo::of(&on(&subsurface::APP, Path::new("/a.xml")), OS, standard);
        let b = SourceInfo::of(&on(&subsurface::APP, Path::new("/b.xml")), OS, standard);

        assert!(writes.begin(&a.id(), || b.clone()).is_err(), "stale log");
        assert!(!writes.in_progress());

        let (first, info) = writes.begin(&a.id(), || a.clone()).unwrap();
        assert_eq!(info, a);
        let (second, _) = writes.begin(&a.id(), || a.clone()).unwrap();
        assert!(writes.in_progress());
        assert!(writes.exclusive(|writing| writing));
        drop(first);
        assert!(writes.exclusive(|writing| writing), "one write left");
        drop(second);
        assert!(!writes.exclusive(|writing| writing));
        assert_eq!(*told.lock().unwrap(), [true, false]);
    }
}
