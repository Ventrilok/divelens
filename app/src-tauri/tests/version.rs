//! The product version is set in one place, `app/package.json`. The bundle and
//! the running app read it from there (tauri.conf.json points to it); every
//! other copy is written by `npm run set-version` and must match it.

mod common;

use std::path::{Path, PathBuf};

fn app_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn json(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// The version people see: `app/package.json`.
fn version() -> String {
    json(&app_dir().join("package.json"))["version"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// A number in a version: digits, without a leading zero (semver).
fn is_number(part: &str) -> bool {
    !part.is_empty()
        && part.bytes().all(|b| b.is_ascii_digit())
        && (part == "0" || !part.starts_with('0'))
}

/// MAJOR.MINOR.PATCH, optionally followed by a pre-release label, following
/// semver: dot-separated parts, each non-empty, made of letters, digits and
/// hyphens, a part made of digits only without a leading zero. No build
/// metadata (`+7`): npm drops it, and Lightroom's numbers could not hold it.
fn is_version(version: &str) -> bool {
    let (numbers, label) = match version.split_once('-') {
        Some((numbers, label)) => (numbers, Some(label)),
        None => (version, None),
    };
    let numbers: Vec<_> = numbers.split('.').collect();
    numbers.len() == 3
        && numbers.iter().all(|n| is_number(n))
        && label.is_none_or(|label| {
            label.split('.').all(|part| {
                !part.is_empty()
                    && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    && (!part.bytes().all(|b| b.is_ascii_digit()) || is_number(part))
            })
        })
}

#[test]
fn version_is_major_minor_patch_with_an_optional_label() {
    for good in [
        "0.1.0",
        "2.0.0",
        "10.20.30",
        "2.1.0-beta.1",
        "2.1.0-rc.10",
        "2.1.0-1",
        "2.1.0-0.3.7",
        "2.1.0-x-y.0a",
    ] {
        assert!(is_version(good), "{good}");
    }
    for bad in [
        "2.1",
        "2.1.0.0",
        "02.1.0",
        "2.01.0",
        "2.1.00",
        "v2.1.0",
        "2.1.0-",
        "2.1.0-beta.",
        "2.1.0-.1",
        "2.1.0-beta..1",
        "2.1.0-beta.01",
        "2.1.0-00",
        "2.1.0-bêta",
        "2.1.0+7",
        "2.1.0-beta+7",
    ] {
        assert!(!is_version(bad), "{bad}");
    }
    let version = version();
    assert!(is_version(&version), "app/package.json version {version}");
}

#[test]
fn app_reads_its_version_from_package_json() {
    let config = json(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json"));
    assert_eq!(config["version"], "../package.json");
}

#[test]
fn package_lock_matches_package_json() {
    let lock = json(&app_dir().join("package-lock.json"));
    assert_eq!(lock["version"], version());
    assert_eq!(lock["packages"][""]["version"], version());
}

#[test]
fn crate_version_matches_package_json() {
    // Every crate of the workspace inherits the workspace version.
    assert_eq!(env!("CARGO_PKG_VERSION"), version());
}

#[test]
fn plugin_version_matches_package_json() {
    let (numbers, display): (String, String) = common::lua()
        .load(
            r#"
            local v = require "Info".VERSION
            return string.format("%d.%d.%d", v.major, v.minor, v.revision), v.display
            "#,
        )
        .eval()
        .unwrap();
    let version = version();
    // What the Plug-in Manager shows, pre-release label included.
    assert_eq!(display, version);
    // The numbers hold MAJOR.MINOR.PATCH, without the label.
    assert_eq!(numbers, version.split('-').next().unwrap());
}
