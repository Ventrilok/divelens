//! App settings (spec F2), stored as one versioned JSON file in the app config
//! folder. Unknown fields are ignored and a missing or unreadable field takes
//! its default, so older and newer files both load without losing the rest
//! (spec F11: a file from another version stays usable).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use divelens_core::tags::{DepthUnit, TempUnit, Units};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

pub const FILE_NAME: &str = "settings.json";
const CURRENT_VERSION: u32 = 3;

/// Name of the collection set "Create Collections" uses by default
/// (compatibility contract: never translated, never renamed).
pub const DEFAULT_GROUP: &str = "Dive Lens";

/// Serializes every read-modify-write of the settings file.
static LOCK: Mutex<()> = Mutex::new(());

/// A field that does not read (value from a newer version, wrong type) takes
/// its default instead of making the whole file unreadable.
fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = serde_json::Value::deserialize(d)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

/// Dive application whose log is read. MacDive exists on macOS only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    MacDive,
    Subsurface,
}

impl Default for SourceKind {
    fn default() -> Self {
        if cfg!(windows) {
            SourceKind::Subsurface
        } else {
            SourceKind::MacDive
        }
    }
}

/// The dive application and the log location chosen for each application
/// (spec F2: switching back finds the previous location again).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Source {
    #[serde(deserialize_with = "lenient")]
    pub kind: SourceKind,
    /// `None` means MacDive's standard location. Version 2 of the file
    /// called it `path` (MacDive was the only application).
    #[serde(deserialize_with = "lenient", alias = "path")]
    pub mac_dive_path: Option<PathBuf>,
    /// A file (XML) or a folder (git storage); `None`: Subsurface's standard
    /// file.
    #[serde(deserialize_with = "lenient")]
    pub subsurface_path: Option<PathBuf>,
}

impl SourceKind {
    /// The application's name, as written in the photos' Source field
    /// (never translated).
    pub fn name(self) -> &'static str {
        match self {
            SourceKind::MacDive => "MacDive",
            SourceKind::Subsurface => "Subsurface",
        }
    }
}

impl Source {
    /// Application in use: Subsurface on Windows, whatever the file says
    /// (settings copied from a Mac).
    pub fn kind(&self) -> SourceKind {
        if cfg!(windows) {
            SourceKind::Subsurface
        } else {
            self.kind
        }
    }

    /// Location chosen for `kind`; `None`: its standard location.
    pub fn path(&self, kind: SourceKind) -> Option<&PathBuf> {
        match kind {
            SourceKind::MacDive => self.mac_dive_path.as_ref(),
            SourceKind::Subsurface => self.subsurface_path.as_ref(),
        }
    }

    pub fn set_path(&mut self, kind: SourceKind, path: Option<PathBuf>) {
        match kind {
            SourceKind::MacDive => self.mac_dive_path = path,
            SourceKind::Subsurface => self.subsurface_path = path,
        }
    }
}

/// Depth unit: the dive application's, or forced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DepthChoice {
    #[default]
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "m")]
    Metres,
    #[serde(rename = "ft")]
    Feet,
}

/// Temperature unit: the dive application's, or forced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TempChoice {
    #[default]
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "c")]
    Celsius,
    #[serde(rename = "f")]
    Fahrenheit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UnitChoices {
    #[serde(deserialize_with = "lenient")]
    pub depth: DepthChoice,
    #[serde(deserialize_with = "lenient")]
    pub temp: TempChoice,
}

