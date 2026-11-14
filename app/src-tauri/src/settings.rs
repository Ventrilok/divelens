//! App settings (spec F2), stored as one versioned JSON file in the app config
//! folder. Unknown fields are ignored and a missing or unreadable field takes
//! its default, so older and newer files both load without losing the rest
//! (spec F11: a file from another version stays usable).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use divelens_core::apps::{self, DiveApp, Os};
use divelens_core::macdive;
use divelens_core::tags::{DepthUnit, TempUnit, Units};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

pub const FILE_NAME: &str = "settings.json";
/// 4: one location per application, in `source.locations`.
const CURRENT_VERSION: u32 = 4;

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

/// The dive application chosen, and the log location chosen for each
/// application (spec F2: switching back finds the previous location again).
/// Applications are those of `divelens_core::apps`, named by their id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Id of the application chosen. Kept as it is when this computer lacks
    /// it (settings from a Mac on Windows) or this version does not know it:
    /// the application in use is then another one ([`Source::app`]).
    pub kind: String,
    /// Location chosen for each application, by id; an application without
    /// one uses its standard location. Those of applications this version
    /// does not know are kept.
    pub locations: BTreeMap<String, PathBuf>,
}

/// Where versions 1 to 3 of the file kept each application's location, one
/// field per application (`path`: version 2, when MacDive was the only
/// application). Read, never written; never grows: applications added later
/// only exist in `locations`.
const OLD_LOCATIONS: [(&str, &str); 3] = [
    ("macDivePath", "macDive"),
    ("path", "macDive"),
    ("subsurfacePath", "subsurface"),
];

impl Default for Source {
    fn default() -> Self {
        Source {
            kind: apps::in_use("", Os::current()).id.into(),
            locations: BTreeMap::new(),
        }
    }
}

/// Reads every version of the file. A value that does not read takes its
/// default without losing the others: a location that is not a path (`null`
/// included) is the standard one, the other locations stay.
impl<'de> Deserialize<'de> for Source {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde_json::Value;
        let value = Value::deserialize(d)?;
        let field = |name: &str| value.get(name);
        let kind = match field("kind").and_then(Value::as_str) {
            Some(kind) => kind.to_string(),
            None => Source::default().kind,
        };
        let mut locations: BTreeMap<String, PathBuf> = field("locations")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(id, path)| Some((id.clone(), PathBuf::from(path.as_str()?))))
            .collect();
        for (name, id) in OLD_LOCATIONS {
            if let Some(path) = field(name).and_then(Value::as_str) {
                locations.entry(id.into()).or_insert_with(|| path.into());
            }
        }
        Ok(Source { kind, locations })
    }
}

