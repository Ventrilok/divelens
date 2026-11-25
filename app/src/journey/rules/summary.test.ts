// Step 5 of the journey. Fictional sites and dives only (fixtures.ts). Texts
// are compared word for word with the specification (F5).
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../i18n";
import { reduce } from "../state";
import { summary, type Applied } from "../rules";
import { after7, before3, catalogAt4, log, noDiveAt4, pickBlueArch, run, selectionAt4, withOptions } from "./fixtures";

beforeEach(() => setLanguage("en"));

// --- Step 5 -----------------------------------------------------------------------

describe("summary, catalog path", () => {
  const j = run(catalogAt4, { type: "goTo", step: 2 }, { type: "toggleDive", id: "d3" }, { type: "next" }, { type: "next" }, { type: "applied" });
  const applied: Applied = {
    path: "catalog",
    result: {
      collections: [
        { collection: "12 - Blue Arch", count: 20, created: true },
        { collection: "13 - Sand Patch", count: 6, created: false },
      ],
      photos: 26,
      withDepth: 15,
      emptyDives: ["d3"],
    },
    outside: 9,
    cameras: [
      { key: "a", model: "Olympus TG-6", serial: null, photos: 30, remembered: null },
      { key: "b", model: "Phone", serial: null, photos: 5, remembered: null },
    ],
    ignored: ["b"],
  };

  it("recalls the run, tells what was written and warns, in both languages", () => {
    expect(summary({ journey: j, applied, dives: log.dives })).toEqual({
      head: "From 30 Dec 2025 to 2 Jan 2026 · 1 camera · dive details, site details, collections",
      stats: [
        { label: "Photos tagged", n: 26 },
        { label: "With depth", n: 15 },
        { label: "Collections", n: 2 },
      ],
      warnings: [
        "Dive 14: no photo, no collection created. If the clock changed that day, handle it separately with its own offset.",
        "9 photos outside the dives got nothing.",
      ],
    });
    setLanguage("fr");
    expect(summary({ journey: j, applied, dives: log.dives })).toEqual({
      head: "Du 30 déc. 2025 au 2 janv. 2026 · 1 appareil · détails de la plongée, détails du site, collections",
      stats: [
        { label: "Photos taguées", n: 26 },
        { label: "Avec profondeur", n: 15 },
        { label: "Collections", n: 2 },
      ],
      warnings: [
        "Plongée 14 : aucune photo, pas de collection créée. Si l'horloge a changé ce jour-là, traitez-la à part avec son propre décalage.",
        "9 photos hors plongée n'ont rien reçu.",
      ],
    });
  });

  it("names only the options ticked, and a single collection in the singular", () => {
    const only = withOptions(j, { dive: false, place: false, gps: false });
    const one = { ...applied, result: { ...applied.result, collections: [applied.result.collections[0]], emptyDives: [] }, outside: 0 };
    const s = summary({ journey: only, applied: one, dives: log.dives });
    expect(s.head).toBe("From 30 Dec 2025 to 2 Jan 2026 · 1 camera · collections");
    expect(s.stats[2]).toEqual({ label: "Collection", n: 1 });
    expect(s.warnings).toEqual([]);
  });
});

describe("summary, selection path", () => {
  const j = reduce(selectionAt4, { type: "applied" });
  const applied: Applied = {
    path: "selection",
    result: { count: 3, withDepth: 2, collection: { collection: "12 - Blue Arch", count: 3, created: true } },
    outside: [after7],
    outsideMore: 0,
    perPhoto: true,
  };

  it("recalls the run, tells what was written and the photos without depth, in both languages", () => {
    expect(summary({ journey: j, applied, dives: log.dives })).toEqual({
      head: "The 3 selected photos · dive 12 Blue Arch · dive details, site details, collection",
      stats: [
        { label: "Photos tagged", n: 3 },
        { label: "With depth", n: 2 },
        { label: "Collection", n: 1 },
      ],
      warnings: ["1 photo without depth: taken after getting out of the water."],
    });
    setLanguage("fr");
    expect(summary({ journey: j, applied, dives: log.dives })).toEqual({
      head: "Les 3 photos sélectionnées · plongée 12 Blue Arch · détails de la plongée, détails du site, collection",
      stats: [
        { label: "Photos taguées", n: 3 },
        { label: "Avec profondeur", n: 2 },
        { label: "Collection", n: 1 },
      ],
      warnings: ["1 photo sans profondeur : prise après la sortie de l'eau."],
    });
  });

  it("tells each side of the dive, or only a count when not every photo is listed", () => {
    const sides = { ...applied, result: { ...applied.result, count: 4, withDepth: 1 }, outside: [before3, { ...before3 }, after7] };
    expect(summary({ journey: j, applied: sides, dives: log.dives }).warnings).toEqual([
      "2 photos without depth: taken before going into the water.",
      "1 photo without depth: taken after getting out of the water.",
    ]);
    const more = { ...applied, result: { ...applied.result, count: 9, withDepth: 2 }, outsideMore: 4 };
    expect(summary({ journey: j, applied: more, dives: log.dives }).warnings).toEqual(["7 photos without depth."]);
    setLanguage("fr");
    expect(summary({ journey: j, applied: more, dives: log.dives }).warnings).toEqual(["7 photos sans profondeur."]);
  });

  it("says nothing of depth when no per-photo value was written", () => {
    expect(summary({ journey: j, applied: { ...applied, perPhoto: false }, dives: log.dives }).warnings).toEqual([]);
  });

  it("no dive: the photos and the site details, no collection", () => {
    const at5 = reduce(withOptions(noDiveAt4, { pick: pickBlueArch }), { type: "applied" });
    const s = summary({
      journey: at5,
      applied: { ...applied, result: { count: 3, withDepth: 0, collection: null }, outside: [], perPhoto: false },
      dives: log.dives,
    });
    expect(s.head).toBe("The 3 selected photos · site details");
    expect(s.stats).toEqual([
      { label: "Photos tagged", n: 3 },
      { label: "With depth", n: 0 },
      { label: "Collections", n: 0 },
    ]);
  });
});
