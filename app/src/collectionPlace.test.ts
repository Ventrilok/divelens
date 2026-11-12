import { describe, expect, it } from "vitest";
import { flattenSets, groupName, pathKey, placeLabel, setExists, unitsInUse } from "./collectionPlace";
import type { CollectionSet } from "./types";

const tree: CollectionSet[] = [
  { name: "Dive Lens", children: [] },
  { name: "Trips", children: [{ name: "2025", children: [] }, { name: "2026", children: [] }] },
];

describe("collections place", () => {
  it("lists sets parents first, with their depth", () => {
    expect(flattenSets(tree)).toEqual([
      { path: ["Dive Lens"], depth: 0 },
      { path: ["Trips"], depth: 0 },
      { path: ["Trips", "2025"], depth: 1 },
      { path: ["Trips", "2026"], depth: 1 },
    ]);
    expect(flattenSets([])).toEqual([]);
  });

  it("finds a set by its path from the root", () => {
    expect(setExists(tree, [])).toBe(true);
    expect(setExists(tree, ["Trips", "2026"])).toBe(true);
    expect(setExists(tree, ["Trips", "2027"])).toBe(false);
    expect(setExists(tree, ["2026"])).toBe(false);
  });

  it("names the place", () => {
    expect(placeLabel({ parent: [], group: "Dive Lens" }, "Root")).toBe("Dive Lens");
    expect(placeLabel({ parent: [], group: "  " }, "Root")).toBe("Root");
    expect(placeLabel({ parent: ["Trips", "2026"], group: "" }, "Root")).toBe("Trips › 2026");
    expect(placeLabel({ parent: ["Trips"], group: " Plongées " }, "Root")).toBe("Trips › Plongées");
    expect(groupName({ parent: [], group: " x " })).toBe("x");
    expect(pathKey(["a", "b"])).not.toBe(pathKey(["a,b"]));
  });

  it("forces units only where asked", () => {
    const canadian = { depth: "ft", temp: "c" } as const;
    expect(unitsInUse({ depth: "source", temp: "source" }, canadian)).toEqual(canadian);
    expect(unitsInUse({ depth: "m", temp: "source" }, canadian)).toEqual({ depth: "m", temp: "c" });
    expect(unitsInUse({ depth: "source", temp: "f" }, canadian)).toEqual({ depth: "ft", temp: "f" });
  });
});
