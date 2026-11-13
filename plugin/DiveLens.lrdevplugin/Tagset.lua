--[[
The "Dive Lens" choice in Lightroom's Metadata panel: DiveLens fields
together with the location fields DiveLens fills, in four titled sections.
The photo's own values come first: they are what one looks for on a photo
(spec section 7).

Part of DiveLens, GPL-3.0-or-later.
]]

local function section(label)
    return { "com.adobe.label", label = label }
end

return {
    title = LOC "$$$/DiveLens/PluginName=Dive Lens",
    id = "DiveLensTagset",

    items = {
        section(LOC "$$$/DiveLens/Tagset/Photo=Photo"),
        "com.querton.divelens.PhotoDepth",
        "com.querton.divelens.PhotoTemp",
        "com.querton.divelens.PhotoElapsed",
        "com.adobe.separator",
        section(LOC "$$$/DiveLens/Tagset/Dive=Dive"),
        "com.querton.divelens.DiveId",
        "com.querton.divelens.StartDate",
        "com.querton.divelens.EndDate",
        "com.querton.divelens.Duration",
        "com.querton.divelens.MaxDepth",
        "com.querton.divelens.TempLow",
        "com.querton.divelens.TempHigh",
        "com.querton.divelens.Rating",
        "com.adobe.separator",
        section(LOC "$$$/DiveLens/Tagset/Site=Site"),
        "com.adobe.location",
        "com.adobe.city",
        "com.adobe.country",
        "com.adobe.GPS",
        "com.adobe.separator",
        section(LOC "$$$/DiveLens/Tagset/Technical=Technical"),
        "com.querton.divelens.ClockOffset",
        "com.querton.divelens.Source",
    },
}
