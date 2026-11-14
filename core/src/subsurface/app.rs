//! Subsurface as the app knows it besides its reader (see [`crate::apps`]).

use std::path::{Path, PathBuf};

use super::{Storage, Subsurface, default_path};
use crate::Result;
use crate::apps::{
    Choose, DiveApp, Kept, Localized, Opened, Os, Places, PreferenceValue, Preferences, Problem,
    Texts,
};
use crate::tags::{DepthUnit, TempUnit, Units};

pub static APP: DiveApp = DiveApp {
    id: "subsurface",
    name: "Subsurface",
    systems: &[Os::MacOs, Os::Windows],
    standard_location,
    // An XML file, or the folder of a git storage.
    choose: Choose {
        file: Some(&["xml", "ssrf"]),
        folder: true,
    },
    open,
    preferences: Preferences {
        // Qt names Subsurface's preferences after its organization domain.
        mac_domain: Some("org.hohndel.subsurface.Subsurface"),
        windows_key: Some(r"Software\Subsurface\Subsurface"),
        units,
    },
    offsets_file: "clock-offsets-subsurface.json",
    texts: Localized {
        en: Texts {
            log: "Subsurface log",
            log_hint: "Subsurface's standard file unless you choose another: a .xml or .ssrf file, or the folder of its git storage. Only read, never modified.",
            reading: "Reading the log…",
            read: &[
                ("xml", "XML file · {dives} · {sites}"),
                ("git", "Git storage · {dives} · {sites}"),
            ],
            file_type: "Subsurface dive log",
            locate: None,
            problems: &[
                (
                    "notFound",
                    Problem {
                        title: "Subsurface dive log not found",
                        text: "There is nothing at this location. Choose your Subsurface log: a .xml or .ssrf file, or the folder of its git storage.",
                    },
                ),
                (
                    "notADiveLog",
                    Problem {
                        title: "This is not a Subsurface dive log",
                        text: "Choose a file saved by Subsurface (.xml or .ssrf), or the folder of its git storage.",
                    },
                ),
                (
                    "database",
                    Problem {
                        title: "The Subsurface dive log could not be read",
                        text: "It may be incomplete, for example while Subsurface is saving it. Try again in a moment.",
                    },
                ),
                (
                    "oldFormat",
                    Problem {
                        title: "This Subsurface log uses an old format",
                        text: "Open it in Subsurface and save it again, then try again.",
                    },
                ),
                (
                    "severalLogs",
                    Problem {
                        title: "Several dive logs in this folder",
                        text: "Dive Lens cannot tell which one to read. Choose another folder.",
                    },
                ),
            ],
        },
        fr: Texts {
            log: "Carnet Subsurface",
            log_hint: "Fichier standard de Subsurface, sauf si vous en choisissez un autre : fichier .xml ou .ssrf, ou dossier du stockage git. Lu sans jamais être modifié.",
            reading: "Lecture du carnet…",
            read: &[
                ("xml", "Fichier XML · {dives} · {sites}"),
                ("git", "Stockage git · {dives} · {sites}"),
            ],
            file_type: "Carnet Subsurface",
            locate: None,
            problems: &[
                (
                    "notFound",
                    Problem {
                        title: "Carnet Subsurface introuvable",
                        text: "Rien à l'emplacement indiqué. Choisissez votre carnet Subsurface : un fichier .xml ou .ssrf, ou le dossier de son stockage git.",
                    },
                ),
                (
                    "notADiveLog",
                    Problem {
                        title: "Ce n'est pas un carnet Subsurface",
                        text: "Choisissez un fichier enregistré par Subsurface (.xml ou .ssrf), ou le dossier de son stockage git.",
                    },
                ),
                (
                    "database",
                    Problem {
                        title: "Impossible de lire le carnet Subsurface",
                        text: "Il est peut-être incomplet, par exemple pendant que Subsurface l'enregistre. Réessayez dans un instant.",
                    },
                ),
                (
                    "oldFormat",
                    Problem {
                        title: "Ce carnet Subsurface est dans un format ancien",
                        text: "Ouvrez-le et réenregistrez-le dans Subsurface, puis réessayez.",
                    },
                ),
                (
                    "severalLogs",
                    Problem {
                        title: "Plusieurs carnets dans ce dossier",
                        text: "Dive Lens ne sait pas lequel lire. Choisissez un autre dossier.",
                    },
                ),
            ],
        },
    },
    #[cfg(test)]
    conformance: super::tests::conformance,
};

