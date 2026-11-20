//! Tests of the plug-in's manifest: its fields, files, menus and texts, run
//! under Lua 5.1 (the version Lightroom embeds).

mod common;

use common::{lua, plugin_dir, run};

#[test]
fn plugin_keeps_its_identifier_and_fields() {
    let lua = lua();
    run(
        &lua,
        r#"
        assert(require "Info".LrToolkitIdentifier == "com.querton.divelens")
        local metadata = require "Metadata"
        assert(metadata.schemaVersion == 4)
        local ids, hidden = {}, {}
        for _, field in ipairs(metadata.metadataFieldsForPhotos) do
            assert(field.dataType == "string" and field.version == 1, field.id)
            ids[#ids + 1] = field.id
            if field.title == nil then
                -- The SDK: searchable and browsable only with a title.
                assert(field.searchable == nil and field.browsable == nil and field.readOnly == nil, field.id)
                hidden[#hidden + 1] = field.id
            end
        end
        assert(table.concat(ids, ",") == "DiveId,StartDate,EndDate,Duration,MaxDepth,TempHigh,TempLow,Rating,"
            .. "PhotoDepth,PhotoTemp,PhotoElapsed,ClockOffset,Source,WrittenAltitude",
            table.concat(ids, ","))
        -- The altitude marker only: the plugin's own record, never shown.
        assert(table.concat(hidden, ",") == "WrittenAltitude", table.concat(hidden, ","))
        assert(require "Tagging".MARKER == "WrittenAltitude")
        "#,
    );
}

#[test]
fn panel_shows_every_field_once_in_titled_sections() {
    run(
        &lua(),
        r#"
        local tagset = require "Tagset"
        local seen, labels = {}, {}
        for _, item in ipairs(tagset.items) do
            if type(item) == "table" then
                assert(item[1] == "com.adobe.label" and item.label ~= "", "bad section")
                labels[#labels + 1] = item.label
            else
                local id = item:match("^com%.querton%.divelens%.(.+)$")
                if id then
                    assert(not seen[id], id .. " shown twice")
                    seen[id] = true
                end
            end
        end
        for _, field in ipairs(require "Metadata".metadataFieldsForPhotos) do
            if field.title then
                assert(seen[field.id], field.id .. " missing from the panel")
            else
                assert(not seen[field.id], field.id .. " is hidden: not in the panel")
            end
        end
        assert(table.concat(labels, ",") == "Photo,Dive,Site,Technical", table.concat(labels, ","))
        "#,
    );
}

#[test]
fn every_plugin_file_compiles_under_lua_5_1() {
    let lua = lua();
    for entry in std::fs::read_dir(plugin_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "lua") {
            let code = std::fs::read_to_string(&path).unwrap();
            if let Err(e) = lua
                .load(&code)
                .set_name(path.display().to_string())
                .into_function()
            {
                panic!("{e}");
            }
        }
    }
}

#[test]
fn menu_items_open_existing_plugin_files() {
    let lua = lua();
    let menus: Vec<(String, String)> = lua
        .load(
            r#"
            local info, found = require "Info", {}
            for _, list in ipairs { info.LrLibraryMenuItems, info.LrHelpMenuItems } do
                for _, item in ipairs(list) do found[#found + 1] = { item.title, item.file } end
            end
            return found
            "#,
        )
        .eval::<Vec<Vec<String>>>()
        .unwrap()
        .into_iter()
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect();
    assert_eq!(
        menus.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
        [
            "Create Collections…",
            "Tag Selection…",
            "Dive Lens Settings…"
        ]
    );
    for (title, file) in &menus {
        assert!(plugin_dir().join(file).is_file(), "{title}: no {file}");
    }
}

/// The `$$$/DiveLens/…` texts of `source`: key and text, in order. A text runs
/// from after the `=` to the closing quote (no plugin text holds a quote).
fn localized_texts(source: &str) -> Vec<(String, String)> {
    let mut texts = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("$$$/DiveLens/") {
        rest = &rest[start..];
        let end = rest.find('"').expect("unterminated text");
        let (key, text) = rest[..end].split_once('=').expect("text without a key");
        texts.push((key.to_string(), text.to_string()));
        rest = &rest[end..];
    }
    texts
}

/// The `^1`…`^9` placeholders of a text, sorted.
fn placeholders(text: &str) -> Vec<char> {
    let mut found: Vec<char> = text
        .split('^')
        .skip(1)
        .filter_map(|s| s.chars().next().filter(char::is_ascii_digit))
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn every_plugin_text_has_its_french_translation() {
    let dir = plugin_dir();
    let mut english = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "lua") {
            let code = std::fs::read_to_string(&path).unwrap();
            for (key, text) in localized_texts(&code) {
                if let Some(other) = english.insert(key.clone(), text.clone()) {
                    assert_eq!(other, text, "{key}: two English texts");
                }
            }
        }
    }
    let french: std::collections::BTreeMap<_, _> =
        localized_texts(&std::fs::read_to_string(dir.join("TranslatedStrings_fr.txt")).unwrap())
            .into_iter()
            .collect();
    for (key, text) in &english {
        let fr = french
            .get(key)
            .unwrap_or_else(|| panic!("{key}: no French text"));
        assert!(!fr.trim().is_empty(), "{key}: empty French text");
        assert_eq!(
            placeholders(fr),
            placeholders(text),
            "{key}: placeholders differ"
        );
    }
    for key in french.keys() {
        assert!(english.contains_key(key), "{key}: translated, never used");
    }
}
