// Step 4 of the journey. Fictional sites and dives only (fixtures.ts). Texts
// are compared word for word with the specification (F5).
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../i18n";
import { applyCount, barSummary, canApply, effects, goodToKnow, otherSite, reviewBlocker, reviewCounts, unitsText } from "../rules";
import {
  after7,
  before3,
  catalogAt4,
  counts,
  input,
  noDiveAt4,
  nothingRead,
  pickBlueArch,
  pickSandPatch,
  preview,
  read,
  ready,
  run,
  selectionAt4,
  withOptions,
} from "./fixtures";

beforeEach(() => setLanguage("en"));

// --- Step 4: what blocks Apply ------------------------------------------------

describe("reviewBlocker, catalog path", () => {
  it("lets Apply through when everything is in place", () => {
    expect(reviewBlocker(input(catalogAt4))).toBeNull();
  });

  it("says Lightroom is disconnected first", () => {
    const all = input(withOptions(catalogAt4, { dive: false, place: false, gps: false, collections: false }), {
      lr: { connected: false, selected: 0 },
      catalog: { search: { state: "searching" }, preview: null, previewFailed: true },
      parentMissing: true,
    });
    expect(reviewBlocker(all)).toBe("Lightroom not connected: photos cannot be found or written.");
  });

  it("says the photos are being looked for, or why they could not be", () => {
    const searching = { search: { state: "searching" }, preview: null, previewFailed: false } as const;
    expect(reviewBlocker(input(catalogAt4, { catalog: searching, parentMissing: true }))).toBe("Looking for the photos in Lightroom…");
    const failed = { search: { state: "failed", message: "Search failed" }, preview: null, previewFailed: false } as const;
    expect(reviewBlocker(input(catalogAt4, { catalog: failed, parentMissing: true }))).toBe("Search failed");
  });

  it("names a missing collection set only when collections are created", () => {
    expect(reviewBlocker(input(catalogAt4, { parentMissing: true }))).toBe(
      "The collection set chosen in Settings (Trips › 2025) is no longer in the catalog. Choose another place in Settings.",
    );
    setLanguage("fr");
    expect(reviewBlocker(input(catalogAt4, { parentMissing: true }))).toBe(
      "L'ensemble choisi dans les réglages (Trips › 2025) n'est plus dans le catalogue. Choisissez un autre emplacement dans les réglages.",
    );
    expect(reviewBlocker(input(withOptions(catalogAt4, { collections: false }), { parentMissing: true }))).toBeNull();
  });

  it("keeps the missing collection set before failed counts, as before", () => {
    const failed = { search: ready, preview: null, previewFailed: true };
    expect(reviewBlocker(input(catalogAt4, { catalog: failed, parentMissing: true }))).toContain("Trips › 2025");
    expect(reviewBlocker(input(catalogAt4, { catalog: failed }))).toBe(
      "The photos found are no longer available: go back to the dives and search again.",
    );
  });

  it("asks to tick at least one option when all three are unticked", () => {
    const none = withOptions(catalogAt4, { dive: false, place: false, gps: false, collections: false });
    expect(reviewBlocker(input(none))).toBe("Tick at least one option");
    setLanguage("fr");
    expect(reviewBlocker(input(none))).toBe("Cochez au moins une option");
    expect(reviewBlocker(input(withOptions(none, { gps: true })))).toBeNull();
  });

  it("asks to choose a site for another site, while location or GPS is ticked", () => {
    const other = withOptions(catalogAt4, { site: "other" });
    expect(reviewBlocker(input(other))).toBe("Choose a site");
    setLanguage("fr");
    expect(reviewBlocker(input(other))).toBe("Choisissez un site");
    expect(reviewBlocker(input(withOptions(other, { place: false, gps: false })))).toBeNull();
    expect(reviewBlocker(input(withOptions(other, { pick: pickBlueArch })))).toBeNull();
  });

  it("refuses a GPS position typed wrong", () => {
    const bad = withOptions(catalogAt4, { site: "other", pick: { ...pickBlueArch, lat: "91" } });
    expect(reviewBlocker(input(bad))).toBe("Latitude between -90 and 90, longitude between -180 and 180, or both empty.");
  });

  it("ignores a GPS position typed wrong while GPS is unticked: nothing of it is sent", () => {
    const placeOnly = withOptions(catalogAt4, { site: "other", gps: false, pick: { ...pickBlueArch, lat: "91" } });
    expect(reviewBlocker(input(placeOnly))).toBeNull();
    expect(otherSite(placeOnly)).toEqual({ kind: "site", siteId: "s1", lat: null, lon: null });
  });

  it("location alone on a site without names: nothing to write, Apply greyed", () => {
    const j = run(catalogAt4, { type: "goTo", step: 2 }, { type: "setDives", ids: ["d4"] }, { type: "next" }, { type: "next" });
    const placeOnly = withOptions(j, { dive: false, gps: false, collections: false });
    const d4Photos = {
      search: ready,
      preview: { ...preview, dives: [{ id: "d4", photos: 5, inDive: 5, cameras: 1 }], total: 5, outside: 0 },
      previewFailed: false,
    };
    expect(reviewBlocker(input(placeOnly, { catalog: d4Photos }))).toBe("No photo falls in the dives: adjust the clock offsets.");
    // Its GPS position is something to write.
    expect(reviewBlocker(input(withOptions(placeOnly, { gps: true }), { catalog: d4Photos }))).toBeNull();
    // The same site picked as another site.
    const picked = withOptions(catalogAt4, { dive: false, gps: false, collections: false, site: "other", pick: { ...pickSandPatch, siteId: "s3" } });
    expect(reviewBlocker(input(picked))).toBe("No photo falls in the dives: adjust the clock offsets.");
  });

  it("says there is nothing to write when no photo falls in the dives", () => {
    const empty = { search: ready, preview: { ...preview, dives: [], total: 0 }, previewFailed: false };
    expect(reviewBlocker(input(catalogAt4, { catalog: empty }))).toBe("No photo falls in the dives: adjust the clock offsets.");
  });

  it("GPS alone with dive sites without GPS: nothing to write, Apply greyed", () => {
    const gpsOnly = withOptions(catalogAt4, { dive: false, place: false, collections: false });
    const onlySandPatch = { search: ready, preview: { ...preview, dives: [preview.dives[1]], total: 6 }, previewFailed: false };
    expect(reviewBlocker(input(gpsOnly, { catalog: onlySandPatch }))).toBe("No photo falls in the dives: adjust the clock offsets.");
    // Dive 12, at a site with GPS, has photos too: its GPS is written.
    expect(reviewBlocker(input(gpsOnly))).toBeNull();
  });

  it("GPS alone with another site without GPS: nothing to write, unless a position is typed", () => {
    const gpsOnly = withOptions(catalogAt4, { dive: false, place: false, collections: false, site: "other", pick: pickSandPatch });
    expect(reviewBlocker(input(gpsOnly))).toBe("No photo falls in the dives: adjust the clock offsets.");
    expect(reviewBlocker(input(withOptions(gpsOnly, { pick: { ...pickSandPatch, lat: "-8.3", lon: "115.4" } })))).toBeNull();
  });
});

