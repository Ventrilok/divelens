// Fictional sites and dives only (rules/fixtures.ts). Texts are compared word
// for word with the specification (F5) in at least one language per rule.
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import { NO_PICK } from "../tag/SitePicker";
import { reduce, start } from "./state";
import { canGoNext, otherSite, siteDetails, tagChoice, writeOptions } from "./rules";
import {
  catalog,
  catalogAt4,
  d1,
  d3,
  noDiveAt4,
  pickBlueArch,
  pickSandPatch,
  read,
  run,
  selection,
  selectionAt4,
  withOptions,
} from "./rules/fixtures";

beforeEach(() => setLanguage("en"));

// --- Next ---------------------------------------------------------------------

describe("canGoNext", () => {
  const allowed = { allowed: true, prompt: null };

  it("step 1: greyed without a prompt until a path is chosen", () => {
    expect(canGoNext(start, { selected: 4 })).toEqual({ allowed: false, prompt: null });
    expect(canGoNext(catalog, { selected: 0 })).toEqual(allowed);
  });

  it("step 1, selection path: greyed while nothing is selected in Lightroom", () => {
    expect(canGoNext(selection, { selected: 0 })).toEqual({ allowed: false, prompt: "Select photos in Lightroom" });
    setLanguage("fr");
    expect(canGoNext(selection, { selected: 0 })).toEqual({ allowed: false, prompt: "Sélectionnez des photos dans Lightroom" });
    expect(canGoNext(selection, { selected: 2 })).toEqual(allowed);
  });

  it("step 2, catalog path: asks to tick dives", () => {
    const at2 = reduce(catalog, { type: "next" });
    expect(canGoNext(at2, { selected: 0 })).toEqual({ allowed: false, prompt: "Check one or more dives" });
    setLanguage("fr");
    expect(canGoNext(at2, { selected: 0 })).toEqual({ allowed: false, prompt: "Cochez une ou plusieurs plongées" });
    expect(canGoNext(reduce(at2, { type: "toggleDive", id: "d1" }), { selected: 0 })).toEqual(allowed);
  });

  it("step 2, selection path: asks to choose a dive, or no dive", () => {
    const at2 = reduce(selection, { type: "next" });
    expect(canGoNext(at2, { selected: 3 })).toEqual({ allowed: false, prompt: "Choose a dive" });
    setLanguage("fr");
    expect(canGoNext(at2, { selected: 3 })).toEqual({ allowed: false, prompt: "Choisissez une plongée" });
    expect(canGoNext(reduce(at2, { type: "pickDive", pick: { kind: "noDive" } }), { selected: 3 })).toEqual(allowed);
    expect(canGoNext(reduce(at2, { type: "pickDive", pick: { kind: "dive", id: "d1" } }), { selected: 3 })).toEqual(allowed);
  });

  it("step 3 lets the diver go on", () => {
    expect(canGoNext(reduce(catalogAt4, { type: "back" }), { selected: 0 })).toEqual(allowed);
  });
});

// --- What is written ------------------------------------------------------------

describe("writeOptions", () => {
  it("sends what is ticked, the altitude unticked on every run", () => {
    expect(writeOptions(catalogAt4)).toEqual({ dive: true, altitude: false, place: true, gps: true, collections: true });
  });

  it("never copies depth to altitude without the dive details", () => {
    expect(writeOptions(withOptions(catalogAt4, { altitude: true })).altitude).toBe(true);
    expect(writeOptions(withOptions(catalogAt4, { altitude: true, dive: false }))).toEqual({
      dive: false,
      altitude: false,
      place: true,
      gps: true,
      collections: true,
    });
  });

  it("writes only the site for no dive", () => {
    expect(writeOptions(withOptions(noDiveAt4, { altitude: true, place: false }))).toEqual({
      dive: false,
      altitude: false,
      place: false,
      gps: true,
      collections: false,
    });
  });

  it("writes neither location nor GPS with the site details unticked", () => {
    const j = withOptions(catalogAt4, { place: false, gps: false });
    expect(siteDetails(j.options)).toBe(false);
    expect(writeOptions(j)).toMatchObject({ place: false, gps: false });
    expect(siteDetails(withOptions(catalogAt4, { place: false }).options)).toBe(true);
  });
});