impl UnitChoices {
    /// Units in use, given those of the dive application.
    pub fn resolve(&self, source: Units) -> Units {
        Units {
            depth: match self.depth {
                DepthChoice::Source => source.depth,
                DepthChoice::Metres => DepthUnit::Metres,
                DepthChoice::Feet => DepthUnit::Feet,
            },
            temp: match self.temp {
                TempChoice::Source => source.temp,
                TempChoice::Celsius => TempUnit::Celsius,
                TempChoice::Fahrenheit => TempUnit::Fahrenheit,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    #[default]
    System,
    En,
    Fr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    #[default]
    Normal,
    Detailed,
}

/// Order of the dive lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiveSort {
    Recent,
    Oldest,
    Depth,
}

/// Where "Create Collections" puts the collections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CollectionsPlace {
    /// Names of the collection sets leading to the chosen one, from the
    /// catalog's root; empty for the root.
    #[serde(deserialize_with = "lenient")]
    pub parent: Vec<String>,
    /// Collection set grouping the collections inside `parent`, created when
    /// needed; empty: the collections go straight into `parent`.
    #[serde(deserialize_with = "lenient")]
    pub group: String,
}

impl Default for CollectionsPlace {
    fn default() -> Self {
        Self {
            parent: Vec::new(),
            group: DEFAULT_GROUP.into(),
        }
    }
}

impl CollectionsPlace {
    /// Group name as written: trimmed, `None` when empty.
    pub fn group_name(&self) -> Option<String> {
        let name = self.group.trim();
        (!name.is_empty()).then(|| name.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    #[serde(deserialize_with = "lenient")]
    pub source: Source,
    #[serde(deserialize_with = "lenient")]
    pub units: UnitChoices,
    #[serde(deserialize_with = "lenient")]
    pub language: Language,
    /// Copy each photo's depth to its GPS altitude (spec F7).
    #[serde(deserialize_with = "lenient")]
    pub depth_to_altitude: bool,
    #[serde(deserialize_with = "lenient")]
    pub collections: CollectionsPlace,
    #[serde(deserialize_with = "lenient")]
    pub log_level: LogLevel,
    /// Last order chosen on a dive list; `None` until the user (or the
    /// previous plugin's settings) chose one.
    #[serde(deserialize_with = "lenient")]
    pub dive_sort: Option<DiveSort>,
    /// The previous plugin's settings were looked at (done once).
    #[serde(deserialize_with = "lenient")]
    pub legacy_imported: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            source: Source::default(),
            units: UnitChoices::default(),
            language: Language::default(),
            depth_to_altitude: false,
            collections: CollectionsPlace::default(),
            log_level: LogLevel::default(),
            dive_sort: None,
            legacy_imported: false,
        }
    }
}

/// Why a settings file chosen for import (spec F11) cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
pub enum ImportError {
    /// The file cannot be read (gone, no permission, not text).
    Unreadable(String),
    /// The file reads, but does not hold Dive Lens settings.
    NotSettings,
    /// The imported settings could not be saved.
    NotSaved(String),
    /// They name another dive log while Lightroom is being written to:
    /// nothing was changed.
    WriteInProgress,
}

/// Settings of the previous plugin (Lightroom plug-in preferences), as the
/// plugin passes them on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Legacy {
    /// `MacDivePath`, possibly starting with `~`.
    pub mac_dive_path: Option<String>,
    /// `DisplayLogOrder`: `asc` or `desc`.
    pub display_log_order: Option<String>,
}

impl Settings {
    /// Loads the settings; a missing or unreadable file gives the defaults.
    pub fn load(dir: &Path) -> Self {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        Self::read(dir)
    }

    fn read(dir: &Path) -> Self {
        std::fs::read_to_string(dir.join(FILE_NAME))
            .ok()
            .and_then(|s| serde_json::from_str::<Self>(&s).ok())
            .map(|mut s| {
                s.version = CURRENT_VERSION;
                s
            })
            .unwrap_or_default()
    }