describe("reviewBlocker, selection path", () => {
  it("lets Apply through when everything is in place", () => {
    expect(reviewBlocker(input(selectionAt4))).toBeNull();
  });

  it("says Lightroom is disconnected first", () => {
    expect(reviewBlocker(input(selectionAt4, { lr: { connected: false, selected: 0 } }))).toBe(
      "Lightroom not connected: nothing can be written",
    );
  });

  it("asks to select photos when the selection becomes empty", () => {
    const empty = input(selectionAt4, { lr: { connected: true, selected: 0 }, selection: { ...read, current: null } });
    expect(reviewBlocker(empty)).toBe("Select photos in Lightroom");
    setLanguage("fr");
    expect(reviewBlocker(empty)).toBe("Sélectionnez des photos dans Lightroom");
  });

  it("waits for the reading of the selection, then of the profile, when per-photo values are written", () => {
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, readFailure: "Reading failed", profile: { status: "loading" } } }))).toBe(
      "Reading failed",
    );
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, profile: { status: "loading" }, current: null } }))).toBe(
      "Loading profile…",
    );
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, current: null } }))).toBe("Reading the selection in Lightroom…");
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, freshCounts: null } }))).toBe("Reading the selection in Lightroom…");
  });

  it("does not wait for the reading with the dive details unticked", () => {
    const waiting = { ...read, readFailure: "Reading failed", profile: { status: "loading" }, current: null } as const;
    expect(reviewBlocker(input(withOptions(selectionAt4, { dive: false }), { selection: waiting }))).toBeNull();
  });

  it("names a missing collection set only when the collection is created", () => {
    expect(reviewBlocker(input(selectionAt4, { parentMissing: true }))).toContain("Trips › 2025");
    expect(reviewBlocker(input(withOptions(selectionAt4, { collections: false }), { parentMissing: true }))).toBeNull();
  });

  it("asks for another site when the dive has no site", () => {
    const j = run(selectionAt4, { type: "goTo", step: 2 }, { type: "pickDive", pick: { kind: "dive", id: "d3" } }, { type: "next" }, { type: "next" });
    expect(reviewBlocker(input(j))).toBe("Choose a site");
    expect(reviewBlocker(input(withOptions(j, { pick: pickBlueArch })))).toBeNull();
  });

  it("GPS alone with a dive site without GPS: nothing to write, Apply greyed", () => {
    const j = run(selectionAt4, { type: "goTo", step: 2 }, { type: "pickDive", pick: { kind: "dive", id: "d2" } }, { type: "next" }, { type: "next" });
    const gpsOnly = withOptions(j, { dive: false, place: false, collections: false });
    expect(reviewBlocker(input(gpsOnly, { selection: nothingRead }))).toBe("Nothing was written: choose a dive or a site first.");
    setLanguage("fr");
    expect(reviewBlocker(input(gpsOnly, { selection: nothingRead }))).toBe(
      "Rien n'a été écrit : choisissez d'abord une plongée ou un site.",
    );
  });

  it("ignores a GPS position typed wrong while GPS is unticked", () => {
    const placeOnly = withOptions(selectionAt4, { site: "other", gps: false, pick: { ...pickBlueArch, lat: "91" } });
    expect(reviewBlocker(input(placeOnly))).toBeNull();
    expect(reviewBlocker(input(withOptions(placeOnly, { gps: true })))).toBe(
      "Latitude between -90 and 90, longitude between -180 and 180, or both empty.",
    );
  });

  it("location alone on a dive site without names: nothing to write, Apply greyed", () => {
    const j = run(selectionAt4, { type: "goTo", step: 2 }, { type: "pickDive", pick: { kind: "dive", id: "d4" } }, { type: "next" }, { type: "next" });
    const placeOnly = withOptions(j, { dive: false, gps: false, collections: false });
    expect(reviewBlocker(input(placeOnly, { selection: nothingRead }))).toBe("Nothing was written: choose a dive or a site first.");
  });

  it("asks to tick at least one option when all three are unticked", () => {
    const none = withOptions(selectionAt4, { dive: false, place: false, gps: false, collections: false });
    expect(reviewBlocker(input(none))).toBe("Tick at least one option");
  });

  it("checks the selection's reading before the collection set, as the photos are checked first", () => {
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, current: null }, parentMissing: true }))).toBe(
      "Reading the selection in Lightroom…",
    );
    expect(reviewBlocker(input(selectionAt4, { selection: { ...read, readFailure: "Reading failed" }, parentMissing: true }))).toBe(
      "Reading failed",
    );
  });

  it("no dive: a missing collection set does not block, as no collection is created", () => {
    expect(reviewBlocker(input(withOptions(noDiveAt4, { pick: pickBlueArch }), { parentMissing: true }))).toBeNull();
  });

  it("no dive: only the site details count, and a site must be chosen", () => {
    expect(reviewBlocker(input(noDiveAt4))).toBe("Choose a site");
    expect(reviewBlocker(input(withOptions(noDiveAt4, { place: false, gps: false })))).toBe("Tick at least one option");
    expect(reviewBlocker(input(withOptions(noDiveAt4, { pick: pickBlueArch })))).toBeNull();
  });
});