describe("tagChoice", () => {
  const base = { journey: selectionAt4, dive: d1, selection: read, offsets: { a: 60 } };

  it("names the selection described and its offsets when per-photo values are written", () => {
    expect(tagChoice(base)).toEqual({
      diveId: "d1",
      location: { kind: "diveSite" },
      search: 7,
      offsets: { a: 60 },
      options: { dive: true, altitude: false, place: true, gps: true, collections: true },
    });
  });

  it("names the description the reading shows, never another", () => {
    const newer = { ...read, current: { ...read.current!, search: 9 } };
    expect(tagChoice({ ...base, selection: newer }).search).toBe(9);
  });

  it("names no selection with the dive details unticked, the dive still sent for its site and collection", () => {
    const choice = tagChoice({ ...base, journey: withOptions(selectionAt4, { dive: false }) });
    expect(choice).toMatchObject({ diveId: "d1", search: null, options: { dive: false, altitude: false } });
    expect(choice.offsets).toEqual({});
  });

  it("names no selection for a dive without a readable profile, or before the selection is described", () => {
    const unreadable = tagChoice({ ...base, selection: { ...read, needsValues: false } });
    expect(unreadable.search).toBeNull();
    expect(unreadable.offsets).toEqual({});
    const undescribed = tagChoice({ ...base, selection: { ...read, current: null } });
    expect(undescribed.search).toBeNull();
    expect(undescribed.offsets).toEqual({});
  });

  it("sends no location with neither location nor GPS ticked", () => {
    expect(tagChoice({ ...base, journey: withOptions(selectionAt4, { place: false, gps: false }) }).location).toBeNull();
    expect(tagChoice({ ...base, journey: withOptions(selectionAt4, { place: false }) }).location).toEqual({ kind: "diveSite" });
  });

  it("sends the site picked and its GPS as typed for another site", () => {
    const j = withOptions(selectionAt4, { site: "other", pick: { ...pickSandPatch, lat: "-8,25", lon: "115.5" } });
    expect(tagChoice({ ...base, journey: j }).location).toEqual({ kind: "site", siteId: "s2", lat: -8.25, lon: 115.5 });
    const empty = withOptions(selectionAt4, { site: "other", pick: pickSandPatch });
    expect(tagChoice({ ...base, journey: empty }).location).toEqual({ kind: "site", siteId: "s2", lat: null, lon: null });
  });

  it("uses the site picked when the dive has no site", () => {
    const j = run(selectionAt4, { type: "goTo", step: 2 }, { type: "pickDive", pick: { kind: "dive", id: "d3" } });
    expect(tagChoice({ ...base, journey: j, dive: d3 }).location).toBeNull();
    const picked = withOptions(j, { pick: pickBlueArch });
    expect(tagChoice({ ...base, journey: picked, dive: d3 }).location).toEqual({ kind: "site", siteId: "s1", lat: -8.5, lon: 115.25 });
  });

  it("for no dive: no dive, the site picked, only the site details", () => {
    const j = withOptions(noDiveAt4, { pick: pickBlueArch });
    expect(tagChoice({ journey: j, dive: null, selection: read, offsets: { a: 60 } })).toEqual({
      diveId: null,
      location: { kind: "site", siteId: "s1", lat: -8.5, lon: 115.25 },
      search: null,
      offsets: {},
      options: { dive: false, altitude: false, place: true, gps: true, collections: false },
    });
  });
});

describe("otherSite", () => {
  it("is absent on each dive's site", () => {
    expect(otherSite(withOptions(catalogAt4, { pick: pickBlueArch }))).toBeUndefined();
  });

  it("is the site picked, its GPS as typed, while location or GPS is ticked", () => {
    const j = withOptions(catalogAt4, { site: "other", pick: pickBlueArch });
    expect(otherSite(j)).toEqual({ kind: "site", siteId: "s1", lat: -8.5, lon: 115.25 });
    expect(otherSite(withOptions(j, { place: false }))).toEqual({ kind: "site", siteId: "s1", lat: -8.5, lon: 115.25 });
  });

  it("is absent with neither location nor GPS ticked, or no site picked yet", () => {
    const j = withOptions(catalogAt4, { site: "other", pick: pickBlueArch });
    expect(otherSite(withOptions(j, { place: false, gps: false }))).toBeUndefined();
    expect(otherSite(withOptions(j, { pick: NO_PICK }))).toBeUndefined();
  });
});

