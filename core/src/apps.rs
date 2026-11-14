//! The dive applications Dive Lens reads, each described once by a
//! [`DiveApp`] next to its reader: its name, where its log is, how the diver
//! chooses it, how to read its unit preferences, the texts the screens show
//! for it, in English and French. The app builds its settings, application
//! list, file chooser and dive-log error screen from these descriptions: it
//! has no code of its own for any application.
//!
//! To add an application, write its reader and its description (`APP`, in
//! `core/src/<application>/app.rs`, with its runs of the common tests), and
//! add it to [`ALL`]. The tests of this module check every description is
//! complete, in both languages; those of `core/src/conformance.rs` run every
//! registered reader on the reference log.

use std::any::Any;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Serializer};

use crate::model::DiveSource;
use crate::profile::Sample;
use crate::tags::Units;
use crate::{Capabilities, Dive, Result, Site};

/// Every application Dive Lens reads, in the order Settings lists them. On
/// each system, the first one available is the one used by default.
pub static ALL: &[&DiveApp] = &[&crate::macdive::APP, &crate::subsurface::APP];

/// A dive application, as Dive Lens knows it besides its reader.
pub struct DiveApp {
    /// Identifier in the settings and between the app's screens and its
    /// back end (`macDive`). Never changes: settings files keep it.
    pub id: &'static str,
    /// The application's name, as Dive Lens shows it and writes it in the
    /// photos' Source field. Never translated, never changed.
    pub name: &'static str,
    /// The systems the application runs on.
    pub systems: &'static [Os],
    /// Where the application keeps its log unless the diver moved it;
    /// `None` when it cannot be told (no home folder, say).
    pub standard_location: fn(&Places) -> Option<PathBuf>,
    /// What the diver chooses as a log: a file, a folder, or either.
    pub choose: Choose,
    /// Opens a log read-only, for one request of the app. A reader that
    /// reads the whole log on open may keep it in `kept` between requests.
    pub open: fn(&Path, &Kept) -> Result<Opened>,
    /// Where the application keeps its unit preferences, and what they mean.
    pub preferences: Preferences,
    /// File of the clock offsets remembered for this application, in the
    /// app's settings folder. Never changes: it holds the diver's offsets.
    pub offsets_file: &'static str,
    /// What the screens say of this application, in each language.
    pub texts: Localized<Texts>,
    /// Its runs of the common tests (`core/src/conformance.rs`): one per
    /// storage format it reads, at least one.
    #[cfg(test)]
    pub(crate) conformance: fn() -> Vec<crate::conformance::Run>,
}

/// A system Dive Lens runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
}

impl Os {
    /// The system this is running on. Dive Lens runs on macOS and Windows;
    /// any other system (development only) is taken as macOS.
    pub fn current() -> Os {
        if cfg!(windows) {
            Os::Windows
        } else {
            Os::MacOs
        }
    }
}

/// Where things are on this computer, for [`DiveApp::standard_location`].
#[derive(Debug, Clone, Copy)]
pub struct Places<'a> {
    pub os: Os,
    /// The diver's home folder.
    pub home: Option<&'a Path>,
    /// Where applications keep their data: `~/Library/Application Support`
    /// on macOS, `%APPDATA%` on Windows.
    pub data: Option<&'a Path>,
    /// The diver's login name.
    pub user: &'a str,
}

/// What the diver chooses as a log.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choose {
    /// Extensions of the files the log may be (without the dot); `None`:
    /// never a file.
    pub file: Option<&'static [&'static str]>,
    /// The log may be a folder.
    pub folder: bool,
}

/// A log opened by its application's reader.
pub struct Opened {
    pub log: Box<dyn DiveSource>,
    /// How the log is stored: a key of the application's [`Texts::read`]
    /// (Subsurface: `xml` or `git`), what Settings says once it is read.
    pub format: &'static str,
}

/// Where an application keeps its unit preferences, for Settings › units
/// "as the dive application" (spec F2). Only read, never written.
pub struct Preferences {
    /// macOS: the application's preference domain, read from its property
    /// list in `~/Library/Preferences` or in its sandbox container. `None`:
    /// none on macOS.
    pub mac_domain: Option<&'static str>,
    /// Windows: the application's registry key, under `HKEY_CURRENT_USER`.
    /// `None`: none on Windows.
    pub windows_key: Option<&'static str>,
    /// The units the application shows, from its preference values:
    /// `value("Group/key")` (Qt's notation; `value("key")` without a group)
    /// gives a number when it is set. Used once the preferences are found;
    /// without them, Dive Lens uses metres and °C.
    pub units: fn(&PreferenceValue<'_>) -> Units,
}

/// A preference value by key, as [`Preferences::units`] reads them.
pub type PreferenceValue<'a> = dyn Fn(&str) -> Option<i64> + 'a;

/// Something said in each language of the app.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Localized<T> {
    pub en: T,
    pub fr: T,
}

