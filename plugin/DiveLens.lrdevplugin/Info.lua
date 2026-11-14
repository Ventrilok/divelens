--[[
DiveLens plugin manifest.

The plugin is invisible: it declares the DiveLens metadata fields, keeps a
local link with the DiveLens app, and offers two Library menu items that open
the app on the matching screen. All user interface lives in the app.

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

    LrLibraryMenuItems = {
        {
            title = LOC "$$$/DiveLens/Menu/CreateCollections=Create Collections…",
            file = "MenuCreateCollections.lua",
        },
        {
            title = LOC "$$$/DiveLens/Menu/TagSelection=Tag Selection…",
            file = "MenuTagSelection.lua",
            enabledWhen = "photosSelected",
        },
    },

    -- `display` is what the Plug-in Manager shows: the full version, with a
    -- pre-release label if any (2.1.0-beta.1). The numbers hold MAJOR.MINOR.PATCH
    -- only; without `display` Lightroom formats them itself (0.1.0.0).
    VERSION = { major = 0, minor = 1, revision = 0, display = "0.1.0" },
}
