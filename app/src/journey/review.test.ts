// Step 4 of the journey, as the screen shows it. Fictional sites and dives only
// (rules/fixtures.ts). Texts are compared word for word with the specification (F5).
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { NO_PICK } from "../tag/SitePicker";
import type { ApplyError, CollectionsResult, Period, TagApplied } from "../types";
import {
  applyPlan,
  catalogApplied,
  chooseOtherSite,
  collectionName,
  collectionPath,
  collectionsText,
  knowLines,
  perPhotoSent,
  readAgain,
  reviewBar,
  reviewBoxes,
  reviewEffects,
  selectionApplied,
  siteDetailsTicked,
  siteRadios,
  staleAnswer,
  type Box,
} from "./review";
import { canApply, tagChoice } from "./rules";
import {
  after7,
  catalogAt4,
  counts,
  d1,
  d3,
  input,
  log,
  noDiveAt4,
  pickBlueArch,
  preview,
  read,
  ready,
  run,
  selectionAt4,
  withOptions,
} from "./rules/fixtures";

beforeEach(() => setLanguage("en"));

const on: Box = { checked: true, enabled: true };
const off: Box = { checked: false, enabled: true };
const greyed: Box = { checked: false, enabled: false };

describe("reviewBoxes", () => {
  it("every option ticked but the altitude, on every run", () => {
    expect(reviewBoxes(catalogAt4)).toEqual({ dive: on, altitude: off, site: on, place: on, gps: on, collections: on });
    expect(reviewBoxes(selectionAt4)).toEqual(reviewBoxes(catalogAt4));
  });

  it("the altitude is greyed, unticked, without the dive details", () => {
    expect(reviewBoxes(withOptions(catalogAt4, { altitude: true })).altitude).toEqual(on);
    expect(reviewBoxes(withOptions(catalogAt4, { altitude: true, dive: false })).altitude).toEqual(greyed);
  });

  it("site details unticked: location and GPS unticked and greyed", () => {
    const b = reviewBoxes(withOptions(catalogAt4, { place: false, gps: false }));
    expect(b.site).toEqual(off);
    expect(b.place).toEqual(greyed);
    expect(b.gps).toEqual(greyed);
    expect(reviewBoxes(withOptions(catalogAt4, { gps: false })).site).toEqual(on);
  });

  it("no dive: dive details, altitude and collections visible but greyed, unticked", () => {
    expect(reviewBoxes(withOptions(noDiveAt4, { altitude: true }))).toEqual({
      dive: greyed,
      altitude: greyed,
      site: on,
      place: on,
      gps: on,
      collections: greyed,
    });
  });
});

describe("siteDetailsTicked", () => {
  it("unticking it unticks location and GPS; ticking it ticks both", () => {
    expect(siteDetailsTicked(false)).toEqual({ place: false, gps: false });
    expect(siteDetailsTicked(true)).toEqual({ place: true, gps: true });
  });
});

describe("reviewEffects", () => {
  it("each effect, in full when the option writes something", () => {
    expect(reviewEffects(input(catalogAt4))).toEqual({
      dive: { text: "26 photos · 15 with depth", on: true },
      altitude: { text: "No", on: false },
      site: { text: "26 photos · location and GPS", on: true },
      collections: { text: "2 collections", on: true },
    });
    expect(reviewEffects(input(withOptions(catalogAt4, { altitude: true }))).altitude).toEqual({ text: "15 photos", on: true });
  });

  it("muted when the option writes nothing", () => {
    const none = withOptions(catalogAt4, { dive: false, altitude: true, place: false, gps: false, collections: false });
    const e = reviewEffects(input(none));
    expect([e.dive.on, e.altitude.on, e.site.on, e.collections.on]).toEqual([false, false, false, false]);
    // GPS alone, and no dive site has a GPS position: nothing for the site either.
    const gpsOnly = withOptions(catalogAt4, { place: false });
    const onlySandPatch = { search: ready, preview: { ...preview, dives: [preview.dives[1]], total: 6 }, previewFailed: false };
    expect(reviewEffects(input(gpsOnly, { catalog: onlySandPatch })).site).toEqual({ text: "Nothing", on: false });
  });

  it("no dive: only the site", () => {
    const e = reviewEffects(input(withOptions(noDiveAt4, { pick: pickBlueArch })));
    expect(e.dive).toEqual({ text: "Nothing", on: false });
    expect(e.site).toEqual({ text: "3 photos · location and GPS", on: true });
    expect(e.collections).toEqual({ text: "None", on: false });
  });
});

