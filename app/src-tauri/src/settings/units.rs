//! Units of the chosen dive application, from its preferences: what
//! Settings › units "as the dive application" means (spec F2). Dive logs
//! always store metric values; only the application's preferences say what
//! the diver reads. Preferences are only read, never written.
//!
//! Each application says where its preferences are and what they mean
//! (`divelens_core::apps::Preferences`); this reads them where the system
//! keeps them: a property list on a Mac, the registry on Windows. Keys are
//! written as Qt does, `Group/key`. Preferences not found (application not
//! installed, log copied from another computer): m and °C.

use std::path::{Path, PathBuf};

use divelens_core::apps::Preferences;
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

/// Units an application shows, from its preferences on this computer
/// (`home`: the diver's home folder).
pub fn read(prefs: &Preferences, home: &Path) -> Units {
    #[cfg(windows)]
    {
        let _ = home;
        read_windows(prefs)
    }
    #[cfg(not(windows))]
    {
        read_mac(prefs, home)
    }
}

/// Where a Mac application keeps its preferences: the usual folder, or its
/// sandbox container.
#[cfg_attr(windows, allow(dead_code))]
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

/// A number, possibly stored as text (Qt does both).
#[cfg_attr(windows, allow(dead_code))]
fn number(value: Option<&plist::Value>) -> Option<i64> {
    match value? {
        plist::Value::Integer(i) => i.as_signed(),
        plist::Value::String(s) => s.trim().parse().ok(),
        plist::Value::Boolean(b) => Some(i64::from(*b)),
        _ => None,
    }
}

/// Qt writes `Group/key` as `Group.key` in a property list.
#[cfg_attr(windows, allow(dead_code))]
fn mac_key(key: &str) -> String {
    key.replace('/', ".")
}

/// An application's units, from the first readable preference file in
/// `home` (Mac).
#[cfg_attr(windows, allow(dead_code))]
fn read_mac(prefs: &Preferences, home: &Path) -> Units {
    let Some(domain) = prefs.mac_domain else {
        return METRIC;
    };
    let Some(dict) = mac_preference_files(home, domain)
        .iter()
        .find_map(|path| plist::Value::from_file(path).ok()?.into_dictionary())
    else {
        return METRIC;
    };
    (prefs.units)(&|key| number(dict.get(&mac_key(key))))
}

/// Qt keeps `Group/key` as the value `key` of the subkey `Group` of the
/// application's registry key `root`: the key to open, and the value's name.
#[cfg_attr(not(windows), allow(dead_code))]
fn registry_value<'a>(root: &str, key: &'a str) -> (String, &'a str) {
    match key.rsplit_once('/') {
        Some((group, name)) => (format!(r"{root}\{}", group.replace('/', r"\")), name),
        None => (root.to_string(), key),
    }
}

/// An application's units, from the registry (Windows).
#[cfg(windows)]
fn read_windows(prefs: &Preferences) -> Units {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let Some(root) = prefs.windows_key else {
        return METRIC;
    };
    let user = RegKey::predef(HKEY_CURRENT_USER);
    if user.open_subkey(root).is_err() {
        return METRIC;
    }
    // Qt stores numbers as DWORD or as text depending on its version.
    let value = |key: &str| -> Option<i64> {
        let (path, name) = registry_value(root, key);
        let key = user.open_subkey(path).ok()?;
        key.get_value::<u32, _>(name)
            .ok()
            .map(i64::from)
            .or_else(|| key.get_value::<String, _>(name).ok()?.trim().parse().ok())
    };
    (prefs.units)(&value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use divelens_core::{macdive, subsurface};

    const FEET_F: Units = Units {
        depth: DepthUnit::Feet,
        temp: TempUnit::Fahrenheit,
    };
    const FEET_C: Units = Units {
        depth: DepthUnit::Feet,
        temp: TempUnit::Celsius,
    };

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
        let md = &macdive::APP.preferences;
        assert_eq!(read_mac(md, &dir), METRIC, "no preferences");
        let file = dir.join("Library/Preferences/com.mintsoftware.MacDive2.plist");
        for (value, expected) in [(0, METRIC), (1, FEET_F), (2, FEET_C), (42, METRIC)] {
            write_prefs(&file, &[("units", plist::Value::Integer(value.into()))]);
            assert_eq!(read_mac(md, &dir), expected, "units = {value}");
        }
        write_prefs(&file, &[("other", plist::Value::Boolean(true))]);
        assert_eq!(read_mac(md, &dir), METRIC, "key absent");
        std::fs::write(&file, b"not a plist").unwrap();
        assert_eq!(read_mac(md, &dir), METRIC, "unreadable");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reads_sandboxed_macdive_preferences() {
        let dir = home("sandbox");
        let file = dir.join(
            "Library/Containers/com.mintsoftware.MacDive2/Data/Library/Preferences/com.mintsoftware.MacDive2.plist",
        );
        write_prefs(&file, &[("units", plist::Value::Integer(2.into()))]);
        assert_eq!(read_mac(&macdive::APP.preferences, &dir), FEET_C);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reads_subsurface_mac_preferences() {
        let dir = home("subsurface");
        let ss = &subsurface::APP.preferences;
        assert_eq!(read_mac(ss, &dir), METRIC, "no preferences");
        let file = dir.join("Library/Preferences/org.hohndel.subsurface.Subsurface.plist");
        write_prefs(
            &file,
            &[
                ("Units.length", plist::Value::Integer(1.into())),
                ("Units.temperature", plist::Value::String("0".into())),
            ],
        );
        assert_eq!(read_mac(ss, &dir), FEET_C);
        write_prefs(
            &file,
            &[("Units.unit_system", plist::Value::Integer(1.into()))],
        );
        assert_eq!(read_mac(ss, &dir), FEET_F);
        write_prefs(
            &file,
            &[("MainWindow.geometry", plist::Value::Boolean(true))],
        );
        assert_eq!(read_mac(ss, &dir), METRIC, "units never set");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn qt_keys_on_each_system() {
        assert_eq!(mac_key("Units/length"), "Units.length");
        assert_eq!(mac_key("units"), "units");
        let root = subsurface::APP.preferences.windows_key.unwrap();
        assert_eq!(root, r"Software\Subsurface\Subsurface");
        assert_eq!(
            registry_value(root, "Units/length"),
            (
                r"Software\Subsurface\Subsurface\Units".to_string(),
                "length"
            )
        );
        assert_eq!(
            registry_value(root, "Units/unit_system"),
            (
                r"Software\Subsurface\Subsurface\Units".to_string(),
                "unit_system"
            )
        );
        assert_eq!(
            registry_value(r"Software\Blue", "units"),
            (r"Software\Blue".to_string(), "units")
        );
    }

    #[test]
    fn no_preferences_on_a_system_means_metric() {
        let none = Preferences {
            mac_domain: None,
            windows_key: None,
            units: |_| FEET_F,
        };
        let dir = home("none");
        assert_eq!(read_mac(&none, &dir), METRIC);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
