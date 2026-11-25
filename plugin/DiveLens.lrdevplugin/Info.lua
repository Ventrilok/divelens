--[[
DiveLens plugin manifest.

The plugin is invisible: it declares the DiveLens metadata fields, keeps a
local link with the DiveLens app, and offers one Library menu item that
opens the app on its journey and one Help menu item that opens it on
Settings. All user interface lives in the app.

The version is a copy of app/package.json's: change it with
`npm run set-version <version>` from app/, never by hand (the app's tests
fail when the two differ). Lightroom cannot read package.json: this file may
not use require or io.

Part of DiveLens, GPL-3.0-or-later.
]]

return {
    LrSdkVersion = 10.0,
    LrSdkMinimumVersion = 6.0,
    LrToolkitIdentifier = "com.querton.divelens", -- compatibility contract: never change
    LrPluginName = LOC "$$$/DiveLens/PluginName=Dive Lens",
    LrPluginInfoUrl = "https://github.com/Ventrilok/divelens",

    LrInitPlugin = "Init.lua",
    LrForceInitPlugin = true, -- start the link when Lightroom starts, not on first use
    LrEnablePlugin = "Init.lua",
    LrDisablePlugin = "Disable.lua",
    LrShutdownPlugin = "Shutdown.lua",
    LrShutdownApp = "Shutdown.lua",

    LrPluginInfoProvider = "PluginInfoProvider.lua",
    LrMetadataProvider = "Metadata.lua",
    LrMetadataTagsetFactory = "Tagset.lua",

    -- Always enabled, with or without photos selected (spec section 3).
    LrLibraryMenuItems = {
        {
            title = LOC "$$$/DiveLens/Menu/Open=Open Dive Lens…",
            file = "MenuOpen.lua",
        },
    },

    -- Help › Plug-in Extras: Lightroom puts plug-in items there, never in the
    -- Help menu itself. The submenu mixes every plug-in's items, hence the
    -- name in the title.
    LrHelpMenuItems = {
        {
            title = LOC "$$$/DiveLens/Menu/Settings=Dive Lens Settings…",
            file = "MenuSettings.lua",
        },
    },

    -- `display` is what the Plug-in Manager shows: the full version, with a
    -- pre-release label if any (2.1.0-beta.1). The numbers hold MAJOR.MINOR.PATCH
    -- only; without `display` Lightroom formats them itself (0.1.0.0).
    VERSION = { major = 2, minor = 2, revision = 0, display = "2.2.0" },
}
