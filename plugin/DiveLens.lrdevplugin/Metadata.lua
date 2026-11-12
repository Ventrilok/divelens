--[[
DiveLens metadata fields, shown in Lightroom's DiveLens metadata panel.

Compatibility contract: these ids and their types keep existing catalogs
valid. New fields are added, never renamed; the schema version goes up when
fields are added.

Part of DiveLens, GPL-3.0-or-later.
]]

return {
    metadataFieldsForPhotos = {
        {
            id = "DiveId",
            title = LOC "$$$/DiveLens/Metadata/DiveID=Dive",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "StartDate",
            title = LOC "$$$/DiveLens/Metadata/StartDate=Start Date",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "EndDate",
            title = LOC "$$$/DiveLens/Metadata/EndDate=End Date",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "Duration",
            title = LOC "$$$/DiveLens/Metadata/Duration=Duration",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "MaxDepth",
            title = LOC "$$$/DiveLens/Metadata/MaxDepth=Max Depth",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "TempHigh",
            title = LOC "$$$/DiveLens/Metadata/TempHigh=Temp. High",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "TempLow",
            title = LOC "$$$/DiveLens/Metadata/TempLow=Temp. Low",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "Rating",
            title = LOC "$$$/DiveLens/Metadata/Rating=Rating",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "PhotoDepth",
            title = LOC "$$$/DiveLens/Metadata/PhotoDepth=Photo Depth",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "PhotoTemp",
            title = LOC "$$$/DiveLens/Metadata/PhotoTemp=Photo Temp.",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "PhotoElapsed",
            title = LOC "$$$/DiveLens/Metadata/PhotoElapsed=Elapsed Time",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "ClockOffset",
            title = LOC "$$$/DiveLens/Metadata/ClockOffset=Clock Offset",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
    },

    -- 2: per-photo fields added (no data to migrate).
    schemaVersion = 2,
}
