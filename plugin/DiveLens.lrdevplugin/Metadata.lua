--[[
DiveLens metadata fields, shown in Lightroom's DiveLens metadata panel.

Compatibility contract: these ids, their types and the schema version keep
existing catalogs valid. New fields are added, never renamed.

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
    },

    schemaVersion = 1,
}