describe("canApply", () => {
  it("catalog path: no blocker, the counts in, no write in progress", () => {
    expect(canApply(input(catalogAt4), false)).toBe(true);
    expect(canApply(input(catalogAt4), true)).toBe(false);
    expect(canApply(input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: false } }), false)).toBe(false);
    expect(canApply(input(catalogAt4, { parentMissing: true }), false)).toBe(false);
  });

  it("selection path: no blocker, no write in progress", () => {
    expect(canApply(input(selectionAt4), false)).toBe(true);
    expect(canApply(input(selectionAt4), true)).toBe(false);
    expect(canApply(input(selectionAt4, { lr: { connected: true, selected: 0 } }), false)).toBe(false);
  });
});

describe("applyCount", () => {
  it("catalog path: the photos found in the dives, 0 until they are counted", () => {
    expect(applyCount(input(catalogAt4))).toBe(26);
    // Not only the photos that get something on them (20 here): every photo found, as before.
    expect(applyCount(input(withOptions(catalogAt4, { dive: false, place: false, collections: false })))).toBe(26);
    expect(applyCount(input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: false } }))).toBe(0);
  });

  it("selection path: the photos selected in Lightroom", () => {
    expect(applyCount(input(selectionAt4, { lr: { connected: true, selected: 24 } }))).toBe(24);
  });
});

