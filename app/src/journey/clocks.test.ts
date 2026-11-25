// Runs in UTC+14 (vitest.config.ts): every time below is a wall-clock time
// and must come out as written.
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../i18n";
import {
  catalogBar,
  catalogOutside,
  catalogRows,
  outsideByCamera,
  selectionBar,
  selectionOutside,
  selectionRow,
  selectionWaiting,
} from "./clocks";
import type { Camera, CollectionsPreview, Dive, OutsidePhoto, TagPreview } from "../types";

beforeEach(() => setLanguage("en"));

/** A fictional dive. */
const dive = (id: string, number: number | null, site: string | null, hasProfile = true): Dive => ({
  id,
  number,
  start: "2026-03-06T14:40:00",
  end: "2026-03-06T15:38:00",
  durationS: 3480,
  maxDepthM: 18,
  tempHighC: 27,
  tempLowC: 26,
  rating: null,
  site: { id: site && `s-${id}`, name: site, location: null, country: null, lat: null, lon: null },
  hasProfile,
  invalid: false,
  notes: null,
});

const camera = (key: string, model: string | null): Camera => ({ key, model, serial: null, photos: 10, remembered: null });

const cameras = [camera("a", "Olympus TG-6"), camera("b", "Sony A7"), camera("c", null)];

const d1 = dive("d1", 706, "Coral Steps");
const d2 = dive("d2", 707, "Blue Arch");
const d3 = dive("d3", 708, "Sand Patch", false);

const preview: CollectionsPreview = {
  dives: [
    { id: "d1", photos: 0, inDive: 0, cameras: 0 },
    { id: "d2", photos: 26, inDive: 24, cameras: 2 },
    { id: "d3", photos: 9, inDive: 9, cameras: 1 },
  ],
  cameras: [
    { key: "a", inDives: 20, outside: 58 },
    { key: "b", inDives: 15, outside: 0 },
    { key: "c", inDives: 0, outside: 7 },
  ],
  total: 35,
  outside: 65,
  ignored: 0,
};

const photo = (time: string, minutes: number): OutsidePhoto => ({ time, camera: "a", minutes });

/** Five photos selected, four in the dive, one taken seven minutes after it. */
const counts: TagPreview = {
  total: 5,
  inDive: 4,
  cameras: [{ key: "a", inDives: 4, outside: 1 }],
  coverFrom: null,
  coverTo: null,
  outside: [photo("2026-03-06T15:45:00", 7)],
  outsideMore: 0,
};

describe("the preview by dive", () => {
  it("catalog path: counts each dive, and points out a dive without photo", () => {
    const rows = catalogRows([d1, d2, d3], preview);
    expect(rows.map((r) => [r.dive.id, r.photos, r.cameras, r.perPhoto, r.warn])).toEqual([
      ["d1", 0, 0, "No photo — clock changed?", true],
      ["d2", 26, 2, "Depth · temp. · time", false],
      ["d3", 9, 1, "No profile: dive data only", false],
    ]);
  });

  it("catalog path: says when a dive has only surface photos", () => {
    const p = { ...preview, dives: [{ id: "d2", photos: 3, inDive: 0, cameras: 1 }] };
    expect(catalogRows([d2], p)[0].perPhoto).toBe("Surface photos only");
  });

  it("catalog path: waits for the counts", () => {
    expect(catalogRows([d2, d3], null).map((r) => [r.photos, r.perPhoto])).toEqual([
      [null, "Depth · temp. · time"],
      [null, "No profile: dive data only"],
    ]);
  });

  it("selection path: one row, every photo selected attached to the dive", () => {
    const row = selectionRow(d2, counts, false);
    expect([row.photos, row.cameras, row.perPhoto, row.warn]).toEqual([5, 1, "Depth · temp. · time", false]);
  });

  it("selection path: no photo in the dive, no profile, or a profile that cannot be read", () => {
    expect(selectionRow(d2, { ...counts, inDive: 0 }, false).perPhoto).toBe("Surface photos only");
    expect(selectionRow(d3, counts, false).perPhoto).toBe("No profile: dive data only");
    expect(selectionRow(d2, counts, true).perPhoto).toBe("Profile could not be read: no per-photo data");
    expect(selectionRow(d2, null, false).photos).toBeNull();
  });

  it("in French", () => {
    setLanguage("fr");
    expect(catalogRows([d1], preview)[0].perPhoto).toBe("Aucune photo — horloge changée ?");
    expect(selectionRow(d2, counts, false).perPhoto).toBe("Profondeur · temp. · temps");
  });
});

