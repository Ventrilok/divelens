//! App settings, stored as one versioned JSON file in the app config folder.
//! The Settings screen will extend this; unknown fields are ignored and
//! missing ones take their default, so older and newer files both load.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "settings.json";
const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    #[default]
    MacDive,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Source {
    pub kind: SourceKind,
    /// `None` means the application's standard location.
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    pub source: Source,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            source: Source::default(),
        }
    }
}

impl Settings {
    /// Loads the settings; a missing or unreadable file gives the defaults.
    pub fn load(dir: &Path) -> Self {
        std::fs::read_to_string(dir.join(FILE_NAME))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join(format!("{FILE_NAME}.tmp"));
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, dir.join(FILE_NAME))
    }
}