describe("siteRadios", () => {
  it("catalog path: each dive's site, or another site the same for all photos", () => {
    expect(siteRadios(catalogAt4, log.dives)).toEqual({
      source: "diveSite",
      diveSite: { label: "Each dive's site", enabled: true },
      other: { label: "Other site, the same for all photos…", enabled: true },
      picker: false,
    });
    setLanguage("fr");
    const other = siteRadios(withOptions(catalogAt4, { site: "other" }), log.dives);
    expect(other.diveSite.label).toBe("Site de chaque plongée");
    expect(other.other.label).toBe("Autre site, le même pour toutes les photos…");
    expect(other.source).toBe("other");
    expect(other.picker).toBe(true);
  });

  it("selection path: the dive's site by name, or another site", () => {
    expect(siteRadios(selectionAt4, log.dives)).toEqual({
      source: "diveSite",
      diveSite: { label: "Dive's site: Blue Arch", enabled: true },
      other: { label: "Other site…", enabled: true },
      picker: false,
    });
    setLanguage("fr");
    expect(siteRadios(selectionAt4, log.dives).diveSite.label).toBe("Site de la plongée : Blue Arch");
  });

  it("a dive without a site: another site, the dive's greyed, as before", () => {
    const j = run(selectionAt4, { type: "goTo", step: 2 }, { type: "pickDive", pick: { kind: "dive", id: "d3" } });
    expect(siteRadios(j, log.dives)).toEqual({
      source: "other",
      diveSite: { label: "Dive's site: this dive has no site", enabled: false },
      other: { label: "Other site…", enabled: true },
      picker: true,
    });
  });

  it("no dive: another site, the dive's greyed", () => {
    expect(siteRadios(noDiveAt4, log.dives)).toEqual({
      source: "other",
      diveSite: { label: "Dive's site: no dive chosen", enabled: false },
      other: { label: "Other site…", enabled: true },
      picker: true,
    });
  });

  it("site details unticked: both greyed, no picker", () => {
    const r = siteRadios(withOptions(catalogAt4, { site: "other", place: false, gps: false }), log.dives);
    expect([r.diveSite.enabled, r.other.enabled, r.picker]).toEqual([false, false, false]);
  });
});

describe("chooseOtherSite", () => {
  it("opens the picker on the chosen dive's country and location, as before", () => {
    expect(chooseOtherSite(selectionAt4, log.dives)).toEqual({
      site: "other",
      pick: { ...NO_PICK, country: "Nowhere Islands", location: "North Bay" },
    });
  });

  it("keeps a site already picked", () => {
    expect(chooseOtherSite(withOptions(selectionAt4, { pick: pickBlueArch }), log.dives)).toEqual({ site: "other", pick: pickBlueArch });
  });

  it("catalog path: opens the picker empty", () => {
    expect(chooseOtherSite(catalogAt4, log.dives)).toEqual({ site: "other", pick: NO_PICK });
  });
});

describe("collectionName: the engine's \"number - site\"", () => {
  it("a dive's number and site, or what it has, else its start", () => {
    expect(collectionName(d1)).toBe("12 - Blue Arch");
    expect(collectionName(d3)).toBe("14");
    expect(collectionName({ ...d1, number: null })).toBe("2025-12-30 09:00 - Blue Arch");
    expect(collectionName({ ...d3, number: null })).toBe("2026-01-02 08:00");
  });
});

describe("collectionPath", () => {
  const place = { parent: ["Trips", "2025"], group: "Dive Lens" };

  it("the collection under the place chosen in Settings", () => {
    expect(collectionPath(place, "12 - Blue Arch")).toBe("Trips › 2025 › Dive Lens › 12 - Blue Arch");
    expect(collectionPath({ parent: [], group: "Dive Lens" }, "12 - Blue Arch")).toBe("Dive Lens › 12 - Blue Arch");
  });

  it("at the catalog's root, the collection alone", () => {
    expect(collectionPath({ parent: [], group: "  " }, "12 - Blue Arch")).toBe("12 - Blue Arch");
  });

  it("without a collection, the place alone", () => {
    expect(collectionPath(place, null)).toBe("Trips › 2025 › Dive Lens");
    expect(collectionPath({ parent: [], group: "" }, null)).toBe("Catalog root");
  });
});

