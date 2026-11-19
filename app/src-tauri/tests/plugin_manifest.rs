//! Tests of the plug-in's manifest: its fields, files, menus and texts, run
//! under Lua 5.1 (the version Lightroom embeds).

mod common;

use std::collections::BTreeSet;

use common::{lua, plugin_dir, run};
use mlua::Lua;

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

/// Loads the link's modules as Lightroom would, without Lightroom: `import`
/// gives a stub that accepts anything, and reading a global Lightroom does
/// not define is an error. Catches a missing module, a `require` cycle and a
/// global read while loading, which compiling alone does not.
#[test]
fn plugin_modules_load_without_lightroom() {
    run(
        &lua(),
        r#"
        local stub
        stub = setmetatable({}, {
            __index = function() return stub end,
            __call = function() return stub end,
        })
        import = function() return stub end
        _PLUGIN = { path = "/tmp", id = "com.querton.divelens" }
        WIN_ENV = false
        setmetatable(_G, { __index = function(_, k) error("undefined global " .. k, 2) end })

        -- Loads `module` and checks it has the functions named in `names`.
        local function check(module, names)
            local m = require(module)
            for name in names:gmatch("%S+") do
                assert(type(m[name]) == "function", module .. "." .. name .. ": not a function")
            end
            return m
        end
        check("Link", "start stop isStopped isConnected show showWhenConnected appSettingsFolder")
        "#,
    );
}

/// Globals a plug-in file may read: Lua 5.1's standard library, and what
/// Lightroom gives every plug-in script.
const KNOWN_GLOBALS: &[&str] = &[
    "assert",
    "error",
    "ipairs",
    "pairs",
    "pcall",
    "require",
    "select",
    "setmetatable",
    "getmetatable",
    "tonumber",
    "tostring",
    "type",
    "unpack",
    "next",
    "rawget",
    "rawset",
    "math",
    "string",
    "table",
    "os",
    "io",
    "_G",
    "_VERSION",
    "import",
    "LOC",
    "_PLUGIN",
    "WIN_ENV",
    "MAC_ENV",
];

/// Other globals the plug-in already read when this test was written: none.
/// The link state is `_G.DiveLensLink`, a field of `_G`, not a global read.
const ALREADY_READ: &[&str] = &[];

/// Globals a plug-in file may set: none.
const SET_GLOBALS: &[&str] = &[];

/// `code` compiled under Lua 5.1, as `string.dump` gives it.
fn bytecode(lua: &Lua, code: &str, name: &str) -> Vec<u8> {
    let function = lua
        .load(code)
        .set_name(name)
        .into_function()
        .unwrap_or_else(|e| panic!("{e}"));
    let dump: mlua::Function = lua
        .globals()
        .get::<mlua::Table>("string")
        .unwrap()
        .get("dump")
        .unwrap();
    dump.call::<mlua::LuaString>(function)
        .unwrap()
        .as_bytes()
        .to_vec()
}

const GETGLOBAL: u32 = 5;
const SETGLOBAL: u32 = 7;

/// Reads Lua 5.1 bytecode in the order `string.dump` writes it (ldump.c):
/// integers and lengths as wide as the header says, little-endian.
struct Bytecode<'a> {
    bytes: &'a [u8],
    at: usize,
    int: usize,
    size_t: usize,
    lua_number: usize,
}