/// `<user>.xml` in Subsurface's folder: `~/Library/Application Support/
/// Subsurface` on macOS, `%APPDATA%\Subsurface` on Windows.
fn standard_location(places: &Places) -> Option<PathBuf> {
    places
        .data
        .map(|data| default_path(&data.join("Subsurface"), places.user))
}

/// The log is read whole (profiles included), so it is kept while it does
/// not change on disk: reading it again for every profile shown would be
/// slow.
fn open(path: &Path, kept: &Kept) -> Result<Opened> {
    let log = kept.open(path, Subsurface::fingerprint(path), || {
        Subsurface::open(path)
    })?;
    let format = match log.storage() {
        Storage::Xml => "xml",
        Storage::Git { .. } => "git",
    };
    Ok(Opened {
        log: Box::new(log),
        format,
    })
}

/// Subsurface's separate depth (`Units/length`: 0 = m, 1 = ft) and
/// temperature (`Units/temperature`: 0 = °C, 1 = °F, 2 = K shown as °C)
/// settings; Subsurface rewrites both when its unit system changes. When one
/// is missing, its `Units/unit_system` (0 metric, 1 imperial,
/// 2 personalized) decides; with nothing at all, Subsurface's default:
/// metric.
fn units(value: &PreferenceValue<'_>) -> Units {
    preference_units(
        value("Units/length"),
        value("Units/temperature"),
        value("Units/unit_system"),
    )
}

fn preference_units(length: Option<i64>, temperature: Option<i64>, system: Option<i64>) -> Units {
    let imperial = system == Some(1);
    Units {
        depth: match length {
            Some(1) => DepthUnit::Feet,
            Some(_) => DepthUnit::Metres,
            None if imperial => DepthUnit::Feet,
            None => DepthUnit::Metres,
        },
        temp: match temperature {
            Some(1) => TempUnit::Fahrenheit,
            Some(_) => TempUnit::Celsius,
            None if imperial => TempUnit::Fahrenheit,
            None => TempUnit::Celsius,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const METRIC: Units = Units {
        depth: DepthUnit::Metres,
        temp: TempUnit::Celsius,
    };
    const FEET_F: Units = Units {
        depth: DepthUnit::Feet,
        temp: TempUnit::Fahrenheit,
    };
    const FEET_C: Units = Units {
        depth: DepthUnit::Feet,
        temp: TempUnit::Celsius,
    };

    #[test]
    fn preference_values() {
        let s = preference_units;
        assert_eq!(s(Some(0), Some(0), None), METRIC);
        assert_eq!(s(Some(1), Some(1), None), FEET_F);
        assert_eq!(s(Some(1), Some(0), None), FEET_C);
        assert_eq!(s(Some(5), Some(9), None), METRIC, "unknown values");
        assert_eq!(s(None, None, None), METRIC, "nothing set");
        assert_eq!(s(None, None, Some(1)), FEET_F);
        assert_eq!(s(None, None, Some(2)), METRIC, "personalized, nothing set");
        // Separate settings win over the unit system.
        assert_eq!(
            s(Some(0), None, Some(1)),
            Units {
                depth: DepthUnit::Metres,
                temp: TempUnit::Fahrenheit,
            }
        );
    }

    #[test]
    fn reads_qt_keys() {
        let prefs = |key: &str| match key {
            "Units/length" => Some(1),
            "Units/temperature" => Some(0),
            _ => None,
        };
        assert_eq!((APP.preferences.units)(&prefs), FEET_C);
        let system = |key: &str| (key == "Units/unit_system").then_some(1);
        assert_eq!((APP.preferences.units)(&system), FEET_F);
    }

    #[test]
    fn opened_logs_tell_their_format_and_are_kept_while_unchanged() {
        let dir = tempfile::TempDir::new().unwrap();
        let kept = Kept::default();
        let xml = super::super::tests::reference_xml_log(dir.path());
        assert_eq!((APP.open)(&xml, &kept).unwrap().format, "xml");
        let git = super::super::tests::reference_git_log(dir.path());
        assert_eq!((APP.open)(&git, &kept).unwrap().format, "git");
        // Kept: the same log, without reading it again.
        let first = kept
            .open(&git, Subsurface::fingerprint(&git), || {
                Subsurface::open(&git)
            })
            .unwrap();
        let again = kept
            .open(&git, Subsurface::fingerprint(&git), || {
                Subsurface::open(&git)
            })
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &again));
    }
}