describe("collectionsText", () => {
  const place = { parent: ["Trips"], group: "Dive Lens" };

  it("catalog path: one collection per dive with photos, the first as an example, in both languages", () => {
    expect(collectionsText(catalogAt4, place, log.dives, preview)).toBe(
      'One "number - site" collection per dive that has photos, for example Trips › Dive Lens › 12 - Blue Arch.',
    );
    setLanguage("fr");
    expect(collectionsText(catalogAt4, place, log.dives, preview)).toBe(
      "Une collection « numéro - site » par plongée qui a des photos, par exemple Trips › Dive Lens › 12 - Blue Arch.",
    );
  });

  it("catalog path: the example is a dive that gets photos", () => {
    const noneIn12 = { ...preview, dives: [{ ...preview.dives[0], photos: 0 }, preview.dives[1]] };
    expect(collectionsText(catalogAt4, place, log.dives, noneIn12)).toContain("Trips › Dive Lens › 13 - Sand Patch.");
    expect(collectionsText(catalogAt4, place, log.dives, null)).toContain("Trips › Dive Lens › 12 - Blue Arch.");
  });

  it("selection path: the chosen dive's collection", () => {
    expect(collectionsText(selectionAt4, place, log.dives, null)).toBe('The dive\'s "number - site" collection: Trips › Dive Lens › 12 - Blue Arch.');
    setLanguage("fr");
    expect(collectionsText(selectionAt4, place, log.dives, null)).toBe(
      "La collection « numéro - site » de la plongée : Trips › Dive Lens › 12 - Blue Arch.",
    );
  });

  it("no dive: the place alone", () => {
    expect(collectionsText(noDiveAt4, place, log.dives, null)).toBe('The dive\'s "number - site" collection: Trips › Dive Lens.');
  });
});

describe("reviewBar", () => {
  it("what Apply writes, once nothing blocks it", () => {
    expect(reviewBar(input(catalogAt4))).toEqual({ text: "26 photos · 2 collections", blocked: false });
    expect(reviewBar(input(selectionAt4))).toEqual({ text: "3 photos · 1 collection", blocked: false });
  });

  it("what blocks Apply, in its place", () => {
    const none = withOptions(catalogAt4, { dive: false, place: false, gps: false, collections: false });
    expect(reviewBar(input(none))).toEqual({ text: "Tick at least one option", blocked: true });
    expect(reviewBar(input(selectionAt4, { lr: { connected: true, selected: 0 } }))).toEqual({
      text: "Select photos in Lightroom",
      blocked: true,
    });
  });

  it("catalog path: nothing while the photos are being counted", () => {
    expect(reviewBar(input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: false } }))).toEqual({
      text: null,
      blocked: false,
    });
  });
});

describe("knowLines", () => {
  it("catalog path: a dive without photo in orange, the photos outside the dives not", () => {
    const j = run(catalogAt4, { type: "goTo", step: 2 }, { type: "toggleDive", id: "d3" }, { type: "next" }, { type: "next" });
    const withEmpty = { ...preview, dives: [...preview.dives, { id: "d3", photos: 0, inDive: 0, cameras: 0 }] };
    expect(knowLines(input(j, { catalog: { search: ready, preview: withEmpty, previewFailed: false } }))).toEqual([
      { text: "Dive 14: no photo, no collection.", warn: true },
      { text: "9 photos outside the dives get nothing: on land, on the boat…", warn: false },
    ]);
  });

  it("selection path: the photos without depth", () => {
    expect(knowLines(input(selectionAt4))).toEqual([
      { text: "1 photo taken after getting out of the water (09:57): dive data and location, without depth.", warn: false },
    ]);
  });
});

describe("what Apply hands to the summary", () => {
  it("catalog path: the result, the photos outside the dives and the cameras of the search", () => {
    const result: CollectionsResult = { collections: [], photos: 26, withDepth: 15, emptyDives: [] };
    const period: Period = (ready as Extract<typeof ready, { state: "ready" }>).period;
    const choices = { offsets: {}, ignored: ["b"], marginMin: 0 };
    expect(catalogApplied(result, preview, period, choices)).toEqual({
      path: "catalog",
      result,
      outside: 9,
      cameras: period.cameras,
      ignored: ["b"],
    });
  });

  it("selection path: per-photo values only when the description was sent, with the photos outside it", () => {
    const result: TagApplied = { count: 3, withDepth: 2, collection: null };
    const sent = tagChoice({ journey: selectionAt4, dive: d1, selection: read, offsets: {} });
    expect(selectionApplied(result, sent, counts)).toEqual({ path: "selection", result, outside: [after7], outsideMore: 0, perPhoto: true });
    const noDetails = tagChoice({ journey: withOptions(selectionAt4, { dive: false }), dive: d1, selection: read, offsets: {} });
    expect(selectionApplied(result, noDetails, { ...counts, outsideMore: 4 })).toEqual({
      path: "selection",
      result,
      outside: [],
      outsideMore: 0,
      perPhoto: false,
    });
  });
});

