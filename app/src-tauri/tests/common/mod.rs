//! Shared by the integration tests: the plugin's folder, and a Lua 5.1 state
//! (the version Lightroom embeds) that loads the plugin's modules.

use std::path::PathBuf;

use mlua::Lua;

pub fn plugin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugin/DiveLens.lrdevplugin")
}

/// Lua that finds the plugin's modules with `require`, with a `LOC` that
/// returns the English text, as Lightroom does without a translation.
pub fn lua() -> Lua {
    let lua = Lua::new();
    let path = plugin_dir().join("?.lua");
    lua.load(format!(
        "package.path = {:?}\n\
         LOC = function(s) return (s:gsub('^%$%$%$/[^=]*=', '')) end",
        path.to_str().unwrap()
    ))
    .exec()
    .unwrap();
    lua
}
