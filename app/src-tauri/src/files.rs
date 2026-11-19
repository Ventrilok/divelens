//! Small JSON files kept in the app's settings folder: one safe way to read
//! and write them.

use std::io;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

/// The app's settings folder.
pub(crate) fn config_dir(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().expect("no config folder")
}

/// Reads a JSON file; a missing or unreadable file gives the default.
pub(crate) fn read_json<T: serde::de::DeserializeOwned + Default>(path: &Path) -> T {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Writes `value` as `name` in `dir` (created if needed), through a
/// temporary file so a reader never sees a half-written file.
pub(crate) fn write_json<T: serde::Serialize>(dir: &Path, name: &str, value: &T) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{name}.tmp"));
    std::fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    std::fs::rename(tmp, dir.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Sample {
        n: u32,
        label: String,
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("divelens-files-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_or_invalid_file_reads_as_default() {
        let dir = temp_dir("default");
        let path = dir.join("a.json");
        assert_eq!(read_json::<Sample>(&path), Sample::default());
        std::fs::write(&path, "not json").unwrap();
        assert_eq!(read_json::<Sample>(&path), Sample::default());
    }

    #[test]
    fn write_creates_the_folder_and_leaves_no_temporary_file() {
        let dir = temp_dir("create").join("inner");
        let value = Sample {
            n: 3,
            label: "x".into(),
        };
        write_json(&dir, "a.json", &value).unwrap();
        assert_eq!(read_json::<Sample>(&dir.join("a.json")), value);
        assert!(!dir.join("a.json.tmp").exists());
    }

    #[test]
    fn written_json_is_pretty() {
        let dir = temp_dir("pretty");
        let value = Sample {
            n: 1,
            label: "y".into(),
        };
        write_json(&dir, "a.json", &value).unwrap();
        let text = std::fs::read_to_string(dir.join("a.json")).unwrap();
        assert_eq!(text, serde_json::to_string_pretty(&value).unwrap());
    }
}
