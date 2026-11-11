--[[
DiveLens plugin manifest.

The plugin is invisible: it declares the DiveLens metadata fields, keeps a
local link with the DiveLens app, and offers two Library menu items that open
the app on the matching screen. All user interface lives in the app.

The version must match app/package.json (checked by the app's tests).

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

    VERSION = { major = 0, minor = 1, revision = 0 },
}