describe("applyPlan: what a click on Apply writes, null exactly when Apply is greyed", () => {
  const period = (ready as Extract<typeof ready, { state: "ready" }>).period;

  it("catalog path: the search shown and its counts", () => {
    expect(applyPlan(input(catalogAt4), false)).toEqual({ path: "catalog", period, preview });
  });

  it("selection path: the selection at the moment of the click", () => {
    expect(applyPlan(input(selectionAt4), false)).toEqual({ path: "selection" });
  });

  it("agrees with canApply", () => {
    const cases = [
      input(catalogAt4),
      input(catalogAt4, { catalog: { search: { state: "idle" }, preview, previewFailed: false } }),
      input(catalogAt4, { catalog: { search: ready, preview: null, previewFailed: false } }),
      input(catalogAt4, { catalog: { search: { state: "searching" }, preview, previewFailed: false } }),
      input(catalogAt4, { parentMissing: true }),
      input(selectionAt4),
      input(selectionAt4, { lr: { connected: true, selected: 0 } }),
      input(withOptions(selectionAt4, { dive: false, place: false, gps: false, collections: false })),
    ];
    for (const i of cases) {
      for (const writing of [false, true]) expect(applyPlan(i, writing) !== null).toBe(canApply(i, writing));
    }
  });
});

describe("staleAnswer: Lightroom's answer to an earlier run, or once the step is gone, changes nothing", () => {
  it("only the answer to the run shown, on the step shown, counts", () => {
    expect(staleAnswer(3, 3, true)).toBe(false);
    expect(staleAnswer(3, 4, true)).toBe(true);
    expect(staleAnswer(3, 3, false)).toBe(true);
  });
});

describe("perPhotoSent", () => {
  it("the description of the selection was sent, with the dive details", () => {
    const sent = tagChoice({ journey: selectionAt4, dive: d1, selection: read, offsets: {} });
    expect(perPhotoSent(sent)).toBe(true);
    expect(perPhotoSent({ ...sent, search: null })).toBe(false);
    // Whatever is sent with the dive details unticked, no per-photo value is written.
    expect(perPhotoSent({ ...sent, options: { ...sent.options, dive: false } })).toBe(false);
  });

  it("selectionApplied follows it", () => {
    const result: TagApplied = { count: 3, withDepth: 0, collection: null };
    const sent = tagChoice({ journey: selectionAt4, dive: d1, selection: read, offsets: {} });
    const noDetails = { ...sent, options: { ...sent.options, dive: false } };
    expect(selectionApplied(result, noDetails, counts)).toMatchObject({ perPhoto: false, outside: [], outsideMore: 0 });
  });
});

describe("readAgain: what is read again once Lightroom answered, as before", () => {
  const none = { searchAgain: false, describeAgain: false, clearTouched: false };
  const busy: ApplyError = { error: "lightroom", reason: "busy", detail: null };

  it("catalog path: the photos are looked for again only when the search is gone", () => {
    expect(readAgain("catalog", true, null)).toEqual(none);
    expect(readAgain("catalog", true, { error: "noPhotos" })).toEqual({ ...none, searchAgain: true });
    expect(readAgain("catalog", true, { error: "lightroom", reason: "staleSearch", detail: null })).toEqual({ ...none, searchAgain: true });
    expect(readAgain("catalog", true, busy)).toEqual(none);
  });

  it("selection path, written: described again with per-photo values sent; the offsets set by hand forgotten", () => {
    expect(readAgain("selection", true, null)).toEqual({ ...none, describeAgain: true, clearTouched: true });
    expect(readAgain("selection", false, null)).toEqual({ ...none, clearTouched: true });
  });

  it("selection path, refused: described again when the selection moved on, or Lightroom answered with per-photo values", () => {
    expect(readAgain("selection", false, { error: "noPhotos" })).toEqual({ ...none, describeAgain: true });
    expect(readAgain("selection", false, { error: "lightroom", reason: "staleSelection", detail: null })).toEqual({
      ...none,
      describeAgain: true,
    });
    expect(readAgain("selection", true, busy)).toEqual({ ...none, describeAgain: true });
    expect(readAgain("selection", false, busy)).toEqual(none);
    expect(readAgain("selection", true, { error: "notConnected" })).toEqual(none);
  });
});
