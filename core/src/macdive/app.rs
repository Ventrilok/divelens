//! MacDive as the app knows it besides its reader (see [`crate::apps`]).

use std::path::{Path, PathBuf};

use super::MacDive;
use crate::Result;
use crate::apps::{
    Choose, DiveApp, Kept, Localized, Opened, Os, Places, PreferenceValue, Preferences, Problem,
    Texts,
};
use crate::tags::{DepthUnit, TempUnit, Units};

pub static APP: DiveApp = DiveApp {
    id: "macDive",
    name: "MacDive",
    systems: &[Os::MacOs],
    standard_location,
    choose: Choose {
        file: Some(&["sqlite"]),
        folder: false,
    },
    open,
    preferences: Preferences {
        mac_domain: Some("com.mintsoftware.MacDive2"),
        windows_key: None,
        units,
    },
    // The one from before Subsurface.
    offsets_file: "clock-offsets.json",
    texts: Localized {
        en: Texts {
            log: "MacDive database",
            log_hint: "MacDive's standard location unless you choose another. Only read, never modified.",
            reading: "Reading the database…",
            read: &[("database", "Valid database · {dives} · {sites}")],
            file_type: "MacDive database",
            locate: Some("Locate the database…"),
            problems: &[
                (
                    "notFound",
                    Problem {
                        title: "MacDive database not found",
                        text: "There is no database at this location. It may have been moved, for example to iCloud.",
                    },
                ),
                (
                    "notADiveLog",
                    Problem {
                        title: "This is not a MacDive database",
                        text: "Choose the MacDive.sqlite file.",
                    },
                ),
                (
                    "database",
                    Problem {
                        title: "The MacDive database could not be read",
                        text: "If it is stored in iCloud, check that it is downloaded on this Mac.",
                    },
                ),
            ],
        },
        fr: Texts {
            log: "Base MacDive",
            log_hint: "Emplacement standard de MacDive, sauf si vous en choisissez un autre. Lue sans jamais être modifiée.",
            reading: "Lecture de la base…",
            read: &[("database", "Base valide · {dives} · {sites}")],
            file_type: "Base MacDive",
            locate: Some("Localiser la base…"),
            problems: &[
                (
                    "notFound",
                    Problem {
                        title: "Base MacDive introuvable",
                        text: "Aucune base à l'emplacement indiqué. Elle a peut-être été déplacée, par exemple dans iCloud.",
                    },
                ),
                (
                    "notADiveLog",
                    Problem {
                        title: "Ce fichier n'est pas une base MacDive",
                        text: "Choisissez le fichier MacDive.sqlite.",
                    },
                ),
                (
                    "database",
                    Problem {
                        title: "Impossible de lire la base MacDive",
                        text: "Si elle est dans iCloud, vérifiez qu'elle est bien téléchargée sur ce Mac.",
                    },
                ),
            ],
        },
    },
    #[cfg(test)]
    conformance: super::tests::conformance,
};

/// `~/Library/Application Support/MacDive/MacDive.sqlite`.
fn standard_location(places: &Places) -> Option<PathBuf> {
    places.home.map(MacDive::default_path)
}

/// The database, opened for one request: it is read in place, never kept.
fn open(path: &Path, _: &Kept) -> Result<Opened> {
    Ok(Opened {
        log: Box::new(MacDive::open(path)?),
        format: "database",
    })
}

/// MacDive's global `units` preference.
fn units(value: &PreferenceValue<'_>) -> Units {
    preference_units(value("units"))
}

/// 0 = m and °C, 1 = ft and °F, 2 (Canadian) = ft and °C, anything else =
/// m and °C.
fn preference_units(value: Option<i64>) -> Units {
    match value {
        Some(1) => Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Fahrenheit,
        },
        Some(2) => Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Celsius,
        },
        _ => Units {
            depth: DepthUnit::Metres,
            temp: TempUnit::Celsius,
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

    #[test]
    fn preference_values() {
        let feet_f = Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Fahrenheit,
        };
        let feet_c = Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Celsius,
        };
        assert_eq!(preference_units(Some(0)), METRIC);
        assert_eq!(preference_units(Some(1)), feet_f);
        assert_eq!(preference_units(Some(2)), feet_c);
        assert_eq!(preference_units(Some(7)), METRIC, "unknown value");
        assert_eq!(preference_units(None), METRIC, "missing value");
        let canadian = |key: &str| (key == "units").then_some(2);
        assert_eq!((APP.preferences.units)(&canadian), feet_c);
    }
}
