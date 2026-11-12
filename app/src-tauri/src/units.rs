//! Display and writing units, until the Settings screen exists: taken from
//! the preferences of the chosen dive application (spec F2). Dive logs always
//! store metric values; only the application's preferences say what the
//! diver reads. Preferences are only read, never written.
//!
//! - MacDive: global preference `units` of `com.mintsoftware.MacDive2`:
//!   0 = m and °C, 1 = ft and °F, 2 (Canadian) = ft and °C, anything else =
//!   m and °C.
//! - Subsurface: its separate depth (`Units/length`: 0 = m, 1 = ft) and
//!   temperature (`Units/temperature`: 0 = °C, 1 = °F, 2 = K shown as °C)
//!   settings, stored by Qt in a property list on a Mac and in the registry
//!   on Windows. Subsurface rewrites both when its unit system changes. When
//!   one is missing, its `Units/unit_system` (0 metric, 1 imperial,
//!   2 personalized) decides; with nothing at all, Subsurface's default:
//!   metric.
//! - Preferences not found (application not installed, log copied from
//!   another computer): m and °C.

use std::path::{Path, PathBuf};

use divelens_core::tags::{DepthUnit, TempUnit, Units};
use serde::Serialize;

/// Units as the UI knows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct UiUnits {
    /// `m` or `ft`.
    pub depth: &'static str,
    /// `c` or `f`.
    pub temp: &'static str,
}

impl From<Units> for UiUnits {
    fn from(u: Units) -> Self {
        Self {
            depth: match u.depth {
                DepthUnit::Metres => "m",
                DepthUnit::Feet => "ft",
            },
            temp: match u.temp {
                TempUnit::Celsius => "c",
                TempUnit::Fahrenheit => "f",
            },
        }
    }
}

const METRIC: Units = Units {
    depth: DepthUnit::Metres,
    temp: TempUnit::Celsius,
};

/// MacDive's `units` preference value to units.
pub fn macdive(value: Option<i64>) -> Units {
    match value {
        Some(1) => Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Fahrenheit,
        },
        Some(2) => Units {
            depth: DepthUnit::Feet,
            temp: TempUnit::Celsius,
        },
        _ => METRIC,
    }
}

