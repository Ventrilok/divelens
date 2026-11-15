--[[
DiveLens metadata fields, shown in Lightroom's DiveLens metadata panel,
except the hidden one (no title), which only the plugin reads.

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
            title = LOC "$$$/DiveLens/Metadata/StartDate=Start Dive",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "EndDate",
            title = LOC "$$$/DiveLens/Metadata/EndDate=End Dive",
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
            title = LOC "$$$/DiveLens/Metadata/PhotoDepth=Depth",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            id = "PhotoTemp",
            title = LOC "$$$/DiveLens/Metadata/PhotoTemp=Temperature",
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
        {
            -- Dive application the dive data come from (MacDive, Subsurface).
            id = "Source",
            title = LOC "$$$/DiveLens/Metadata/Source=Source",
            dataType = "string",
            searchable = true,
            browsable = true,
            version = 1,
        },
        {
            -- Altitude marker (spec F7, section 7): the GPS altitude, in
            -- metres, Dive Lens last wrote on the photo, as text. It tells
            -- the altitude Dive Lens wrote from one the camera or the user
            -- set (Tagging.altitudeClears). Hidden: without a title, the
            -- field shows in no panel and cannot be searched or browsed
            -- (Lightroom SDK: searchable and browsable need a title).
            id = "WrittenAltitude",
            dataType = "string",
            version = 1,
        },
    },

    -- 2: per-photo fields added; 3: Source added; 4: WrittenAltitude added
    -- (no data to migrate).
    schemaVersion = 4,
}