describe("the photos outside the dives", () => {
  it("names each camera that has some, in the cameras' order", () => {
    expect(outsideByCamera(cameras, preview.cameras)).toBe("58 Olympus TG-6 · 7 Unknown camera");
  });

  it("catalog path: the block with its count, its text and what to do", () => {
    expect(catalogOutside(cameras, preview)).toEqual({
      title: "Outside the dives · 65 photos",
      lines: ["Photos from the period that fall in no dive, even with the margin: on land, on the boat… They get nothing."],
      cameras: "58 Olympus TG-6 · 7 Unknown camera",
      how: 'To attach some to a dive anyway, or give them only a location: select them in Lightroom, then choose "The selected photos" at the Photos step.',
    });
  });

  it("catalog path, in French", () => {
    setLanguage("fr");
    const block = catalogOutside(cameras, preview);
    expect(block?.title).toBe("Hors plongée · 65 photos");
    expect(block?.cameras).toBe("58 Olympus TG-6 · 7 Appareil inconnu");
  });

  it("catalog path: no block without photos outside, or before the counts", () => {
    expect(catalogOutside(cameras, { ...preview, outside: 0 })).toBeNull();
    expect(catalogOutside(cameras, null)).toBeNull();
  });

  it("selection path: each photo outside the dive's times, and their number by camera", () => {
    expect(selectionOutside(cameras, counts)).toEqual({
      title: "Outside the dive's times · 1 photo",
      lines: [
        "Taken at 15:45, 7 minutes after getting out of the water. Attached to the dive anyway: it gets its data and its location, without depth.",
      ],
      cameras: "1 Olympus TG-6",
      how: null,
    });
  });

  it("selection path: counts the photos not detailed", () => {
    const many = { ...counts, outside: [photo("2026-03-06T14:30:00", -10)], outsideMore: 6 };
    const block = selectionOutside(cameras, many);
    expect(block?.title).toBe("Outside the dive's times · 7 photos");
    expect(block?.lines[block.lines.length - 1]).toBe("and 6 more");
  });

  it("selection path, in French", () => {
    setLanguage("fr");
    expect(selectionOutside(cameras, counts)?.title).toBe("Hors de l'horaire de la plongée · 1 photo");
  });

  it("selection path: no block when every photo is in the dive, or before the counts", () => {
    expect(selectionOutside(cameras, { ...counts, outside: [], outsideMore: 0 })).toBeNull();
    expect(selectionOutside(cameras, null)).toBeNull();
  });
});

describe("the bottom bar", () => {
  it("catalog path: the total, and each dive without photo", () => {
    expect(catalogBar([d1, d2, d3], preview)).toEqual({
      total: "35 photos in 2 dives · 65 outside the dives",
      emptyDives: ["Dive 706 Coral Steps: no photo — clock changed?"],
    });
  });

  it("catalog path, in French", () => {
    setLanguage("fr");
    expect(catalogBar([d1, d2, d3], preview)).toEqual({
      total: "35 photos dans 2 plongées · 65 hors plongée",
      emptyDives: ["Plongée 706 Coral Steps : aucune photo — horloge changée ?"],
    });
  });

  it("catalog path: nothing before the counts", () => {
    expect(catalogBar([d1, d2], null)).toEqual({ total: null, emptyDives: [] });
  });

  const lr = { connected: true, selected: 5 };
  const ok = { status: "ok" as const };
  const reading = {
    described: { photos: { search: 1, photos: 5, undated: 0, cameras }, version: 1 },
    counts,
    profile: ok,
    needsValues: true,
  };

  it("selection path: every photo attached, with depth, and nothing else to set", () => {
    expect(selectionBar(d2, reading, lr)).toEqual({
      total: "5 photos attached to 707 Blue Arch · 4 with depth",
      waiting: null,
      hint: "Nothing else to set.",
    });
  });

  it("selection path, in French", () => {
    setLanguage("fr");
    expect(selectionBar(d2, reading, lr)).toEqual({
      total: "5 photos rattachées à 707 Blue Arch · 4 avec profondeur",
      waiting: null,
      hint: "Rien d'autre à régler.",
    });
  });

  it("selection path: no depth without a profile that can be read", () => {
    expect(selectionBar(d3, { ...reading, needsValues: false }, lr).total).toBe(
      "5 photos attached to 708 Sand Patch · 0 with depth",
    );
    const unread = { described: null, counts: null, profile: ok, needsValues: false };
    expect(selectionBar(d3, unread, lr).total).toBe("5 photos attached to 708 Sand Patch · 0 with depth");
  });

  it("selection path: says what the cards wait for instead of the total", () => {
    const waiting = { described: null, counts: null, profile: ok, needsValues: true };
    expect(selectionBar(d2, waiting, lr)).toEqual({
      total: null,
      waiting: "Reading the selection in Lightroom…",
      hint: null,
    });
    expect(selectionBar(d2, waiting, { connected: false, selected: 5 })).toEqual({
      total: null,
      waiting: null,
      hint: null,
    });
  });
});

describe("what the camera clocks wait for, on the selection path", () => {
  const lr = { connected: true, selected: 5 };
  const none = { described: null, counts: null, profile: { status: "ok" as const } };

  it("the selection read in Lightroom, then the profile", () => {
    expect(selectionWaiting(none, lr)).toBe("Reading the selection in Lightroom…");
    expect(selectionWaiting({ ...none, profile: { status: "loading" } }, lr)).toBe("Loading profile…");
  });

  it("photos selected in Lightroom", () => {
    expect(selectionWaiting(none, { connected: true, selected: 0 })).toBe("Select photos in Lightroom");
  });

  it("nothing once the cards are there, nor while Lightroom is away (the warning says it)", () => {
    const described = { photos: { search: 1, photos: 5, undated: 0, cameras }, version: 1 };
    expect(selectionWaiting({ described, counts, profile: { status: "ok" } }, lr)).toBeNull();
    expect(selectionWaiting(none, { connected: false, selected: 5 })).toBeNull();
  });

  it("in French", () => {
    setLanguage("fr");
    expect(selectionWaiting(none, lr)).toBe("Lecture de la sélection dans Lightroom…");
  });
});