impl<'a> Bytecode<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let bytes = self.bytes;
        self.at += n;
        &bytes[self.at - n..self.at]
    }

    fn unsigned(&mut self, size: usize) -> usize {
        self.take(size)
            .iter()
            .rev()
            .fold(0, |n, &b| (n << 8) | b as usize)
    }

    fn int(&mut self) -> usize {
        self.unsigned(self.int)
    }

    /// A string: its length counting the closing NUL, then its bytes; a
    /// length of 0 for no string.
    fn string(&mut self) -> Option<String> {
        let n = self.unsigned(self.size_t);
        (n > 0).then(|| String::from_utf8_lossy(&self.take(n)[..n - 1]).into_owned())
    }

    /// One function, then those it defines: the global names its
    /// instructions read (GETGLOBAL) and set (SETGLOBAL), the name being the
    /// constant at index Bx.
    fn function(&mut self, reads: &mut BTreeSet<String>, sets: &mut BTreeSet<String>) {
        self.string(); // source
        // First and last line; upvalue and parameter counts, vararg flag, stack size.
        self.take(2 * self.int + 4);
        let code: Vec<u32> = (0..self.int()).map(|_| self.unsigned(4) as u32).collect();
        let constants: Vec<Option<String>> = (0..self.int())
            .map(|_| match self.take(1)[0] {
                0 => None,
                1 => {
                    self.take(1);
                    None
                }
                3 => {
                    self.take(self.lua_number);
                    None
                }
                4 => self.string(),
                other => panic!("unknown constant type {other}"),
            })
            .collect();
        for instruction in code {
            let opcode = instruction & 0x3f;
            if opcode == GETGLOBAL || opcode == SETGLOBAL {
                let name = constants[(instruction >> 14) as usize]
                    .clone()
                    .expect("a global's name is a string");
                let found = if opcode == GETGLOBAL {
                    &mut *reads
                } else {
                    &mut *sets
                };
                found.insert(name);
            }
        }
        for _ in 0..self.int() {
            self.function(reads, sets);
        }
        // Debug information: each instruction's line, local variables, upvalue names.
        let lines = self.int();
        self.take(lines * self.int);
        for _ in 0..self.int() {
            self.string();
            self.take(2 * self.int);
        }
        for _ in 0..self.int() {
            self.string();
        }
    }
}

/// The global names a compiled chunk reads and sets, in every function.
fn globals(bytes: &[u8]) -> (BTreeSet<String>, BTreeSet<String>) {
    // Signature, version 5.1, official format, little-endian; then the sizes
    // of int, size_t, an instruction and a number, and the integral flag.
    assert_eq!(&bytes[..7], b"\x1bLua\x51\x00\x01", "not Lua 5.1 bytecode");
    assert_eq!(bytes[9], 4, "instructions are not 32 bits");
    let mut chunk = Bytecode {
        bytes,
        at: 12,
        int: bytes[7].into(),
        size_t: bytes[8].into(),
        lua_number: bytes[10].into(),
    };
    let (mut reads, mut sets) = (BTreeSet::new(), BTreeSet::new());
    chunk.function(&mut reads, &mut sets);
    assert_eq!(chunk.at, bytes.len(), "bytecode not read to its end");
    (reads, sets)
}

/// Every global each plug-in file reads, function bodies included, is a
/// known one. A `local` of one file used from another still compiles, as a
/// global that is nil when it runs: only Lightroom would show it, and only
/// on the branch that uses it.
#[test]
fn plugin_globals_are_known() {
    let lua = lua();
    let names = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>();
    // The reader itself, on globals read and set at the top and in a nested function.
    let (reads, sets) = globals(&bytecode(
        &lua,
        "outside = inside; local function f() return nested end",
        "sample",
    ));
    assert_eq!(reads, names(&["inside", "nested"]));
    assert_eq!(sets, names(&["outside"]));

    let readable = |name: &str| KNOWN_GLOBALS.contains(&name) || ALREADY_READ.contains(&name);
    let mut unknown = Vec::new();
    for entry in std::fs::read_dir(plugin_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "lua") {
            let file = path.file_name().unwrap().to_string_lossy().into_owned();
            let code = std::fs::read_to_string(&path).unwrap();
            let (reads, sets) = globals(&bytecode(&lua, &code, &file));
            for name in reads.iter().filter(|name| !readable(name)) {
                unknown.push(format!("{file} reads {name}"));
            }
            for name in sets
                .iter()
                .filter(|name| !SET_GLOBALS.contains(&name.as_str()))
            {
                unknown.push(format!("{file} sets {name}"));
            }
        }
    }
    assert!(unknown.is_empty(), "unknown globals: {unknown:?}");
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