/// Subsurface's unit settings to units.
pub fn subsurface(length: Option<i64>, temperature: Option<i64>, system: Option<i64>) -> Units {
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

const MACDIVE_DOMAIN: &str = "com.mintsoftware.MacDive2";
/// Qt names Subsurface's preferences after its organization domain.
const SUBSURFACE_DOMAIN: &str = "org.hohndel.subsurface.Subsurface";

/// Where a Mac application keeps its preferences: the usual folder, or its
/// sandbox container.
fn mac_preference_files(home: &Path, domain: &str) -> [PathBuf; 2] {
    let file = format!("{domain}.plist");
    [
        home.join("Library/Preferences").join(&file),
        home.join("Library/Containers")
            .join(domain)
            .join("Data/Library/Preferences")
            .join(&file),
    ]
}

/// The first readable preference file of a Mac application.
fn read_mac_preferences(home: &Path, domain: &str) -> Option<plist::Dictionary> {
    mac_preference_files(home, domain)
        .iter()
        .find_map(|path| plist::Value::from_file(path).ok()?.into_dictionary())
}

/// A number, possibly stored as text (Qt does both).
fn number(value: Option<&plist::Value>) -> Option<i64> {
    match value? {
        plist::Value::Integer(i) => i.as_signed(),
        plist::Value::String(s) => s.trim().parse().ok(),
        plist::Value::Boolean(b) => Some(i64::from(*b)),
        _ => None,
    }
}

/// MacDive's units, from its preferences in `home`.
pub fn read_macdive(home: &Path) -> Units {
    let prefs = read_mac_preferences(home, MACDIVE_DOMAIN);
    macdive(prefs.as_ref().and_then(|p| number(p.get("units"))))
}

/// Subsurface's units, from its preferences in `home` (Mac).
#[cfg_attr(windows, allow(dead_code))]
pub fn read_subsurface_mac(home: &Path) -> Units {
    let Some(prefs) = read_mac_preferences(home, SUBSURFACE_DOMAIN) else {
        return METRIC;
    };
    // Qt writes "Group/key" as "Group.key" in a property list.
    subsurface(
        number(prefs.get("Units.length")),
        number(prefs.get("Units.temperature")),
        number(prefs.get("Units.unit_system")),
    )
}

/// Subsurface's units, from the registry (Windows).
#[cfg(windows)]
pub fn read_subsurface_windows() -> Units {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let Ok(key) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Subsurface\Subsurface\Units")
    else {
        return METRIC;
    };
    // Qt stores numbers as DWORD or as text depending on its version.
    let number = |name: &str| -> Option<i64> {
        key.get_value::<u32, _>(name)
            .ok()
            .map(i64::from)
            .or_else(|| key.get_value::<String, _>(name).ok()?.trim().parse().ok())
    };
    subsurface(
        number("length"),
        number("temperature"),
        number("unit_system"),
    )
}

/// Units of Subsurface on this computer.
#[allow(dead_code)] // used once Subsurface is a source
pub fn read_subsurface(home: &Path) -> Units {
    #[cfg(windows)]
    {
        let _ = home;
        read_subsurface_windows()
    }
    #[cfg(not(windows))]
    {
        read_subsurface_mac(home)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEET_F: Units = Units {
        depth: DepthUnit::Feet,
        temp: TempUnit::Fahrenheit,
    };
    const FEET_C: Units = Units {
        depth: DepthUnit::Feet,
        temp: TempUnit::Celsius,
    };

    #[test]
    fn macdive_values() {
        assert_eq!(macdive(Some(0)), METRIC);
        assert_eq!(macdive(Some(1)), FEET_F);
        assert_eq!(macdive(Some(2)), FEET_C);
        assert_eq!(macdive(Some(7)), METRIC, "unknown value");
        assert_eq!(macdive(None), METRIC, "missing value");
    }

    #[test]
    fn subsurface_values() {
        assert_eq!(subsurface(Some(0), Some(0), None), METRIC);
        assert_eq!(subsurface(Some(1), Some(1), None), FEET_F);
        assert_eq!(subsurface(Some(1), Some(0), None), FEET_C);
        assert_eq!(subsurface(Some(5), Some(9), None), METRIC, "unknown values");
        assert_eq!(subsurface(None, None, None), METRIC, "nothing set");
        assert_eq!(subsurface(None, None, Some(1)), FEET_F);
        assert_eq!(
            subsurface(None, None, Some(2)),
            METRIC,
            "personalized, nothing set"
        );
        // Separate settings win over the unit system.
        assert_eq!(
            subsurface(Some(0), None, Some(1)),
            Units {
                depth: DepthUnit::Metres,
                temp: TempUnit::Fahrenheit,
            }
        );
    }

    fn home(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("divelens-units-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Library/Preferences")).unwrap();
        dir
    }

    fn write_prefs(path: &Path, entries: &[(&str, plist::Value)]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut dict = plist::Dictionary::new();
        for (k, v) in entries {
            dict.insert((*k).into(), v.clone());
        }
        plist::Value::Dictionary(dict).to_file_binary(path).unwrap();
    }

    #[test]
    fn reads_macdive_preferences() {
        let dir = home("macdive");
        assert_eq!(read_macdive(&dir), METRIC, "no preferences");
        let file = dir.join("Library/Preferences/com.mintsoftware.MacDive2.plist");
        for (value, expected) in [(0, METRIC), (1, FEET_F), (2, FEET_C), (42, METRIC)] {
            write_prefs(&file, &[("units", plist::Value::Integer(value.into()))]);
            assert_eq!(read_macdive(&dir), expected, "units = {value}");
        }
        write_prefs(&file, &[("other", plist::Value::Boolean(true))]);
        assert_eq!(read_macdive(&dir), METRIC, "key absent");
        std::fs::write(&file, b"not a plist").unwrap();
        assert_eq!(read_macdive(&dir), METRIC, "unreadable");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reads_sandboxed_macdive_preferences() {
        let dir = home("sandbox");
        let file = dir.join(
            "Library/Containers/com.mintsoftware.MacDive2/Data/Library/Preferences/com.mintsoftware.MacDive2.plist",
        );
        write_prefs(&file, &[("units", plist::Value::Integer(2.into()))]);
        assert_eq!(read_macdive(&dir), FEET_C);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reads_subsurface_mac_preferences() {
        let dir = home("subsurface");
        assert_eq!(read_subsurface_mac(&dir), METRIC, "no preferences");
        let file = dir.join("Library/Preferences/org.hohndel.subsurface.Subsurface.plist");
        write_prefs(
            &file,
            &[
                ("Units.length", plist::Value::Integer(1.into())),
                ("Units.temperature", plist::Value::String("0".into())),
            ],
        );
        assert_eq!(read_subsurface_mac(&dir), FEET_C);
        write_prefs(
            &file,
            &[("Units.unit_system", plist::Value::Integer(1.into()))],
        );
        assert_eq!(read_subsurface_mac(&dir), FEET_F);
        write_prefs(
            &file,
            &[("MainWindow.geometry", plist::Value::Boolean(true))],
        );
        assert_eq!(read_subsurface_mac(&dir), METRIC, "units never set");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
