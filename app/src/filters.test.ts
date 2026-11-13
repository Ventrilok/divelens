import { describe, expect, it, vi } from "vitest";
import type { Dive } from "./types";

vi.stubGlobal("navigator", { language: "en-GB" });
const { applyFilters, countryCounts, NO_FILTERS, searchIndex, sortDives } = await import("./filters");

function dive(p: Partial<Dive> & { id: string }): Dive {
  return {
    number: null,
    start: "2025-01-01T10:00:00",
    end: "2025-01-01T10:40:00",
    durationS: 2400,
    maxDepthM: null,
    tempHighC: null,
    tempLowC: null,
    rating: null,
    site: { id: null, name: null, location: null, country: null, lat: null, lon: null },
    hasProfile: false,
    invalid: false,
    notes: null,
    ...p,
  };
}

const dives = [
  dive({ id: "a", number: 1, start: "2024-06-01T09:00:00", maxDepthM: 12, rating: 3, hasProfile: true, site: { id: null, name: "Coral Garden", location: "North Reef", country: "Égypte", lat: null, lon: null } }),
  dive({ id: "b", number: 2, start: "2025-03-10T14:00:00", maxDepthM: 30, site: { id: null, name: "Blue Hole", location: null, country: "Atlantis", lat: null, lon: null } }),
  dive({ id: "c", number: 13, start: "2025-03-11T08:00:00", maxDepthM: 22, rating: 5, hasProfile: true }),
];
const index = searchIndex(dives);
const ids = (ds: Dive[]) => ds.map((d) => d.id);
const filter = (patch: object) => ids(applyFilters(dives, { ...NO_FILTERS, ...patch }, index));

describe("filters", () => {
  it("searches number, site, location and country, ignoring accents", () => {
    expect(filter({ query: "egypte" })).toEqual(["a"]);
    expect(filter({ query: "north coral" })).toEqual(["a"]);
    expect(filter({ query: "13" })).toEqual(["c"]);
    expect(filter({ query: "BLUE" })).toEqual(["b"]);
  });
  it("filters by period (inclusive days)", () => {
    expect(filter({ from: "2025-03-10", to: "2025-03-10" })).toEqual(["b"]);
    expect(filter({ from: "2025-01-01" })).toEqual(["b", "c"]);
  });
  it("filters by country, including dives without one", () => {
    expect(filter({ countries: ["Atlantis", ""] })).toEqual(["b", "c"]);
  });
  it("filters by depth, rating and profile", () => {
    expect(filter({ depthMin: 20 })).toEqual(["b", "c"]);
    expect(filter({ depthMin: 20, depthMax: 25 })).toEqual(["c"]);
    expect(filter({ minRating: 4 })).toEqual(["c"]);
    expect(filter({ withProfile: true })).toEqual(["a", "c"]);
  });
});

describe("sorting and counts", () => {
  it("sorts by date and by depth", () => {
    expect(ids(sortDives(dives, "recent"))).toEqual(["c", "b", "a"]);
    expect(ids(sortDives(dives, "oldest"))).toEqual(["a", "b", "c"]);
    expect(ids(sortDives(dives, "depth"))).toEqual(["b", "c", "a"]);
  });
  it("counts countries", () => {
    expect(countryCounts(dives)).toEqual([["", 1], ["Atlantis", 1], ["Égypte", 1]]);
  });
});