impl Source {
    /// Application in use on `os`: the one chosen, unless `os` lacks it or
    /// this version does not know it; then the first one available.
    pub fn app(&self, os: Os) -> &'static DiveApp {
        apps::in_use(&self.kind, os)
    }

    /// Location chosen for `app`; `None`: its standard location.
    pub fn location(&self, app: &DiveApp) -> Option<&PathBuf> {
        self.locations.get(app.id)
    }

    /// Chooses `app`'s location; `None`: its standard location.
    pub fn set_location(&mut self, app: &DiveApp, path: Option<PathBuf>) {
        match path {
            Some(path) => self.locations.insert(app.id.into(), path),
            None => self.locations.remove(app.id),
        };
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
    /// dive order. Never replaces a setting already chosen in the app. The
    /// previous plugin only read MacDive.
    pub fn take_over(&mut self, legacy: &Legacy, home: &Path, standard_path: &Path) {
        if self.legacy_imported {
            return;
        }
        self.legacy_imported = true;
        if self.source.location(&macdive::APP).is_none()
            && let Some(path) = legacy.mac_dive_path.as_deref().map(str::trim)
            && !path.is_empty()
        {
            let path = match path.strip_prefix("~/") {
                Some(rest) => home.join(rest),
                None => PathBuf::from(path),
            };
            if path != standard_path && path.is_file() {
                self.source.set_location(&macdive::APP, Some(path));
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
    use divelens_core::subsurface;

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
            s.source.location(&macdive::APP).map(PathBuf::as_path),
            Some(Path::new("/Dives/MacDive.sqlite"))
        );
        assert_eq!(s.collections.group, "Dive Lens");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn version_3_file_keeps_both_locations_and_is_saved_as_version_4() {
        let dir = dir("v3");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(FILE_NAME),
            r#"{"version":3,"source":{"kind":"subsurface","macDivePath":"/A.sqlite",
                "subsurfacePath":"/Logs/diver.ssrf"},"language":"fr"}"#,
        )
        .unwrap();
        let s = Settings::load(&dir);
        assert_eq!(s.source.kind, "subsurface");
        assert_eq!(
            s.source.locations,
            locations(&[("macDive", "/A.sqlite"), ("subsurface", "/Logs/diver.ssrf")])
        );
        // Saved in today's form: one list of locations, nothing else.
        Settings::update(&dir, |s| s.depth_to_altitude = true).unwrap();
        let file: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join(FILE_NAME)).unwrap()).unwrap();
        assert_eq!(file["version"], 4);
        assert_eq!(
            file["source"],
            serde_json::json!({"kind": "subsurface",
                "locations": {"macDive": "/A.sqlite", "subsurface": "/Logs/diver.ssrf"}})
        );
        assert_eq!(Settings::load(&dir).source, s.source);
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn locations(list: &[(&str, &str)]) -> BTreeMap<String, PathBuf> {
        list.iter()
            .map(|(id, p)| (id.to_string(), PathBuf::from(p)))
            .collect()
    }

    #[test]
    fn one_location_per_application() {
        let mut s: Settings = serde_json::from_str(
            r#"{"version":3,"source":{"kind":"subsurface","macDivePath":"/A.sqlite",
                "subsurfacePath":"/Logs/diver.ssrf"}}"#,
        )
        .unwrap();
        let (md, ss) = (&macdive::APP, &subsurface::APP);
        assert_eq!(s.source.location(md), Some(&PathBuf::from("/A.sqlite")));
        assert_eq!(
            s.source.location(ss),
            Some(&PathBuf::from("/Logs/diver.ssrf"))
        );
        s.source.set_location(ss, None);
        assert_eq!(s.source.location(ss), None);
        assert_eq!(s.source.location(md), Some(&PathBuf::from("/A.sqlite")));
        // The application in use on each system.
        assert_eq!(s.source.app(Os::MacOs), ss);
        assert_eq!(s.source.app(Os::Windows), ss);
        s.source.kind = "macDive".into();
        assert_eq!(s.source.app(Os::MacOs), md);
        assert_eq!(s.source.app(Os::Windows), ss, "no MacDive on Windows");
        assert_eq!(
            s.source.location(md),
            Some(&PathBuf::from("/A.sqlite")),
            "the MacDive location stays"
        );
        assert_eq!(
            Source::default().app(Os::current()).id,
            if cfg!(windows) {
                "subsurface"
            } else {
                "macDive"
            }
        );
        // A kind this version does not know keeps the locations.
        let s: Settings =
            serde_json::from_str(r#"{"source":{"kind":"blueDive","macDivePath":"/A.sqlite"}}"#)
                .unwrap();
        assert_eq!(s.source.app(Os::MacOs), md);
        assert_eq!(s.source.app(Os::Windows), ss);
        assert_eq!(s.source.location(md), Some(&PathBuf::from("/A.sqlite")));
    }

    #[test]
    fn each_location_reads_on_its_own() {
        // Standard (null), a newer version's application, a value that is
        // not a path: the other locations stay.
        let s: Settings = serde_json::from_str(
            r#"{"version":5,"source":{"kind":"blueDive","locations":{"macDive":null,
                "subsurface":"/Logs/diver.xml","blueDive":"/Logs/blue.db","oops":3}}}"#,
        )
        .unwrap();
        assert_eq!(
            s.source.locations,
            locations(&[
                ("blueDive", "/Logs/blue.db"),
                ("subsurface", "/Logs/diver.xml")
            ])
        );
        assert_eq!(s.source.kind, "blueDive", "kept for the newer version");
        // Kept through a save.
        let dir = dir("each-location");
        Settings::update(&dir, |t| *t = s.clone()).unwrap();
        assert_eq!(Settings::load(&dir).source, s.source);
        std::fs::remove_dir_all(dir).unwrap();
        // Locations not a list: none, the application stays.
        let s: Settings =
            serde_json::from_str(r#"{"source":{"kind":"subsurface","locations":"/x"}}"#).unwrap();
        assert_eq!(s.source.kind, "subsurface");
        assert!(s.source.locations.is_empty());
        // When both are there, the list wins over a field of versions 1 to 3.
        let s: Settings = serde_json::from_str(
            r#"{"source":{"kind":"macDive","macDivePath":"/Old.sqlite",
                "locations":{"macDive":"/New.sqlite"}}}"#,
        )
        .unwrap();
        assert_eq!(s.source.locations, locations(&[("macDive", "/New.sqlite")]));
        for (_, id) in OLD_LOCATIONS {
            assert!(apps::by_id(id).is_some(), "{id}");
        }
    }

    #[test]
    fn patch_changes_one_location() {
        let mut s: Settings = serde_json::from_str(
            r#"{"source":{"kind":"subsurface","locations":{"macDive":"/A.sqlite","subsurface":"/x.xml"}}}"#,
        )
        .unwrap();
        // As the Settings screen sends it: "Standard location" for MacDive.
        s.patch(&serde_json::json!({"source": {"kind": "subsurface",
            "locations": {"macDive": null, "subsurface": "/y.xml"}}}));
        assert_eq!(s.source.locations, locations(&[("subsurface", "/y.xml")]));
        s.patch(&serde_json::json!({"source": {"kind": "subsurface", "locations": {}}}));
        assert!(s.source.locations.is_empty());
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
                kind: "subsurface".into(),
                locations: locations(&[
                    ("macDive", "/Dives/MacDive.sqlite"),
                    ("subsurface", "/Dives/diver.ssrf"),
                ]),
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
            s.source.location(&macdive::APP).map(PathBuf::as_path),
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

    /// A file exported by the version before one list of locations (version
    /// 3, written by that version's export), imported now.
    #[test]
    fn imports_a_version_3_export() {
        let text = include_str!("../tests/fixtures/fictional-settings-export-v3.json");
        assert!(text.contains("\"macDivePath\""), "today's format: {text}");
        let s = Settings::from_export(text).unwrap();
        assert_eq!(
            s,
            Settings {
                legacy_imported: false,
                ..chosen()
            }
        );
        // Exported again: today's form, which reads back the same.
        let again = s.to_export();
        assert!(!again.contains("macDivePath"), "{again}");
        assert_eq!(Settings::from_export(&again).unwrap(), s);
        // On Windows, the MacDive location of a Mac's file stays saved.
        let mut on_mac = s.clone();
        on_mac.source.kind = "macDive".into();
        assert_eq!(on_mac.source.app(Os::Windows), &subsurface::APP);
        assert_eq!(
            on_mac.source.location(&macdive::APP),
            Some(&PathBuf::from("/Dives/MacDive.sqlite"))
        );
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
            s.source.location(&macdive::APP).map(PathBuf::as_path),
            Some(home.join("Dropbox/MacDive.sqlite").as_path())
        );
        assert_eq!(s.dive_sort, Some(DiveSort::Oldest));
        assert!(s.legacy_imported);
        // Done once: later values are ignored.
        s.source.set_location(&macdive::APP, None);
        s.take_over(
            &legacy(Some("/Other.sqlite"), Some("desc")),
            Path::new(HOME),
            Path::new(STANDARD),
        );
        assert_eq!(s.source.location(&macdive::APP), None);
        assert_eq!(s.dive_sort, Some(DiveSort::Oldest));
        std::fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn take_over_never_replaces_app_choices() {
        let mut s = Settings {
            source: Source {
                kind: "macDive".into(),
                locations: locations(&[("macDive", "/Chosen.sqlite")]),
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
            s.source.location(&macdive::APP).map(PathBuf::as_path),
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
            assert_eq!(s.source.location(&macdive::APP), None, "{path:?}");
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