    fn write(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join(format!("{FILE_NAME}.tmp"));
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, dir.join(FILE_NAME))
    }

    /// Changes the settings on disk and returns the result. `change` sees
    /// the file's latest content; nothing is written when it leaves the
    /// settings as they were.
    pub fn update(dir: &Path, change: impl FnOnce(&mut Self)) -> std::io::Result<Self> {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let before = Self::read(dir);
        let mut after = before.clone();
        change(&mut after);
        if after != before {
            after.write(dir)?;
        }
        Ok(after)
    }

    /// Applies the fields of `patch` (a JSON object, as the Settings screen
    /// sends it) on top of these settings. Fields that do not read keep their
    /// value; `version` and `legacyImported` are never changed this way.
    pub fn patch(&mut self, patch: &serde_json::Value) {
        let Some(fields) = patch.as_object() else {
            return;
        };
        let mut current = serde_json::to_value(&*self).expect("settings serialize");
        let target = current.as_object_mut().expect("settings are an object");
        for (key, value) in fields {
            if key != "version" && key != "legacyImported" && target.contains_key(key) {
                target.insert(key.clone(), value.clone());
            }
        }
        if let Ok(next) = serde_json::from_value::<Self>(current) {
            *self = next;
        }
    }
    /// The settings as an export file (spec F11): the settings file itself,
    /// readable, without what only concerns this computer (whether the
    /// previous plugin's settings were taken over).
    pub fn to_export(&self) -> String {
        let mut value = serde_json::to_value(self).expect("settings serialize");
        if let Some(fields) = value.as_object_mut() {
            fields.remove("legacyImported");
        }
        serde_json::to_string_pretty(&value).expect("settings serialize")
    }

    /// Writes the export file at `path`.
    pub fn export(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_export())
    }

    /// Reads an export file, or a settings file copied from another
    /// computer, of any version: unknown fields are ignored and a field that
    /// does not read takes its default. A file without a version number is
    /// not Dive Lens settings.
    pub fn from_export(text: &str) -> Result<Self, ImportError> {
        let value: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|_| ImportError::NotSettings)?;
        if !value.get("version").is_some_and(serde_json::Value::is_u64) {
            return Err(ImportError::NotSettings);
        }
        let mut settings: Self =
            serde_json::from_value(value).map_err(|_| ImportError::NotSettings)?;
        settings.version = CURRENT_VERSION;
        settings.legacy_imported = false;
        Ok(settings)
    }

    /// Reads the export file at `path`.
    pub fn read_export(path: &Path) -> Result<Self, ImportError> {
        let text =
            std::fs::read_to_string(path).map_err(|e| ImportError::Unreadable(e.to_string()))?;
        Self::from_export(&text)
    }

    /// Replaces every setting with imported ones. The previous plugin's
    /// settings are then never taken over: the imported file is the user's
    /// choice, which the takeover must never replace.
    pub fn replace_with(&mut self, imported: Settings) {
        *self = Settings {
            version: CURRENT_VERSION,
            legacy_imported: true,
            ..imported
        };
    }

    /// Takes over the previous plugin's settings, once: its dive-log location
    /// (when it is not the standard one and the file is still there) and its
    /// dive order. Never replaces a setting already chosen in the app.
    pub fn take_over(&mut self, legacy: &Legacy, home: &Path, standard_path: &Path) {
        if self.legacy_imported {
            return;
        }
        self.legacy_imported = true;
        if self.source.mac_dive_path.is_none()
            && let Some(path) = legacy.mac_dive_path.as_deref().map(str::trim)
            && !path.is_empty()
        {
            let path = match path.strip_prefix("~/") {
                Some(rest) => home.join(rest),
                None => PathBuf::from(path),
            };
            if path != standard_path && path.is_file() {
                self.source.mac_dive_path = Some(path);
            }
        }
        if self.dive_sort.is_none() {
            self.dive_sort = match legacy.display_log_order.as_deref() {
                Some("asc") => Some(DiveSort::Oldest),
                Some("desc") => Some(DiveSort::Recent),
                _ => None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("divelens-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn defaults() {
        let s = Settings::default();
        assert_eq!(s.collections.parent, Vec::<String>::new());
        assert_eq!(s.collections.group, "Dive Lens");
        assert!(!s.depth_to_altitude);
        assert_eq!(s.units, UnitChoices::default());
        assert_eq!(s.language, Language::System);
        assert_eq!(s.log_level, LogLevel::Normal);
        assert_eq!(s.dive_sort, None);
    }

    #[test]
    fn missing_file_gives_defaults_and_save_round_trips() {
        let dir = dir("roundtrip");
        assert_eq!(Settings::load(&dir), Settings::default());
        let saved = Settings::update(&dir, |s| {
            s.units.depth = DepthChoice::Feet;
            s.language = Language::Fr;
            s.collections.parent = vec!["Trips".into(), "2026".into()];
            s.dive_sort = Some(DiveSort::Depth);
        })
        .unwrap();
        assert_eq!(Settings::load(&dir), saved);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn version_1_file_keeps_its_source() {
        let dir = dir("v1");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(FILE_NAME),
            r#"{"version":1,"source":{"kind":"macDive","path":"/Dives/MacDive.sqlite"}}"#,
        )
        .unwrap();
        let s = Settings::load(&dir);
        assert_eq!(s.version, CURRENT_VERSION);
        assert_eq!(
            s.source.mac_dive_path.as_deref(),
            Some(Path::new("/Dives/MacDive.sqlite"))
        );
        assert_eq!(s.collections.group, "Dive Lens");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn one_location_per_application() {
        let mut s: Settings = serde_json::from_str(
            r#"{"version":3,"source":{"kind":"subsurface","macDivePath":"/A.sqlite",
                "subsurfacePath":"/Logs/diver.ssrf"}}"#,
        )
        .unwrap();
        assert_eq!(
            s.source.path(SourceKind::MacDive),
            Some(&PathBuf::from("/A.sqlite"))
        );
        assert_eq!(
            s.source.path(SourceKind::Subsurface),
            Some(&PathBuf::from("/Logs/diver.ssrf"))
        );
        s.source.set_path(SourceKind::Subsurface, None);
        assert_eq!(s.source.path(SourceKind::Subsurface), None);
        assert_eq!(
            s.source.path(SourceKind::MacDive),
            Some(&PathBuf::from("/A.sqlite"))
        );
        if cfg!(windows) {
            assert_eq!(Source::default().kind(), SourceKind::Subsurface);
            s.source.kind = SourceKind::MacDive;
            assert_eq!(
                s.source.kind(),
                SourceKind::Subsurface,
                "no MacDive on Windows"
            );
        } else {
            assert_eq!(Source::default().kind(), SourceKind::MacDive);
            assert_eq!(s.source.kind(), SourceKind::Subsurface);
        }
        // A kind this version does not know keeps the locations.
        let s: Settings =
            serde_json::from_str(r#"{"source":{"kind":"blueDive","macDivePath":"/A.sqlite"}}"#)
                .unwrap();
        assert_eq!(s.source.kind, SourceKind::default());
        assert_eq!(s.source.mac_dive_path, Some(PathBuf::from("/A.sqlite")));
    }

    #[test]
    fn unknown_values_fall_back_field_by_field() {
        let s: Settings = serde_json::from_str(
            r#"{"version":9,"language":"de","units":{"depth":"fathoms","temp":"f"},
                "depthToAltitude":true,"logLevel":42,"futureSetting":{"a":1},
                "collections":{"parent":"oops","group":"Plongées"}}"#,
        )
        .unwrap();
        assert_eq!(s.language, Language::System);
        assert_eq!(s.units.depth, DepthChoice::Source);
        assert_eq!(s.units.temp, TempChoice::Fahrenheit);
        assert!(s.depth_to_altitude);
        assert_eq!(s.log_level, LogLevel::Normal);
        assert_eq!(s.collections.parent, Vec::<String>::new());
        assert_eq!(s.collections.group, "Plongées");
    }

    #[test]
    fn units_follow_the_dive_application_unless_forced() {
        let canadian = Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Celsius,
        };
        assert_eq!(UnitChoices::default().resolve(canadian), canadian);
        let forced = UnitChoices {
            depth: DepthChoice::Metres,
            temp: TempChoice::Fahrenheit,
        };
        assert_eq!(
            forced.resolve(canadian),
            Units {
                depth: DepthUnit::Metres,
                temp: TempUnit::Fahrenheit,
            }
        );
        let depth_only = UnitChoices {
            depth: DepthChoice::Metres,
            temp: TempChoice::Source,
        };
        assert_eq!(depth_only.resolve(canadian).temp, TempUnit::Celsius);
    }

    #[test]
    fn patch_changes_only_what_it_names() {
        let mut s = Settings {
            depth_to_altitude: true,
            ..Settings::default()
        };
        s.patch(&serde_json::json!({
            "units": {"depth": "ft", "temp": "source"},
            "collections": {"parent": ["Trips"], "group": ""},
            "legacyImported": true,
            "version": 99,
            "nonsense": 1,
        }));
        assert_eq!(s.units.depth, DepthChoice::Feet);
        assert_eq!(s.collections.parent, ["Trips"]);
        assert_eq!(s.collections.group_name(), None);
        assert!(s.depth_to_altitude, "untouched");
        assert!(!s.legacy_imported, "never patched");
        assert_eq!(s.version, CURRENT_VERSION);
        // A value that does not read is ignored, not a reset.
        s.patch(&serde_json::json!({"language": "klingon"}));
        assert_eq!(s.language, Language::System);
        s.patch(&serde_json::json!(["not", "an", "object"]));
        assert_eq!(s.units.depth, DepthChoice::Feet);
    }

    #[test]
    fn update_writes_only_on_change() {
        let dir = dir("nochange");
        Settings::update(&dir, |_| {}).unwrap();
        assert!(!dir.join(FILE_NAME).exists());
        Settings::update(&dir, |s| s.depth_to_altitude = true).unwrap();
        assert!(Settings::load(&dir).depth_to_altitude);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn group_name_is_trimmed() {
        let mut place = CollectionsPlace::default();
        assert_eq!(place.group_name().as_deref(), Some("Dive Lens"));
        place.group = "  Plongées 2026 ".into();
        assert_eq!(place.group_name().as_deref(), Some("Plongées 2026"));
        place.group = "   ".into();
        assert_eq!(place.group_name(), None);
    }

    fn chosen() -> Settings {
        Settings {
            source: Source {
                kind: SourceKind::Subsurface,
                mac_dive_path: Some("/Dives/MacDive.sqlite".into()),
                subsurface_path: Some("/Dives/diver.ssrf".into()),
            },
            units: UnitChoices {
                depth: DepthChoice::Feet,
                temp: TempChoice::Celsius,
            },
            language: Language::Fr,
            depth_to_altitude: true,
            collections: CollectionsPlace {
                parent: vec!["Trips".into(), "2026".into()],
                group: "Plongées".into(),
            },
            log_level: LogLevel::Detailed,
            dive_sort: Some(DiveSort::Depth),
            legacy_imported: true,
            ..Settings::default()
        }
    }

    #[test]
    fn export_round_trips_without_machine_state() {
        let dir = dir("export");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("Dive Lens settings.json");
        let settings = chosen();
        settings.export(&file).unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("\"version\""), "readable, versioned: {text}");
        assert!(!text.contains("legacyImported"), "{text}");
        let read = Settings::read_export(&file).unwrap();
        assert_eq!(
            read,
            Settings {
                legacy_imported: false,
                ..settings.clone()
            }
        );
        // Imported, the settings are the file's, and the previous plugin's
        // settings are never taken over afterwards.
        let mut current = Settings::default();
        current.replace_with(read);
        assert_eq!(current, settings);
        current.take_over(
            &legacy(Some("/Other.sqlite"), Some("asc")),
            Path::new(HOME),
            Path::new(STANDARD),
        );
        assert_eq!(current, settings);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn imports_older_and_newer_files() {
        // Version 2: MacDive was the only application, its location `path`.
        let s = Settings::from_export(
            r#"{"version":2,"source":{"kind":"macDive","path":"/Dives/MacDive.sqlite"},
                "units":{"depth":"ft","temp":"source"},"legacyImported":true}"#,
        )
        .unwrap();
        assert_eq!(s.version, CURRENT_VERSION);
        assert_eq!(
            s.source.mac_dive_path.as_deref(),
            Some(Path::new("/Dives/MacDive.sqlite"))
        );
        assert_eq!(s.units.depth, DepthChoice::Feet);
        assert_eq!(s.collections, CollectionsPlace::default());
        assert!(!s.legacy_imported, "machine state is never imported");
        // A newer version: unknown settings are ignored, the rest is kept.
        let s = Settings::from_export(
            r#"{"version":7,"language":"fr","trips":{"on":true},"logLevel":"verbose"}"#,
        )
        .unwrap();
        assert_eq!(s.language, Language::Fr);
        assert_eq!(s.log_level, LogLevel::Normal);
        // A byte-order mark (file saved by a Windows editor) is fine.
        assert!(Settings::from_export("\u{feff}{\"version\":3}").is_ok());
    }

    #[test]
    fn refuses_what_is_not_settings() {
        for text in [
            "",
            "not json",
            "[1, 2]",
            r#"{"name":"divelens-app"}"#,
            r#"{"version":"3"}"#,
        ] {
            assert_eq!(
                Settings::from_export(text),
                Err(ImportError::NotSettings),
                "{text}"
            );
        }
        let gone = dir("gone").join("settings.json");
        assert!(matches!(
            Settings::read_export(&gone),
            Err(ImportError::Unreadable(_))
        ));
    }

    const HOME: &str = "/Users/diver";
    const STANDARD: &str = "/Users/diver/Library/Application Support/MacDive/MacDive.sqlite";

    fn legacy(path: Option<&str>, order: Option<&str>) -> Legacy {
        Legacy {
            mac_dive_path: path.map(Into::into),
            display_log_order: order.map(Into::into),
        }
    }

    #[test]
    fn takes_over_the_previous_plugin_once() {
        let home = dir("legacy-home");
        std::fs::create_dir_all(home.join("Dropbox")).unwrap();
        std::fs::write(home.join("Dropbox/MacDive.sqlite"), b"").unwrap();
        let mut s = Settings::default();
        s.take_over(
            &legacy(Some("~/Dropbox/MacDive.sqlite"), Some("asc")),
            &home,
            Path::new(STANDARD),
        );
        assert_eq!(
            s.source.mac_dive_path.as_deref(),
            Some(home.join("Dropbox/MacDive.sqlite").as_path())
        );
        assert_eq!(s.dive_sort, Some(DiveSort::Oldest));
        assert!(s.legacy_imported);
        // Done once: later values are ignored.
        s.source.mac_dive_path = None;
        s.take_over(
            &legacy(Some("/Other.sqlite"), Some("desc")),
            Path::new(HOME),
            Path::new(STANDARD),
        );
        assert_eq!(s.source.mac_dive_path, None);
        assert_eq!(s.dive_sort, Some(DiveSort::Oldest));
        std::fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn take_over_never_replaces_app_choices() {
        let mut s = Settings {
            source: Source {
                kind: SourceKind::MacDive,
                mac_dive_path: Some("/Chosen.sqlite".into()),
                subsurface_path: None,
            },
            dive_sort: Some(DiveSort::Depth),
            ..Settings::default()
        };
        s.take_over(
            &legacy(Some("/Old.sqlite"), Some("asc")),
            Path::new(HOME),
            Path::new(STANDARD),
        );
        assert_eq!(
            s.source.mac_dive_path.as_deref(),
            Some(Path::new("/Chosen.sqlite"))
        );
        assert_eq!(s.dive_sort, Some(DiveSort::Depth));
    }

    #[test]
    fn take_over_ignores_standard_and_empty_values() {
        for (path, order) in [
            (
                Some("~/Library/Application Support/MacDive/MacDive.sqlite"),
                Some("desc"),
            ),
            (Some(STANDARD), Some("sideways")),
            (Some("  "), None),
            (Some("/Gone/MacDive.sqlite"), None),
            (None, None),
        ] {
            let mut s = Settings::default();
            s.take_over(&legacy(path, order), Path::new(HOME), Path::new(STANDARD));
            assert_eq!(s.source.mac_dive_path, None, "{path:?}");
            assert!(s.legacy_imported);
        }
        let mut s = Settings::default();
        s.take_over(
            &legacy(None, Some("desc")),
            Path::new(HOME),
            Path::new(STANDARD),
        );
        assert_eq!(s.dive_sort, Some(DiveSort::Recent));
    }
}
