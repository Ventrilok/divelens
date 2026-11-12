//! The app's log file (spec F10), written next to the plugin's own log in
//! Lightroom's log folder, so "Show log" opens one folder holding both.
//!
//! Lightroom writes the plugin's `DiveLens.log` to a fixed place (Lightroom
//! SDK, LrLogger): `~/Library/Logs/Adobe/Lightroom/LrClassicLogs` on a Mac,
//! `%LOCALAPPDATA%\Adobe\Lightroom\Logs\LrClassicLogs` on Windows.
//!
//! The file is kept small: past [`MAX_BYTES`] it becomes the "previous" log
//! (replacing the one before) and a new one starts.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record};

use crate::settings::LogLevel;

pub const FILE_NAME: &str = "DiveLens App.log";
const PREVIOUS: &str = "DiveLens App (previous).log";
const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// Lightroom's log folder, where the plugin's log is.
pub fn folder(home: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let local = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| home.join("AppData").join("Local"));
        local
            .join("Adobe")
            .join("Lightroom")
            .join("Logs")
            .join("LrClassicLogs")
    }
    #[cfg(not(windows))]
    {
        home.join("Library/Logs/Adobe/Lightroom/LrClassicLogs")
    }
}

pub fn level_filter(level: LogLevel) -> LevelFilter {
    match level {
        LogLevel::Normal => LevelFilter::Info,
        LogLevel::Detailed => LevelFilter::Debug,
    }
}

/// Changes what gets written from now on.
pub fn set_level(level: LogLevel) {
    log::set_max_level(level_filter(level));
}

struct FileLog {
    dir: PathBuf,
    file: Mutex<Option<File>>,
}

impl FileLog {
    fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    fn open(&self) -> Option<File> {
        std::fs::create_dir_all(&self.dir).ok()?;
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path())
            .ok()
    }

    fn rotate_if_big(&self, file: &mut Option<File>) {
        let big = file
            .as_ref()
            .and_then(|f| f.metadata().ok())
            .is_some_and(|m| m.len() > MAX_BYTES);
        if big {
            *file = None;
            let _ = std::fs::rename(self.path(), self.dir.join(PREVIOUS));
        }
    }
}

impl Log for FileLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Only DiveLens's own messages: libraries stay out of the user's log.
        metadata.target().starts_with("divelens") && metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        );
        let mut file = self.file.lock().unwrap_or_else(|e| e.into_inner());
        self.rotate_if_big(&mut file);
        if file.is_none() {
            *file = self.open();
        }
        if let Some(f) = file.as_mut()
            && f.write_all(line.as_bytes()).is_err()
        {
            *file = None; // folder removed meanwhile: reopen next time
        }
        if cfg!(debug_assertions) {
            eprint!("{line}");
        }
    }

    fn flush(&self) {
        if let Some(f) = self.file.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            let _ = f.flush();
        }
    }
}

/// Starts writing the log in `dir`; returns the log file's path.
pub fn start(dir: PathBuf, level: LogLevel) -> PathBuf {
    let logger = FileLog {
        dir,
        file: Mutex::new(None),
    };
    let path = logger.path();
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        set_level(level);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_rotates() {
        let dir = std::env::temp_dir().join(format!("divelens-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let log = FileLog {
            dir: dir.clone(),
            file: Mutex::new(None),
        };
        log::set_max_level(LevelFilter::Info);
        let record = |level, text: &str| {
            log.log(
                &Record::builder()
                    .level(level)
                    .target("divelens_lib::test")
                    .args(format_args!("{text}"))
                    .build(),
            )
        };
        record(log::Level::Info, "first");
        record(log::Level::Debug, "hidden");
        log.log(
            &Record::builder()
                .level(log::Level::Warn)
                .target("tao::platform")
                .args(format_args!("library"))
                .build(),
        );
        log.flush();
        let text = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap();
        assert!(text.contains("INFO  first"), "{text}");
        assert!(!text.contains("hidden") && !text.contains("library"));
        // Past the size limit, the log starts afresh and keeps one previous.
        std::fs::write(dir.join(FILE_NAME), vec![b'x'; MAX_BYTES as usize + 1]).unwrap();
        *log.file.lock().unwrap() = None;
        record(log::Level::Info, "opens big file");
        record(log::Level::Info, "after rotation");
        let text = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap();
        assert!(text.contains("after rotation") && !text.contains('x'));
        assert!(dir.join(PREVIOUS).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn levels() {
        assert_eq!(level_filter(LogLevel::Normal), LevelFilter::Info);
        assert_eq!(level_filter(LogLevel::Detailed), LevelFilter::Debug);
    }

    #[cfg(not(windows))]
    #[test]
    fn next_to_the_plugin_log() {
        assert_eq!(
            folder(Path::new("/Users/diver")),
            Path::new("/Users/diver/Library/Logs/Adobe/Lightroom/LrClassicLogs")
        );
    }
}
