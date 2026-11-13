--[[
The "Dive Lens" choice in Lightroom's Metadata panel: DiveLens fields
together with the location fields DiveLens fills.

Part of DiveLens, GPL-3.0-or-later.
]]

return {
    title = LOC "$$$/DiveLens/PluginName=Dive Lens",
    id = "DiveLensTagset",

    items = {
        "com.querton.divelens.DiveId",
        "com.adobe.location",
        "com.adobe.city",
        "com.adobe.country",
        "com.adobe.GPS",
        "com.adobe.separator",
        "com.querton.divelens.StartDate",
        "com.querton.divelens.EndDate",
        "com.querton.divelens.Duration",
        "com.adobe.separator",
        "com.querton.divelens.TempHigh",
        "com.querton.divelens.TempLow",
        "com.querton.divelens.MaxDepth",
        "com.adobe.separator",
        "com.querton.divelens.PhotoDepth",
        "com.querton.divelens.PhotoTemp",
        "com.querton.divelens.PhotoElapsed",
        "com.querton.divelens.ClockOffset",
        "com.querton.divelens.Source",
        "com.adobe.separator",
        "com.querton.divelens.Rating",
    },
}