// --- Step 4: the effect of each option ------------------------------------------

describe("reviewCounts", () => {
  it("counts the photos written, with a depth, given a site, and the collections (catalog)", () => {
    expect(reviewCounts(input(catalogAt4))).toEqual({ dive: 26, depth: 15, site: 26, place: true, gps: true, collections: 2, photos: 26 });
  });

  it("counts the selection (selection path)", () => {
    expect(reviewCounts(input(selectionAt4))).toEqual({ dive: 3, depth: 2, site: 3, place: true, gps: true, collections: 1, photos: 3 });
  });
});

describe("effects", () => {
  it("tells each option's effect on this run, in both languages (catalog)", () => {
    expect(effects(input(catalogAt4))).toEqual({
      dive: "26 photos · 15 with depth",
      altitude: "No",
      site: "26 photos · location and GPS",
      collections: "2 collections",
    });
    setLanguage("fr");
    expect(effects(input(catalogAt4))).toEqual({
      dive: "26 photos · 15 avec profondeur",
      altitude: "Non",
      site: "26 photos · lieu et GPS",
      collections: "2 collections",
    });
  });

  it("copies the depth to the altitude of the photos with a depth", () => {
    expect(effects(input(withOptions(catalogAt4, { altitude: true }))).altitude).toBe("15 photos");
  });

  it("says nothing is written for an option unticked", () => {
    const none = withOptions(catalogAt4, { dive: false, altitude: true, place: false, gps: false, collections: false });
    expect(effects(input(none))).toEqual({ dive: "Nothing", altitude: "No", site: "Nothing", collections: "None" });
    setLanguage("fr");
    expect(effects(input(none))).toEqual({ dive: "Rien", altitude: "Non", site: "Rien", collections: "Aucune" });
  });

  it("tells the site's part ticked, and only the photos of dives whose site gives it", () => {
    expect(effects(input(withOptions(catalogAt4, { gps: false }))).site).toBe("26 photos · location");
    expect(effects(input(withOptions(catalogAt4, { place: false }))).site).toBe("20 photos · GPS");
    setLanguage("fr");
    expect(effects(input(withOptions(catalogAt4, { gps: false }))).site).toBe("26 photos · lieu");
  });

  it("selection path: the selection, the photos taken during the dive, one collection", () => {
    expect(effects(input(selectionAt4))).toEqual({
      dive: "3 photos · 2 with depth",
      altitude: "No",
      site: "3 photos · location and GPS",
      collections: "1 collection",
    });
  });

  it("no dive: only the site, the other options greyed", () => {
    expect(effects(input(withOptions(noDiveAt4, { pick: pickBlueArch, altitude: true })))).toEqual({
      dive: "Nothing",
      altitude: "No",
      site: "3 photos · location and GPS",
      collections: "None",
    });
  });
});