impl<T> Localized<T> {
    /// Each language, with its code.
    pub fn each(&self) -> [(&'static str, &T); 2] {
        [("en", &self.en), ("fr", &self.fr)]
    }
}

/// What the screens say of an application, in one language.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Texts {
    /// Settings: what the log is called ("MacDive database"), the title of
    /// its location.
    pub log: &'static str,
    /// Settings: the hint next to it.
    pub log_hint: &'static str,
    /// Settings, while the log is read.
    pub reading: &'static str,
    /// Settings, once the log is read: one line per format the reader
    /// reports ([`Opened::format`]), with `{dives}` and `{sites}` for the
    /// counts.
    #[serde(serialize_with = "as_map")]
    pub read: &'static [(&'static str, &'static str)],
    /// File chooser: the kind of file shown.
    pub file_type: &'static str,
    /// Dive-log error screen: the button choosing the log elsewhere, when
    /// it is a file or a folder but not either; `None`: "Choose…".
    pub locate: Option<&'static str>,
    /// What the screens say of a reader error, by its kind as the reader
    /// reports it: `notFound`, `notADiveLog` and `database` at least; any
    /// other kind reads as `database`.
    #[serde(serialize_with = "as_map")]
    pub problems: &'static [(&'static str, Problem)],
}

/// A reader error as the screens tell it.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Problem {
    pub title: &'static str,
    pub text: &'static str,
}

fn as_map<S: Serializer, V: Serialize>(
    entries: &&'static [(&'static str, V)],
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    s.collect_map(entries.iter().map(|(k, v)| (k, v)))
}

impl DiveApp {
    pub fn runs_on(&self, os: Os) -> bool {
        self.systems.contains(&os)
    }
}

impl PartialEq for DiveApp {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DiveApp {}

impl fmt::Debug for DiveApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiveApp({})", self.id)
    }
}

/// What the screens know of an application: its id, name, how the log is
/// chosen, and its texts.
impl Serialize for DiveApp {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct View<'a> {
            id: &'a str,
            name: &'a str,
            choose: &'a Choose,
            texts: &'a Localized<Texts>,
        }
        View {
            id: self.id,
            name: self.name,
            choose: &self.choose,
            texts: &self.texts,
        }
        .serialize(s)
    }
}

/// The application with this id.
pub fn by_id(id: &str) -> Option<&'static DiveApp> {
    ALL.iter().copied().find(|a| a.id == id)
}

/// The applications that run on `os`, in [`ALL`]'s order.
pub fn available(os: Os) -> impl Iterator<Item = &'static DiveApp> {
    ALL.iter().copied().filter(move |a| a.runs_on(os))
}

/// The application used on `os` when `chosen` is: that one, unless `os`
/// lacks it or this version does not know it (settings from a Mac on
/// Windows, from a newer version); then the first one available.
pub fn in_use(chosen: &str, os: Os) -> &'static DiveApp {
    by_id(chosen)
        .filter(|a| a.runs_on(os))
        .or_else(|| available(os).next())
        .unwrap_or(ALL[0])
}

/// An application written as its id, for serde (`#[serde(with = "...")]`).
pub mod id {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::DiveApp;

    pub fn serialize<S: Serializer>(app: &&'static DiveApp, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(app.id)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<&'static DiveApp, D::Error> {
        let id = String::deserialize(d)?;
        super::by_id(&id)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown dive application {id:?}")))
    }
}

/// The last log read whole when opened, kept while it does not change on
/// disk: a reader that reads everything on open (Subsurface) would otherwise
/// read the log again for every profile shown. A database read in place
/// (MacDive) is never kept.
#[derive(Default)]
pub struct Kept(Mutex<Option<KeptLog>>);

struct KeptLog {
    path: PathBuf,
    fingerprint: String,
    log: Arc<dyn Any + Send + Sync>,
}

impl Kept {
    /// The log at `path`: the one kept when it was opened from `path` with
    /// the same `fingerprint`, else `open`'s, then kept. A `fingerprint` of
    /// `None` (the reader cannot tell) is never kept.
    pub fn open<T: Any + Send + Sync>(
        &self,
        path: &Path,
        fingerprint: Option<String>,
        open: impl FnOnce() -> Result<T>,
    ) -> Result<Arc<T>> {
        if let Some(f) = &fingerprint
            && let Some(kept) = self.0.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
            && kept.path == path
            && &kept.fingerprint == f
            && let Ok(log) = Arc::clone(&kept.log).downcast::<T>()
        {
            return Ok(log);
        }
        let log = Arc::new(open()?);
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) =
            fingerprint.map(|fingerprint| KeptLog {
                path: path.to_path_buf(),
                fingerprint,
                log: log.clone(),
            });
        Ok(log)
    }
}

