//! Example's description (see [`crate::apps`]): what the app needs to know of
//! the application besides its reader. Built only with the tests and never
//! in [`crate::apps::ALL`], so Dive Lens never offers it; the tests check it
//! like every registered description.
//!
//! A copy becomes `core/src/<application>/app.rs`, its `APP` is
//! `pub static` (declared `mod app; pub use app::APP;` in the reader's
//! module) and is added to [`crate::apps::ALL`]: the application then
//! appears in Settings, on the systems it runs on.

use std::path::{Path, PathBuf};

use super::Example;
use crate::Result;
use crate::apps::{
    Choose, DiveApp, Kept, Localized, Opened, Os, Places, PreferenceValue, Preferences, Problem,
    Texts,
};
use crate::tags::Units;

pub(crate) static APP: DiveApp = DiveApp {
    // ADAPT: the id settings files keep (camelCase, never changed) and the
    // name written on photos (never translated, never changed).
    id: "example",
    name: "Example",
    // ADAPT: the systems the application runs on.
    systems: &[Os::MacOs, Os::Windows],
    standard_location,
    // ADAPT: a file (its extensions, lower case, without the dot), a folder
    // (Subsurface's git storage), or either.
    choose: Choose {
        file: Some(&["sqlite"]),
        folder: false,
    },
    open,
    // ADAPT: where the application keeps its unit preferences (see MacDive's
    // and Subsurface's descriptions). None here: metres and °C.
    preferences: Preferences {
        mac_domain: None,
        windows_key: None,
        units,
    },
    // ADAPT: `clock-offsets-<application>.json`, never changed afterwards.
    offsets_file: "clock-offsets-example.json",
    // ADAPT: every text, in English and in French. The tests refuse one
    // missing, empty, or with other placeholders than its translation.
    texts: Localized {
        en: Texts {
            log: "Example log",
            log_hint: "Example's standard location unless you choose another. Only read, never modified.",
            reading: "Reading the log…",
            read: &[("database", "Valid log · {dives} · {sites}")],
            file_type: "Example log",
            locate: Some("Locate the log…"),
            problems: &[
                (
                    "notFound",
                    Problem {
                        title: "Example log not found",
                        text: "There is no log at this location. It may have been moved.",
                    },
                ),
                (
                    "notADiveLog",
                    Problem {
                        title: "This is not an Example log",
                        text: "Choose the log saved by Example.",
                    },
                ),
                (
                    "database",
                    Problem {
                        title: "The Example log could not be read",
                        text: "Try again in a moment.",
                    },
                ),
            ],
        },
        fr: Texts {
            log: "Carnet Example",
            log_hint: "Emplacement standard d'Example, sauf si vous en choisissez un autre. Lu sans jamais être modifié.",
            reading: "Lecture du carnet…",
            read: &[("database", "Carnet valide · {dives} · {sites}")],
            file_type: "Carnet Example",
            locate: Some("Localiser le carnet…"),
            problems: &[
                (
                    "notFound",
                    Problem {
                        title: "Carnet Example introuvable",
                        text: "Aucun carnet à l'emplacement indiqué. Il a peut-être été déplacé.",
                    },
                ),
                (
                    "notADiveLog",
                    Problem {
                        title: "Ce n'est pas un carnet Example",
                        text: "Choisissez le carnet enregistré par Example.",
                    },
                ),
                (
                    "database",
                    Problem {
                        title: "Impossible de lire le carnet Example",
                        text: "Réessayez dans un instant.",
                    },
                ),
            ],
        },
    },
    conformance: super::tests::conformance,
};

/// ADAPT: where the application keeps its log unless the diver moved it.
fn standard_location(places: &Places) -> Option<PathBuf> {
    places
        .data
        .map(|data| data.join("Example").join("Example.sqlite"))
}

/// The database, opened for one request: read in place, never kept. A
/// reader that reads the whole log on open keeps it in `kept` instead (see
/// Subsurface's description).
fn open(path: &Path, _: &Kept) -> Result<Opened> {
    Ok(Opened {
        log: Box::new(Example::open(path)?),
        // ADAPT: one key per way the log can be stored, each with its line
        // in `texts.read`.
        format: "database",
    })
}

/// ADAPT: what the application's preference values mean.
fn units(_: &PreferenceValue<'_>) -> Units {
    Units::default()
}
