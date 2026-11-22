//! The plug-in, the app's engine and its screens must name the same things:
//! the reasons an apply fails, the Undo names the texts quote, and the
//! screens the Lightroom menus open. Each test reads the other side as text,
//! so a rename on one side breaks here, not in front of a diver.

mod common;

use std::collections::BTreeSet;
use std::path::PathBuf;

use common::{lua, plugin_dir};

fn repo_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"))
}

fn plugin_file(name: &str) -> String {
    let path = plugin_dir().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"))
}

/// The text between the first `"` after `marker` and the next `"`, on the
/// line that holds `marker`.
fn quoted_after<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let rest = &line[line.find(marker)? + marker.len()..];
    let start = rest.find('"')? + 1;
    let end = start + rest[start..].find('"')?;
    Some(&rest[start..end])
}

/// The English and French Undo names of the key `$$$/DiveLens/Undo/<key>`.
fn undo_names(plugin_source: &str, key: &str) -> (String, String) {
    let marker = format!("$$$/DiveLens/Undo/{key}=");
    let english = plugin_source
        .lines()
        .find_map(|l| quoted_after(l, "LOC ").filter(|s| s.starts_with(&marker)))
        .unwrap_or_else(|| panic!("no LOC string {marker} in the plug-in source"))[marker.len()..]
        .to_string();
    let french = plugin_file("TranslatedStrings_fr.txt")
        .lines()
        .find_map(|l| quoted_after(l, "").filter(|s| s.starts_with(&marker)))
        .unwrap_or_else(|| panic!("no French string {marker} in TranslatedStrings_fr.txt"))
        [marker.len()..]
        .to_string();
    (english, french)
}

#[test]
fn plugin_and_app_know_the_same_reasons() {
    let lua = lua();
    let reasons: mlua::Table = lua
        .load(r#"return require("Tagging").REASONS"#)
        .eval()
        .expect("Tagging.REASONS must exist");
    let plugin: BTreeSet<String> = reasons
        .sequence_values::<String>()
        .map(Result::unwrap)
        .collect();
    let app: BTreeSet<String> = divelens_lib::APPLY_REASONS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert!(!plugin.is_empty(), "Tagging.REASONS is empty");
    assert_eq!(
        plugin, app,
        "Tagging.REASONS (plug-in) and APPLY_REASONS (app) differ"
    );
}

#[test]
fn every_reason_has_a_text() {
    let failure = repo_file("app/src/apply/failure.ts");
    // Only the switch on the plug-in's reason in `applyFailure`: from its
    // `switch (e.reason)` line to the closing brace at the same indent.
    let mut lines = failure.lines();
    let anchor = lines
        .by_ref()
        .find(|l| l.trim() == "switch (e.reason) {")
        .expect("`switch (e.reason) {` not found in app/src/apply/failure.ts");
    let indent = &anchor[..anchor.len() - anchor.trim_start().len()];
    let closing = format!("{indent}}}");
    let switch: Vec<&str> = lines.take_while(|l| l.trim_end() != closing).collect();
    assert!(
        !switch.is_empty(),
        "the switch on e.reason in failure.ts is empty"
    );
    let missing: Vec<&str> = divelens_lib::APPLY_REASONS
        .iter()
        .copied()
        .filter(|reason| {
            let wanted = format!("case \"{reason}\"");
            !switch.iter().any(|l| l.trim().starts_with(&wanted))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "the switch on e.reason in app/src/apply/failure.ts has no `case \"<reason>\"` for: {missing:?}"
    );
}

#[test]
fn the_undo_names_match() {
    let i18n = repo_file("app/src/i18n.ts");
    // The i18n has an English block, then a French one: the first and the
    // last line of a key are the English and the French text.
    let line_of = |key: &str, french: bool| -> String {
        let prefix = format!("\"{key}\":");
        let mut found = i18n
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with(&prefix));
        let line = if french {
            found.next_back()
        } else {
            found.next()
        };
        line.unwrap_or_else(|| panic!("no text {key} in app/src/i18n.ts"))
            .to_string()
    };
    for (source, key, i18n_key) in [
        ("ApplyTags.lua", "TagSelection", "tag.err.gpsNotWritten"),
        (
            "ApplyTags.lua",
            "TagSelection",
            "tag.err.gpsNotWrittenDetail",
        ),
        (
            "ApplyCollections.lua",
            "CreateCollections",
            "coll.err.gpsNotWritten",
        ),
        (
            "ApplyCollections.lua",
            "CreateCollections",
            "coll.err.gpsNotWrittenDetail",
        ),
    ] {
        let (english, french) = undo_names(&plugin_file(source), key);
        assert!(
            !english.is_empty() && !french.is_empty(),
            "empty Undo name for {key}"
        );
        assert!(
            line_of(i18n_key, false).contains(&english),
            "{i18n_key} (English) does not name the Undo \"{english}\" of {source}"
        );
        assert!(
            line_of(i18n_key, true).contains(&french),
            "{i18n_key} (French) does not name the Undo \"{french}\" of {source}"
        );
    }
}

#[test]
fn the_screens_match() {
    let sidebar = repo_file("app/src/ui/Sidebar.tsx");
    let line = sidebar
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("export type Screen ="))
        .expect("`export type Screen =` not found in app/src/ui/Sidebar.tsx");
    let accepted: BTreeSet<&str> = line
        .trim_start_matches("export type Screen =")
        .trim_end_matches(';')
        .split('|')
        .map(|n| n.trim().trim_matches('"'))
        .collect();
    assert!(
        accepted.len() >= 3 && !accepted.contains(""),
        "cannot read the Screen type: {line}"
    );

    for menu in [
        "MenuTagSelection.lua",
        "MenuCreateCollections.lua",
        "MenuSettings.lua",
    ] {
        let source = plugin_file(menu);
        let name = source
            .lines()
            .find_map(|l| quoted_after(l, "require \"Launch\".open("))
            .unwrap_or_else(|| {
                panic!("{menu} does not call require \"Launch\".open(\"<screen>\")")
            });
        assert!(
            accepted.contains(name),
            "{menu} opens screen \"{name}\", which the app does not accept: {accepted:?}"
        );
    }
}