/// A kept log, shared between requests.
impl<T: DiveSource + ?Sized> DiveSource for Arc<T> {
    fn capabilities(&self) -> Capabilities {
        (**self).capabilities()
    }
    fn dives(&self) -> Result<Vec<Dive>> {
        (**self).dives()
    }
    fn profile(&self, dive_id: &str) -> Result<Vec<Sample>> {
        (**self).profile(dive_id)
    }
    fn sites(&self) -> Result<Vec<Site>> {
        (**self).sites()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::Error;

    /// Every kind of reader error, as the screens receive it.
    fn error_kinds() -> BTreeSet<String> {
        let p = PathBuf::from("/x");
        [
            Error::NotFound(p.clone()),
            Error::NotADiveLog(p.clone()),
            Error::OldFormat(p.clone()),
            Error::SeveralLogs(p),
            Error::Database(String::new()),
            Error::UnknownDive(String::new()),
            Error::BadProfile(String::new()),
            Error::Decoder(String::new()),
        ]
        .iter()
        .map(|e| {
            serde_json::to_value(e).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
    }

    /// The `{name}` placeholders of a text.
    fn placeholders(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|s| s.split_once('}').map(|(name, _)| name))
            .collect()
    }

    /// Every text of `t`, by what it is.
    fn all_texts(t: &Texts) -> Vec<(String, &'static str)> {
        let mut out = vec![
            ("log".to_string(), t.log),
            ("log hint".into(), t.log_hint),
            ("reading".into(), t.reading),
            ("file type".into(), t.file_type),
        ];
        out.extend(t.locate.map(|l| ("locate".into(), l)));
        out.extend(t.read.iter().map(|(f, l)| (format!("read {f}"), *l)));
        for (kind, p) in t.problems {
            out.push((format!("{kind} title"), p.title));
            out.push((format!("{kind} text"), p.text));
        }
        out
    }

    fn keys<V>(entries: &[(&'static str, V)]) -> Vec<&'static str> {
        entries.iter().map(|(k, _)| *k).collect()
    }

    /// Everything wrong with `app`'s description.
    fn problems_of(app: &DiveApp) -> Vec<String> {
        let mut out = Vec::new();
        let mut bad = |what: String| out.push(format!("{}: {what}", app.id));
        let camel = app
            .id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
            && app.id.chars().all(|c| c.is_ascii_alphanumeric());
        if !camel {
            bad("id must be camelCase letters and digits".into());
        }
        if app.name.trim().is_empty() || app.name.trim() != app.name {
            bad(format!("name {:?}", app.name));
        }
        if app.systems.is_empty() {
            bad("runs on no system".into());
        }
        for (i, os) in app.systems.iter().enumerate() {
            if app.systems[..i].contains(os) {
                bad(format!("{os:?} listed twice"));
            }
            let places = Places {
                os: *os,
                home: Some(Path::new("/Users/diver")),
                data: Some(Path::new("/Users/diver/Library/Application Support")),
                user: "diver",
            };
            if (app.standard_location)(&places).is_none() {
                bad(format!("no standard location on {os:?}"));
            }
        }
        let file = app.choose.file;
        if file.is_none() && !app.choose.folder {
            bad("the log is neither a file nor a folder".into());
        }
        if file.is_some_and(|x| {
            x.is_empty()
                || x.iter()
                    .any(|e| e.is_empty() || e.starts_with('.') || e.to_lowercase() != **e)
        }) {
            bad(format!(
                "file extensions {file:?}: lower case, without the dot"
            ));
        }
        let offsets = app.offsets_file;
        if !offsets.starts_with("clock-offsets") || !offsets.ends_with(".json") {
            bad(format!("offsets file {offsets:?}"));
        }
        if offsets.contains(['/', '\\']) {
            bad(format!("offsets file {offsets:?} is not a file name"));
        }

        let kinds = error_kinds();
        let en = &app.texts.en;
        for (lang, t) in app.texts.each() {
            let mut bad = |what: String| out.push(format!("{} ({lang}): {what}", app.id));
            for (what, text) in all_texts(t) {
                if text.trim().is_empty() || text.trim() != text {
                    bad(format!("{what} {text:?}"));
                }
            }
            for kind in ["notFound", "notADiveLog", "database"] {
                if !keys(t.problems).contains(&kind) {
                    bad(format!("no text for the reader error {kind}"));
                }
            }
            for (i, kind) in keys(t.problems).iter().enumerate() {
                if !kinds.contains(*kind) {
                    bad(format!("{kind} is not a reader error (one of {kinds:?})"));
                }
                if keys(t.problems)[..i].contains(kind) {
                    bad(format!("{kind} told twice"));
                }
            }
            if t.read.is_empty() {
                bad("nothing to say once the log is read".into());
            }
            for (i, (format, line)) in t.read.iter().enumerate() {
                if keys(t.read)[..i].contains(format) {
                    bad(format!("read {format} twice"));
                }
                if placeholders(line) != BTreeSet::from(["dives", "sites"]) {
                    bad(format!(
                        "read {format}: {line:?} needs {{dives}} and {{sites}}"
                    ));
                }
            }
            if app.choose.file.is_some() && app.choose.folder && t.locate.is_some() {
                bad("locate is for a log that is a file or a folder, not either".into());
            }
            // Same texts in every language, with the same placeholders.
            if lang != "en" {
                let mine: Vec<_> = all_texts(t);
                let theirs: Vec<_> = all_texts(en);
                let names =
                    |v: &[(String, &str)]| v.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
                if names(&mine) != names(&theirs) {
                    bad(format!(
                        "not the same texts as in English: {:?} / {:?}",
                        names(&mine),
                        names(&theirs)
                    ));
                }
                for ((what, a), (_, b)) in mine.iter().zip(&theirs) {
                    if placeholders(a) != placeholders(b) {
                        bad(format!("{what}: placeholders differ from English"));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn every_application_is_fully_described_in_both_languages() {
        assert!(!ALL.is_empty());
        let mut problems: Vec<String> = ALL.iter().flat_map(|a| problems_of(a)).collect();
        for (i, app) in ALL.iter().enumerate() {
            for other in &ALL[..i] {
                if app.id == other.id || app.name == other.name {
                    problems.push(format!("{} and {}: same id or name", app.id, other.id));
                }
                if app.offsets_file == other.offsets_file {
                    problems.push(format!("{} and {}: same offsets file", app.id, other.id));
                }
            }
        }
        assert!(problems.is_empty(), "\n- {}", problems.join("\n- "));
    }

    #[test]
    fn the_check_finds_an_incomplete_description() {
        static INCOMPLETE: DiveApp = DiveApp {
            id: "Blue Dive",
            name: "BlueDive",
            systems: &[Os::MacOs],
            standard_location: |_| None,
            choose: Choose {
                file: Some(&[".bdl"]),
                folder: false,
            },
            open: |path, _| Err(crate::Error::NotFound(path.to_path_buf())),
            preferences: Preferences {
                mac_domain: None,
                windows_key: None,
                units: |_| Units::default(),
            },
            offsets_file: "offsets.json",
            texts: Localized {
                en: Texts {
                    log: "BlueDive log",
                    log_hint: "Only read.",
                    reading: "Reading…",
                    read: &[("db", "{dives} · {sites}")],
                    file_type: "BlueDive log",
                    locate: None,
                    problems: &[
                        (
                            "notFound",
                            Problem {
                                title: "Not found",
                                text: "Choose it.",
                            },
                        ),
                        (
                            "database",
                            Problem {
                                title: "Unreadable",
                                text: "Try again.",
                            },
                        ),
                        (
                            "lost",
                            Problem {
                                title: "Lost",
                                text: "Gone.",
                            },
                        ),
                    ],
                },
                fr: Texts {
                    log: "Carnet BlueDive",
                    log_hint: "",
                    reading: "Lecture…",
                    read: &[("db", "{dives}")],
                    file_type: "Carnet BlueDive",
                    locate: None,
                    problems: &[],
                },
            },
            conformance: Vec::new,
        };
        let found = problems_of(&INCOMPLETE).join("\n");
        for expected in [
            "id must be camelCase",
            "no standard location on MacOs",
            "file extensions",
            "offsets file",
            "(en): no text for the reader error notADiveLog",
            "(en): lost is not a reader error",
            "(fr): log hint \"\"",
            "(fr): no text for the reader error notFound",
            "(fr): read db: \"{dives}\" needs {dives} and {sites}",
            "(fr): not the same texts as in English",
        ] {
            assert!(found.contains(expected), "{expected:?} not in:\n{found}");
        }
    }

    #[test]
    fn the_first_application_available_is_the_default() {
        assert_eq!(in_use("", Os::MacOs).id, "macDive");
        assert_eq!(in_use("", Os::Windows).id, "subsurface");
        assert_eq!(in_use("subsurface", Os::MacOs).id, "subsurface");
        // Settings from a Mac on Windows, or from a newer version.
        assert_eq!(in_use("macDive", Os::Windows).id, "subsurface");
        assert_eq!(in_use("blueDive", Os::MacOs).id, "macDive");
        let windows: Vec<_> = available(Os::Windows).map(|a| a.id).collect();
        assert_eq!(windows, ["subsurface"], "no MacDive on Windows");
        let mac: Vec<_> = available(Os::MacOs).map(|a| a.id).collect();
        assert_eq!(
            mac,
            ["macDive", "subsurface"],
            "Settings lists them in this order"
        );
    }

    /// Settings files, photos and remembered offsets already hold these.
    #[test]
    fn ids_names_and_offsets_files_never_change() {
        let frozen: Vec<_> = ALL.iter().map(|a| (a.id, a.name, a.offsets_file)).collect();
        assert_eq!(
            frozen[..2],
            [
                ("macDive", "MacDive", "clock-offsets.json"),
                ("subsurface", "Subsurface", "clock-offsets-subsurface.json"),
            ]
        );
    }

    #[test]
    fn standard_locations() {
        let places = |os, home: &'static str, data: &'static str| Places {
            os,
            home: Some(Path::new(home)),
            data: Some(Path::new(data)),
            user: "diver",
        };
        let mac = places(
            Os::MacOs,
            "/Users/diver",
            "/Users/diver/Library/Application Support",
        );
        assert_eq!(
            (crate::macdive::APP.standard_location)(&mac),
            Some(PathBuf::from(
                "/Users/diver/Library/Application Support/MacDive/MacDive.sqlite"
            ))
        );
        assert_eq!(
            (crate::subsurface::APP.standard_location)(&mac),
            Some(PathBuf::from(
                "/Users/diver/Library/Application Support/Subsurface/diver.xml"
            ))
        );
        let windows = places(
            Os::Windows,
            "C:/Users/diver",
            "C:/Users/diver/AppData/Roaming",
        );
        assert_eq!(
            (crate::subsurface::APP.standard_location)(&windows),
            Some(
                Path::new("C:/Users/diver/AppData/Roaming")
                    .join("Subsurface")
                    .join("diver.xml")
            )
        );
        let nowhere = Places {
            home: None,
            data: None,
            ..mac
        };
        for app in ALL {
            assert_eq!((app.standard_location)(&nowhere), None, "{}", app.id);
        }
    }

    #[test]
    fn screens_receive_ids_names_choosers_and_texts() {
        let json = serde_json::to_value(&crate::subsurface::APP).unwrap();
        assert_eq!(json["id"], "subsurface");
        assert_eq!(json["name"], "Subsurface");
        assert_eq!(json["choose"]["file"], serde_json::json!(["xml", "ssrf"]));
        assert_eq!(json["choose"]["folder"], true);
        assert_eq!(
            json["texts"]["fr"]["read"]["git"],
            "Stockage git · {dives} · {sites}"
        );
        assert_eq!(
            json["texts"]["en"]["problems"]["severalLogs"]["title"],
            "Several dive logs in this folder"
        );
        assert_eq!(json["texts"]["en"]["locate"], serde_json::Value::Null);
        assert!(json.get("offsetsFile").is_none(), "the back end's only");
    }

    #[derive(Debug)]
    struct Fixed(u32);

    #[test]
    fn keeps_a_log_while_it_does_not_change() {
        let kept = Kept::default();
        let path = Path::new("/Logs/diver.xml");
        let opened = std::cell::Cell::new(0);
        let open = |fingerprint: Option<&str>| {
            kept.open(path, fingerprint.map(Into::into), || {
                opened.set(opened.get() + 1);
                Ok(Fixed(opened.get()))
            })
            .unwrap()
            .0
        };
        assert_eq!(open(Some("a")), 1);
        assert_eq!(open(Some("a")), 1, "kept");
        assert_eq!(open(Some("b")), 2, "changed on disk");
        assert_eq!(open(None), 3, "cannot tell: read again");
        assert_eq!(open(None), 4, "and never kept");
        let other = kept
            .open(Path::new("/Logs/other.xml"), Some("b".into()), || {
                Ok(Fixed(9))
            })
            .unwrap();
        assert_eq!(other.0, 9, "another log");
        // A failure keeps nothing new.
        let failed = kept.open(path, Some("c".into()), || -> Result<Fixed> {
            Err(Error::NotFound(path.to_path_buf()))
        });
        assert!(failed.is_err());
    }
}
