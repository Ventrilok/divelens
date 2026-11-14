//! The product version is set in one place, `app/package.json`. The bundle and
//! the running app read it from there (tauri.conf.json points to it); every
//! other copy is written by `npm run set-version` and must match it.

use std::path::{Path, PathBuf};

use mlua::Lua;

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

#[test]
fn version_is_major_minor_patch_with_an_optional_label() {
    let version = version();
    let (numbers, label) = version.split_once('-').unwrap_or((&version, ""));
    let numbers: Vec<_> = numbers.split('.').collect();
    assert!(
        numbers.len() == 3
            && numbers
                .iter()
                .all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())),
        "{version}"
    );
    assert!(
        label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-'),
        "{version}"
    );
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
    let lua = Lua::new();
    let path = app_dir().join("../plugin/DiveLens.lrdevplugin/?.lua");
    lua.load(format!(
        "package.path = {:?}\n\
         LOC = function(s) return (s:gsub('^%$%$%$/[^=]*=', '')) end",
        path.to_str().unwrap()
    ))
    .exec()
    .unwrap();
    let (numbers, display): (String, String) = lua
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