describe("goodToKnow", () => {
  it("catalog path: dives without photo and photos outside the dives, in both languages", () => {
    const j = run(catalogAt4, { type: "goTo", step: 2 }, { type: "toggleDive", id: "d3" }, { type: "next" }, { type: "next" });
    const withEmpty = { search: ready, preview: { ...preview, dives: [...preview.dives, { id: "d3", photos: 0, inDive: 0, cameras: 0 }] }, previewFailed: false };
    expect(goodToKnow(input(j, { catalog: withEmpty }))).toEqual([
      "Dive 14: no photo, no collection.",
      "9 photos outside the dives get nothing: on land, on the boat…",
    ]);
    setLanguage("fr");
    expect(goodToKnow(input(j, { catalog: withEmpty }))).toEqual([
      "Plongée 14 : aucune photo, pas de collection.",
      "9 photos hors plongée ne reçoivent rien : à terre, sur le bateau…",
    ]);
  });

  it("catalog path: nothing to say before the counts arrive", () => {
    expect(goodToKnow(input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: false } }))).toEqual([]);
  });

  it("selection path: the photos without depth, by side of the dive, with their times", () => {
    expect(goodToKnow(input(selectionAt4))).toEqual([
      "1 photo taken after getting out of the water (09:57): dive data and location, without depth.",
    ]);
    setLanguage("fr");
    expect(goodToKnow(input(selectionAt4))).toEqual([
      "1 photo prise après la sortie de l'eau (09:57) : infos de la plongée et lieu, sans profondeur.",
    ]);
    const both = { ...counts, total: 5, inDive: 2, outside: [before3, { ...before3, time: "2025-12-30T08:59:00" }, after7] };
    expect(goodToKnow(input(selectionAt4, { selection: { ...read, freshCounts: both } }))).toEqual([
      "2 photos prises avant la mise à l'eau (08:57, 08:59) : infos de la plongée et lieu, sans profondeur.",
      "1 photo prise après la sortie de l'eau (09:57) : infos de la plongée et lieu, sans profondeur.",
    ]);
  });

  it("selection path: only a count when not every photo without depth is listed", () => {
    const more = { ...counts, total: 10, inDive: 2, outsideMore: 3 };
    expect(goodToKnow(input(selectionAt4, { selection: { ...read, freshCounts: more } }))).toEqual([
      "8 photos without depth: dive data and location.",
    ]);
    setLanguage("fr");
    expect(goodToKnow(input(selectionAt4, { selection: { ...read, freshCounts: more } }))).toEqual([
      "8 photos sans profondeur : infos de la plongée et lieu.",
    ]);
  });

  it("selection path: nothing about depth when no per-photo value is written", () => {
    expect(goodToKnow(input(withOptions(selectionAt4, { dive: false })))).toEqual([]);
    expect(goodToKnow(input(selectionAt4, { selection: { ...read, needsValues: false } }))).toEqual([]);
    expect(goodToKnow(input(selectionAt4, { selection: { ...read, freshCounts: { ...counts, inDive: 3, outside: [] } } }))).toEqual([]);
  });
});

describe("unitsText", () => {
  it("names the units in use, in both languages", () => {
    expect(unitsText({ depth: "m", temp: "c" })).toBe("Units: m, °C");
    expect(unitsText({ depth: "ft", temp: "f" })).toBe("Units: ft, °F");
    setLanguage("fr");
    expect(unitsText({ depth: "m", temp: "c" })).toBe("Unités : m, °C");
  });
});

describe("barSummary", () => {
  it("photos and collections", () => {
    expect(barSummary(input(catalogAt4))).toBe("26 photos · 2 collections");
    setLanguage("fr");
    expect(barSummary(input(catalogAt4))).toBe("26 photos · 2 collections");
    expect(barSummary(input(selectionAt4))).toBe("3 photos · 1 collection");
  });

  it("photos without a collection", () => {
    const j = withOptions(catalogAt4, { collections: false });
    expect(barSummary(input(j))).toBe("26 photos · no collection");
    setLanguage("fr");
    expect(barSummary(input(j))).toBe("26 photos · pas de collection");
  });

  it("collections only: nothing written on the photos", () => {
    const j = withOptions(catalogAt4, { dive: false, place: false, gps: false });
    expect(barSummary(input(j))).toBe("2 collections · nothing is written on the photos");
    setLanguage("fr");
    expect(barSummary(input(j))).toBe("2 collections · rien n'est écrit sur les photos");
  });

  it("counts only the photos that get something", () => {
    expect(barSummary(input(withOptions(catalogAt4, { dive: false, place: false, collections: false })))).toBe(
      "20 photos · no collection",
    );
  });
});

