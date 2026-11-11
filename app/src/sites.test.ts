import { describe, expect, it } from "vitest";
import { gpsInput, parseCoordinate, siteTree } from "./sites";
import type { Site } from "./types";

const site = (id: string, name: string | null, location: string | null, country: string | null): Site => ({
  id,
  name,
  location,
  country,
  lat: null,
  lon: null,
});

describe("siteTree", () => {
  it("groups by country then location, sorted, missing ones last", () => {
    const tree = siteTree([
      site("1", "Fottheyo", "Vaavu Atoll", "Maldives"),
      site("2", "Alimatha", "Vaavu Atoll", "Maldives"),
      site("3", "Kuda Rah Thila", "Ari Atoll", "Maldives"),
      site("4", "Lonely Rock", null, null),
      site("5", "Abu Dabab", "Marsa Alam", "Egypt"),
      site("6", "Pier", null, "Egypt"),
      site("7", null, "Nowhere", "Egypt"),
    ]);
    expect(tree.map((c) => [c.name, c.count])).toEqual([
      ["Egypt", 2],
      ["Maldives", 3],
      [null, 1],
    ]);
    const maldives = tree[1];
    expect(maldives.locations.map((l) => l.name)).toEqual(["Ari Atoll", "Vaavu Atoll"]);
    expect(maldives.locations[1].sites.map((s) => s.name)).toEqual(["Alimatha", "Fottheyo"]);
    expect(tree[0].locations.map((l) => l.name)).toEqual(["Marsa Alam", null]);
  });

  it("keeps two sites with the same name apart", () => {
    const tree = siteTree([site("a", "Reef", "North", "X"), site("b", "Reef", "North", "X")]);
    expect(tree[0].locations[0].sites.map((s) => s.id)).toEqual(["a", "b"]);
  });
});

describe("GPS input", () => {
  it("accepts a comma or a dot", () => {
    expect(parseCoordinate(" 3,4878 ")).toBe(3.4878);
    expect(parseCoordinate("-73.5")).toBe(-73.5);
    expect(parseCoordinate("")).toBeNull();
    expect(parseCoordinate("3.4.5")).toBeNaN();
    expect(parseCoordinate("N3")).toBeNaN();
  });

  it("needs both coordinates, in range", () => {
    expect(gpsInput("", "")).toEqual({ lat: null, lon: null });
    expect(gpsInput("3.5", "73.7")).toEqual({ lat: 3.5, lon: 73.7 });
    expect(gpsInput("3.5", "")).toBeUndefined();
    expect(gpsInput("91", "0")).toBeUndefined();
    expect(gpsInput("0", "-180.5")).toBeUndefined();
    expect(gpsInput("abc", "1")).toBeUndefined();
  });
});
