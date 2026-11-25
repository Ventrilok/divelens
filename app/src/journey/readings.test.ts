// Runs in UTC+14 (vitest.config.ts): every time below is a wall-clock time.
import { describe, expect, it } from "vitest";
import {
  camerasToSuggest,
  currentDescription,
  diveKey,
  freshCountsOf,
  keysAfterSearch,
  marginAfterRun,
  marginAfterSearch,
  notesOfCameras,
  offsetsAfterSearch,
  onlyCameras,
  readsSelection,
  suggestionNotes,
  withSuggestions,
} from "./readings";
import type { Camera, Dive, SelectionPhotos, TagPreview } from "../types";

/** A fictional dive. */
const dive = (id: string, start: string): Dive => ({
  id,
  number: null,
  start,
  end: start.replace("T09", "T10"),
  durationS: 3600,
  maxDepthM: 18,
  tempHighC: null,
  tempLowC: null,
  rating: null,
  site: { id: null, name: null, location: null, country: null, lat: null, lon: null },
  hasProfile: true,
  invalid: false,
  notes: null,
});

const camera = (key: string, remembered: number | null = null): Camera => ({
  key,
  model: key,
  serial: null,
  photos: 10,
  remembered,
});

const dives = [
  dive("b", "2026-03-07T09:00:00"),
  dive("a", "2026-03-06T09:00:00"),
  dive("c", "2026-03-08T09:00:00"),
];

describe("the dives a photo search names", () => {
  it("are those of the log, by start time, whatever the order they were ticked in", () => {
    expect(diveKey(dives, ["c", "a", "b"])).toBe("a,b,c");
    expect(diveKey(dives, new Set(["b", "a"]))).toBe("a,b");
  });

  it("leave out dives no longer in the log, and are empty without dives", () => {
    expect(diveKey(dives, ["gone", "b"])).toBe("b");
    expect(diveKey(dives, [])).toBe("");
  });
});

describe("what the diver set, after a photo search", () => {
  const cameras = [camera("TG-6", 240), camera("Z9")];

  it("starts from the remembered offsets when other dives were found", () => {
    expect(offsetsAfterSearch({ "TG-6": 30, Z9: 15 }, cameras, false)).toEqual({ "TG-6": 240, Z9: 0 });
    expect(keysAfterSearch(new Set(["Z9"]), cameras, false)).toEqual(new Set());
  });

  it("keeps it when the same dives were found again, Lightroom having come back", () => {
    expect(offsetsAfterSearch({ "TG-6": 30, Z9: 15 }, cameras, true)).toEqual({ "TG-6": 30, Z9: 15 });
    expect(keysAfterSearch(new Set(["Z9"]), cameras, true)).toEqual(new Set(["Z9"]));
  });

  it("keeps only the cameras still found, and a new camera starts from its remembered offset", () => {
    const now = [camera("TG-6", 240), camera("iPhone", 60)];
    expect(offsetsAfterSearch({ "TG-6": 30, Z9: 15 }, now, true)).toEqual({ "TG-6": 30, iPhone: 60 });
    expect(keysAfterSearch(new Set(["Z9", "TG-6"]), now, true)).toEqual(new Set(["TG-6"]));
  });
});

describe("what the diver set, after the selection was described", () => {
  it("keeps the cameras still in the selection", () => {
    const cameras = [camera("TG-6")];
    expect(onlyCameras(new Set(["TG-6", "Z9"]), cameras)).toEqual(new Set(["TG-6"]));
    expect(notesOfCameras({ "TG-6": "", Z9: "none" }, cameras)).toEqual({ "TG-6": "" });
  });
});

describe("the surface margin, on a new run", () => {
  it("is kept when the run goes on with dives ticked, as 'handle separately' kept it", () => {
    expect(marginAfterRun(15, ["d2"])).toBe(15);
  });

  it("starts from 0 again otherwise", () => {
    expect(marginAfterRun(15, [])).toBe(0);
  });
});

describe("the surface margin, after a photo search", () => {
  it("is kept when the same dives were found again, Lightroom having come back", () => {
    expect(marginAfterSearch(15, true, false)).toBe(15);
  });

  it("is left as the new run set it, on the run's first search", () => {
    expect(marginAfterSearch(15, false, true)).toBe(15);
  });

  it("starts from 0 again when other dives were ticked in the same run", () => {
    expect(marginAfterSearch(15, false, false)).toBe(0);
  });
});

describe("Suggest", () => {
  const cameras = [camera("TG-6"), camera("Z9"), camera("iPhone")];

  it("for all: leaves out the cameras ignored and the offsets set by hand", () => {
    expect(camerasToSuggest(cameras, new Set(["Z9"]), new Set(["iPhone"]))).toEqual(["TG-6"]);
  });

  it("for all, on the selection path: leaves out the offsets set by hand", () => {
    expect(camerasToSuggest(cameras, new Set(["Z9"]))).toEqual(["TG-6", "iPhone"]);
  });

  it("takes each offset found, and keeps the offset shown where none was found", () => {
    const found = { "TG-6": 240, Z9: null };
    expect(withSuggestions({ "TG-6": 0, Z9: 30, iPhone: 5 }, ["TG-6", "Z9"], found)).toEqual({
      "TG-6": 240,
      Z9: 30,
      iPhone: 5,
    });
  });

  it("says when no offset was found, and clears the note of a camera that got one", () => {
    const found = { "TG-6": 240, Z9: null };
    expect(suggestionNotes({ "TG-6": "none", iPhone: "none" }, ["TG-6", "Z9"], found, "none")).toEqual({
      "TG-6": "",
      Z9: "none",
      iPhone: "none",
    });
  });
});

describe("reading the selection photo by photo", () => {
  const lr = { connected: true, selected: 5 };

  it("waits for a dive with a profile, as before, when the journey does not ask for it", () => {
    expect(readsSelection(false, false, lr)).toBe(false);
    expect(readsSelection(false, true, lr)).toBe(true);
  });

  it("starts as soon as the journey asks for it (choice no. 3)", () => {
    expect(readsSelection(true, false, lr)).toBe(true);
  });

  it("needs Lightroom and photos selected", () => {
    expect(readsSelection(true, true, { connected: false, selected: 5 })).toBe(false);
    expect(readsSelection(true, true, { connected: true, selected: 0 })).toBe(false);
  });
});

describe("the description Apply may use", () => {
  const photos: SelectionPhotos = { search: 4, photos: 5, undated: 0, cameras: [camera("TG-6")] };
  const described = { photos, version: 7 };

  it("is the one of the selection shown, once read", () => {
    expect(currentDescription(described, 7, false, null)).toBe(photos);
  });

  it("is none while the selection is read again, after a failure, or for another selection", () => {
    expect(currentDescription(described, 7, true, null)).toBeNull();
    expect(currentDescription(described, 7, false, "failed")).toBeNull();
    expect(currentDescription(described, 8, false, null)).toBeNull();
    expect(currentDescription(null, 7, false, null)).toBeNull();
  });

  it("has fresh counts only when they were computed for it", () => {
    const preview: TagPreview = {
      total: 5,
      inDive: 4,
      cameras: [],
      coverFrom: null,
      coverTo: null,
      outside: [],
      outsideMore: 0,
    };
    expect(freshCountsOf({ search: 4, preview }, photos)).toBe(preview);
    expect(freshCountsOf({ search: 3, preview }, photos)).toBeNull();
    expect(freshCountsOf({ search: 4, preview }, null)).toBeNull();
    expect(freshCountsOf(null, photos)).toBeNull();
  });
});
